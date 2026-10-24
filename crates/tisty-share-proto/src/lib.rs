pub mod identity;
pub mod phrase;
pub mod seed;

pub use ed25519_dalek::VerifyingKey;

#[cfg(feature = "mint")]
fn minted() -> Option<zeroize::Zeroizing<[u8; 32]>> {
    let mut fresh = zeroize::Zeroizing::new([0u8; 32]);
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, fresh.as_mut()).ok()?;
    Some(fresh)
}
