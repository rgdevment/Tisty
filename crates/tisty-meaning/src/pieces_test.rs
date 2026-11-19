use super::*;

fn shipped() -> Pieces {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../app/src-tauri/resources/brain/vocab.txt");
    Pieces::read(&std::fs::read_to_string(at).unwrap()).unwrap()
}

fn small() -> Pieces {
    Pieces::read("[PAD]\n[UNK]\n[CLS]\n[SEP]\n[MASK]\nun\n##able\n##a\nable\n!\nhola\n中\n##b")
        .unwrap()
}

#[test]
fn the_shipped_vocabulary_pieces_every_fixture_text_as_the_published_tokenizer_does() {
    let pieces = shipped();
    let fixture = include_str!("../fixtures/pieces.jsonl");
    let mut differed = Vec::new();
    let mut read = 0;
    for line in fixture.lines() {
        let said: serde_json::Value = serde_json::from_str(line).unwrap();
        let text = said["text"].as_str().unwrap();
        let wanted: Vec<u32> = serde_json::from_value(said["ids"].clone()).unwrap();
        if pieces.of(text) != wanted {
            differed.push(text.to_string());
        }
        read += 1;
    }
    assert!(read > 300, "only {read} texts in the fixture");
    assert!(
        differed.is_empty(),
        "{} differ: {differed:?}",
        differed.len()
    );
}

#[test]
fn a_word_is_pieced_longest_first_and_goes_on_with_its_mark() {
    assert_eq!(small().of("unable"), vec![5, 6]);
    assert_eq!(small().of("able"), vec![8]);
}

#[test]
fn a_word_that_cannot_be_pieced_whole_is_one_unknown() {
    assert_eq!(small().of("unz"), vec![1]);
    assert_eq!(small().of("hola unz able"), vec![10, 1, 8]);
}

#[test]
fn a_word_longer_than_a_hundred_characters_is_unknown_without_being_looked_at() {
    let long = format!("un{}", "a".repeat(99));
    assert_eq!(small().of(&long), vec![1]);
    let fits = format!("un{}", "a".repeat(98));
    assert_eq!(small().of(&fits).len(), 99);
}

#[test]
fn punctuation_stands_alone_and_accents_and_case_fall_away() {
    assert_eq!(small().of("HÓLA!able"), vec![10, 9, 8]);
    assert_eq!(small().of("ho\u{301}la"), vec![10]);
}

#[test]
fn a_chinese_character_is_a_word_of_its_own() {
    assert_eq!(small().of("中b"), vec![11, 1]);
    assert_eq!(small().of("hola中"), vec![10, 11]);
}

#[test]
fn a_special_piece_is_taken_as_written_before_the_text_is_folded() {
    assert_eq!(small().of("[CLS]hola[SEP]"), vec![2, 10, 3]);
    assert_eq!(small().of("[cls]"), vec![1, 1, 1]);
}

#[test]
fn control_characters_go_and_whitespace_of_any_kind_splits() {
    assert_eq!(small().of("ho\u{7}la\u{0}\u{fffd}"), vec![10]);
    assert_eq!(small().of("hola\u{2003}able\tun"), vec![10, 8, 5]);
}

#[test]
fn a_vocabulary_without_its_unknown_piece_is_refused() {
    assert!(Pieces::read("[PAD]\nhola").is_none());
}

#[test]
fn nothing_written_gives_no_pieces() {
    assert!(small().of("").is_empty());
    assert!(small().of(" \n\t ").is_empty());
}

#[test]
fn a_vocabulary_ending_in_a_line_break_reads_as_written() {
    let ended = Pieces::read("[PAD]\n[UNK]\nhola\n").unwrap();
    assert_eq!(ended.len(), 3);
    assert_eq!(ended.of("hola"), vec![2]);
}
