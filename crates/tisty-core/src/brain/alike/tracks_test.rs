use std::collections::HashSet;

use super::super::likeness::CAPS;
use super::*;

fn caps() -> Caps {
    Caps {
        common: 1,
        common_at_least: 100,
        steps: u64::MAX,
        ..CAPS
    }
}

fn said(words: &[&str]) -> Item {
    Item {
        lang: None,
        kin: Vec::new(),
        words: words
            .iter()
            .map(|one| (crate::text::folded(one), one.to_string(), 1))
            .collect(),
    }
}

fn base(mut items: Vec<Item>) -> Vec<Item> {
    for n in 0..8 {
        items.push(said(&[&format!("filler{n}")]));
    }
    items
}

fn dictionary() -> Dictionary {
    Dictionary::parsed("agua\twater\ncuenta\tbill\npagar\tpay\n".to_string()).unwrap()
}

fn tracked(
    items: &[Item],
    guesses: &[Option<&str>],
    dictionary: Option<&Dictionary>,
) -> Vec<(usize, usize, &'static str, String)> {
    let guesses: Vec<Option<String>> = guesses
        .iter()
        .map(|one| one.map(str::to_string))
        .chain(std::iter::repeat(None))
        .take(items.len())
        .collect();
    let mut all: Vec<_> = alike(items, &guesses, &HashSet::new(), &caps(), dictionary)
        .found
        .into_iter()
        .map(|one| (one.a, one.b, one.method, one.shared.join(" ")))
        .collect();
    all.sort();
    all
}

#[test]
fn a_spanish_task_and_its_english_twin_are_kept_alike_by_the_dictionary() {
    let items = base(vec![
        said(&["pagar", "cuenta", "agua"]),
        said(&["pay", "water", "bill"]),
    ]);
    let guesses = [Some("es"), Some("en")];
    assert!(
        tracked(&items, &guesses, None).is_empty(),
        "no word in common"
    );
    assert_eq!(
        tracked(&items, &guesses, Some(&dictionary())),
        [(
            0,
            1,
            DICTIONARY,
            "pagar≈pay cuenta≈bill agua≈water".to_string()
        )]
    );
}

#[test]
fn english_text_is_never_read_as_spanish() {
    let items = base(vec![said(&["pagar", "agua"]), said(&["pagar", "water"])]);
    assert!(tracked(&items, &[Some("es"), Some("en")], Some(&dictionary())).is_empty());
    assert!(
        tracked(&items, &[Some("es"), None], Some(&dictionary())).is_empty(),
        "a text of no known language crosses to none"
    );
}

#[test]
fn the_dictionary_joins_only_two_languages_never_one() {
    let items = base(vec![said(&["pagar", "cuenta"]), said(&["pay", "bill"])]);
    let crossing = [(0, 1, DICTIONARY, "pagar≈pay cuenta≈bill".to_string())];
    assert_eq!(
        tracked(&items, &[Some("es"), Some("en")], Some(&dictionary())),
        crossing
    );
    assert!(
        tracked(&items, &[Some("es"), Some("es")], Some(&dictionary())).is_empty(),
        "two Spanish texts are never joined through English"
    );
}

#[test]
fn a_typo_joins_what_shared_words_missed() {
    let items = base(vec![
        said(&["impresora", "nueva"]),
        said(&["imrpesora", "nueva"]),
        said(&["presupuesto", "familiar"]),
        said(&["presupuesto", "familiar"]),
    ]);
    assert_eq!(
        tracked(&items, &[], None),
        [
            (0, 1, TYPO, "impresora≈imrpesora nueva".to_string()),
            (2, 3, WORDS, "presupuesto familiar".to_string()),
        ]
    );
}

#[test]
fn shared_words_keep_their_place_and_a_track_only_fills_the_room_left() {
    let mut items = vec![said(&["impresora", "nueva"])];
    for _ in 0..3 {
        items.push(said(&["impresora", "nueva"]));
    }
    items.push(said(&["imrpesora", "nueva"]));
    let found = tracked(&base(items), &[], None);
    assert!(
        found.iter().all(|(_, _, method, _)| *method == WORDS),
        "{found:?}"
    );
    let mut held = [0usize; 5];
    for (a, b, _, _) in &found {
        held[*a] += 1;
        held[*b] += 1;
    }
    assert!(held.iter().all(|one| *one <= caps().closest), "{held:?}");
}

#[test]
fn two_words_one_key_stands_for_count_once_under_the_first_written() {
    let item = said(&["cuenta", "cuneta", "agua"]);
    let again = keyed_again(&item, |key| (key == "cuneta").then_some("cuenta"));
    assert_eq!(
        again.words,
        [
            ("cuenta".to_string(), "cuenta".to_string(), 2),
            ("agua".to_string(), "agua".to_string(), 1),
        ]
    );
}

#[test]
fn readers_show_every_track_the_pass_writes() {
    for method in [WORDS, TYPO, DICTIONARY] {
        assert!(SHOWN.contains(&format!("'{method}'")), "{method}");
    }
}
