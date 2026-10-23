use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::identity::Root;

const SALT: &[u8] = b"tisty.seed.v1";
const ROOT: &[u8] = b"tisty.root.v1";
const CLOUD: &[u8] = b"tisty.cloud.v1";

/// What the 24 words hold. Every key a person has comes out of it under its own label, so a new
/// use never needs a new phrase and one use's key never opens another's.
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
        crate::identity::minted().map(Self)
    }

    pub fn root(&self) -> Root {
        Root::from_bytes(&self.derived(ROOT))
    }

    /// The key for a person's own cloud, never sent anywhere.
    pub fn cloud(&self) -> Zeroizing<[u8; 32]> {
        self.derived(CLOUD)
    }

    fn derived(&self, label: &[u8]) -> Zeroizing<[u8; 32]> {
        let mut out = Zeroizing::new([0u8; 32]);
        // 32 bytes is far below what HKDF-SHA256 can expand to, so this cannot fail.
        let _ = Hkdf::<Sha256>::new(Some(SALT), self.0.as_ref()).expand(label, out.as_mut());
        out
    }
}

#[cfg(test)]
#[path = "seed_test.rs"]
mod tests;
