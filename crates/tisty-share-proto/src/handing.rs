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
use crate::identity::{Certificate, Revocation, ShareKey, Signed, Verdict};

pub const SAID: u32 = 1;
const ASKED: &[u8] = b"tisty.ask.v1\0";
const HANDED: &[u8] = b"tisty.handed.v1\0";
const SALT: [u8; 15] = *b"tisty.handed.v1";
const CODE: &[u8] = b"tisty.ask.code.v1\0";
const COMMIT: &[u8] = b"tisty.ask.commit.v1\0";
// As slow as a locked parcel, so a provider cannot mint, while an ask stands, another key that reads the same digits.
const WORK: u8 = 16;

// The reveal stays here until a machine has taken the ask, so whoever put that machine up could not pick it knowing the code.
pub struct OneUse {
    secret: Zeroizing<[u8; 32]>,
    reveal: Zeroizing<[u8; 32]>,
}

impl OneUse {
    #[cfg(feature = "mint")]
    pub fn minted() -> Option<Self> {
        Some(Self {
            secret: crate::minted()?,
            reveal: crate::minted()?,
        })
    }

    pub fn public(&self) -> [u8; 32] {
        MontgomeryPoint::mul_base_clamped(*self.secret).to_bytes()
    }

    pub fn revealed(&self) -> String {
        coded(self.reveal.as_ref())
    }

    fn commit(&self) -> String {
        coded(&committed(self.reveal.as_ref()))
    }
}

fn committed(reveal: &[u8]) -> [u8; 32] {
    let mut over = Sha256::new();
    over.update(COMMIT);
    over.update(reveal);
    over.finalize().into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ask {
    pub v: u32,
    pub device: String,
    pub name: String,
    pub key: String,
    pub one_use: String,
    pub commit: String,
    pub cloud: String,
    pub at: i64,
    pub sig: String,
}

/// The machine that takes an ask from a machine no log names, put up before it answers so both can show one code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Giver {
    pub v: u32,
    pub by: String,
    pub name: String,
    pub key: String,
}

impl Giver {
    pub fn of(by: &str, name: &str, key: &VerifyingKey) -> Self {
        Self {
            v: SAID,
            by: by.to_string(),
            name: name.to_string(),
            key: coded(key.as_bytes()),
        }
    }

    pub fn key(&self) -> Option<VerifyingKey> {
        key_of(&self.key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Handed {
    pub v: u32,
    pub by: String,
    pub key: String,
    pub from: String,
    pub sealed: String,
    pub sig: String,
    /// The same answer signed with a share key the person's root certified, for an asker whose log never met this giver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vouched: Option<Signed>,
}

const VOUCHED: &str = "tisty.handed.vouched";

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
            commit: one_use.commit(),
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
                self.commit.as_bytes(),
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

    /// About 66 bits of the ask, the key of whoever took it and the reveal the ask committed to, so one comparison proves both ways.
    pub fn code(&self, giver: &VerifyingKey, reveal: &str) -> Option<String> {
        let reveal = decoded::<32>(reveal)?;
        if coded(&committed(&reveal)) != self.commit {
            return None;
        }
        let how = scrypt::Params::new(WORK, 8, 1).ok()?;
        let mut ground = Zeroizing::new([0u8; 16]);
        let salt = framed(CODE, &[self.cloud.as_bytes()]);
        let said = framed(CODE, &[&self.digest(), giver.as_bytes(), &reveal]);
        scrypt::scrypt(&said, &salt, &how, &mut ground[7..]).ok()?;
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

    pub fn vouch(&mut self, ask: &Ask, share: &ShareKey, certificate: &Certificate) -> bool {
        let said = self.said(&ask.digest());
        self.vouched = share.sign(certificate, VOUCHED, &said);
        self.vouched.is_some()
    }

    pub fn vouched_for(&self, ask: &Ask, revoked: &[Revocation]) -> Verdict {
        match &self.vouched {
            Some(signed) => {
                crate::identity::check(signed, VOUCHED, &self.said(&ask.digest()), revoked)
            }
            None => Verdict::Refused,
        }
    }

    pub fn giver(&self) -> Option<VerifyingKey> {
        key_of(&self.key)
    }

    pub fn opened(&self, ask: &Ask, one_use: &OneUse) -> Option<CloudKey> {
        let digest = ask.digest();
        let (giver, sig) = (self.giver()?, signature(&self.sig)?);
        if self.v != SAID || giver.verify(&self.said(&digest), &sig).is_err() {
            return None;
        }
        let from = decoded::<32>(&self.from)?;
        let point = shared(&one_use.secret, from)?;
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
    let point = shared(&mine.secret, asked)?;
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
        vouched: None,
    };
    handed.sig = coded(&signer.sign(&handed.said(&digest)).to_bytes());
    Some(handed)
}

#[cfg(test)]
#[path = "handing_test.rs"]
mod tests;
