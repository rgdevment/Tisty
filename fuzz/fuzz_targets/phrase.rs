#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::phrase::{self, Unread, WORDS, Words};
use tisty_share_proto::seed::Seed;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    said: &'a str,
    word: &'a str,
    seed: [u8; 32],
    english: bool,
    at: u8,
}

fn bare(said: &str) -> String {
    said.nfd().filter(|one| !is_combining_mark(*one)).collect()
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

    let words = if input.english {
        Words::English
    } else {
        Words::Spanish
    };
    let written = phrase::phrase(&Seed::from_bytes(&input.seed), words).expect("written");
    let listed: Vec<&str> = written.split(' ').collect();
    let word = listed[usize::from(input.at) % listed.len()];
    for copied in [
        word.nfc().collect::<String>(),
        word.nfd().collect(),
        word.to_uppercase(),
        bare(word),
        format!(" {word}\t"),
    ] {
        assert!(phrase::same_word(&copied, word));
        assert!(phrase::same_word(word, &copied));
    }
    let _ = phrase::same_word(input.word, word);

    let by_hand = bare(&written).to_uppercase();
    let read = phrase::seed_from(&by_hand).expect("read by hand");
    assert_eq!(*read.to_bytes(), input.seed);
});
