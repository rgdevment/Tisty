use tisty_core::arriving::{from_pasted_html, from_plain, tidied};

const WORD_LIST: &str = "<p class=MsoListParagraphCxSpFirst style='mso-list:l0 level1 lfo1'><span style='mso-list:Ignore'>·<span>&nbsp;&nbsp; </span></span>Primero</p><p class=MsoListParagraphCxSpLast style='mso-list:l0 level1 lfo1'><span style='mso-list:Ignore'>·<span>&nbsp; </span></span>Segundo</p>";

const CODE_EDITOR: &str = "<div style=\"white-space: pre;\"><div><span style=\"color:#569cd6;\"># Título</span></div></div>";

#[test]
fn markdown_written_in_pasted_html_stays_markdown() {
    let said = tidied(&from_pasted_html("<p># Título</p><p>Texto **negrita**</p>")).body;
    assert!(said.starts_with("# Título"), "{said:?}");
    assert!(
        said.contains("**negrita**") && !said.contains(r"\*"),
        "{said:?}"
    );
}

#[test]
fn a_line_copied_from_a_code_editor_does_not_become_a_colour() {
    let said = tidied(&from_pasted_html(CODE_EDITOR)).body;
    assert!(!said.contains("=="), "{said:?}");
}

#[test]
fn a_plain_list_with_commas_is_left_as_written() {
    assert_eq!(
        from_plain("- rojo, verde, azul\n- uno, dos, tres\n- a, b, c"),
        None
    );
}

#[test]
fn a_word_list_comes_in_as_a_markdown_list() {
    let said = tidied(&from_pasted_html(WORD_LIST)).body;
    assert!(said.contains("- Primero\n- Segundo"), "{said:?}");
}
