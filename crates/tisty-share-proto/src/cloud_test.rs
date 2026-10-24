use super::*;

const CLOUD: &str = "01JSTORE";
const NAME: &str = "store/dev_a/active.tisty";

fn key(n: u8) -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[n; 32]))
}

fn sealed_here(said: &[u8]) -> Vec<u8> {
    sealed(&key(1), CLOUD, NAME, said).unwrap()
}

fn refused(sealed: &[u8]) -> Option<Unsealed> {
    opened(&key(1), CLOUD, NAME, sealed).err()
}

#[test]
fn what_is_sealed_opens_whole_whatever_its_size() {
    for size in [
        0,
        1,
        100,
        BLOCK - 1,
        BLOCK,
        BLOCK + 1,
        2 * BLOCK,
        3 * BLOCK + 17,
    ] {
        let said: Vec<u8> = (0..size).map(|at| (at % 251) as u8).collect();
        let sealed = sealed_here(&said);

        assert!(is_sealed(&sealed));
        let back = opened(&key(1), CLOUD, NAME, &sealed).unwrap();
        assert_eq!(*back, said, "size {size}");
    }
}

#[test]
fn sealing_streams_from_a_reader_into_a_writer() {
    let said = vec![9u8; 2 * BLOCK + 5];
    let mut sealed = Vec::new();
    seal(&key(1), CLOUD, NAME, &mut &said[..], &mut sealed).unwrap();
    let mut back = Vec::new();

    open(&key(1), CLOUD, NAME, &mut &sealed[..], &mut back).unwrap();

    assert_eq!(back, said);
}

#[test]
fn the_same_text_never_seals_the_same_way_twice() {
    assert_ne!(sealed_here(b"hello"), sealed_here(b"hello"));
}

#[test]
fn another_key_name_or_cloud_does_not_open_it() {
    let sealed = sealed_here(b"hello");

    assert_eq!(
        opened(&key(2), CLOUD, NAME, &sealed).err(),
        Some(Unsealed::Refused)
    );
    assert_eq!(
        opened(&key(1), CLOUD, "store/dev_b/active.tisty", &sealed).err(),
        Some(Unsealed::Refused)
    );
    assert_eq!(
        opened(&key(1), "01JOTHER", NAME, &sealed).err(),
        Some(Unsealed::Refused)
    );
}

#[test]
fn a_name_spelled_either_way_is_one_name() {
    let composed = "docs/Reuni\u{f3}n.md";
    let decomposed = "docs/Reunio\u{301}n.md";
    let sealed = sealed(&key(1), CLOUD, composed, b"hello").unwrap();

    assert_eq!(
        opened(&key(1), CLOUD, decomposed, &sealed)
            .unwrap()
            .as_slice(),
        b"hello"
    );
}

#[test]
fn a_changed_byte_is_refused() {
    let mut sealed = sealed_here(b"hello");
    let last = sealed.len() - 1;
    sealed[last] ^= 1;

    assert_eq!(refused(&sealed), Some(Unsealed::Refused));
}

#[test]
fn a_block_dropped_cut_or_moved_is_refused() {
    let said = vec![7u8; 3 * BLOCK];
    let sealed = sealed_here(&said);
    let start = MAGIC.len() + 1 + HEAD;
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
        assert_eq!(refused(&broken), Some(Unsealed::Refused));
    }
}

#[test]
fn bytes_tisty_never_sealed_and_newer_ones_are_told_apart() {
    assert!(!is_sealed(b"{\"v\":18}"));
    assert_eq!(refused(b"{\"v\":18}"), Some(Unsealed::Foreign));
    assert_eq!(refused(b"TCS"), Some(Unsealed::Foreign));

    let mut newer = sealed_here(b"hello");
    newer[3] = VERSION + 1;
    assert_eq!(refused(&newer), Some(Unsealed::Newer));

    let header_only = sealed_here(b"hello")[..MAGIC.len() + 1 + HEAD].to_vec();
    assert_eq!(refused(&header_only), Some(Unsealed::Refused));
}

#[test]
fn the_mark_tells_the_right_phrase_another_and_a_damaged_mark_apart() {
    let mark = mark(&key(1), CLOUD).unwrap();

    assert_eq!(mark_says(&key(1), CLOUD, &mark), Mark::Holds);
    assert_eq!(mark_says(&key(2), CLOUD, &mark), Mark::OtherPhrase);

    let mut damaged = mark.clone();
    let last = damaged.len() - 1;
    damaged[last] ^= 1;
    assert_eq!(mark_says(&key(1), CLOUD, &damaged), Mark::Damaged);
    assert_eq!(mark_says(&key(1), CLOUD, &mark[..10]), Mark::Damaged);
    assert_eq!(mark_says(&key(1), "01JOTHER", &mark), Mark::Damaged);
}

#[test]
fn a_mark_never_opens_as_a_file_nor_a_file_as_a_mark() {
    let mark = mark(&key(1), CLOUD).unwrap();
    let as_file = [&MAGIC[..], &[VERSION], &mark[KEY_ID..]].concat();

    assert_eq!(
        opened(&key(1), CLOUD, "", &as_file).err(),
        Some(Unsealed::Refused)
    );
}
