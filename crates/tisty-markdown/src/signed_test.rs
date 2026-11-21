use super::{signer, signers};
use crate::{html, html_placed, read};

const PAGE: &str = "## Descripción\n\n**2026-10-28 09:00** · Mario\nLo que escribí yo.\n\n## Notas del asistente\n\n**2026-10-29 12:04** · Claude Code\nJira actualizado.\n\n#### Borrador\n\nTexto del borrador.\n\n**2026-10-30 08:15** · Copilot Cli\nOtra nota.\n\n## Al cerrarla\n\n**2026-10-31 10:00** · Lucas\n";

fn names(body: &str) -> Vec<String> {
    let doc = read(body);
    let mut found: Vec<_> = signers(&doc).into_values().collect();
    found.sort_by_key(|at| at.start);
    found
        .iter()
        .map(|at| body[at.clone()].to_string())
        .collect()
}

#[test]
fn reads_the_name_after_the_stamp() {
    assert_eq!(
        signer("**2026-10-29 12:04** · Claude Code"),
        Some("Claude Code")
    );
    assert_eq!(
        signer("**2026-10-29 12:04** · Claude Code\r"),
        Some("Claude Code")
    );
    assert_eq!(signer("**2026-10-29** · Claude Code"), None);
    assert_eq!(signer("**2026-10-29 12:04** · "), None);
    assert_eq!(signer("Claude Code dijo algo"), None);
}

#[test]
fn finds_the_signers_under_the_assistant_heading_only() {
    assert_eq!(names(PAGE), ["Claude Code", "Copilot Cli"]);
}

#[test]
fn reads_the_english_heading_as_well() {
    let page = "## Assistant notes\n\n**2026-10-29 12:04** · Codex\nDone.\n";
    assert_eq!(names(page), ["Codex"]);
}

#[test]
fn leaves_a_note_without_a_stamp_alone() {
    let page = "## Notas del asistente\n\nClaude Code dejó esto sin firma.\n";
    assert!(names(page).is_empty());
}

#[test]
fn draws_the_signer_whole_where_the_page_is_written_on() {
    let drawn = html_placed(&read(PAGE));
    let at = PAGE.find("Claude Code").unwrap();
    let span = format!(
        "<span class=\"signer\" data-by=\"Claude Code\" data-src=\"{at}-{}\" contenteditable=\"false\">Claude Code</span>",
        at + "Claude Code".len()
    );
    assert!(drawn.contains(&span), "{drawn}");
    assert_eq!(drawn.matches("class=\"signer\"").count(), 2);
}

#[test]
fn keeps_the_words_around_the_signer_where_they_are_written() {
    let drawn = html_placed(&read(PAGE));
    let at = PAGE.find("Jira actualizado").unwrap();
    assert!(drawn.contains(&format!("data-src=\"{at}-")), "{drawn}");
    assert!(!drawn.contains("data-by=\"Mario\""));
    assert!(!drawn.contains("data-by=\"Lucas\""));
}

#[test]
fn draws_a_name_written_in_pieces_as_one_signer() {
    for name in ["Codex *beta*", "A&B :dog:", "Kiro_2"] {
        let page = format!("## Assistant notes\n\n**2026-10-29 12:04** · {name}\nDone.\n");
        let drawn = html_placed(&read(&page));
        let at = page.find(name).unwrap();
        assert_eq!(drawn.matches("class=\"signer\"").count(), 1, "{drawn}");
        assert!(
            drawn.contains(&format!("data-src=\"{at}-{}\"", at + name.len())),
            "{drawn}"
        );
        assert!(!drawn.contains("<em>"), "{drawn}");
        assert!(drawn.contains("Done."), "{drawn}");
    }
}

#[test]
fn marks_the_signer_on_a_page_only_read() {
    let drawn = html(&read(PAGE));
    assert!(drawn.contains("<span class=\"signer\" data-by=\"Copilot Cli\">Copilot Cli</span>"));
    assert!(!drawn.contains("data-src"));
}
