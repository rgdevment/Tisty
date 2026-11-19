use std::path::PathBuf;

use super::*;

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../app/src-tauri/resources/brain/dictionary.tsv")
}

#[test]
fn each_line_is_a_spanish_word_and_its_english_one() {
    let dictionary = Dictionary::parsed("agua\twater\npagar\tpay\n").unwrap();
    assert_eq!(dictionary.english_of("pagar"), Some("pay"));
    assert_eq!(dictionary.english_of("agua"), Some("water"));
    assert_eq!(dictionary.english_of("water"), None);
    assert_eq!(dictionary.len(), 2);
    assert!(!dictionary.is_empty());
}

#[test]
fn a_line_that_is_not_a_pair_refuses_the_whole_table() {
    assert!(
        Dictionary::parsed("agua\twater\npagar\n")
            .unwrap_err()
            .contains("line 2")
    );
    assert!(Dictionary::parsed("\tpay\n").is_err());
    assert!(Dictionary::parsed("pagar\t\n").is_err());
    assert!(Dictionary::parsed("").unwrap().is_empty());
}

#[test]
fn a_missing_file_says_why() {
    assert!(Dictionary::read(&shipped().with_file_name("missing.tsv")).is_err());
}

#[test]
fn the_shipped_table_reads_and_knows_the_everyday_words() {
    let dictionary = Dictionary::read(&shipped()).unwrap();
    assert!(dictionary.len() > 40_000, "{}", dictionary.len());
    for (spanish, english) in [
        ("pagar", "pay"),
        ("agua", "water"),
        ("techo", "roof"),
        ("gotera", "leak"),
        ("entrevista", "interview"),
    ] {
        assert_eq!(dictionary.english_of(spanish), Some(english), "{spanish}");
    }
    assert_eq!(
        dictionary.english_of("chile"),
        None,
        "a word that is the same is left out"
    );
}
