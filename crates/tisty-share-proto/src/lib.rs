pub mod cloud;
pub mod handing;
pub mod identity;
pub mod notice;
pub mod phrase;
pub mod seed;

pub use ed25519_dalek::VerifyingKey;

#[cfg(all(feature = "mint", not(fuzzing)))]
fn minted() -> Option<zeroize::Zeroizing<[u8; 32]>> {
    let mut fresh = zeroize::Zeroizing::new([0u8; 32]);
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, fresh.as_mut()).ok()?;
    Some(fresh)
}

#[cfg(all(feature = "mint", fuzzing))]
fn minted() -> Option<zeroize::Zeroizing<[u8; 32]>> {
    Some(zeroize::Zeroizing::new([9; 32]))
}
