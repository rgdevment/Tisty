use std::path::PathBuf;

use super::*;

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../app/src-tauri/resources/brain/dictionary.tsv")
}

fn parsed(said: &str) -> Result<Dictionary, String> {
    Dictionary::parsed(said.to_string())
}

#[test]
fn each_line_is_a_spanish_word_and_its_english_one() {
    let dictionary = parsed("agua\twater\npagar\tpay\n").unwrap();
    assert_eq!(dictionary.english_of("pagar"), Some("pay"));
    assert_eq!(dictionary.english_of("agua"), Some("water"));
    assert_eq!(dictionary.english_of("water"), None);
    assert_eq!(dictionary.english_of("pag"), None);
    assert_eq!(dictionary.len(), 2);
}

#[test]
fn the_last_line_needs_no_newline() {
    assert_eq!(
        parsed("agua\twater\npagar\tpay")
            .unwrap()
            .english_of("pagar"),
        Some("pay")
    );
}

#[test]
fn a_line_that_is_not_a_pair_refuses_the_whole_table() {
    assert!(
        parsed("agua\twater\npagar\n")
            .unwrap_err()
            .contains("line 2")
    );
    assert!(parsed("\tpay\n").is_err());
    assert!(parsed("pagar\t\n").is_err());
    assert!(parsed("pagar\tpay\tpaid\n").is_err());
    assert_eq!(parsed("").unwrap().len(), 0);
}

#[test]
fn a_table_out_of_order_is_refused_since_it_is_searched_in_order() {
    assert!(
        parsed("pagar\tpay\nagua\twater\n")
            .unwrap_err()
            .contains("line 2")
    );
    assert!(
        parsed("agua\twater\nagua\tdrink\n").is_err(),
        "a word twice"
    );
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
