use tisty_markdown::{html, printed, read};

fn page(text: &str) -> String {
    html(&read(text))
}

fn marks(text: &str) -> String {
    printed(&read(text))["content"][0]["content"].to_string()
}

#[test]
fn a_pen_names_its_colour_after_the_closing_wall() {
    let drawn = page("Revisar ==esto=={green} y ==aquello==");
    assert!(drawn.contains("<mark data-pen=\"green\">esto</mark>"));
    assert!(drawn.contains("<mark>aquello</mark>"));
    assert!(!drawn.contains("{green}"));
    assert!(!drawn.contains("=="));
}

#[test]
fn yellow_named_out_loud_is_the_same_pen_as_the_unnamed_one() {
    let drawn = page("==otro=={yellow}");
    assert!(drawn.contains("<mark>otro</mark>"));
    assert!(!drawn.contains("{yellow}"));
}

#[test]
fn a_colour_the_palette_does_not_have_stays_in_sight() {
    let drawn = page("==raro=={chartreuse}");
    assert!(drawn.contains("<mark>raro</mark>"));
    assert!(drawn.contains("{chartreuse}"));
}

#[test]
fn an_icon_is_written_between_double_colons_with_an_optional_hue() {
    let drawn = page("::star:: listo y ::flag red::");
    assert!(drawn.contains("data-ico=\"star\""));
    assert!(drawn.contains("data-ico=\"flag\" data-hue=\"red\""));
    assert!(!drawn.contains("::"));
}

#[test]
fn a_name_or_hue_the_language_does_not_have_is_left_as_words() {
    assert!(page("::nadaquever::").contains("::nadaquever::"));
    assert!(page("::star chartreuse::").contains("::star chartreuse::"));
    assert!(!page("a::star::b").contains("data-ico"));
}

#[test]
fn an_icon_is_not_read_a_second_time_as_the_emoji_inside_it() {
    let drawn = page("::star:: y <span data-ico=\"star\">:star:</span>");
    assert_eq!(drawn.matches("data-ico=\"star\"").count(), 2);
    assert!(!drawn.contains('⭐'));
}

#[test]
fn double_plus_underlines_and_code_is_left_alone() {
    let drawn = page("++subrayado++ y C++ y i++ y ++ suelto ++");
    assert!(drawn.contains("<u>subrayado</u>"));
    assert!(drawn.contains("C++ y i++ y ++ suelto ++"));
    assert_eq!(drawn.matches("<u>").count(), 1);
}

#[test]
fn the_underline_an_older_file_wrote_is_the_same_mark() {
    let drawn = page("<u>viejo</u> y <U>mayus</U>");
    assert_eq!(drawn.matches("<u>").count(), 2);
    let said = marks("<u>viejo</u> y ++nuevo++");
    assert_eq!(said.matches("\"underline\"").count(), 2);
}

#[test]
fn paper_learns_the_new_marks_too() {
    let said = marks("**==negrita=={blue}** y ++sub++ y ~~no~~");
    assert!(said.contains("\"blue\""));
    assert!(said.contains("\"underline\""));
    assert!(said.contains("\"strike\""));
    assert!(!said.contains("{blue}"));
}

#[test]
fn every_word_the_language_has_is_known_once() {
    use tisty_markdown::vocab::{HUES, ICONS, YELLOW, hue, icon, pen};
    assert!(pen(YELLOW) && !hue(YELLOW));
    assert!(HUES.iter().all(|one| pen(one) && hue(one)));
    assert!(ICONS.iter().all(|one| icon(one)));
    assert!(!pen("chartreuse") && !icon("nadaquever"));
}
