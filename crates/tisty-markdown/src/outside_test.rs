use super::{file_as_tisty, read_file};
use crate::{html, pieces, read};

const CUT: &str = "Este párrafo viene cortado a ochenta columnas, como casi todo lo escrito\nen un editor de código, y se lee como uno solo.\n";

fn drawn(text: &str) -> String {
    html(&read_file(text))
}

#[test]
fn a_paragraph_cut_at_its_column_reads_as_one() {
    assert!(html(&read(CUT)).contains("<br>"));
    let page = drawn(CUT);
    assert!(!page.contains("<br>"));
    assert!(page.contains("escrito\nen un editor"));
}

#[test]
fn short_lines_keep_the_break_their_writer_gave_them() {
    assert!(
        drawn("Calle Uno 123\nSantiago\nChile\n")
            .matches("<br>")
            .count()
            == 2
    );
}

#[test]
fn a_break_asked_for_stays_even_in_a_long_paragraph() {
    let long =
        "Una línea bien larga que llega de sobra a las sesenta columnas y sigue  \nla siguiente\n";
    assert!(drawn(long).contains("<br>"));
}

#[test]
fn a_cut_item_reads_as_one_too() {
    let item = format!("- {CUT}");
    assert!(!drawn(&item).contains("<br>"));
}

#[test]
fn a_colour_or_a_number_is_not_a_tag_and_a_word_is() {
    let page = drawn("El issue #123 y el color #ff0000 y #hashtag suelto.\n");
    assert!(page.contains("#123"));
    assert!(page.contains("#ff0000"));
    assert!(!page.contains("data-tag=\"ff0000\""));
    assert!(!page.contains("data-tag=\"123\""));
}

#[test]
fn double_brackets_are_only_what_they_say() {
    let page = drawn("Wiki [[Otra página]] aquí.\n");
    assert!(page.contains("[[Otra página]]"));
    assert!(html(&read("Wiki [[Otra página]] aquí.\n")) != page);
}

#[test]
fn a_reference_links_and_its_definition_prints_nothing() {
    let page = drawn("Ve [al sitio][1].\n\n[1]: https://example.com\n");
    assert!(page.contains("href=\"https://example.com\""));
    assert!(!page.contains("refdef"));
}

#[test]
fn a_tag_it_does_not_keep_leaves_its_words_and_not_itself() {
    let page = drawn("Un <span style=\"color:red\">rojo</span> y <video src=\"a\">x</video>.\n");
    assert!(page.contains("rojo"));
    assert!(!page.contains("span style"));
    assert!(!page.contains("&lt;span"));
    assert!(!page.contains("&lt;video"));
}

#[test]
fn a_link_with_a_script_keeps_only_its_words() {
    let page = drawn("Mira <a href=\"javascript:alert(1)\">esto</a> ya.\n");
    assert!(page.contains("esto"));
    assert!(!page.contains("javascript"));
}

#[test]
fn the_tags_tisty_keeps_are_still_drawn() {
    assert!(drawn("Pulsa <kbd>Ctrl</kbd> y <u>ya</u>.\n").contains("<kbd>Ctrl</kbd>"));
}

#[test]
fn a_centred_logo_stays_centred_in_a_file() {
    let body = "<div align=\"center\"><img src=\"./logo.png\" width=\"200\"></div>\n";
    let doc = read_file(body);
    assert!(pieces(&doc, 1024).concat().contains("align=\"center\""));
    assert!(!html(&read(body)).contains("align=\"center\""));
}

#[test]
fn what_a_document_of_tisty_says_is_still_drawn() {
    let page =
        drawn("> [!WARNING]\n> Una alerta.\n\n- [x] hecho\n\n| a | b |\n|---|---|\n| 1 | 2 |\n");
    assert!(page.contains("data-callout"));
    assert!(page.contains("<table"));
    assert!(page.contains("hecho"));
}

#[test]
fn a_script_is_still_taken_out() {
    let page = drawn("<script>alert(1)</script>\n\n<iframe src=\"https://a.b\"></iframe>\n");
    assert!(!page.contains("alert(1)</script>"));
    assert!(!page.contains("<iframe"));
}

fn seen(page: &str) -> String {
    let mut out = String::new();
    let mut rest = page;
    while let Some(from) = rest.find(" data-at=\"") {
        out.push_str(&rest[..from]);
        let after = &rest[from + 10..];
        rest = &after[after.find('"').map_or(0, |at| at + 1)..];
    }
    out.push_str(rest);
    out.replace("&#35;", "#")
        .replace("&#91;", "[")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn says_the_same(file: &str) {
    let before = seen(&drawn(file));
    let after = seen(&html(&read(&file_as_tisty(file))));
    assert_eq!(before, after, "as written: {}", file_as_tisty(file));
}

#[test]
fn a_cut_paragraph_says_the_same_in_tisty() {
    says_the_same(CUT);
    assert!(!file_as_tisty(CUT).contains("escrito\n"));
}

#[test]
fn a_cut_quote_loses_its_margin_with_the_line() {
    let quote = format!("> {}", CUT.replace('\n', "\n> "));
    says_the_same(quote.trim_end_matches("> "));
}

#[test]
fn short_lines_say_the_same_in_tisty() {
    says_the_same("Calle Uno 123\nSantiago\nChile\n");
}

#[test]
fn a_colour_and_a_number_say_the_same_in_tisty() {
    says_the_same("El issue #123 y el color #ff0000 aquí.\n");
}

#[test]
fn double_brackets_say_the_same_in_tisty() {
    says_the_same("Wiki [[Otra página]] aquí.\n");
}

#[test]
fn a_reference_becomes_a_link_in_place() {
    let file = "Ve [al sitio][1] y a [otro][].\n\n[1]: https://example.com\n[otro]: https://b.c\n";
    says_the_same(file);
    assert!(file_as_tisty(file).contains("[al sitio](https://example.com)"));
    assert!(!file_as_tisty(file).contains("[1]:"));
}

#[test]
fn a_tag_it_does_not_keep_says_the_same_in_tisty() {
    says_the_same("Un <span style=\"color:red\">rojo</span> y <kbd>Ctrl</kbd>.\n");
}

#[test]
fn code_is_never_turned() {
    let file = "```\n#ff0000 [[x]] <span>\n```\n";
    assert_eq!(file_as_tisty(file), file);
}

#[test]
fn a_document_of_tisty_comes_in_as_it_was() {
    let file = "# Título\n\nUna línea corta\ny otra.\n\n- [x] hecho\n";
    assert_eq!(file_as_tisty(file), file);
}
