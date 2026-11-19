use super::*;

fn said(body: &str) -> Vec<(usize, String)> {
    highlighted(body)
        .into_iter()
        .map(|one| (one.line, one.said))
        .collect()
}

#[test]
fn bold_and_pens_come_as_written_with_their_line() {
    let body =
        "# Plan\n\nThe walk is **cached per file** and ==read once=={green}.\n\nPlain words.\n";
    assert_eq!(
        said(body),
        [
            (3, "**cached per file**".to_string()),
            (3, "==read once=={green}".to_string()),
        ]
    );
}

#[test]
fn a_pen_inside_bold_comes_once_with_the_bold() {
    let body = "**all of ==this== matters**\n";
    assert_eq!(said(body), [(1, "**all of ==this== matters**".to_string())]);
}

#[test]
fn marks_in_lists_and_callouts_count_and_code_does_not() {
    let body = "- one **item**\n\n> [!NOTE]\n> a ==quoted== line\n\n```\n**not bold**\n```\n\n`**nor this**`\n";
    assert_eq!(
        said(body),
        [(1, "**item**".to_string()), (4, "==quoted==".to_string())]
    );
}

#[test]
fn italics_strikes_and_underlines_are_no_marks() {
    assert!(said("*soft* ~~gone~~ ++under++ plain\n").is_empty());
}
