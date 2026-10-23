use super::*;

#[test]
fn one_seed_gives_the_same_keys_every_time() {
    let one = Seed::from_bytes(&[7; 32]);
    let again = Seed::from_bytes(&one.to_bytes());

    assert_eq!(one.root().person(), again.root().person());
    assert_eq!(*one.cloud(), *again.cloud());
}

#[test]
fn every_use_gets_a_key_of_its_own() {
    let seed = Seed::from_bytes(&[7; 32]);

    assert_ne!(*seed.cloud(), *seed.root().to_bytes());
    assert_ne!(*seed.cloud(), *seed.to_bytes());
    assert_ne!(*seed.root().to_bytes(), *seed.to_bytes());
}

#[test]
fn two_seeds_give_two_people() {
    let one = Seed::from_bytes(&[7; 32]);
    let two = Seed::from_bytes(&[8; 32]);

    assert_ne!(one.root().person(), two.root().person());
    assert_ne!(*one.cloud(), *two.cloud());
}

#[cfg(feature = "mint")]
#[test]
fn minted_seeds_differ() {
    let one = Seed::minted().unwrap();
    let two = Seed::minted().unwrap();

    assert_ne!(*one.to_bytes(), *two.to_bytes());
}

#[test]
fn the_keys_are_what_hkdf_gives() {
    let seed = Seed::from_bytes(&[7; 32]);
    let mut cloud = [0u8; 32];
    hkdf::Hkdf::<Sha256>::new(Some(&SALT[..]), &[7; 32])
        .expand(CLOUD, &mut cloud)
        .unwrap();

    assert_eq!(*seed.cloud(), cloud);
}
