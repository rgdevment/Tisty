use std::io::{Read, Write};

use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{AeadInOut, KeyInit};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::seed::Seed;

/// A leading byte no text starts with, so a note that begins «TCS» is never taken for a sealed file.
const FILE: &[u8; 4] = b"\xffTCS";
const MARK: &[u8; 4] = b"\xffTCM";
const VERSION: u8 = 1;
pub const HEAD: usize = 19;
const BLOCK: usize = 64 * 1024;
const TAG: usize = 16;
const KEY_ID: usize = 16;
const MARK_SAID: &[u8] = b"this is the person's cloud";

#[derive(Clone)]
pub struct CloudKey(Zeroizing<[u8; 32]>);

impl CloudKey {
    pub fn of(seed: &Seed) -> Self {
        Self(seed.cloud())
    }

    pub fn from_bytes(key: &[u8; 32]) -> Self {
        Self(Zeroizing::new(*key))
    }

    pub fn to_bytes(&self) -> Zeroizing<[u8; 32]> {
        self.0.clone()
    }

    pub fn to_text(&self) -> Zeroizing<String> {
        use base64::Engine;
        Zeroizing::new(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(self.0.as_ref()))
    }

    pub fn from_text(said: &str) -> Option<Self> {
        use base64::Engine;
        let bytes = Zeroizing::new(
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(said)
                .ok()?,
        );
        let key = Zeroizing::new(<[u8; 32]>::try_from(bytes.as_slice()).ok()?);
        Some(Self::from_bytes(&key))
    }

    pub fn named_for(&self, cloud: &str) -> String {
        self.id(cloud)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new((&*self.0).into())
    }

    /// Tied to the cloud too, so the marks of two clouds never show they share a person.
    fn id(&self, cloud: &str) -> [u8; KEY_ID] {
        use sha2::{Digest, Sha256};
        let mut over = Sha256::new();
        over.update(cloud.nfc().collect::<String>().as_bytes());
        over.update(b"tisty.cloud.id.v1\0");
        over.update(self.0.as_ref());
        let digest: [u8; 32] = over.finalize().into();
        let mut id = [0u8; KEY_ID];
        id.copy_from_slice(&digest[..KEY_ID]);
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Unsealed {
    #[error("this was not sealed by Tisty")]
    Foreign,
    #[error("this was sealed by a newer Tisty")]
    Newer,
    #[error("this does not open with this key, or something changed it")]
    Refused,
    #[error("{0}")]
    Io(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Holds,
    OtherPhrase,
    /// The mark itself is damaged, which says nothing about the phrase.
    Damaged,
    Newer,
}

/// The block's place and whether it is the last live in the nonce, so no block can be dropped, cut or moved.
pub fn nonced(head: &[u8; HEAD], at: u32, last: bool) -> [u8; 24] {
    let mut nonce = [0u8; 24];
    nonce[..HEAD].copy_from_slice(head);
    nonce[HEAD..HEAD + 4].copy_from_slice(&at.to_be_bytes());
    nonce[23] = u8::from(last);
    nonce
}

/// Windows, macOS and providers spell one name several ways; the seal must see only one.
fn bound(kind: &str, cloud: &str, name: &str) -> Vec<u8> {
    let cloud: String = cloud.nfc().collect();
    let name: String = name
        .nfc()
        .map(|one| if one == '\\' { '/' } else { one })
        .collect();
    let name = name
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/");
    let mut said = b"tisty.cloud.v1\0".to_vec();
    // Each part says its length, so no two pairs of names ever read as the same bytes.
    for part in [kind, cloud.as_str(), name.as_str()] {
        said.extend_from_slice(&(part.len() as u64).to_be_bytes());
        said.extend_from_slice(part.as_bytes());
    }
    said
}

pub fn is_sealed(bytes: &[u8]) -> bool {
    bytes.starts_with(FILE)
}

fn read_full(from: &mut dyn Read, into: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < into.len() {
        match from.read(&mut into[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(filled)
}

#[cfg(feature = "mint")]
fn fresh_head() -> std::io::Result<[u8; HEAD]> {
    let minted =
        crate::minted().ok_or_else(|| std::io::Error::other("no randomness to seal with"))?;
    let mut head = [0u8; HEAD];
    head.copy_from_slice(&minted[..HEAD]);
    Ok(head)
}

fn buffer(len: usize) -> Zeroizing<Vec<u8>> {
    // Room for the tag from the start, so sealing in place never leaves a freed copy behind.
    let mut held = Zeroizing::new(Vec::with_capacity(BLOCK + TAG));
    held.resize(len, 0);
    held
}

/// `cloud` and `name` say where the bytes go, so a sealed file never passes for another nor travels.
#[cfg(feature = "mint")]
pub fn seal(
    key: &CloudKey,
    cloud: &str,
    name: &str,
    from: &mut dyn Read,
    into: &mut dyn Write,
) -> std::io::Result<()> {
    let refused = || std::io::Error::other("the cloud key could not seal");
    let cipher = key.cipher();
    let head = fresh_head()?;
    let aad = bound("file", cloud, name);
    into.write_all(FILE)?;
    into.write_all(&[VERSION])?;
    into.write_all(&head)?;

    let mut block = buffer(BLOCK);
    let mut filled = read_full(from, &mut block)?;
    let mut ahead = buffer(BLOCK);
    let mut at = 0u32;
    loop {
        let next = match filled {
            BLOCK => {
                ahead.resize(BLOCK, 0);
                read_full(from, &mut ahead)?
            }
            _ => 0,
        };
        let last = next == 0;
        block.truncate(filled);
        cipher
            .encrypt_in_place(&nonced(&head, at, last).into(), &aad, &mut *block)
            .map_err(|_| refused())?;
        into.write_all(&block)?;
        if last {
            return Ok(());
        }
        std::mem::swap(&mut block, &mut ahead);
        filled = next;
        at = at.checked_add(1).ok_or_else(refused)?;
    }
}

/// What it writes before a refusal is not to be kept: only a whole stream answers for itself.
pub fn open(
    key: &CloudKey,
    cloud: &str,
    name: &str,
    from: &mut dyn Read,
    into: &mut dyn Write,
) -> Result<(), Unsealed> {
    let io = |e: std::io::Error| Unsealed::Io(e.to_string());
    let mut magic = [0u8; 5];
    let read = read_full(from, &mut magic).map_err(io)?;
    // A header cut short is a sealed file torn on the way, never plain text to pass through.
    if read < magic.len() {
        let torn = read > 0 && FILE.starts_with(&magic[..read.min(FILE.len())]);
        return Err(if torn {
            Unsealed::Refused
        } else {
            Unsealed::Foreign
        });
    }
    if !magic.starts_with(FILE) {
        return Err(Unsealed::Foreign);
    }
    match magic[4] {
        VERSION => {}
        newer if newer > VERSION => return Err(Unsealed::Newer),
        _ => return Err(Unsealed::Refused),
    }
    let mut head = [0u8; HEAD];
    if read_full(from, &mut head).map_err(io)? < HEAD {
        return Err(Unsealed::Refused);
    }
    let cipher = key.cipher();
    let aad = bound("file", cloud, name);
    let mut block = buffer(BLOCK + TAG);
    let mut filled = read_full(from, &mut block).map_err(io)?;
    if filled < TAG {
        return Err(Unsealed::Refused);
    }
    let mut ahead = buffer(BLOCK + TAG);
    let mut at = 0u32;
    loop {
        let next = match filled {
            full if full == BLOCK + TAG => {
                ahead.resize(BLOCK + TAG, 0);
                read_full(from, &mut ahead).map_err(io)?
            }
            _ => 0,
        };
        let last = next == 0;
        block.truncate(filled);
        cipher
            .decrypt_in_place(&nonced(&head, at, last).into(), &aad, &mut *block)
            .map_err(|_| Unsealed::Refused)?;
        into.write_all(&block).map_err(io)?;
        if last {
            return Ok(());
        }
        std::mem::swap(&mut block, &mut ahead);
        filled = next;
        at = at.checked_add(1).ok_or(Unsealed::Refused)?;
    }
}

#[cfg(feature = "mint")]
pub fn sealed(key: &CloudKey, cloud: &str, name: &str, plain: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    seal(key, cloud, name, &mut &plain[..], &mut out).ok()?;
    Some(out)
}

pub fn opened(
    key: &CloudKey,
    cloud: &str,
    name: &str,
    sealed: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Unsealed> {
    // Sized once, so growing never leaves a freed copy of what was opened behind.
    let mut out = Zeroizing::new(Vec::with_capacity(sealed.len()));
    open(key, cloud, name, &mut &sealed[..], &mut *out)?;
    Ok(out)
}

/// Kept beside a sealed cloud, so a phrase is told apart before anything comes down.
#[cfg(feature = "mint")]
pub fn mark(key: &CloudKey, cloud: &str) -> Option<Vec<u8>> {
    let head = fresh_head().ok()?;
    let mut said = MARK_SAID.to_vec();
    key.cipher()
        .encrypt_in_place(
            &nonced(&head, 0, true).into(),
            &bound("mark", cloud, ""),
            &mut said,
        )
        .ok()?;
    let mut out = MARK.to_vec();
    out.push(VERSION);
    out.extend_from_slice(&key.id(cloud));
    out.extend_from_slice(&head);
    out.extend_from_slice(&said);
    Some(out)
}

/// The seal is tried before the key's id, so a damaged id never makes the right phrase look wrong.
pub fn mark_says(key: &CloudKey, cloud: &str, mark: &[u8]) -> Mark {
    let Some(rest) = mark.strip_prefix(MARK) else {
        return Mark::Damaged;
    };
    let Some((&version, rest)) = rest.split_first() else {
        return Mark::Damaged;
    };
    match version {
        VERSION => {}
        newer if newer > VERSION => return Mark::Newer,
        _ => return Mark::Damaged,
    }
    let Some((id, rest)) = rest.split_first_chunk::<KEY_ID>() else {
        return Mark::Damaged;
    };
    let Some((head, sealed)) = rest.split_first_chunk::<HEAD>() else {
        return Mark::Damaged;
    };
    let mut said = Zeroizing::new(sealed.to_vec());
    let held = key.cipher().decrypt_in_place(
        &nonced(head, 0, true).into(),
        &bound("mark", cloud, ""),
        &mut *said,
    );
    match held {
        Ok(()) if said.as_slice() == MARK_SAID => Mark::Holds,
        _ if *id != key.id(cloud) => Mark::OtherPhrase,
        _ => Mark::Damaged,
    }
}

#[cfg(all(test, feature = "mint"))]
#[path = "cloud_test.rs"]
mod tests;
