use tisty_core::arriving::{arrived, tidied};
use tisty_markdown::to_standard;

const NOTE: &str = "spaces written as &nbsp;, as plain spaces now";

#[test]
fn an_interior_nbsp_run_arrives_as_plain_spaces_and_says_so() {
    let made = arrived("uno &nbsp;&nbsp;dos");
    assert_eq!(made.body.trim(), "uno   dos");
    assert!(made.changed.contains(&NOTE), "{:?}", made.changed);
}

#[test]
fn a_body_without_nbsp_gets_no_note() {
    let made = arrived("uno   dos");
    assert!(!made.changed.contains(&NOTE), "{:?}", made.changed);
}

#[test]
fn a_line_that_is_only_nbsp_stays_a_blank_line() {
    let made = arrived("uno\n\n&nbsp;\n\ndos");
    assert!(made.body.contains("&nbsp;"), "{:?}", made.body);
}

#[test]
fn a_leading_run_at_a_line_start_stays_as_written() {
    let made = arrived("uno\n\n&nbsp;&nbsp;&nbsp;&nbsp;dos &nbsp;tres");
    assert!(
        made.body.contains("&nbsp;&nbsp;&nbsp;&nbsp;dos  tres"),
        "{:?}",
        made.body
    );
}

#[test]
fn fenced_code_and_inline_code_keep_their_nbsp() {
    let made = arrived("```\na&nbsp;&nbsp;b\n```\n\ny `a&nbsp;&nbsp;b` z &nbsp;w");
    assert!(made.body.contains("a&nbsp;&nbsp;b\n```"), "{:?}", made.body);
    assert!(made.body.contains("`a&nbsp;&nbsp;b`"), "{:?}", made.body);
    assert!(made.body.contains("z  w"), "{:?}", made.body);
}

#[test]
fn what_goes_out_for_another_reader_comes_back_as_the_same_runs() {
    for original in [
        "uno   dos",
        "# Un   título",
        "uno  dos y   tres",
        "a **b  c** d",
    ] {
        let out = to_standard(original, &|_| None);
        assert!(out.contains("&nbsp;"), "{out:?}");
        assert_eq!(arrived(&out).body.trim(), original);
    }
}

#[test]
fn a_document_already_here_is_mended_without_turning_its_nbsp() {
    let made = tidied("uno &nbsp;&nbsp;dos");
    assert!(made.body.contains("&nbsp;&nbsp;"), "{:?}", made.body);
    assert!(!made.changed.contains(&NOTE), "{:?}", made.changed);
}
