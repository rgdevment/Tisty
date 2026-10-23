use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

use crate::identity::Root;

const SALT: &[u8] = b"tisty.seed.v1";
const ROOT: &[u8] = b"tisty.root.v1";
const CLOUD: &[u8] = b"tisty.cloud.v1";

/// Every key comes out under its own label, so a new use never needs a new phrase.
pub struct Seed(Zeroizing<[u8; 32]>);

impl Seed {
    pub fn from_bytes(seed: &[u8; 32]) -> Self {
        Self(Zeroizing::new(*seed))
    }

    pub fn to_bytes(&self) -> Zeroizing<[u8; 32]> {
        self.0.clone()
    }

    #[cfg(feature = "mint")]
    pub fn minted() -> Option<Self> {
        crate::minted().map(Self)
    }

    pub fn root(&self) -> Root {
        Root::from_bytes(&self.derived(ROOT))
    }

    /// The key for a person's own cloud, never sent anywhere.
    pub fn cloud(&self) -> Zeroizing<[u8; 32]> {
        self.derived(CLOUD)
    }

    /// HKDF-SHA256 for one block, written out because the `hkdf` crate cannot wipe what it holds.
    fn derived(&self, label: &[u8]) -> Zeroizing<[u8; 32]> {
        let extracted = mac(SALT, &[self.0.as_ref()]);
        mac(extracted.as_ref(), &[label, &[1]])
    }
}

fn mac(key: &[u8], parts: &[&[u8]]) -> Zeroizing<[u8; 32]> {
    let mut out = Zeroizing::new([0u8; 32]);
    // HMAC takes a key of any length, so this never refuses.
    if let Ok(mut over) = <Hmac<Sha256> as KeyInit>::new_from_slice(key) {
        parts.iter().for_each(|part| over.update(part));
        let mut said = over.finalize().into_bytes();
        out.copy_from_slice(&said);
        said.zeroize();
    }
    out
}

#[cfg(test)]
#[path = "seed_test.rs"]
mod tests;
