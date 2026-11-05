use std::io::Write as _;

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use hkdf::Hkdf;
use hmac::{Hmac, KeyInit as _, Mac};
use rand_core::{OsRng, TryRngCore};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tisty_contracts::{
    Document, Envelope, EnvelopeKind, EnvelopePart, EnvelopeWriter, Owner, PartName, Picture,
    Sealing, Signature, VERSION, check, check_html, pictures_in, references_in,
};
use tisty_share_proto::identity::{self, Certificate, Permit, ShareKey, Signed};
use zeroize::Zeroizing;

pub(crate) const MARK: &[u8; 4] = b"TENV";
const TAG_BYTES: u64 = 16;
const COMMITMENT: &[u8] = b"tisty-envelope-commit";
const KEY_INFO: &[u8] = b"tisty-envelope-key";
pub(crate) const ABOUT: &str = "envelope";
pub(crate) const OWNER_ABOUT: &str = "owner";
const NAME_TAG: &[u8] = b"tisty-owner-name\0";
const NO_PERMIT: [u8; 32] = [0; 32];
pub(crate) const MANIFEST_AT_MOST: usize = 16_384;
pub(crate) const SIGNATURE_AT_MOST: usize = 2_048;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("the {0} is outside the contract: {1:?}")]
    OutsideTheContract(&'static str, Vec<String>),
    #[error("{0}")]
    DoesNotAdd(&'static str),
    #[error("the link key is not 32 bytes in base64url")]
    LinkKey,
    #[error("this machine's key cannot sign under this certificate")]
    Unsigned,
    #[error("this machine gave no randomness")]
    NoRandomness,
    #[error("it could not be sealed")]
    Unsealed,
}

/// What travels of one document in one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sealed {
    pub main: Vec<u8>,
    pub files: Option<Vec<u8>>,
    /// What answers for its files, sent now or kept, so the next version can keep them.
    pub files_hash: Option<String>,
}

/// The one who seals and signs, with this machine's key under its certificate.
pub struct Writer<'a> {
    key: &'a ShareKey,
    certificate: &'a Certificate,
    head: Box<dyn FnMut() -> Option<[u8; 32]> + 'a>,
}

fn random_head() -> Option<[u8; 32]> {
    let mut head = [0u8; 32];
    OsRng.try_fill_bytes(&mut head).ok()?;
    Some(head)
}

/// A new link key, 32 random bytes in base64url.
pub fn link_key() -> Result<String, Refused> {
    random_head()
        .map(|key| URL_SAFE_NO_PAD.encode(key))
        .ok_or(Refused::NoRandomness)
}

fn deflated(bytes: &[u8]) -> Result<Vec<u8>, Refused> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes).map_err(|_| Refused::Unsealed)?;
    encoder.finish().map_err(|_| Refused::Unsealed)
}

struct Going {
    name: PartName,
    compressed: bool,
    open_size: u64,
    bytes: Vec<u8>,
}

fn squeezed(name: PartName, open: &[u8]) -> Result<Going, Refused> {
    Ok(Going {
        name,
        compressed: true,
        open_size: open.len() as u64,
        bytes: deflated(open)?,
    })
}

fn as_it_is(name: PartName, bytes: Vec<u8>) -> Going {
    Going {
        name,
        compressed: false,
        open_size: bytes.len() as u64,
        bytes,
    }
}

fn opening_of(link_key: &str) -> Result<Zeroizing<Vec<u8>>, Refused> {
    let key = Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(link_key)
            .map_err(|_| Refused::LinkKey)?,
    );
    if key.len() == 32 {
        Ok(key)
    } else {
        Err(Refused::LinkKey)
    }
}

fn committed_to(link_key: &[u8], head: &[u8; 32]) -> Result<[u8; 32], Refused> {
    let mut mac = Hmac::<Sha256>::new_from_slice(link_key).map_err(|_| Refused::LinkKey)?;
    mac.update(COMMITMENT);
    mac.update(head);
    Ok(mac.finalize().into_bytes().into())
}

pub(crate) fn commitment_holds(link_key: &[u8], head: &[u8; 32], said: &[u8]) -> bool {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(link_key) else {
        return false;
    };
    mac.update(COMMITMENT);
    mac.update(head);
    mac.verify_slice(said).is_ok()
}

pub(crate) fn name_tag(link_key: &[u8], id: &str, name: &str) -> Result<[u8; 32], Refused> {
    let mut mac = Hmac::<Sha256>::new_from_slice(link_key).map_err(|_| Refused::LinkKey)?;
    mac.update(NAME_TAG);
    mac.update(&(id.len() as u64).to_be_bytes());
    mac.update(id.as_bytes());
    mac.update(name.as_bytes());
    Ok(mac.finalize().into_bytes().into())
}

pub(crate) fn owner_said(id: &str, permit: Option<&[u8]>, tag: &[u8]) -> Vec<u8> {
    let mut said = (id.len() as u64).to_be_bytes().to_vec();
    said.extend_from_slice(id.as_bytes());
    said.extend_from_slice(permit.unwrap_or(&NO_PERMIT));
    said.extend_from_slice(tag);
    said
}

// Sealing is the last look on this machine, so a block no reader would take goes no further.
fn received_owner_holds(owner: &Owner, id: &str, tag: &[u8]) -> Result<(), Refused> {
    checked("owner", &json!(owner))?;
    let refused = Refused::DoesNotAdd("its owner's block does not hold");
    let permit = owner
        .permit
        .as_deref()
        .map(|one| URL_SAFE_NO_PAD.decode(one))
        .transpose()
        .map_err(|_| refused.clone())?;
    let signed = Signed {
        certificate: Certificate {
            root: owner.root.clone(),
            share: owner.machine.clone(),
            since: owner.since.parse().map_err(|_| refused.clone())?,
            sig: owner.certificate.clone(),
        },
        sig: owner.signed.clone(),
    };
    let said = owner_said(id, permit.as_deref(), tag);
    match identity::check(&signed, OWNER_ABOUT, &said, &[]) {
        identity::Verdict::Holds { .. } => Ok(()),
        _ => Err(refused),
    }
}

fn permit_of(token: &str) -> Result<Permit, Refused> {
    let seed = Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| Refused::DoesNotAdd("the token is not 32 bytes in base64url"))?,
    );
    let seed = <&[u8; 32]>::try_from(seed.as_slice())
        .map_err(|_| Refused::DoesNotAdd("the token is not 32 bytes in base64url"))?;
    Ok(Permit::from_token(seed))
}

pub(crate) fn cipher_of(link_key: &[u8], head: &[u8; 32]) -> Result<Aes256Gcm, Refused> {
    let mut key = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(head), link_key)
        .expand(KEY_INFO, key.as_mut())
        .map_err(|_| Refused::Unsealed)?;
    Aes256Gcm::new_from_slice(key.as_ref()).map_err(|_| Refused::Unsealed)
}

pub(crate) fn nonce(number: usize) -> Result<[u8; 12], Refused> {
    let mut out = [0u8; 12];
    let number = u32::try_from(number).map_err(|_| Refused::Unsealed)?;
    out[8..].copy_from_slice(&number.to_be_bytes());
    Ok(out)
}

fn checked(contract: &'static str, value: &Value) -> Result<(), Refused> {
    let wrong = check(contract, value);
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(Refused::OutsideTheContract(contract, wrong))
    }
}

fn html_holds(html: &str) -> Result<(), Refused> {
    let wrong = check_html(html);
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(Refused::OutsideTheContract("html", wrong))
    }
}

// A reference that points past what travels would open nothing, so it never leaves here.
fn references_fit(html: &str, document: &Document) -> Result<(), Refused> {
    let pages: Vec<u64> = document.pages.iter().flatten().map(|one| one.at).collect();
    let pictures = document.pictures.as_ref().map_or(0, Vec::len);
    let files = document.files.as_ref().map_or(0, Vec::len);
    let cited = document.cited.as_ref().map_or(0, Vec::len);
    let holds = references_in(html)
        .into_iter()
        .all(|(kind, at)| match kind.as_str() {
            "page" => pages.contains(&at),
            "picture" => (at as usize) < pictures,
            "file" => (at as usize) < files,
            _ => (at as usize) < cited,
        });
    if !holds {
        return Err(Refused::DoesNotAdd(
            "a reference points at something that does not travel",
        ));
    }
    let at_its_size = pictures_in(html).into_iter().all(|(at, width, height)| {
        document
            .pictures
            .iter()
            .flatten()
            .nth(at as usize)
            .is_some_and(|one| width == one.width.to_string() && height == one.height.to_string())
    });
    if at_its_size {
        Ok(())
    } else {
        Err(Refused::DoesNotAdd(
            "a picture is drawn at another size than its own",
        ))
    }
}

fn pictures_are_what_they_say(said: &[Picture], all: &[Vec<u8>]) -> Result<(), Refused> {
    if said.len() != all.len() {
        return Err(Refused::DoesNotAdd(
            "it says another number of pictures than it holds",
        ));
    }
    for (number, (picture, bytes)) in said.iter().zip(all).enumerate() {
        let (first, ending): (&[u8], &str) = match picture.kind {
            tisty_contracts::PictureKind::ImagePng => (b"\x89PNG\r\n\x1a\n", "png"),
            tisty_contracts::PictureKind::ImageJpeg => (b"\xff\xd8\xff", "jpg"),
        };
        if picture.name != format!("picture-{number}.{ending}") || all[..number].contains(bytes) {
            return Err(Refused::DoesNotAdd(
                "a picture says where it came from, or travels twice",
            ));
        }
        let size = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .ok()
            .and_then(|one| one.into_dimensions().ok());
        if picture.at != number as u64
            || picture.size != bytes.len() as u64
            || !bytes.starts_with(first)
            || size
                != u32::try_from(picture.width)
                    .ok()
                    .zip(u32::try_from(picture.height).ok())
        {
            return Err(Refused::DoesNotAdd(
                "a picture is not what its document says",
            ));
        }
    }
    Ok(())
}

// Files kept from an earlier version travel in the envelope already sent, so here they are only named.
fn files_are_what_they_say(
    document: &Document,
    bytes: &[Vec<u8>],
    kept: bool,
) -> Result<(), Refused> {
    let said = document.files.as_deref().unwrap_or_default();
    let in_place = said
        .iter()
        .enumerate()
        .all(|(number, file)| file.at == number as u64);
    if kept {
        if said.is_empty() || !in_place {
            return Err(Refused::DoesNotAdd(
                "it keeps files its document does not say",
            ));
        }
        return if bytes.is_empty() {
            Ok(())
        } else {
            Err(Refused::DoesNotAdd(
                "it keeps the files already sent and sends others too",
            ))
        };
    }
    if said.len() != bytes.len() {
        return Err(Refused::DoesNotAdd(
            "it says another number of files than it holds",
        ));
    }
    let adds = said
        .iter()
        .zip(bytes)
        .enumerate()
        .all(|(number, (file, bytes))| {
            file.at == number as u64 && file.size.is_none_or(|size| size == bytes.len() as u64)
        });
    if adds {
        Ok(())
    } else {
        Err(Refused::DoesNotAdd("a file is not what its document says"))
    }
}

// What only the writer knows, and a reader has no way to look at until it has opened everything.
fn adds_up(document: &Document, sealing: &Sealing) -> Result<(), Refused> {
    if sealing.from.is_some_and(|from| from >= sealing.version) {
        return Err(Refused::DoesNotAdd(
            "it starts from a version that is not an earlier one",
        ));
    }
    if sealing.files_hash.is_some() && sealing.files_key != sealing.link_key {
        return Err(Refused::DoesNotAdd(
            "it keeps files sealed under another key",
        ));
    }
    if document.languages.is_some() && document.lang.is_none() {
        return Err(Refused::DoesNotAdd(
            "it holds the map of its languages without saying its own",
        ));
    }
    let mut ids = std::collections::BTreeSet::new();
    let mut last = 0;
    for page in document.pages.iter().flatten() {
        if page.page_of != document.id || page.at <= last || !ids.insert(&page.id) {
            return Err(Refused::DoesNotAdd(
                "a page is not in its place in this document",
            ));
        }
        last = page.at;
    }
    Ok(())
}

fn split(document: &Document) -> Result<(Vec<u8>, Vec<u8>), Refused> {
    let Value::Object(mut first) = serde_json::to_value(document).map_err(|_| Refused::Unsealed)?
    else {
        return Err(Refused::Unsealed);
    };
    let mut rest = serde_json::Map::new();
    for field in ["body", "pages"] {
        if let Some(value) = first.remove(field) {
            rest.insert(field.to_owned(), value);
        }
    }
    let written = |one| serde_json::to_vec(&Value::Object(one)).map_err(|_| Refused::Unsealed);
    Ok((written(first)?, written(rest)?))
}

impl<'a> Writer<'a> {
    pub fn new(key: &'a ShareKey, certificate: &'a Certificate) -> Self {
        Self::with_head(key, certificate, random_head)
    }

    /// A head of one's own makes the same bytes each time, which only sample envelopes want.
    pub fn with_head(
        key: &'a ShareKey,
        certificate: &'a Certificate,
        head: impl FnMut() -> Option<[u8; 32]> + 'a,
    ) -> Self {
        Self {
            key,
            certificate,
            head: Box::new(head),
        }
    }

    /// Seals a document that holds to the contract, and nothing else leaves: this is the last look at it on this machine.
    pub fn sealed(
        &mut self,
        mut document: Document,
        pictures: Vec<Vec<u8>>,
        files: Vec<Vec<u8>>,
        sealing: &Sealing,
    ) -> Result<Sealed, Refused> {
        if document.id != sealing.id {
            return Err(Refused::DoesNotAdd(
                "its id is not the one it is sealed under",
            ));
        }
        adds_up(&document, sealing)?;
        let owner = self.owner_of(&document, sealing)?;
        pictures_are_what_they_say(document.pictures.as_deref().unwrap_or_default(), &pictures)?;
        files_are_what_they_say(&document, &files, sealing.files_hash.is_some())?;
        let files_envelope = if files.is_empty() {
            document.files_hash.clone_from(&sealing.files_hash);
            None
        } else {
            let parts = files
                .into_iter()
                .map(|one| as_it_is(PartName::File, one))
                .collect();
            let (bytes, hash) =
                self.envelope(EnvelopeKind::Files, parts, sealing, owner.as_ref())?;
            document.files_hash = Some(URL_SAFE_NO_PAD.encode(hash));
            Some(bytes)
        };
        checked(
            "document",
            &serde_json::to_value(&document).map_err(|_| Refused::Unsealed)?,
        )?;
        html_holds(&document.html)?;
        references_fit(&document.html, &document)?;
        for page in document.pages.iter().flatten() {
            html_holds(&page.html)?;
            references_fit(&page.html, &document)?;
        }
        let (first, rest) = split(&document)?;
        let mut parts = vec![
            squeezed(PartName::First, &first)?,
            squeezed(PartName::Rest, &rest)?,
        ];
        parts.extend(
            pictures
                .into_iter()
                .map(|one| as_it_is(PartName::Picture, one)),
        );
        let (main, _) = self.envelope(EnvelopeKind::Main, parts, sealing, owner.as_ref())?;
        Ok(Sealed {
            main,
            files: files_envelope,
            files_hash: document.files_hash,
        })
    }

    // What is public has no key to tie an owner to, so its signer still answers for it.
    fn owner_of(&self, document: &Document, sealing: &Sealing) -> Result<Option<Owner>, Refused> {
        let Some(link_key) = sealing.link_key.as_deref() else {
            return if sealing.owner.is_some()
                || sealing.token.is_some()
                || document.owner_name.is_some()
            {
                Err(Refused::DoesNotAdd(
                    "what is public has no owner's block yet",
                ))
            } else {
                Ok(None)
            };
        };
        let key = opening_of(link_key)?;
        let name = document.owner_name.as_deref().unwrap_or_default();
        let tag = name_tag(&key, &document.id, name)?;
        if let Some(owner) = &sealing.owner {
            if owner.name_tag != URL_SAFE_NO_PAD.encode(tag) {
                return Err(Refused::DoesNotAdd(
                    "its owner_name is not the one its owner signed",
                ));
            }
            received_owner_holds(owner, &document.id, &tag)?;
            return Ok(Some(owner.clone()));
        }
        let permit = sealing.token.as_deref().map(permit_of).transpose()?;
        let permit = permit.as_ref().map(Permit::shown);
        let permit_bytes = permit
            .as_deref()
            .map(|one| URL_SAFE_NO_PAD.decode(one))
            .transpose()
            .map_err(|_| Refused::Unsealed)?;
        let said = owner_said(&document.id, permit_bytes.as_deref(), &tag);
        let signed = self
            .key
            .sign(self.certificate, OWNER_ABOUT, &said)
            .ok_or(Refused::Unsigned)?;
        let owner = Owner {
            root: signed.certificate.root,
            machine: signed.certificate.share,
            since: signed.certificate.since.to_string(),
            certificate: signed.certificate.sig,
            permit,
            name_tag: URL_SAFE_NO_PAD.encode(tag),
            signed: signed.sig,
        };
        checked("owner", &json!(owner))?;
        Ok(Some(owner))
    }

    fn envelope(
        &mut self,
        kind: EnvelopeKind,
        parts: Vec<Going>,
        sealing: &Sealing,
        owner: Option<&Owner>,
    ) -> Result<(Vec<u8>, [u8; 32]), Refused> {
        let key = sealing.link_key.as_deref().map(opening_of).transpose()?;
        let head = key
            .as_ref()
            .map(|_| (self.head)().ok_or(Refused::NoRandomness))
            .transpose()?;
        let growth = if key.is_some() { TAG_BYTES } else { 0 };
        let manifest = Envelope {
            contract: VERSION.to_owned(),
            kind,
            id: sealing.id.clone(),
            version: sealing.version,
            from: sealing.from,
            sealed: key.is_some(),
            head: head.as_ref().map(|one| URL_SAFE_NO_PAD.encode(one)),
            commitment: match (&key, &head) {
                (Some(key), Some(head)) => Some(URL_SAFE_NO_PAD.encode(committed_to(key, head)?)),
                _ => None,
            },
            owner: owner.cloned(),
            parts: parts
                .iter()
                .map(|one| EnvelopePart {
                    name: one.name,
                    compressed: one.compressed,
                    sealed_size: one.bytes.len() as u64 + growth,
                    open_size: one.open_size,
                })
                .collect(),
        };
        checked(
            "envelope",
            &serde_json::to_value(&manifest).map_err(|_| Refused::Unsealed)?,
        )?;
        let manifest = serde_json::to_vec(&manifest).map_err(|_| Refused::Unsealed)?;
        if manifest.len() > MANIFEST_AT_MOST {
            return Err(Refused::DoesNotAdd("its manifest is past its size"));
        }
        let cipher = match (&key, &head) {
            (Some(key), Some(head)) => Some(cipher_of(key, head)?),
            _ => None,
        };
        let mut going = Vec::with_capacity(parts.len());
        for (number, part) in parts.into_iter().enumerate() {
            going.push(match &cipher {
                Some(cipher) => {
                    let payload = Payload {
                        msg: &part.bytes,
                        aad: &manifest,
                    };
                    cipher
                        .encrypt(Nonce::from_slice(&nonce(number)?), payload)
                        .map_err(|_| Refused::Unsealed)?
                }
                None => part.bytes,
            });
        }
        let mut index = Vec::with_capacity(32 * going.len());
        for one in &going {
            index.extend_from_slice(&Sha256::digest(one));
        }
        let signature = self.signature(&manifest, &index, sealing, owner)?;
        if signature.len() > SIGNATURE_AT_MOST {
            return Err(Refused::DoesNotAdd("its signature is past its size"));
        }
        let mut out = Vec::new();
        out.extend_from_slice(MARK);
        out.extend_from_slice(&length_of(&manifest)?);
        out.extend_from_slice(&manifest);
        out.extend_from_slice(&length_of(&signature)?);
        out.extend_from_slice(&signature);
        out.extend_from_slice(&index);
        for one in going {
            out.extend_from_slice(&one);
        }
        let mut hash = Sha256::new();
        hash.update(&manifest);
        hash.update(&index);
        Ok((out, hash.finalize().into()))
    }

    fn signature(
        &self,
        manifest: &[u8],
        index: &[u8],
        sealing: &Sealing,
        owner: Option<&Owner>,
    ) -> Result<Vec<u8>, Refused> {
        let mut said = Sha256::digest(manifest).to_vec();
        said.extend_from_slice(index);
        let signed = self
            .key
            .sign(self.certificate, ABOUT, &said)
            .ok_or(Refused::Unsigned)?;
        let permit_signed = match owner.filter(|one| one.root != signed.certificate.root) {
            None => None,
            Some(owner) => {
                let token = sealing.token.as_deref().ok_or(Refused::DoesNotAdd(
                    "what is another's asks for its edit token",
                ))?;
                let permit = permit_of(token)?;
                if owner.permit.as_deref() != Some(permit.shown().as_str()) {
                    return Err(Refused::DoesNotAdd(
                        "its token is not the one its owner allowed",
                    ));
                }
                let made = permit.sign(&signed.certificate.root, &signed.certificate.share, &said);
                Some(made.ok_or(Refused::Unsigned)?)
            }
        };
        let signature = Signature {
            root: signed.certificate.root,
            machine: signed.certificate.share,
            since: signed.certificate.since.to_string(),
            certificate: signed.certificate.sig,
            signed: signed.sig,
            permit_signed,
        };
        checked("signature", &json!(signature))?;
        serde_json::to_vec(&signature).map_err(|_| Refused::Unsealed)
    }
}

fn length_of(bytes: &[u8]) -> Result<[u8; 4], Refused> {
    u32::try_from(bytes.len())
        .map(u32::to_be_bytes)
        .map_err(|_| Refused::Unsealed)
}

impl EnvelopeWriter for Writer<'_> {
    fn seal(
        &mut self,
        document: Document,
        pictures: Vec<Vec<u8>>,
        files: Vec<Vec<u8>>,
        sealing: Sealing,
    ) -> Result<Vec<Vec<u8>>, String> {
        let sealed = self
            .sealed(document, pictures, files, &sealing)
            .map_err(|wrong| wrong.to_string())?;
        Ok(std::iter::once(sealed.main).chain(sealed.files).collect())
    }
}
