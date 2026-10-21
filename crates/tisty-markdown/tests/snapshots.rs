use tisty_markdown::{html, outline, printed, read, to_standard, to_tisty};

fn page(text: &str) -> String {
    html(&read(text))
}

fn standard(text: &str) -> String {
    to_standard(text, &|_| None)
}

#[test]
fn headings_paragraphs_and_inline_marks_drawn_as_html() {
    insta::assert_snapshot!(page(
        "# Title\n\nSome **strong**, *soft*, ~~gone~~ and `code` with a [link](https://example.org).\n\n## Section\n\nLine one\\\nline two.\n"
    ));
}

#[test]
fn lists_and_tasks_drawn_as_html() {
    insta::assert_snapshot!(page(
        "- one\n- two\n  - nested\n\n1. first\n2. second\n\n- [ ] open task\n- [x] done task\n"
    ));
}

#[test]
fn callouts_drawn_as_html() {
    insta::assert_snapshot!(page(
        "> [!NOTE]\n> Remember this.\n\n> [!WARNING] Careful **now**\n\n:::tip\n\nA colon callout.\n\n:::\n"
    ));
}

#[test]
fn pens_icons_and_underline_drawn_as_html() {
    insta::assert_snapshot!(page(
        "==yellow== and ==blue=={blue} and ++under++ with ::star:: and ::flag red::\n"
    ));
}

#[test]
fn tables_code_and_footnotes_drawn_as_html() {
    insta::assert_snapshot!(page(
        "| a | b |\n| --- | :-: |\n| 1 | 2 |\n\n```rust\nfn main() {}\n```\n\nText[^1].\n\n[^1]: The note.\n"
    ));
}

#[test]
fn front_matter_and_tags_drawn_as_html() {
    insta::assert_snapshot!(page(
        "---\ntitle: Plan\ntags: [work, home]\n---\n\n# Plan\n\nA #hashtag in text.\n"
    ));
}

#[test]
fn the_outline_of_a_document() {
    let doc = read("# One\n\n## Two\n\n### Three\n\n## Four\n");
    insta::assert_debug_snapshot!(outline(&doc));
}

#[test]
fn the_printed_form_of_a_document() {
    let doc = read("# Title\n\nSome ==marked== text.\n\n- [ ] task\n");
    insta::assert_json_snapshot!(printed(&doc));
}

#[test]
fn tisty_words_exported_as_standard_markdown() {
    insta::assert_snapshot!(standard(
        "# Title\n\n==blue=={blue} and ++under++ and ::star::\n"
    ));
}

#[test]
fn standard_markdown_turned_into_tisty_words() {
    insta::assert_snapshot!(to_tisty(&standard(
        "==blue=={blue} and ++under++ and ::star:: and ![p](plano-0006bf09.png)\n"
    )));
}
