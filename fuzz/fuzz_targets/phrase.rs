#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::phrase::{self, Unread, WORDS, Words};

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    said: &'a str,
    word: &'a str,
}

fuzz_target!(|input: Input| {
    match phrase::seed_from(input.said) {
        Ok(seed) => {
            let written = phrase::phrase(&seed, Words::Spanish).expect("written");
            let read = phrase::seed_from(&written).expect("read back");
            assert_eq!(*read.to_bytes(), *seed.to_bytes());
            let written = phrase::phrase(&seed, Words::English).expect("written");
            assert!(phrase::seed_from(&written).is_ok());
        }
        Err(Unread::Count(count)) => assert_ne!(count, WORDS),
        Err(Unread::Word(at)) => assert!((1..=WORDS).contains(&at)),
        Err(Unread::Checksum) => {}
    }
    assert!(phrase::same_word(input.word, input.word));
});
