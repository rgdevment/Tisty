use serde_json::json;
use tisty_contracts::check;

use super::{file_name, is_a_tag, line, name};

fn a_title_holds(said: &str) -> bool {
    check(
        "document",
        &json!({ "id": "x", "title": said, "html": "", "body": "" }),
    )
    .is_empty()
}

#[test]
fn a_title_of_two_lines_becomes_one() {
    assert_eq!(
        line("Paseos\n\nVeredicto: holds").unwrap(),
        "Paseos Veredicto: holds"
    );
}

#[test]
fn what_turns_the_text_around_or_hides_a_letter_comes_off() {
    assert_eq!(line("Firmado \u{202E}exe.fdp").unwrap(), "Firmado exe.fdp");
    assert_eq!(line("Ma\u{200B}rio\u{FEFF}").unwrap(), "Mario");
    assert_eq!(line("Ma\u{200D}rio").unwrap(), "Mario");
}

#[test]
fn a_joiner_a_script_needs_stays() {
    assert_eq!(line("علی\u{200C}رضا").unwrap(), "علی\u{200C}رضا");
}

#[test]
fn nothing_but_what_is_unseen_is_no_line() {
    assert_eq!(line("\u{200B}\u{202E} \n"), None);
    assert_eq!(line(""), None);
}

#[test]
fn what_comes_out_is_what_sealing_takes_however_it_went_in() {
    for said in [
        "a\u{2028}b",
        "a\u{0007}b",
        "\u{1D173}a",
        "\u{E0041}a",
        "a\u{200D}",
        "\u{200C}a",
        "a\tb\r\nc",
        &"x".repeat(9000),
    ] {
        let clean = line(said).unwrap();
        assert!(a_title_holds(&clean), "{said:?} became {clean:?}");
    }
    assert_eq!(line(&"x".repeat(9000)).unwrap().chars().count(), 8192);
}

#[test]
fn a_file_is_named_without_the_path_it_came_from() {
    for (said, named) in [
        ("..\\x.exe", "x.exe"),
        ("C:\\Users\\mario\\cartilla.pdf", "cartilla.pdf"),
        ("/home/mario/a b.pdf", "a b.pdf"),
        ("a\nb.pdf", "a b.pdf"),
    ] {
        assert_eq!(file_name(said).unwrap(), named);
        let file = json!({ "at": 0, "name": file_name(said).unwrap() });
        assert_eq!(check("attachment", &file), Vec::<String>::new());
    }
}

#[test]
fn a_file_with_no_name_left_has_none() {
    for said in ["", "a/", "..", "C:\\", "\u{200B}"] {
        assert_eq!(file_name(said), None, "{said:?}");
    }
}

#[test]
fn a_name_is_what_the_contract_calls_one() {
    assert_eq!(name("  Rina \n").unwrap(), "Rina");
    assert_eq!(name(&"a".repeat(90)).unwrap().chars().count(), 80);
    assert_eq!(name("a 123 456"), None);
    assert_eq!(name("12345"), None);
    assert_eq!(name("\u{200B}"), None);
}

#[test]
fn a_tag_is_only_the_form_the_contract_keeps() {
    assert!(is_a_tag("rina"));
    assert!(is_a_tag("हिन्दी"));
    assert!(!is_a_tag("Rina"));
    assert!(!is_a_tag("camión"));
}
