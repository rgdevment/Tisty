use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{AeadInOut, KeyInit};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::cloud::{HEAD, TAG, nonced};
use crate::identity::{Certificate, Revocation, ShareKey, Signed, Verdict, check};

pub const TEXT_AT_MOST: usize = 512 * 1024;
pub const PICTURE_AT_MOST: usize = 512 * 1024;
pub const PICTURES_AT_MOST: usize = 10;
pub const ALL_PICTURES_AT_MOST: usize = 2 * 1024 * 1024;
/// A signature, its certificate and their names fit well inside this.
const SIGNATURE_AT_MOST: usize = 2 * 1024;
/// Longer than any manifest the limits above allow, short enough that a forged length costs nothing.
const MANIFEST_AT_MOST: usize = 4 * 1024;
const ID_AT_MOST: usize = 64;

const MAGIC: &[u8; 4] = b"\xffTSE";
const VERSION: u8 = 1;
const SAID: u32 = 1;
const PART: &[u8] = b"tisty-share.part.v1\0";
const SEALED: &[u8] = b"tisty-share.sealed.v1\0";
const WRITTEN: &[u8] = b"tisty-share.written.v1\0";
/// What the outer signature answers for: these sealed bytes, uploaded as this id and version.
pub const SENT: &str = "tisty-share.envelope.v1";
/// What the inner signature answers for: these words and pictures, wherever they travel.
pub const WROTE: &str = "tisty-share.text.v1";

/// The key in the link's `#`: the server never sees it, and nothing about it comes from a password.
pub struct LinkKey(Zeroizing<[u8; 32]>);

impl LinkKey {
    #[cfg(feature = "mint")]
    pub fn minted() -> Option<Self> {
        crate::minted().map(Self)
    }

    pub fn from_bytes(key: &[u8; 32]) -> Self {
        Self(Zeroizing::new(*key))
    }

    pub fn to_text(&self) -> Zeroizing<String> {
        Zeroizing::new(URL_SAFE_NO_PAD.encode(self.0.as_ref()))
    }

    pub fn from_text(said: &str) -> Option<Self> {
        let bytes = Zeroizing::new(URL_SAFE_NO_PAD.decode(said).ok()?);
        Some(Self::from_bytes(
            &<[u8; 32]>::try_from(bytes.as_slice()).ok()?,
        ))
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new((&*self.0).into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Text,
    /// The inner signature, sealed like the words it answers for.
    Signature,
    Png,
    Jpeg,
    Webp,
    Gif,
}

impl Kind {
    pub fn is_picture(self) -> bool {
        !matches!(self, Kind::Text | Kind::Signature)
    }

    pub fn media(self) -> &'static str {
        match self {
            Kind::Text => "text/markdown",
            Kind::Signature => "application/json",
            Kind::Png => "image/png",
            Kind::Jpeg => "image/jpeg",
            Kind::Webp => "image/webp",
            Kind::Gif => "image/gif",
        }
    }

    /// What the bytes say they are, by their first bytes: a declared kind has to match them.
    pub fn of_picture(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some(Kind::Png)
        } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            Some(Kind::Jpeg)
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some(Kind::Webp)
        } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            Some(Kind::Gif)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub kind: Kind,
    /// Sealed, tag included, so the server checks it against the bytes it holds.
    pub size: u64,
}

impl Part {
    pub fn plain(&self) -> usize {
        usize::try_from(self.size)
            .unwrap_or(usize::MAX)
            .saturating_sub(TAG)
    }
}

/// What the server reads without the key: enough to hold every limit and nothing of what is said.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub v: u32,
    pub id: String,
    pub version: u64,
    /// The version this upload was written on, so a change made over someone else's is caught.
    pub from: Option<u64>,
    pub head: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("this is not a Tisty Share envelope")]
    Foreign,
    #[error("this envelope was sealed by a newer Tisty")]
    Newer,
    #[error("this envelope is damaged or was changed on the way")]
    Damaged,
    #[error("the text is over 512 KB")]
    TextTooBig,
    #[error("a picture is over 512 KB")]
    PictureTooBig,
    #[error("there are more than 10 pictures")]
    TooManyPictures,
    #[error("the pictures are over 2 MB together")]
    PicturesTooBig,
    #[error("a picture is not the kind it says")]
    NotAPicture,
    #[error("this envelope does not open with this key, or something changed it")]
    Locked,
    #[error("the signature could not be made")]
    Unsigned,
    #[error("there was no randomness to seal with")]
    NoRandomness,
}

impl Manifest {
    /// Every rule a server can hold without the key, so the server and every Tisty refuse the same things.
    pub fn holds(&self) -> Result<(), Refused> {
        match self.v {
            SAID => {}
            newer if newer > SAID => return Err(Refused::Newer),
            _ => return Err(Refused::Damaged),
        }
        let named = !self.id.is_empty()
            && self.id.len() <= ID_AT_MOST
            && self
                .id
                .bytes()
                .all(|one| one.is_ascii_alphanumeric() || one == b'-' || one == b'_');
        let ordered = self.version >= 1 && self.from.is_none_or(|from| from < self.version);
        if !named || !ordered || self.head().is_none() {
            return Err(Refused::Damaged);
        }
        self.parts_hold()
    }

    fn parts_hold(&self) -> Result<(), Refused> {
        let (text, rest) = self.parts.split_first().ok_or(Refused::Damaged)?;
        if text.kind != Kind::Text || text.size < TAG as u64 {
            return Err(Refused::Damaged);
        }
        if text.plain() > TEXT_AT_MOST {
            return Err(Refused::TextTooBig);
        }
        let pictures = match rest.split_first() {
            Some((signature, pictures)) if signature.kind == Kind::Signature => {
                if signature.size < TAG as u64 || signature.plain() > SIGNATURE_AT_MOST {
                    return Err(Refused::Damaged);
                }
                pictures
            }
            _ => rest,
        };
        if pictures
            .iter()
            .any(|one| !one.kind.is_picture() || one.size < TAG as u64)
        {
            return Err(Refused::Damaged);
        }
        if pictures.len() > PICTURES_AT_MOST {
            return Err(Refused::TooManyPictures);
        }
        if pictures.iter().any(|one| one.plain() > PICTURE_AT_MOST) {
            return Err(Refused::PictureTooBig);
        }
        let all: usize = pictures.iter().map(Part::plain).sum();
        if all > ALL_PICTURES_AT_MOST {
            return Err(Refused::PicturesTooBig);
        }
        Ok(())
    }

    fn head(&self) -> Option<[u8; HEAD]> {
        <[u8; HEAD]>::try_from(URL_SAFE_NO_PAD.decode(&self.head).ok()?.as_slice()).ok()
    }

    pub fn signed(&self) -> bool {
        self.parts
            .get(1)
            .is_some_and(|one| one.kind == Kind::Signature)
    }
}

/// A document sealed for one version: the manifest in the clear, each part sealed on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    manifest: Manifest,
    /// The manifest as it was written, since the seal holds these bytes and not their meaning.
    written: Vec<u8>,
    parts: Vec<Vec<u8>>,
    sent: Option<Signed>,
}

/// What a person shares: the words and their pictures, already recompressed by Tisty.
pub struct Draft<'a> {
    pub id: &'a str,
    pub version: u64,
    pub from: Option<u64>,
    pub text: &'a str,
    pub pictures: &'a [(Kind, &'a [u8])],
}

/// The machine's key and the certificate its person gave it.
pub struct Signer<'a> {
    pub key: &'a ShareKey,
    pub certificate: &'a Certificate,
}

pub struct Opened {
    pub manifest: Manifest,
    pub text: Zeroizing<String>,
    pub pictures: Vec<(Kind, Vec<u8>)>,
    /// Who wrote these words and pictures, when the envelope carries their signature.
    pub wrote: Option<Verdict>,
    /// Who sent these bytes as this version, when the envelope carries that signature.
    pub sent: Option<Verdict>,
}

#[cfg(feature = "mint")]
pub fn seal(key: &LinkKey, draft: &Draft, by: Option<&Signer>) -> Result<Envelope, Refused> {
    check_draft(draft)?;
    let inner = by
        .map(|signer| sign(signer, WROTE, &written(draft.text, draft.pictures)))
        .transpose()?
        .map(|signed| serde_json::to_vec(&signed).map_err(|_| Refused::Unsigned))
        .transpose()?;
    let mut plain: Vec<(Kind, &[u8])> = vec![(Kind::Text, draft.text.as_bytes())];
    if let Some(inner) = inner.as_deref() {
        plain.push((Kind::Signature, inner));
    }
    plain.extend(draft.pictures.iter().copied());

    let head: [u8; HEAD] = {
        let minted = crate::minted().ok_or(Refused::NoRandomness)?;
        minted[..HEAD]
            .try_into()
            .map_err(|_| Refused::NoRandomness)?
    };
    let manifest = Manifest {
        v: SAID,
        id: draft.id.to_owned(),
        version: draft.version,
        from: draft.from,
        head: URL_SAFE_NO_PAD.encode(head),
        parts: plain
            .iter()
            .map(|(kind, bytes)| Part {
                kind: *kind,
                size: (bytes.len() + TAG) as u64,
            })
            .collect(),
    };
    manifest.holds()?;
    let written = serde_json::to_vec(&manifest).map_err(|_| Refused::Damaged)?;
    let aad = bound(&written);
    let cipher = key.cipher();
    let last = plain.len() - 1;
    let parts = plain
        .iter()
        .enumerate()
        .map(|(at, (_, bytes))| {
            let mut sealed = bytes.to_vec();
            cipher
                .encrypt_in_place(
                    &nonced(&head, at as u32, at == last).into(),
                    &aad,
                    &mut sealed,
                )
                .map(|()| sealed)
                .map_err(|_| Refused::Locked)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut envelope = Envelope {
        manifest,
        written,
        parts,
        sent: None,
    };
    envelope.sent = by
        .map(|signer| sign(signer, SENT, &envelope.answered()))
        .transpose()?;
    Ok(envelope)
}

/// The pictures must be what they say, so a file passed off as a picture never leaves Tisty.
#[cfg(feature = "mint")]
fn check_draft(draft: &Draft) -> Result<(), Refused> {
    if draft
        .pictures
        .iter()
        .any(|(kind, bytes)| !kind.is_picture() || Kind::of_picture(bytes) != Some(*kind))
    {
        return Err(Refused::NotAPicture);
    }
    Ok(())
}

#[cfg(feature = "mint")]
fn sign(signer: &Signer, about: &str, said: &[u8]) -> Result<Signed, Refused> {
    signer
        .key
        .sign(signer.certificate, about, said)
        .ok_or(Refused::Unsigned)
}

pub fn open(key: &LinkKey, envelope: &Envelope, revoked: &[Revocation]) -> Result<Opened, Refused> {
    let manifest = &envelope.manifest;
    let head = manifest.head().ok_or(Refused::Damaged)?;
    let aad = bound(&envelope.written);
    let cipher = key.cipher();
    let last = envelope.parts.len() - 1;
    let mut plain = Vec::with_capacity(envelope.parts.len());
    for (at, sealed) in envelope.parts.iter().enumerate() {
        let mut bytes = Zeroizing::new(sealed.clone());
        cipher
            .decrypt_in_place(
                &nonced(&head, at as u32, at == last).into(),
                &aad,
                &mut *bytes,
            )
            .map_err(|_| Refused::Locked)?;
        plain.push(bytes);
    }
    let text = Zeroizing::new(String::from_utf8(plain[0].to_vec()).map_err(|_| Refused::Damaged)?);
    let first_picture = if manifest.signed() { 2 } else { 1 };
    let pictures = manifest.parts[first_picture..]
        .iter()
        .zip(&plain[first_picture..])
        .map(|(part, bytes)| match Kind::of_picture(bytes) {
            Some(kind) if kind == part.kind => Ok((kind, bytes.to_vec())),
            _ => Err(Refused::NotAPicture),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let wrote = if manifest.signed() {
        let inner: Signed = serde_json::from_slice(&plain[1]).map_err(|_| Refused::Damaged)?;
        let shown: Vec<(Kind, &[u8])> = pictures
            .iter()
            .map(|(kind, bytes)| (*kind, bytes.as_slice()))
            .collect();
        Some(check(&inner, WROTE, &written(&text, &shown), revoked))
    } else {
        None
    };
    Ok(Opened {
        manifest: manifest.clone(),
        text,
        pictures,
        wrote,
        sent: envelope.sender(revoked),
    })
}

impl Envelope {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Who sent these sealed bytes, checked without the key: what the server can say about an upload.
    pub fn sender(&self, revoked: &[Revocation]) -> Option<Verdict> {
        self.sent
            .as_ref()
            .map(|sent| check(sent, SENT, &self.answered(), revoked))
    }

    /// The id and the version are signed with the sealed bytes, so a signature never travels to another upload.
    fn answered(&self) -> Vec<u8> {
        let mut said = Vec::with_capacity(8 + self.manifest.id.len() + 8 + 32);
        said.extend_from_slice(&(self.manifest.id.len() as u64).to_be_bytes());
        said.extend_from_slice(self.manifest.id.as_bytes());
        said.extend_from_slice(&self.manifest.version.to_be_bytes());
        said.extend_from_slice(&self.digest());
        said
    }

    /// Over the manifest as written and every sealed part, each with its length.
    pub fn digest(&self) -> [u8; 32] {
        let mut over = Sha256::new();
        over.update(SEALED);
        over.update((self.written.len() as u64).to_be_bytes());
        over.update(&self.written);
        for part in &self.parts {
            over.update((part.len() as u64).to_be_bytes());
            over.update(part);
        }
        over.finalize().into()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let sent = self
            .sent
            .as_ref()
            .and_then(|sent| serde_json::to_vec(sent).ok())
            .unwrap_or_default();
        let mut out = MAGIC.to_vec();
        out.push(VERSION);
        out.extend_from_slice(&(self.written.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.written);
        self.parts
            .iter()
            .for_each(|part| out.extend_from_slice(part));
        out.extend_from_slice(&(sent.len() as u32).to_be_bytes());
        out.extend_from_slice(&sent);
        out
    }

    /// Reads and holds every limit without the key, so a server can refuse before it stores anything.
    pub fn read(bytes: &[u8]) -> Result<Self, Refused> {
        let rest = bytes.strip_prefix(MAGIC).ok_or(Refused::Foreign)?;
        let (&version, rest) = rest.split_first().ok_or(Refused::Damaged)?;
        match version {
            VERSION => {}
            newer if newer > VERSION => return Err(Refused::Newer),
            _ => return Err(Refused::Damaged),
        }
        let (written, mut rest) = framed(rest, MANIFEST_AT_MOST)?;
        let manifest: Manifest = serde_json::from_slice(written).map_err(|_| Refused::Damaged)?;
        manifest.holds()?;
        let mut parts = Vec::with_capacity(manifest.parts.len());
        for part in &manifest.parts {
            let size = usize::try_from(part.size).map_err(|_| Refused::Damaged)?;
            let (sealed, after) = rest.split_at_checked(size).ok_or(Refused::Damaged)?;
            parts.push(sealed.to_vec());
            rest = after;
        }
        let (sent, rest) = framed(rest, SIGNATURE_AT_MOST)?;
        if !rest.is_empty() {
            return Err(Refused::Damaged);
        }
        let sent = match sent {
            [] => None,
            said => Some(serde_json::from_slice(said).map_err(|_| Refused::Damaged)?),
        };
        Ok(Self {
            manifest,
            written: written.to_vec(),
            parts,
            sent,
        })
    }
}

/// A length, then that many bytes, never more than `most`.
fn framed(bytes: &[u8], most: usize) -> Result<(&[u8], &[u8]), Refused> {
    let (len, rest) = bytes.split_first_chunk::<4>().ok_or(Refused::Damaged)?;
    let len = u32::from_be_bytes(*len) as usize;
    if len > most {
        return Err(Refused::Damaged);
    }
    rest.split_at_checked(len).ok_or(Refused::Damaged)
}

/// Every part is bound to this very manifest, so none can be moved to another or have its size or kind changed.
fn bound(written: &[u8]) -> Vec<u8> {
    let mut aad = PART.to_vec();
    aad.extend_from_slice(&Sha256::digest(written));
    aad
}

/// The words and every picture, each with its kind and length, as the inner signature reads them.
fn written(text: &str, pictures: &[(Kind, &[u8])]) -> Vec<u8> {
    let mut over = Sha256::new();
    over.update(WRITTEN);
    over.update((text.len() as u64).to_be_bytes());
    over.update(text.as_bytes());
    for (kind, bytes) in pictures {
        over.update(kind.media().as_bytes());
        over.update((bytes.len() as u64).to_be_bytes());
        over.update(bytes);
    }
    over.finalize().to_vec()
}

#[cfg(all(test, feature = "mint"))]
#[path = "envelope_test.rs"]
mod tests;
