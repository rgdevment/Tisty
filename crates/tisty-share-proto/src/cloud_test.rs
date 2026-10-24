use super::*;

fn key(n: u8) -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[n; 32]))
}

fn sealed_here(said: &[u8]) -> Vec<u8> {
    sealed(&key(1), "store/dev_a/active.tisty", said).unwrap()
}

#[test]
fn what_is_sealed_opens_whole_whatever_its_size() {
    for size in [0, 1, 100, BLOCK - 1, BLOCK, BLOCK + 1, 3 * BLOCK + 17] {
        let said: Vec<u8> = (0..size).map(|at| (at % 251) as u8).collect();
        let sealed = sealed_here(&said);

        assert!(is_sealed(&sealed));
        let back = opened(&key(1), "store/dev_a/active.tisty", &sealed).unwrap();
        assert_eq!(*back, said, "size {size}");
    }
}

#[test]
fn the_same_text_never_seals_the_same_way_twice() {
    assert_ne!(sealed_here(b"hello"), sealed_here(b"hello"));
}

#[test]
fn another_key_or_another_name_does_not_open_it() {
    let sealed = sealed_here(b"hello");

    assert_eq!(
        opened(&key(2), "store/dev_a/active.tisty", &sealed).err(),
        Some(Unsealed::Refused)
    );
    assert_eq!(
        opened(&key(1), "store/dev_b/active.tisty", &sealed).err(),
        Some(Unsealed::Refused)
    );
}

#[test]
fn a_changed_byte_is_refused() {
    let mut sealed = sealed_here(b"hello");
    let last = sealed.len() - 1;
    sealed[last] ^= 1;

    assert_eq!(
        opened(&key(1), "store/dev_a/active.tisty", &sealed).err(),
        Some(Unsealed::Refused)
    );
}

#[test]
fn a_block_dropped_cut_or_moved_is_refused() {
    let said = vec![7u8; 3 * BLOCK];
    let sealed = sealed_here(&said);
    let start = SEALED.len() + HEAD;
    let one = BLOCK + TAG;

    let dropped_last = sealed[..start + 2 * one].to_vec();
    let mut moved = sealed.clone();
    let (first, second) = (start..start + one, start + one..start + 2 * one);
    let a = sealed[first.clone()].to_vec();
    let b = sealed[second.clone()].to_vec();
    moved[first].copy_from_slice(&b);
    moved[second].copy_from_slice(&a);
    let cut = sealed[..sealed.len() - 5].to_vec();

    for broken in [dropped_last, moved, cut] {
        assert_eq!(
            opened(&key(1), "store/dev_a/active.tisty", &broken).err(),
            Some(Unsealed::Refused)
        );
    }
}

#[test]
fn bytes_tisty_never_sealed_are_told_apart() {
    assert!(!is_sealed(b"{\"v\":18}"));
    assert_eq!(
        opened(&key(1), "x", b"{\"v\":18}").err(),
        Some(Unsealed::Foreign)
    );
    assert_eq!(opened(&key(1), "x", SEALED).err(), Some(Unsealed::Refused));
}

#[test]
fn the_mark_tells_the_right_phrase_from_another() {
    let mark = mark(&key(1)).unwrap();

    assert!(mark_holds(&key(1), &mark));
    assert!(!mark_holds(&key(2), &mark));
}
