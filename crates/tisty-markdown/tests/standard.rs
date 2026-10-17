use tisty_markdown::{Toward, to_standard, to_tisty};

fn alone(text: &str) -> String {
    to_standard(text, &|_| None)
}

#[test]
fn a_reader_outside_tisty_gets_what_markdown_can_say_and_the_tags_any_reader_passes_over() {
    assert_eq!(
        alone("Un ==azul=={blue}, ==amarillo==, ++sub++ y ::flag red::"),
        "Un <mark data-pen=\"blue\">azul</mark>, <mark>amarillo</mark>, <ins>sub</ins> y <span data-ico=\"flag\" data-hue=\"red\">:flag:</span>"
    );
    let linked = "![p](plano-0006bf09.png) y [web](https://x.y)";
    assert_eq!(alone(linked), linked);
}

#[test]
fn what_a_person_wrote_as_code_or_plain_markdown_reaches_the_reader_untouched() {
    let teaching = "Así se subraya:\n\n```\n++hola++\n```\n\ny `==así==`";
    assert_eq!(alone(teaching), teaching);
    let whole = "# Título\n\n**fuerte** y *suave*\n\n- uno\n- dos\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\nsi a < b entonces";
    assert_eq!(alone(whole), whole);
}

#[test]
fn what_goes_out_comes_back_in_tisty_words_but_the_underline() {
    let body = "==azul=={blue} y ==sí== y ::star:: y ![p](plano-0006bf09.png)";
    assert_eq!(to_tisty(&alone(body)), body);
}

#[test]
fn a_single_line_break_stays_a_break() {
    assert_eq!(alone("uno\ndos\\\ntres"), "uno\\\ndos\\\ntres");
}

#[test]
fn colon_callouts_and_folds_become_the_forms_github_reads() {
    assert_eq!(
        alone(":::tip\n\nOjo con esto.\n\n:::"),
        "> [!TIP]\n>\n> Ojo con esto.\n>"
    );
    assert_eq!(
        alone(":::details Más\n\nAdentro.\n\n:::"),
        "<details>\n<summary>Más</summary>\n\nAdentro.\n\n</details>"
    );
    assert_eq!(
        alone("> [!NOTE] Antes de salir\n> Lleva agua."),
        "> [!NOTE]\n> Antes de salir\\\n> Lleva agua."
    );
}

#[test]
fn another_document_is_a_link_where_it_travels_and_its_words_where_it_does_not() {
    let body = "[[Viaje]] y [plan](tisty:doc/abc.md)\n\n![Capítulo](tisty:doc/def.md)";
    let toward = |to: Toward| match to {
        Toward::Ref("Viaje") => Some("02 Viaje.md".to_string()),
        Toward::Doc("def.md") => Some("01 Capítulo.md".to_string()),
        _ => None,
    };
    assert_eq!(
        to_standard(body, &toward),
        "[Viaje](<02 Viaje.md>) y plan\n\n[Capítulo](<01 Capítulo.md>)"
    );
}

#[test]
fn emoji_go_out_as_their_glyph() {
    assert_eq!(alone("listo :tada:"), "listo 🎉");
}

#[test]
fn every_tag_of_the_document_lands_in_its_front_matter() {
    assert_eq!(
        alone("Sobre #viaje y #casa"),
        "---\ntags: [viaje, casa]\n---\n\nSobre #viaje y #casa"
    );
    assert_eq!(
        alone("---\ntitle: Plan\ntags:\n  - casa\n---\nSobre #viaje"),
        "---\ntitle: Plan\ntags: [casa, viaje]\n---\nSobre #viaje"
    );
}
