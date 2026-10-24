use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

use crate::seed::Seed;

const SEALED: &[u8; 4] = b"TCS1";
const HEAD: usize = 19;
const BLOCK: usize = 64 * 1024;
const TAG: usize = 16;
const MARK: &str = "tisty.cloud.mark";
const MARK_SAID: &[u8] = b"this is the person's cloud";

pub struct CloudKey(Zeroizing<[u8; 32]>);

impl CloudKey {
    pub fn of(seed: &Seed) -> Self {
        Self(seed.cloud())
    }

    fn cipher(&self) -> Option<XChaCha20Poly1305> {
        XChaCha20Poly1305::new_from_slice(self.0.as_ref()).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unsealed {
    #[error("this was not sealed by Tisty")]
    Foreign,
    #[error("this does not open with this key, or something changed it")]
    Refused,
}

pub fn is_sealed(bytes: &[u8]) -> bool {
    bytes.starts_with(SEALED)
}

/// Each block's nonce says its place and whether it is the last, so no block can be dropped, cut or moved.
fn nonce(head: &[u8; HEAD], at: u32, last: bool) -> XNonce {
    let mut nonce = [0u8; 24];
    nonce[..HEAD].copy_from_slice(head);
    nonce[HEAD..HEAD + 4].copy_from_slice(&at.to_be_bytes());
    nonce[23] = u8::from(last);
    nonce.into()
}

/// `about` is the name the bytes go under, so a sealed file never passes for another.
#[cfg(feature = "mint")]
pub fn sealed(key: &CloudKey, about: &str, plain: &[u8]) -> Option<Vec<u8>> {
    let mut head = [0u8; HEAD];
    head.copy_from_slice(&crate::minted()?[..HEAD]);
    let cipher = key.cipher()?;
    let blocks: Vec<&[u8]> = match plain.is_empty() {
        true => vec![&[]],
        false => plain.chunks(BLOCK).collect(),
    };
    let mut out = Vec::with_capacity(SEALED.len() + HEAD + plain.len() + blocks.len() * TAG);
    out.extend_from_slice(SEALED);
    out.extend_from_slice(&head);
    for (at, block) in blocks.iter().enumerate() {
        let last = at + 1 == blocks.len();
        let nonce = nonce(&head, u32::try_from(at).ok()?, last);
        let said = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: block,
                    aad: about.as_bytes(),
                },
            )
            .ok()?;
        out.extend_from_slice(&said);
    }
    Some(out)
}

pub fn opened(key: &CloudKey, about: &str, sealed: &[u8]) -> Result<Zeroizing<Vec<u8>>, Unsealed> {
    let rest = sealed.strip_prefix(SEALED).ok_or(Unsealed::Foreign)?;
    let (head, rest) = rest.split_at_checked(HEAD).ok_or(Unsealed::Refused)?;
    let head: [u8; HEAD] = head.try_into().map_err(|_| Unsealed::Refused)?;
    if rest.is_empty() {
        return Err(Unsealed::Refused);
    }
    let cipher = key.cipher().ok_or(Unsealed::Refused)?;
    let blocks: Vec<&[u8]> = rest.chunks(BLOCK + TAG).collect();
    let mut plain = Zeroizing::new(Vec::with_capacity(rest.len()));
    for (at, block) in blocks.iter().enumerate() {
        let last = at + 1 == blocks.len();
        let at = u32::try_from(at).map_err(|_| Unsealed::Refused)?;
        let said = cipher
            .decrypt(
                &nonce(&head, at, last),
                Payload {
                    msg: block,
                    aad: about.as_bytes(),
                },
            )
            .map_err(|_| Unsealed::Refused)?;
        plain.extend_from_slice(&said);
    }
    Ok(plain)
}

/// Kept beside a cloud that is sealed, so a phrase is told apart before anything comes down.
#[cfg(feature = "mint")]
pub fn mark(key: &CloudKey) -> Option<Vec<u8>> {
    sealed(key, MARK, MARK_SAID)
}

pub fn mark_holds(key: &CloudKey, mark: &[u8]) -> bool {
    opened(key, MARK, mark).is_ok_and(|said| said.as_slice() == MARK_SAID)
}

#[cfg(all(test, feature = "mint"))]
#[path = "cloud_test.rs"]
mod tests;
