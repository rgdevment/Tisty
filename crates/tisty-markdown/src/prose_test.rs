use super::*;

fn words(source: &str) -> Vec<String> {
    prose(source)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[test]
fn plain_words_come_back_as_written() {
    assert_eq!(
        prose("Llevar a Rina al veterinario."),
        "Llevar a Rina al veterinario."
    );
}

#[test]
fn a_fenced_block_says_nothing() {
    let said = prose("Antes\n\n```html\n<span class=\"dog\">Roger</span>\n```\n\nDespués");
    assert_eq!(said, "Antes Después");
}

#[test]
fn an_indented_block_says_nothing() {
    assert_eq!(
        prose("Antes\n\n    let roger = dog;\n\nDespués"),
        "Antes Después"
    );
}

#[test]
fn inline_code_is_left_out_and_the_words_around_it_stay_apart() {
    assert_eq!(
        words("Escribe `<div>` y`span`listo"),
        ["Escribe", "y", "listo"]
    );
}

#[test]
fn raw_html_says_nothing_but_the_words_between_its_tags_do() {
    assert_eq!(words("<div class=\"note\">\n\nHola\n\n</div>"), ["Hola"]);
    assert_eq!(
        words("Un <span style=\"color:blue\">azul</span> claro"),
        ["Un", "azul", "claro"]
    );
}

#[test]
fn properties_at_the_top_say_nothing() {
    assert_eq!(
        prose("---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n"),
        "Ideas"
    );
}

#[test]
fn a_link_keeps_its_words_and_drops_where_it_goes() {
    assert_eq!(
        words("Ver [la guía](https://tisty.app/guia) hoy"),
        ["Ver", "la", "guía", "hoy"]
    );
}

#[test]
fn a_link_definition_says_nothing() {
    assert_eq!(
        words("Ver [guía][g]\n\n[g]: https://tisty.app/guia \"Guía\""),
        ["Ver", "guía"]
    );
}

#[test]
fn a_callout_drops_its_kind_and_an_idea_its_date_and_colour() {
    assert_eq!(
        words("> [!TIP]\n> Riega las plantas"),
        ["Riega", "las", "plantas"]
    );
    assert_eq!(
        words("> [!IDEA] 2026-07-01 16:09 · pink\n> Que las vacunas avisen antes"),
        ["Que", "las", "vacunas", "avisen", "antes"]
    );
}

#[test]
fn a_pen_keeps_its_words_and_drops_its_walls_and_colour() {
    assert_eq!(
        words("Lo ==importante=={blue} hoy"),
        ["Lo", "importante", "hoy"]
    );
}

#[test]
fn an_icon_says_nothing() {
    assert_eq!(words("Paseo ::dog orange:: diario"), ["Paseo", "diario"]);
}

#[cfg(feature = "tables")]
#[test]
fn a_column_type_says_nothing_but_its_heading_does() {
    let said = words("| Gasto {money} | Pagado |\n| --- | --- |\n| Arriendo | sí |");
    assert_eq!(said, ["Gasto", "Pagado", "Arriendo", "sí"]);
}

#[cfg(feature = "details")]
#[test]
fn folded_details_keep_their_summary_and_their_words() {
    let said = words(":::details Más detalles\n\nAdentro hay texto\n\n:::");
    assert_eq!(said, ["Más", "detalles", "Adentro", "hay", "texto"]);
    let said = words("<details>\n<summary>Ver más</summary>\n\nAdentro\n\n</details>");
    assert_eq!(said, ["Ver", "más", "Adentro"]);
}

#[cfg(feature = "details")]
#[test]
fn a_callout_written_between_colons_keeps_only_its_words() {
    assert_eq!(
        words(":::tip\n\nRiega las plantas\n\n:::"),
        ["Riega", "las", "plantas"]
    );
}

#[test]
fn a_rule_and_an_empty_text_say_nothing() {
    assert_eq!(prose("---"), "");
    assert_eq!(prose(""), "");
}

#[test]
fn lists_and_quotes_keep_every_word_in_order() {
    assert_eq!(words("- uno\n- [x] dos\n\n> tres"), ["uno", "dos", "tres"]);
}
