use super::*;

#[test]
fn a_name_is_a_readable_line_of_at_most_eighty_characters_with_nothing_unseen() {
    let long = "x".repeat(NAME_AT_MOST + 1);
    for name in [
        "",
        "   ",
        " Mario",
        "una\nlínea",
        "\u{200B}",
        "Ana\u{202E}zeréP",
        "\u{3164}",
        "Ana\u{2028}Pérez",
        "12345 67890 12345 67890",
        "12345 67890 12345 6789O",
        "Ana\u{180B}",
        "Ana\u{17B4}",
        "Ana\u{1D173}",
        "Ana\u{E0041}",
        long.as_str(),
    ] {
        assert!(!named_well(name), "{name:?}");
    }
}

#[test]
fn a_name_of_any_alphabet_up_to_eighty_characters_is_well_named() {
    let longest = "日".repeat(NAME_AT_MOST);
    for name in [
        "Mario Hidalgo",
        "José Núñez",
        "王小明",
        "R2D2",
        "Sala 101",
        longest.as_str(),
    ] {
        assert!(named_well(name), "{name}");
    }
}
