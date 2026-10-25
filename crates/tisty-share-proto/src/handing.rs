use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{AeadInOut, KeyInit};
use curve25519_dalek::montgomery::MontgomeryPoint;
pub use ed25519_dalek::SigningKey;
use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::cloud::CloudKey;

pub const SAID: u32 = 1;
const ASKED: &[u8] = b"tisty.ask.v1\0";
const HANDED: &[u8] = b"tisty.handed.v1\0";
const SALT: [u8; 15] = *b"tisty.handed.v1";
const CODE: &[u8] = b"tisty.ask.code.v1\0";
/// As slow as a locked parcel, so a provider cannot mint another key that reads the same digits while the ask stands.
const WORK: u8 = 16;

/// Kept only by the machine that asks, and dropped once a key came or the ask ran out.
pub struct OneUse(Zeroizing<[u8; 32]>);

impl OneUse {
    pub fn from_bytes(secret: &[u8; 32]) -> Self {
        Self(Zeroizing::new(*secret))
    }

    pub fn to_bytes(&self) -> Zeroizing<[u8; 32]> {
        self.0.clone()
    }

    #[cfg(feature = "mint")]
    pub fn minted() -> Option<Self> {
        crate::minted().map(Self)
    }

    pub fn public(&self) -> [u8; 32] {
        MontgomeryPoint::mul_base_clamped(*self.0).to_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ask {
    pub v: u32,
    pub device: String,
    pub name: String,
    pub key: String,
    pub one_use: String,
    pub cloud: String,
    pub at: i64,
    pub sig: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Handed {
    pub v: u32,
    pub by: String,
    pub key: String,
    pub from: String,
    pub sealed: String,
    pub sig: String,
}

fn coded(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn decoded<const N: usize>(said: &str) -> Option<[u8; N]> {
    <[u8; N]>::try_from(URL_SAFE_NO_PAD.decode(said).ok()?.as_slice()).ok()
}

fn signature(said: &str) -> Option<Signature> {
    decoded::<64>(said).map(|bytes| Signature::from_bytes(&bytes))
}

fn key_of(said: &str) -> Option<VerifyingKey> {
    VerifyingKey::from_bytes(&decoded::<32>(said)?).ok()
}

/// Each part goes with its length, so no two asks ever read as the same bytes.
fn framed(label: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut said = label.to_vec();
    for part in parts {
        said.extend_from_slice(&(part.len() as u64).to_be_bytes());
        said.extend_from_slice(part);
    }
    said
}

impl Ask {
    pub fn signed(
        device: &str,
        name: &str,
        signer: &SigningKey,
        one_use: &OneUse,
        cloud: &str,
        at: i64,
    ) -> Self {
        let mut ask = Self {
            v: SAID,
            device: device.to_string(),
            name: name.to_string(),
            key: coded(signer.verifying_key().as_bytes()),
            one_use: coded(&one_use.public()),
            cloud: cloud.to_string(),
            at,
            sig: String::new(),
        };
        ask.sig = coded(&signer.sign(&ask.said()).to_bytes());
        ask
    }

    fn said(&self) -> Vec<u8> {
        framed(
            ASKED,
            &[
                &self.v.to_be_bytes(),
                self.device.as_bytes(),
                self.name.as_bytes(),
                self.key.as_bytes(),
                self.one_use.as_bytes(),
                self.cloud.as_bytes(),
                &self.at.to_be_bytes(),
            ],
        )
    }

    pub fn key(&self) -> Option<VerifyingKey> {
        key_of(&self.key)
    }

    /// Signed by the key it carries; whether that key is the machine's own is for the log to say.
    pub fn holds(&self) -> bool {
        match (self.key(), signature(&self.sig)) {
            (Some(key), Some(sig)) => self.v == SAID && key.verify(&self.said(), &sig).is_ok(),
            _ => false,
        }
    }

    fn digest(&self) -> [u8; 32] {
        let mut over = Sha256::new();
        over.update(self.said());
        over.update(self.sig.as_bytes());
        over.finalize().into()
    }

    /// About 66 bits of both keys the ask carries, shown on the two machines for the person to compare.
    pub fn code(&self) -> Option<String> {
        let (key, one_use) = (decoded::<32>(&self.key)?, decoded::<32>(&self.one_use)?);
        let how = scrypt::Params::new(WORK, 8, 1).ok()?;
        let mut ground = Zeroizing::new([0u8; 16]);
        let salt = framed(CODE, &[self.cloud.as_bytes()]);
        scrypt::scrypt(&[key, one_use].concat(), &salt, &how, &mut ground[7..]).ok()?;
        let digits = format!(
            "{:020}",
            u128::from_be_bytes(*ground) % 100_000_000_000_000_000_000
        );
        Some(
            digits
                .as_bytes()
                .chunks(5)
                .filter_map(|five| std::str::from_utf8(five).ok())
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

/// A shared point of all zeros means the other side gave a key of low order, and nothing secret came of it.
fn shared(secret: &[u8; 32], public: [u8; 32]) -> Option<Zeroizing<[u8; 32]>> {
    let point = Zeroizing::new(MontgomeryPoint(public).mul_clamped(*secret).to_bytes());
    (*point != [0u8; 32]).then_some(point)
}

fn cipher(shared: &[u8; 32], ask: &[u8; 32], from: &[u8; 32]) -> XChaCha20Poly1305 {
    let extracted = crate::seed::mac(&SALT, &[shared]);
    let key = crate::seed::mac(&extracted, &[HANDED, ask, from, &[1]]);
    XChaCha20Poly1305::new((&*key).into())
}

/// Each answer is sealed under a key made for it alone, so the nonce never has to change.
const NONCE: [u8; 24] = [0; 24];

impl Handed {
    fn said(&self, ask: &[u8; 32]) -> Vec<u8> {
        framed(
            HANDED,
            &[
                &self.v.to_be_bytes(),
                ask,
                self.by.as_bytes(),
                self.key.as_bytes(),
                self.from.as_bytes(),
                self.sealed.as_bytes(),
            ],
        )
    }

    pub fn giver(&self) -> Option<VerifyingKey> {
        key_of(&self.key)
    }

    /// The key it brings, once its signature and its seal both hold for this very ask.
    pub fn opened(&self, ask: &Ask, one_use: &OneUse) -> Option<CloudKey> {
        let digest = ask.digest();
        let (giver, sig) = (self.giver()?, signature(&self.sig)?);
        if self.v != SAID || giver.verify(&self.said(&digest), &sig).is_err() {
            return None;
        }
        let from = decoded::<32>(&self.from)?;
        let point = shared(&one_use.0, from)?;
        let mut key = Zeroizing::new(URL_SAFE_NO_PAD.decode(&self.sealed).ok()?);
        cipher(&point, &digest, &from)
            .decrypt_in_place(&NONCE.into(), &digest, &mut *key)
            .ok()?;
        let bytes = <[u8; 32]>::try_from(key.as_slice()).ok()?;
        Some(CloudKey::from_bytes(&bytes))
    }
}

#[cfg(feature = "mint")]
pub fn handed(ask: &Ask, key: &CloudKey, by: &str, signer: &SigningKey) -> Option<Handed> {
    let asked = decoded::<32>(&ask.one_use)?;
    let mine = OneUse::minted()?;
    let from = mine.public();
    let point = shared(&mine.0, asked)?;
    let digest = ask.digest();
    let mut sealed = Zeroizing::new(Vec::with_capacity(48));
    sealed.extend_from_slice(key.to_bytes().as_ref());
    cipher(&point, &digest, &from)
        .encrypt_in_place(&NONCE.into(), &digest, &mut *sealed)
        .ok()?;
    let mut handed = Handed {
        v: SAID,
        by: by.to_string(),
        key: coded(signer.verifying_key().as_bytes()),
        from: coded(&from),
        sealed: coded(&sealed),
        sig: String::new(),
    };
    handed.sig = coded(&signer.sign(&handed.said(&digest)).to_bytes());
    Some(handed)
}

#[cfg(test)]
#[path = "handing_test.rs"]
mod tests;
