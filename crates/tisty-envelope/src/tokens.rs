//! A language's edit token rides in its anchor's map, sealed so that only the anchor's edit link opens it.

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hkdf::Hkdf;
use rand_core::{OsRng, TryRngCore};
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::sealing::Refused;

const INFO: &[u8] = b"tisty-language-token";
const NONCE_BYTES: usize = 12;

fn token_bytes(token: &str) -> Option<Zeroizing<[u8; 32]>> {
    let bytes = Zeroizing::new(URL_SAFE_NO_PAD.decode(token).ok()?);
    let mut out = Zeroizing::new([0u8; 32]);
    out.copy_from_slice(bytes.get(..32).filter(|_| bytes.len() == 32)?);
    Some(out)
}

fn cipher(anchor: &str, language: &str) -> Option<Aes256Gcm> {
    let anchor = token_bytes(anchor)?;
    let mut key = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(language.as_bytes()), anchor.as_ref())
        .expand(INFO, key.as_mut())
        .ok()?;
    Aes256Gcm::new_from_slice(key.as_ref()).ok()
}

/// The language's token sealed under the anchor's, as the map's `edit` carries it.
pub fn sealed_for(anchor: &str, language: &str, token: &str) -> Result<String, Refused> {
    let unsealable = || Refused::DoesNotAdd("a token is not 32 bytes in base64url");
    let cipher = cipher(anchor, language).ok_or_else(unsealable)?;
    let token = token_bytes(token).ok_or_else(unsealable)?;
    let mut nonce = [0u8; NONCE_BYTES];
    OsRng
        .try_fill_bytes(&mut nonce)
        .map_err(|_| Refused::NoRandomness)?;
    let sealed = cipher
        .encrypt(
            &Nonce::from(nonce),
            Payload {
                msg: token.as_ref(),
                aad: language.as_bytes(),
            },
        )
        .map_err(|_| Refused::Unsealed)?;
    Ok(URL_SAFE_NO_PAD.encode([nonce.as_slice(), &sealed].concat()))
}

/// The language's token, when the anchor's token opens what the map carries for it.
pub fn opened_with(anchor: &str, language: &str, sealed: &str) -> Option<String> {
    let bytes = URL_SAFE_NO_PAD.decode(sealed).ok()?;
    let nonce: [u8; NONCE_BYTES] = bytes.get(..NONCE_BYTES)?.try_into().ok()?;
    let sealed = bytes.get(NONCE_BYTES..)?;
    let token = Zeroizing::new(
        cipher(anchor, language)?
            .decrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: sealed,
                    aad: language.as_bytes(),
                },
            )
            .ok()?,
    );
    (token.len() == 32).then(|| URL_SAFE_NO_PAD.encode(token.as_slice()))
}

#[cfg(test)]
#[path = "tokens_test.rs"]
mod tests;
