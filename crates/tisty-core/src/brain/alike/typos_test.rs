use super::*;

#[test]
fn a_long_word_one_edit_away_takes_the_key_of_the_more_used_one() {
    let heads = classes(&[("impresora", 9), ("imrpesora", 1), ("impresoras", 2)]);
    assert_eq!(
        heads.get("imrpesora").map(String::as_str),
        Some("impresora")
    );
    assert_eq!(
        heads.get("impresoras").map(String::as_str),
        Some("impresora")
    );
    assert_eq!(heads.get("impresora"), None);
}

#[test]
fn a_short_word_keeps_its_own_key() {
    let heads = classes(&[("cuenta", 9), ("cuneta", 1), ("casa", 5), ("cosa", 1)]);
    assert_eq!(heads.get("cuneta").map(String::as_str), Some("cuenta"));
    assert_eq!(
        heads.get("cosa"),
        None,
        "four letters are too few to tell a typo"
    );
}

#[test]
fn two_edits_away_is_another_word() {
    assert!(classes(&[("impresora", 9), ("imprimir", 1)]).is_empty());
}

#[test]
fn a_class_never_chains_through_its_members() {
    let heads = classes(&[("abcdefg", 9), ("abcdefh", 5), ("abcdehh", 1)]);
    assert_eq!(heads.get("abcdefh").map(String::as_str), Some("abcdefg"));
    assert_eq!(heads.get("abcdehh"), None, "two edits from the head");
}

#[test]
fn equally_used_words_follow_the_alphabet() {
    let heads = classes(&[("presupuesto", 3), ("prseupuesto", 3)]);
    assert_eq!(
        heads.get("prseupuesto").map(String::as_str),
        Some("presupuesto")
    );
}

#[test]
fn words_told_apart_by_a_digit_are_no_typo() {
    assert!(classes(&[("capitulo1", 5), ("capitulo2", 1)]).is_empty());
}
