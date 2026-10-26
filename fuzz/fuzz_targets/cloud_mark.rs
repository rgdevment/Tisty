#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::cloud::{self, CloudKey, Mark};

const KEY_ID: std::ops::Range<usize> = 5..21;

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    cloud: &'a str,
    mark: &'a [u8],
    at: usize,
    flip: u8,
    key: &'a str,
}

fuzz_target!(|input: Input| {
    let ours = CloudKey::from_bytes(&[7; 32]);
    let theirs = CloudKey::from_bytes(&[8; 32]);
    assert_ne!(
        cloud::mark_says(&ours, input.cloud, input.mark),
        Mark::Holds
    );

    let genuine = cloud::mark(&ours, input.cloud).expect("marked");
    assert_eq!(cloud::mark_says(&ours, input.cloud, &genuine), Mark::Holds);
    assert_eq!(
        cloud::mark_says(&theirs, input.cloud, &genuine),
        Mark::OtherPhrase
    );
    let mut changed = genuine;
    let at = input.at % changed.len();
    changed[at] ^= input.flip.max(1);
    // The key's id is not sealed: a change there is what tells a damaged mark from another phrase.
    if !KEY_ID.contains(&at) {
        assert_ne!(cloud::mark_says(&ours, input.cloud, &changed), Mark::Holds);
    }

    if let Some(read) = CloudKey::from_text(input.key) {
        assert_eq!(read.to_text().as_str(), input.key);
    }
});
