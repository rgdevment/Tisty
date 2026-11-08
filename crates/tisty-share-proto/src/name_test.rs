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

#[test]
fn a_name_that_looks_like_the_makers_imitates_it_as_folio_reads_it() {
    for alias in [
        "rgdevment",
        "RGDEVMENT",
        "rg-devment",
        "rg devment",
        "rgdevrnent",
        "rgd\u{0435}vment",
        "rgdevm\u{0435}nt",
        "rgde\u{03bd}ment",
        "rgdevm\u{00e9}nt",
        "rgdevment\u{200b}",
        "r\u{0261}devment",
        "rg\u{0501}evment",
        "r\u{0261}\u{0501}evment",
        "rgdeevment",
        "rgdvment",
        "rgdevmnet",
        "rgdevvment",
        "rgdevments",
        "xgdevment",
    ] {
        assert!(imitates(alias, "rgdevment"), "{alias:?}");
    }
    let rain = "\u{96e8}".repeat(9);
    for alias in [
        "Ana",
        "rgdev",
        "devment",
        "rgdevmentor",
        "",
        rain.as_str(),
        "\u{0436}\u{0436}\u{0436}devment",
        "\u{0410}\u{043d}\u{043d}\u{0430}",
        "\u{041c}\u{0430}\u{0440}\u{0456}\u{044f}",
    ] {
        assert!(!imitates(alias, "rgdevment"), "{alias:?}");
    }
}

#[test]
fn two_unknown_letters_may_stand_in_never_three() {
    assert!(imitates("r\u{0261}\u{0501}evment", "rgdevment"));
    assert!(!imitates("r\u{0261}\u{0501}\u{0436}vment", "rgdevment"));
}

#[test]
fn a_short_name_matches_only_whole_since_one_slip_would_catch_too_many() {
    assert!(!imitates("anna", "ana"));
    assert!(imitates("ANA", "ana"));
}

#[test]
fn the_makers_domain_is_imitated_like_its_name() {
    assert!(imitates("rgdevment.dev", "rgdevment.dev"));
    assert!(imitates("RG-DEVMENT.DEV", "rgdevment.dev"));
}
