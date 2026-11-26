pub mod cloud;
pub mod editing;
pub mod handing;
pub mod identity;
pub mod name;
pub mod notice;
pub mod phrase;
pub mod reviewing;
pub mod seed;

pub use ed25519_dalek::VerifyingKey;

#[cfg(feature = "mint")]
fn minted() -> Option<zeroize::Zeroizing<[u8; 32]>> {
    let mut fresh = zeroize::Zeroizing::new([0u8; 32]);
    getrandom::fill(fresh.as_mut()).ok()?;
    Some(fresh)
}
