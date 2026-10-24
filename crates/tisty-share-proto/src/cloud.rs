use std::io::{Read, Write};

use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{AeadInOut, KeyInit};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::seed::Seed;

const MAGIC: &[u8; 3] = b"TCS";
const VERSION: u8 = 1;
pub const HEAD: usize = 19;
const BLOCK: usize = 64 * 1024;
const TAG: usize = 16;
const KEY_ID: usize = 16;
const MARK_SAID: &[u8] = b"this is the person's cloud";

pub struct CloudKey(Zeroizing<[u8; 32]>);

impl CloudKey {
    pub fn of(seed: &Seed) -> Self {
        Self(seed.cloud())
    }

    fn cipher(&self) -> Option<XChaCha20Poly1305> {
        XChaCha20Poly1305::new_from_slice(self.0.as_ref()).ok()
    }

    /// Says which key sealed a mark without saying anything of the key itself.
    fn id(&self) -> [u8; KEY_ID] {
        use sha2::{Digest, Sha256};
        let mut over = Sha256::new();
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
}

/// The block's place and whether it is the last live in the nonce, so no block can be dropped, cut or moved.
pub fn nonced(head: &[u8; HEAD], at: u32, last: bool) -> [u8; 24] {
    let mut nonce = [0u8; 24];
    nonce[..HEAD].copy_from_slice(head);
    nonce[HEAD..HEAD + 4].copy_from_slice(&at.to_be_bytes());
    nonce[23] = u8::from(last);
    nonce
}

/// Names reach here spelled by Windows, macOS or a provider, so one name is one spelling.
fn bound(kind: &str, cloud: &str, name: &str) -> Vec<u8> {
    let name: String = name.nfc().collect();
    let mut said = b"tisty.cloud.v1\0".to_vec();
    for part in [kind, cloud, name.as_str()] {
        said.extend_from_slice(part.as_bytes());
        said.push(0);
    }
    said
}

pub fn is_sealed(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
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
    let cipher = key.cipher().ok_or_else(refused)?;
    let head = fresh_head()?;
    let aad = bound("file", cloud, name);
    into.write_all(MAGIC)?;
    into.write_all(&[VERSION])?;
    into.write_all(&head)?;

    let mut block = Zeroizing::new(Vec::with_capacity(BLOCK + TAG));
    block.resize(BLOCK, 0);
    let mut filled = read_full(from, &mut block)?;
    let mut ahead = Zeroizing::new(vec![0u8; BLOCK]);
    let mut at = 0u32;
    loop {
        let next = match filled {
            BLOCK => read_full(from, &mut ahead)?,
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
        block.clear();
        block.extend_from_slice(&ahead[..next]);
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
    let mut magic = [0u8; 4];
    if read_full(from, &mut magic).map_err(io)? < magic.len() || !magic.starts_with(MAGIC) {
        return Err(Unsealed::Foreign);
    }
    match magic[3] {
        VERSION => {}
        newer if newer > VERSION => return Err(Unsealed::Newer),
        _ => return Err(Unsealed::Foreign),
    }
    let mut head = [0u8; HEAD];
    if read_full(from, &mut head).map_err(io)? < HEAD {
        return Err(Unsealed::Refused);
    }
    let cipher = key.cipher().ok_or(Unsealed::Refused)?;
    let aad = bound("file", cloud, name);
    let mut block = Zeroizing::new(vec![0u8; BLOCK + TAG]);
    let mut filled = read_full(from, &mut block).map_err(io)?;
    if filled < TAG {
        return Err(Unsealed::Refused);
    }
    let mut ahead = Zeroizing::new(vec![0u8; BLOCK + TAG]);
    let mut at = 0u32;
    loop {
        let next = match filled {
            full if full == BLOCK + TAG => read_full(from, &mut ahead).map_err(io)?,
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
        block.clear();
        block.extend_from_slice(&ahead[..next]);
        block.resize(BLOCK + TAG, 0);
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
    let cipher = key.cipher()?;
    let head = fresh_head().ok()?;
    let mut said = MARK_SAID.to_vec();
    cipher
        .encrypt_in_place(
            &nonced(&head, 0, true).into(),
            &bound("mark", cloud, ""),
            &mut said,
        )
        .ok()?;
    let mut out = key.id().to_vec();
    out.extend_from_slice(&head);
    out.extend_from_slice(&said);
    Some(out)
}

pub fn mark_says(key: &CloudKey, cloud: &str, mark: &[u8]) -> Mark {
    let Some((id, rest)) = mark.split_first_chunk::<KEY_ID>() else {
        return Mark::Damaged;
    };
    if *id != key.id() {
        return Mark::OtherPhrase;
    }
    let (Some((head, sealed)), Some(cipher)) = (rest.split_first_chunk::<HEAD>(), key.cipher())
    else {
        return Mark::Damaged;
    };
    let mut said = Zeroizing::new(sealed.to_vec());
    let held = cipher.decrypt_in_place(
        &nonced(head, 0, true).into(),
        &bound("mark", cloud, ""),
        &mut *said,
    );
    match held {
        Ok(()) if said.as_slice() == MARK_SAID => Mark::Holds,
        _ => Mark::Damaged,
    }
}

#[cfg(all(test, feature = "mint"))]
#[path = "cloud_test.rs"]
mod tests;
