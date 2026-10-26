#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::cloud::{self, CloudKey};

const CLOUD: &str = "Tisty";
const NAME: &str = "store/device/events.jsonl";
const BLOCK: usize = 64 * 1024;

#[derive(Arbitrary, Debug)]
struct Input {
    plain: Vec<u8>,
    blocks: u8,
    at: usize,
    flip: u8,
    cut: bool,
}

fuzz_target!(|input: Input| {
    let key = CloudKey::from_bytes(&[7; 32]);
    let wide = input.plain.len() + usize::from(input.blocks % 3) * BLOCK;
    let plain: Vec<u8> = input
        .plain
        .iter()
        .copied()
        .chain(std::iter::repeat(0))
        .take(wide)
        .collect();
    let mut sealed = cloud::sealed(&key, CLOUD, NAME, &plain).expect("sealed");
    assert_eq!(
        cloud::opened(&key, CLOUD, NAME, &sealed).as_deref(),
        Ok(&plain)
    );
    assert!(cloud::opened(&key, CLOUD, "store/device/other.jsonl", &sealed).is_err());

    let at = input.at % sealed.len();
    if input.cut {
        sealed.truncate(at);
    } else {
        sealed[at] ^= input.flip.max(1);
    }
    assert!(cloud::opened(&key, CLOUD, NAME, &sealed).is_err());
});
