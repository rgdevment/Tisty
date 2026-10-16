use tisty_markdown::{html, read};

fn page(text: &str) -> String {
    html(&read(text))
}

#[test]
fn every_block_tells_the_bytes_it_was_written_in() {
    let drawn = page("# Title\n\nWords.\n");
    assert!(drawn.contains("data-at=\"0-8\">Title</h1>"));
    assert!(drawn.contains("<p data-at=\"9-16\">Words.</p>"));
}

#[test]
fn a_callout_shows_its_words_without_its_marker() {
    let drawn = page("> [!WARNING] Careful **now**\n");
    assert!(drawn.contains("data-callout=\"warning\""));
    assert!(drawn.contains("<strong>now</strong>"));
    assert!(!drawn.contains("[!WARNING]"));
}

#[cfg(feature = "pens")]
#[test]
fn a_yellow_pen_hides_the_equals_that_wrote_it() {
    let drawn = page("Some ==bright== words.\n");
    assert!(drawn.contains("<mark>bright</mark>"));
    assert!(!drawn.contains("=="));
}

#[test]
fn a_task_says_whether_it_is_done() {
    assert!(page("- [x] done\n").contains("data-done=\"true\""));
    assert!(page("- [ ] open\n").contains("data-done=\"false\""));
}

#[test]
fn code_is_shown_as_words_and_never_as_html() {
    let drawn = page("```\n<b>x</b>\n```\n");
    assert!(drawn.contains("&lt;b&gt;x&lt;/b&gt;"));
    assert!(!drawn.contains("<b>"));
}

#[test]
fn nothing_written_can_run_in_the_reader() {
    let drawn = page(
        "<script>alert(1)</script>\n\n<img src=x onerror=alert(2)>\n\n[a](javascript:alert(3)) ![b](javascript:alert(4))\n\nInline <span onclick=\"alert(5)\">c</span>.\n",
    );
    assert!(!drawn.contains("<script"));
    #[cfg(feature = "html")]
    assert!(!drawn.contains("onerror="));
    assert!(!drawn.contains("javascript:"));
    assert!(!drawn.contains("<span onclick"));
}

#[test]
fn only_plain_inline_tags_are_kept() {
    let drawn = page("Press <kbd>Ctrl</kbd> and <u>go</u>.\n");
    assert!(drawn.contains("<kbd>Ctrl</kbd>"));
    assert!(drawn.contains("<u>go</u>"));
}

#[test]
fn a_tag_left_open_is_shown_as_words() {
    let drawn = page("Some <u>words\n\nNext.\n");
    assert!(drawn.contains("&lt;u&gt;words"));
    assert!(!drawn.contains("<u>"));
}

#[test]
fn a_callout_marker_alone_on_its_line_leaves_only_the_body() {
    let drawn = page("> [!NOTE]\n> Only body.\n");
    assert!(drawn.contains("data-callout=\"note\""));
    assert!(drawn.contains(">Only body.</p>"));
    assert!(!drawn.contains("[!NOTE]"));
}

#[test]
fn a_link_with_no_address_yet_reads_as_its_words() {
    let drawn = page("See [the plan]() today.\n");
    assert!(drawn.contains("See <span>the plan</span> today."));
    assert!(!drawn.contains("href"));
}

#[test]
fn an_email_in_brackets_opens_the_mail() {
    assert!(page("Write <ana@example.com>.\n").contains("href=\"mailto:ana@example.com\""));
}

#[test]
fn inline_code_over_two_lines_of_a_quote_reads_as_one() {
    assert!(page("> `a\n> b` end\n").contains("<code>a b</code>"));
}

#[cfg(feature = "html")]
#[test]
fn html_inside_a_quote_leaves_the_quote_marks_out() {
    let drawn = page("> <div>\n> hi\n> </div>\n");
    assert!(drawn.contains("<div>\nhi\n</div>"));
    assert!(!drawn.contains("&gt; hi"));
}

#[cfg(feature = "math")]
#[test]
fn a_formula_keeps_every_symbol_between_its_dollars() {
    let drawn = page("Sum $a+b$ here.\n");
    assert!(drawn.contains("<mi>a</mi>"));
    assert!(drawn.contains("<mi>b</mi>"));
}

#[cfg(feature = "mermaid")]
#[test]
fn a_diagram_comes_light_and_dark_for_the_reader_to_pick() {
    let drawn = page("```mermaid\ngraph TD; A-->B\n```\n");
    assert!(drawn.contains("<img class=\"light\""));
    assert!(drawn.contains("<img class=\"dark\""));
    for src in drawn.split("src=\"").skip(1) {
        let url = src.split('"').next().unwrap_or_default();
        assert!(url.starts_with("data:image/svg+xml,"));
        assert!(!url.contains(' '));
    }
}

#[test]
fn a_path_on_a_windows_drive_is_a_place_and_not_a_scheme() {
    let drawn = page("[clip](<C:/Users/Mario/clip.mkv>) [b](vbscript:x)\n");
    assert!(drawn.contains("href=\"C:/Users/Mario/clip.mkv\""));
    assert!(!drawn.contains("vbscript:"));
}

#[test]
fn a_step_named_in_a_note_keeps_its_number_for_the_window() {
    let drawn = page("para arreglar [[#4]] ejecuté\nluego\n");
    assert!(drawn.contains("data-ref=\"#4\""));
    assert!(drawn.contains("<br>"));
}

#[test]
fn a_footnote_is_called_where_it_is_named_and_kept_at_its_place() {
    let drawn = page("A claim[^src] here.\n\n[^src]: The source.\n");
    assert!(drawn.contains("<sup class=\"fnref\"><a href=\"#fn-src\">src</a></sup>"));
    assert!(drawn.contains("<div class=\"footnote\" id=\"fn-src\""));
    assert!(drawn.contains("The source."));
}
