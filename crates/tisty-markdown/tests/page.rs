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
