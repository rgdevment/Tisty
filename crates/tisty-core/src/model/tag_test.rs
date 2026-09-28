use super::*;

fn tag(raw: &str) -> String {
    Tag::new(raw).unwrap().to_string()
}

#[test]
fn casing_and_spacing_collapse_into_one_tag() {
    assert_eq!(tag("Work"), "work");
    assert_eq!(tag("  WORK  "), "work");
    assert_eq!(tag("mi etiqueta"), "mi-etiqueta");
    assert_eq!(tag("mi_etiqueta"), "mi-etiqueta");
    assert_eq!(tag("mi   etiqueta"), "mi-etiqueta");
}

#[test]
fn an_accent_written_apart_from_its_letter_lands_on_the_same_tag() {
    assert_eq!(tag("disen\u{0303}o"), tag("diseño"));
    assert_eq!(tag("gestio\u{0301}n"), tag("gestión"));
}

#[test]
fn punctuation_is_dropped_not_kept_as_separator() {
    assert_eq!(tag("b2b/b2c"), "b2bb2c");
    assert_eq!(tag("#bug!"), "bug");
}

/// One word, one tag: nobody writes the accent the same way twice, and a tag people cannot
/// hit reliably is a tag that quietly splits their work in two.
#[test]
fn a_word_is_one_tag_however_its_accents_were_typed() {
    assert_eq!(tag("migración"), "migracion");
    assert_eq!(tag("camión"), tag("camion"));
    assert_eq!(tag("camión"), tag("CAMIÓN"));
    assert_eq!(tag("niño"), "nino");
}

#[test]
fn an_accent_that_is_the_whole_letter_still_leaves_something_behind() {
    assert_eq!(tag("año"), "ano");
    assert_eq!(Tag::new("´"), Err(InvalidTag));
}

#[test]
fn a_tag_without_letters_or_digits_is_rejected() {
    assert_eq!(Tag::new("---"), Err(InvalidTag));
    assert_eq!(Tag::new("  "), Err(InvalidTag));
    assert_eq!(Tag::new(""), Err(InvalidTag));
}

/// The rule lives here and not in `new` on purpose: a line of the log that fails to parse
/// stops the whole store from opening, and tags like these were saved before it existed.
#[test]
fn what_a_reader_takes_is_narrower_than_what_the_log_keeps() {
    for passed_over in ["1", "1234", "12-34", "2026", "a", "x"] {
        let tag = Tag::new(passed_over).unwrap();
        assert!(!tag.worth_reading(), "{passed_over}");
        assert!(serde_json::from_str::<Tag>(&format!("\"{passed_over}\"")).is_ok());
    }
    for kept in ["ia", "ux", "b2b", "pepe32", "1a", "legal"] {
        assert!(Tag::new(kept).unwrap().worth_reading(), "{kept}");
    }
}

#[test]
fn what_is_written_now_answers_to_the_rule_of_now() {
    for turned_away in ["1", "1234", "2026", "a", "x", "---", ""] {
        assert_eq!(Tag::written(turned_away), Err(InvalidTag), "{turned_away}");
    }
    assert_eq!(Tag::written("  Legal  ").unwrap().as_str(), "legal");
    assert_eq!(Tag::written("b2b").unwrap().as_str(), "b2b");
}

#[test]
fn deserialisation_normalises() {
    let tag: Tag = serde_json::from_str(r#""  Work  ""#).unwrap();
    assert_eq!(tag.as_str(), "work");
}

#[test]
fn deserialising_an_empty_tag_fails() {
    assert!(serde_json::from_str::<Tag>(r#""!!""#).is_err());
}
