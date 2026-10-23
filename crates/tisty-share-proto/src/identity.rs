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
        let (root, share, since, sig) = attest(&self.0, CERTIFIED, share, since);
        Certificate {
            root,
            share,
            since,
            sig,
        }
    }

    pub fn revoke(&self, share: &VerifyingKey, at: Timestamp) -> Revocation {
        let (root, share, at, sig) = attest(&self.0, REVOKED, share, at);
        Revocation {
            root,
            share,
            at,
            sig,
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
        crate::minted().map(|seed| Self::from_bytes(&seed))
    }

    pub fn public(&self) -> VerifyingKey {
        self.0.verifying_key()
    }

    /// None unless the certificate holds and vouches for this very key: a signature nobody could check.
    pub fn sign(&self, certificate: &Certificate, about: &str, said: &[u8]) -> Option<Signed> {
        let vouched = certificate.vouches().ok()?;
        if vouched.share != self.public() {
            return None;
        }
        Some(Signed {
            certificate: certificate.clone(),
            sig: encoded(&self.0.sign(&message(about, &vouched, said)).to_bytes()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Person(VerifyingKey);

impl Person {
    pub fn read(said: &str) -> Option<Self> {
        key(said).map(Self)
    }

    pub fn shown(&self) -> String {
        encoded(self.0.as_bytes())
    }

    /// 66 bits, in four groups of five digits that two people can read to each other.
    pub fn fingerprint(&self) -> String {
        let digest = hashed(FINGERPRINT, self.0.as_bytes());
        let mut wide = [0u8; 16];
        wide[7..].copy_from_slice(&digest[..9]);
        // 2^66 stays under 10^20, so every code is as likely as any other.
        let digits = format!("{:020}", u128::from_be_bytes(wide) >> 6);
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
        let (root, share) = attested(CERTIFIED, &self.root, &self.share, self.since, &self.sig)?;
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
    /// Only the root that vouched can take a key back, and for good: a machine found again gets a new key.
    pub fn takes_back(&self, vouched: &Vouched) -> bool {
        attested(REVOKED, &self.root, &self.share, self.at, &self.sig)
            .is_ok_and(|(root, share)| root == vouched.person.0 && share == vouched.share)
    }
}

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
    /// The key was taken back; nothing signed says when, so earlier work and a thief's look alike.
    Revoked {
        person: Person,
        at: Timestamp,
    },
    Refused,
    Unreadable,
}

impl From<Fault> for Verdict {
    fn from(fault: Fault) -> Self {
        match fault {
            Fault::Unreadable => Verdict::Unreadable,
            Fault::Refused => Verdict::Refused,
        }
    }
}

/// `about` names what was signed, so a signature over one kind of thing never answers for another.
pub fn check(signed: &Signed, about: &str, said: &[u8], revoked: &[Revocation]) -> Verdict {
    let vouched = match signed.certificate.vouches() {
        Ok(vouched) => vouched,
        Err(fault) => return fault.into(),
    };
    let Some(sig) = signature(&signed.sig) else {
        return Verdict::Unreadable;
    };
    if vouched
        .share
        .verify_strict(&message(about, &vouched, said), &sig)
        .is_err()
    {
        return Verdict::Refused;
    }
    // The earliest, so every machine reads the same date whatever order the list came in.
    let taken_back = revoked
        .iter()
        .filter(|one| one.takes_back(&vouched))
        .map(|one| one.at)
        .min();
    match taken_back {
        Some(at) => Verdict::Revoked {
            person: vouched.person,
            at,
        },
        None => Verdict::Holds {
            person: vouched.person,
            since: vouched.since,
        },
    }
}

/// The root is signed too, or another root could certify this key and claim what it signed.
fn message(about: &str, vouched: &Vouched, said: &[u8]) -> Vec<u8> {
    let mut out = SIGNED.to_vec();
    out.extend_from_slice(vouched.person.0.as_bytes());
    out.extend_from_slice(vouched.share.as_bytes());
    out.extend_from_slice(&(about.len() as u64).to_be_bytes());
    out.extend_from_slice(about.as_bytes());
    out.extend_from_slice(said);
    out
}

/// What a certificate and a revocation share: a root, a share key and a second, signed by the root.
fn attest(
    root: &SigningKey,
    kind: &[u8],
    share: &VerifyingKey,
    when: Timestamp,
) -> (String, String, Timestamp, String) {
    let when = Timestamp::from_second(when.as_second()).unwrap_or(when);
    let said = between(kind, &root.verifying_key(), share, when);
    (
        encoded(root.verifying_key().as_bytes()),
        encoded(share.as_bytes()),
        when,
        encoded(&root.sign(&said).to_bytes()),
    )
}

/// Only whole seconds are signed, so a fraction added later is a change nobody signed.
fn attested(
    kind: &[u8],
    root: &str,
    share: &str,
    when: Timestamp,
    sig: &str,
) -> Result<(VerifyingKey, VerifyingKey), Fault> {
    let (Some(root), Some(share), Some(sig)) = (key(root), key(share), signature(sig)) else {
        return Err(Fault::Unreadable);
    };
    if when.subsec_nanosecond() != 0
        || root
            .verify_strict(&between(kind, &root, &share, when), &sig)
            .is_err()
    {
        return Err(Fault::Refused);
    }
    Ok((root, share))
}

fn between(kind: &[u8], root: &VerifyingKey, share: &VerifyingKey, when: Timestamp) -> Vec<u8> {
    let mut out = kind.to_vec();
    out.extend_from_slice(root.as_bytes());
    out.extend_from_slice(share.as_bytes());
    out.extend_from_slice(&when.as_second().to_be_bytes());
    out
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

#[cfg(test)]
#[path = "identity_test.rs"]
mod tests;
