use tisty_core::arriving::{from_pasted_html, from_text, tidied};

const EDGE_PAGE_HTML: &str = include_str!("fixtures/clipboard/edge-page.html");
const EDGE_PAGE_TXT: &str = include_str!("fixtures/clipboard/edge-page.txt");
const EDGE_COLORED_HTML: &str = include_str!("fixtures/clipboard/edge-colored.html");
const EDGE_CHAT_HTML: &str = include_str!("fixtures/clipboard/edge-chat.html");
const EDGE_CHAT_TXT: &str = include_str!("fixtures/clipboard/edge-chat.txt");
const CONHOST_TXT: &str = include_str!("fixtures/clipboard/powershell-conhost.txt");
const TERMINAL_TXT: &str = include_str!("fixtures/clipboard/windows-terminal.txt");
const TERMINAL_HTML: &str = include_str!("fixtures/clipboard/windows-terminal-formatted.html");
const TERMINAL_FORMATTED_TXT: &str =
    include_str!("fixtures/clipboard/windows-terminal-formatted.txt");

const BACKSLASH: char = '\\';

fn landed(html: &str) -> String {
    tidied(&from_pasted_html(html)).body.trim().to_string()
}

fn lines(body: &str) -> Vec<&str> {
    body.lines().collect()
}

fn position(body: &str, line: &str) -> usize {
    lines(body)
        .iter()
        .position(|l| *l == line)
        .unwrap_or_else(|| panic!("line {line:?} missing in:\n{body}"))
}

fn in_order(body: &str, wanted: &[&str]) {
    let mut from = 0;
    for want in wanted {
        let at = body[from..]
            .find(want)
            .unwrap_or_else(|| panic!("{want:?} missing or out of order in:\n{body}"));
        from += at + want.len();
    }
}

fn fenced_terminal(said: &str) {
    assert!(said.starts_with("```"), "{said}");
    assert!(said.trim_end().ends_with("```"), "{said}");
    assert!(!said.contains('\r'), "{said:?}");
    in_order(
        said,
        &["Windows PowerShell", "Name", "AggregatorHost", "PS C:\\"],
    );
}

#[test]
fn edge_page_html_keeps_every_mark() {
    let body = landed(EDGE_PAGE_HTML);
    assert!(body.contains("# Título principal"), "{body}");
    assert!(body.contains("## Sección número 2"), "{body}");
    assert!(body.contains("**negrita**"), "{body}");
    assert!(body.contains("*cursiva*"), "{body}");
    assert!(body.contains("[enlace](https://ejemplo.cl/ruta)"), "{body}");
    assert!(body.contains("`código_inline()`"), "{body}");
    assert_eq!(
        position(&body, "- Primero") + 1,
        position(&body, "  - Anidado uno"),
        "{body}"
    );
    assert!(body.contains("1. Paso uno"), "{body}");
    assert!(body.contains("| Nombre | Edad |"), "{body}");
    assert!(
        body.lines()
            .any(|l| l.starts_with('|') && l.contains("---") && !l.contains(char::is_alphanumeric)),
        "{body}"
    );
    assert!(body.contains("```"), "{body}");
    assert!(body.contains("fn main() {"), "{body}");
}

#[test]
fn edge_page_html_has_no_stray_escapes_or_spaces() {
    let body = landed(EDGE_PAGE_HTML);
    assert!(!body.contains('\u{a0}'), "{body:?}");
    for escape in [r"\#", r"\*", r"\["] {
        assert!(!body.contains(escape), "{escape} in {body}");
    }
    assert!(body.contains("==texto rojo=={red}"), "{body}");
    assert_eq!(body.matches("==").count(), 2, "{body}");
    assert_eq!(body.matches("=={").count(), 1, "{body}");
    assert!(body.contains("con **negrita**"), "{body:?}");
}

#[test]
fn edge_chat_html_stays_the_markdown_the_person_typed() {
    let body = landed(EDGE_CHAT_HTML);
    assert!(!body.contains(BACKSLASH), "{body}");
    let first = position(&body, "# Título");
    assert_eq!(position(&body, "- uno"), first + 1, "{body}");
    assert_eq!(position(&body, "- dos"), first + 2, "{body}");
    let rows = position(&body, "| A | B |");
    assert_eq!(position(&body, "|---|---|"), rows + 1, "{body}");
    assert_eq!(position(&body, "| 1 | 2 |"), rows + 2, "{body}");
    assert_eq!(position(&body, "| 3 | 4 |"), rows + 3, "{body}");
    let all = lines(&body);
    let rule = all.iter().rposition(|l| *l == "---").unwrap();
    assert_eq!(all[rule - 1], "", "{body}");
    assert_eq!(all[rule + 1], "", "{body}");
    assert_eq!(body.matches("# Título").count(), 2, "both messages: {body}");
}

#[test]
fn console_text_lands_as_one_fenced_block() {
    for plain in [CONHOST_TXT, TERMINAL_TXT, TERMINAL_FORMATTED_TXT] {
        fenced_terminal(&from_text(plain).expect("a console copy is fenced"));
    }
}

#[test]
fn formatted_terminal_html_lands_as_one_fenced_block() {
    let body = landed(TERMINAL_HTML);
    fenced_terminal(&body);
    assert_eq!(body.matches("```").count(), 2, "{body}");
    assert!(!body.contains("=="), "{body}");
}

#[test]
fn edge_plain_text_is_left_for_the_page_with_its_real_crlf() {
    assert_eq!(from_text(EDGE_PAGE_TXT), None);
    assert_eq!(from_text(EDGE_CHAT_TXT), None);
    assert!(EDGE_PAGE_TXT.contains("\r\n"));
    assert!(EDGE_CHAT_TXT.contains("\r\n"));
}

#[test]
fn a_no_break_space_between_tags_becomes_a_plain_space() {
    assert_eq!(landed("a<span>\u{a0}</span><strong>b</strong>"), "a **b**");
}

#[test]
fn a_styled_no_break_space_between_tags_is_still_a_plain_space() {
    for style in [
        "color: rgb(208, 0, 0);",
        "font-weight: 700;",
        "font-style: italic;",
    ] {
        let html = format!("a<span style=\"{style}\">\u{a0}</span><strong>b</strong>");
        assert_eq!(landed(&html), "a **b**", "{style}");
    }
}

#[test]
fn a_run_of_no_break_spaces_keeps_its_gap() {
    let body = landed("<p>Total:<span>\u{a0}\u{a0}\u{a0}</span>42</p>");
    assert!(body.contains("\u{a0}\u{a0}\u{a0}"), "{body:?}");
}

#[test]
fn a_rule_between_typed_markdown_lines_stays_markdown() {
    assert_eq!(
        landed("<div># T</div><hr><div>- a</div><div>- b</div>"),
        "# T\n\n---\n\n- a\n- b"
    );
}

#[test]
fn a_page_with_a_rule_and_no_markdown_goes_through_the_converter() {
    let body = landed("<p>uno</p><hr><p>dos</p>");
    assert!(body.contains("---"), "{body}");
    assert!(!body.contains(BACKSLASH), "{body}");
    assert!(body.contains("uno") && body.contains("dos"), "{body}");
}

#[test]
fn edge_colored_html_has_plain_spaces() {
    let body = landed(EDGE_COLORED_HTML);
    assert!(!body.contains('\u{a0}'), "{body:?}");
    assert!(body.contains("con **negrita**"), "{body}");
    assert!(body.contains("*cursiva*"), "{body}");
    assert!(body.contains("[enlace](https://ejemplo.cl)"), "{body}");
    assert_eq!(
        body,
        "Un párrafo con **negrita**, *cursiva* y [enlace](https://ejemplo.cl) final.\n\nTodo en negrita con *cursiva* dentro."
    );
}
