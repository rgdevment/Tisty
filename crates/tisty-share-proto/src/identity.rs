use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

const CERTIFIED: &[u8] = b"tisty-share.certificate.v1\0";
const REVOKED: &[u8] = b"tisty-share.revocation.v1\0";
const SIGNED: &[u8] = b"tisty-share.signed.v1\0";
const FINGERPRINT: &[u8] = b"tisty-share.person.v1\0";
const SUFFIX: &[u8] = b"tisty-share.suffix.v1\0";

/// Who a person is. It only vouches for the keys that sign, so it seldom has to leave its machine.
pub struct Root(SigningKey);

impl Root {
    pub fn from_bytes(seed: &[u8; 32]) -> Self {
        Self(SigningKey::from_bytes(seed))
    }

    pub fn to_bytes(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.0.to_bytes())
    }

    pub fn person(&self) -> Person {
        Person(self.0.verifying_key())
    }

    pub fn certify(&self, share: &VerifyingKey, since: Timestamp) -> Certificate {
        let since = whole_second(since);
        let said = between(CERTIFIED, &self.0.verifying_key(), share, since);
        Certificate {
            root: encoded(self.0.verifying_key().as_bytes()),
            share: encoded(share.as_bytes()),
            since,
            sig: encoded(&self.0.sign(&said).to_bytes()),
        }
    }

    pub fn revoke(&self, share: &VerifyingKey, at: Timestamp) -> Revocation {
        let at = whole_second(at);
        let said = between(REVOKED, &self.0.verifying_key(), share, at);
        Revocation {
            root: encoded(self.0.verifying_key().as_bytes()),
            share: encoded(share.as_bytes()),
            at,
            sig: encoded(&self.0.sign(&said).to_bytes()),
        }
    }
}

/// The key one machine signs with. Losing it costs a new certificate, never the person's name.
pub struct ShareKey(SigningKey);

impl ShareKey {
    pub fn from_bytes(seed: &[u8; 32]) -> Self {
        Self(SigningKey::from_bytes(seed))
    }

    pub fn to_bytes(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.0.to_bytes())
    }

    #[cfg(feature = "mint")]
    pub fn minted() -> Option<Self> {
        minted().map(|seed| Self::from_bytes(&seed))
    }

    pub fn public(&self) -> VerifyingKey {
        self.0.verifying_key()
    }

    /// None when the certificate vouches for some other key: a signature it could never answer for.
    pub fn sign(&self, certificate: &Certificate, about: &str, said: &[u8]) -> Option<Signed> {
        if decoded::<32>(&certificate.share)? != *self.0.verifying_key().as_bytes() {
            return None;
        }
        Some(Signed {
            certificate: certificate.clone(),
            sig: encoded(&self.0.sign(&message(about, said)).to_bytes()),
        })
    }
}

/// A person, known by the public half of their root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Person(VerifyingKey);

impl Person {
    pub fn read(said: &str) -> Option<Self> {
        key(said).map(Self)
    }

    pub fn shown(&self) -> String {
        encoded(self.0.as_bytes())
    }

    /// About 66 bits, in four groups of five digits that two people can read to each other.
    pub fn fingerprint(&self) -> String {
        let digest = hashed(FINGERPRINT, self.0.as_bytes());
        let mut wide = [0u8; 16];
        wide[7..].copy_from_slice(&digest[..9]);
        let digits = format!(
            "{:020}",
            u128::from_be_bytes(wide) % 100_000_000_000_000_000_000
        );
        digits
            .as_bytes()
            .chunks(5)
            .map(|five| std::str::from_utf8(five).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Four characters after an alias, enough to tell two «mario» apart; never enough to prove one.
    pub fn suffix(&self) -> String {
        const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
        let digest = hashed(SUFFIX, self.0.as_bytes());
        let bits = u32::from_be_bytes([0, digest[0], digest[1], digest[2]]) >> 4;
        (0..4)
            .rev()
            .map(|at| char::from(ALPHABET[((bits >> (at * 5)) & 31) as usize]))
            .collect()
    }
}

/// A root saying that a share key is the person's, from a given second on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    pub root: String,
    pub share: String,
    pub since: Timestamp,
    pub sig: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vouched {
    pub person: Person,
    pub share: VerifyingKey,
    pub since: Timestamp,
}

/// Unreadable is what a damaged or half-written record looks like; refused is a forged one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    Unreadable,
    Refused,
}

impl Certificate {
    pub fn vouches(&self) -> Result<Vouched, Fault> {
        let (root, share, sig) = parts(&self.root, &self.share, &self.sig)?;
        root.verify_strict(&between(CERTIFIED, &root, &share, self.since), &sig)
            .map_err(|_| Fault::Refused)?;
        Ok(Vouched {
            person: Person(root),
            share,
            since: self.since,
        })
    }
}

/// A root taking back a share key, for a machine lost or a key that leaked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revocation {
    pub root: String,
    pub share: String,
    pub at: Timestamp,
    pub sig: String,
}

impl Revocation {
    /// Only the root that vouched for a key can take it back.
    pub fn takes_back(&self, vouched: &Vouched) -> bool {
        let Ok((root, share, sig)) = parts(&self.root, &self.share, &self.sig) else {
            return false;
        };
        root == vouched.person.0
            && share == vouched.share
            && root
                .verify_strict(&between(REVOKED, &root, &share, self.at), &sig)
                .is_ok()
    }
}

/// What a share key signed, carried with the certificate that says whose key it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signed {
    pub certificate: Certificate,
    pub sig: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Holds {
        person: Person,
        since: Timestamp,
    },
    /// The signature answers, but the person has since taken that key back.
    Revoked {
        person: Person,
        at: Timestamp,
    },
    Refused,
    Unreadable,
}

/// `about` names what was signed, so a signature over one kind of thing never answers for another.
pub fn check(signed: &Signed, about: &str, said: &[u8], revoked: &[Revocation]) -> Verdict {
    let vouched = match signed.certificate.vouches() {
        Ok(vouched) => vouched,
        Err(Fault::Unreadable) => return Verdict::Unreadable,
        Err(Fault::Refused) => return Verdict::Refused,
    };
    let Some(sig) = signature(&signed.sig) else {
        return Verdict::Unreadable;
    };
    if vouched
        .share
        .verify_strict(&message(about, said), &sig)
        .is_err()
    {
        return Verdict::Refused;
    }
    match revoked.iter().find(|one| one.takes_back(&vouched)) {
        Some(one) => Verdict::Revoked {
            person: vouched.person,
            at: one.at,
        },
        None => Verdict::Holds {
            person: vouched.person,
            since: vouched.since,
        },
    }
}

/// The length goes first, or «ab» over «c» and «a» over «bc» would read as the same signature.
fn message(about: &str, said: &[u8]) -> Vec<u8> {
    let mut out = SIGNED.to_vec();
    out.extend_from_slice(&u64::try_from(about.len()).unwrap_or(u64::MAX).to_be_bytes());
    out.extend_from_slice(about.as_bytes());
    out.extend_from_slice(said);
    out
}

fn between(kind: &[u8], root: &VerifyingKey, share: &VerifyingKey, when: Timestamp) -> Vec<u8> {
    let mut out = kind.to_vec();
    out.extend_from_slice(root.as_bytes());
    out.extend_from_slice(share.as_bytes());
    out.extend_from_slice(&when.as_second().to_be_bytes());
    out
}

/// Only whole seconds are signed, so what is kept must not claim more than that.
fn whole_second(when: Timestamp) -> Timestamp {
    Timestamp::from_second(when.as_second()).unwrap_or(when)
}

fn parts(
    root: &str,
    share: &str,
    sig: &str,
) -> Result<(VerifyingKey, VerifyingKey, Signature), Fault> {
    match (key(root), key(share), signature(sig)) {
        (Some(root), Some(share), Some(sig)) => Ok((root, share, sig)),
        _ => Err(Fault::Unreadable),
    }
}

fn key(said: &str) -> Option<VerifyingKey> {
    VerifyingKey::from_bytes(&decoded::<32>(said)?).ok()
}

fn signature(said: &str) -> Option<Signature> {
    Signature::from_slice(&decoded::<64>(said)?).ok()
}

fn hashed(kind: &[u8], bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut over = Sha256::new();
    over.update(kind);
    over.update(bytes);
    over.finalize().into()
}

fn encoded(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn decoded<const N: usize>(said: &str) -> Option<[u8; N]> {
    <[u8; N]>::try_from(URL_SAFE_NO_PAD.decode(said).ok()?.as_slice()).ok()
}

#[cfg(feature = "mint")]
pub(crate) fn minted() -> Option<Zeroizing<[u8; 32]>> {
    let mut fresh = Zeroizing::new([0u8; 32]);
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, fresh.as_mut()).ok()?;
    Some(fresh)
}

#[cfg(test)]
#[path = "identity_test.rs"]
mod tests;
