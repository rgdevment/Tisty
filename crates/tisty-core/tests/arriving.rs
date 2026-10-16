use tisty_core::arriving::tidied;
use tisty_core::docs::survives;

fn plain(said: &str) -> String {
    let out = tidied(said);
    assert!(
        survives(&out.body).is_ok(),
        "what came out is still refused: {:?} -> {:?} ({:?})",
        said,
        out.body,
        survives(&out.body)
    );
    out.body
}

#[test]
fn front_matter_from_an_exported_note_is_taken_off() {
    let out = tidied("---\ntitle: Reunion\ntags: [a, b]\n---\n\n# Reunion\n\nlo dicho\n");

    assert_eq!(out.body, "# Reunion\n\nlo dicho\n");
    assert!(out.changed.contains(&"front matter"));
}

#[test]
fn html_that_markdown_can_say_is_said_in_markdown() {
    assert_eq!(
        plain("Un <b>fuerte</b> y un <i>suave</i> y <code>codigo</code>."),
        "Un **fuerte** y un *suave* y `codigo`.\n"
    );
    assert_eq!(plain("Algo <strong>asi</strong>."), "Algo **asi**.\n");
    assert_eq!(plain("Algo <del>fuera</del>."), "Algo ~~fuera~~.\n");
}

#[test]
fn html_that_markdown_cannot_say_is_dropped_rather_than_left_to_be_destroyed() {
    let out = tidied("<div class=\"callout\">Aviso</div>\n");

    assert_eq!(out.body, "Aviso\n");
    assert!(survives(&out.body).is_ok());
    assert!(out.changed.contains(&"HTML with nothing markdown can say"));
}

#[test]
fn the_tags_an_older_tisty_wrote_come_in_its_own_words() {
    let said = "Un <mark data-pen=\"blue\">azul</mark> y un <u>subrayado</u>.\n";

    let out = tidied(said);

    assert_eq!(out.body, "Un ==azul=={blue} y un ++subrayado++.\n");
    assert_eq!(
        out.changed,
        ["what an older Tisty wrote, in its own words now"]
    );
}

#[test]
fn the_standard_forms_the_language_adopts_come_in_as_they_were_written() {
    for said in [
        "Hola <!-- oculto --> y &amp; y &nbsp;fin\n",
        "Ver [el sitio][uno] y [otro][dos].\n\n[uno]: https://a.example\n[dos]: <https://b.example>\n",
        "Antes\n\n$$\n\\int_0^1 x\n$$\n\nDespues\n",
        "$$a^2 + b^2$$\n",
        "Ver esta formula $$x=1$$ en el medio.\n",
        "```js showLineNumbers title=\"a.js\"\nconst a = 1;\n```\n",
        "- # Revisar la duda\n- * No deberia mostrar eso\n1. > citado\n",
        "Antes\n\n<!--\noculto\n-->\n\nDespues\n",
        "esto es &amp;amp; y listo\n",
        "Antes\n\n  ```sh\ngit cherry-pick <uno> <dos>\n```\n\nDespues\n",
    ] {
        assert!(survives(said).is_ok(), "{said:?} is the premise");
        let out = tidied(said);
        assert_eq!(out.body, said, "nothing to fix is nothing to touch");
        assert!(out.changed.is_empty(), "{:?}", out.changed);
    }
}

#[test]
fn a_line_break_written_as_a_tag_stops_being_one() {
    assert_eq!(plain("una<br>otra<br />tercera"), "unaotratercera\n");
}

#[test]
fn a_fence_keeps_every_byte_it_holds() {
    let said = "```js\nconst a = \"<b>no</b> &amp; $$\";\n```\n";

    let out = tidied(said);

    assert_eq!(out.body, said, "a fence is not text to be tidied");
    assert!(out.changed.is_empty());
}

#[test]
fn a_document_that_was_already_plain_comes_out_untouched() {
    let said = "# Compras\n\n- leche\n- pan\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n";

    let out = tidied(said);

    assert_eq!(out.body, said);
    assert!(out.changed.is_empty(), "{:?}", out.changed);
}

#[test]
fn everything_it_hands_back_is_something_tisty_will_take() {
    for said in [
        "---\na: b\n---\n<div><b>hola</b> &amp; <!-- x --></div>\n",
        "$$x$$ y <br> y [a][b]\n\n[b]: https://x.example\n",
        "<span style=\"color:red\">rojo</span> y <mark>amarillo</mark>\n",
        "# t\n\n<table><tr><td>a</td></tr></table>\n",
    ] {
        let out = tidied(said);
        assert!(
            survives(&out.body).is_ok(),
            "{said:?} came out as {:?}, still refused for {:?}",
            out.body,
            survives(&out.body)
        );
    }
}

#[test]
fn a_fence_written_in_that_tisty_already_takes_is_left_where_it_was() {
    let said = "Antes\n\n  ```json\n  {\"a\": 1}\n```\n\nDespues\n";

    let out = tidied(said);

    assert_eq!(out.body, said, "nothing to fix is nothing to touch");
    assert!(out.changed.is_empty());
}

#[test]
fn a_bullet_whose_words_merely_start_with_a_dash_is_left_alone() {
    let said = "- -5 grados
- #hashtag suelto
";

    let out = tidied(said);

    assert_eq!(out.body, said);
    assert!(out.changed.is_empty(), "{:?}", out.changed);
}

#[test]
fn a_run_of_code_left_open_is_shut_rather_than_read_as_html() {
    let out = tidied(
        "El comienzo es `<?php
",
    );

    assert_eq!(
        out.body,
        "El comienzo es `<?php`
"
    );
    assert!(survives(&out.body).is_ok());
    assert!(out.changed.contains(&"a run of code left open"));
}

#[test]
fn a_card_is_what_names_a_page_and_a_link_to_it_is_a_mention() {
    let said = "[Marzo](tisty:doc/ab12-0002)";
    let card = "![Marzo](tisty:doc/ab12-0002)";

    assert!(
        tisty_core::refs::papers(said).is_empty(),
        "a sentence that points at a page is not the page's place in the book"
    );
    assert_eq!(
        tisty_core::refs::papers(card),
        vec!["ab12-0002".to_string()],
        "the shape Tisty writes, which the window draws as a card"
    );
    assert_eq!(
        tisty_core::refs::extract(said).len(),
        1,
        "the mention is still a reference, and still keeps what it points at"
    );
}

#[test]
fn what_is_written_between_backticks_is_not_tidied() {
    for said in [
        "Escribe `images/<id>.png` en la plantilla.

<div>fuera</div>
",
        "Usa `<b>bold</b>` como ejemplo.

<div>fuera</div>
",
        "Pon `<!-- nota -->` ahi.

<div>fuera</div>
",
        "El literal `a &amp; b` va tal cual.

<div>fuera</div>
",
    ] {
        let out = tidied(said);
        let kept = said.split('`').nth(1).unwrap();
        assert!(
            out.body.contains(kept),
            "{said:?} lost {kept:?}: {:?}",
            out.body
        );
        assert!(survives(&out.body).is_ok());
    }
}

#[test]
fn a_reference_named_inside_a_run_of_code_is_left_as_it_reads() {
    let out = tidied(
        "Ejemplo: `[texto][uno]` y de verdad [texto][uno].

[uno]: https://a.example

<div>x</div>
",
    );

    assert!(out.body.contains("`[texto][uno]`"), "{:?}", out.body);
    assert!(
        out.body.contains("[uno]: https://a.example"),
        "{:?}",
        out.body
    );
}

#[test]
fn the_blank_lines_a_code_block_holds_are_its_own() {
    let said = "<div>x</div>

```python
def a():
    pass


def b():
    pass
```
";

    let out = tidied(said);

    assert!(
        out.body.contains(
            "pass


def b()"
        ),
        "{:?}",
        out.body
    );
}

#[test]
fn a_document_tisty_already_takes_comes_back_byte_for_byte() {
    for said in [
        "# Sin salto final",
        "# Uno



muchas lineas en blanco
",
        "- uno
- dos",
    ] {
        assert!(survives(said).is_ok(), "{said:?} is the premise");
        let out = tidied(said);
        assert_eq!(out.body, said, "nothing to fix is nothing to touch");
        assert!(out.changed.is_empty());
    }
}

#[test]
fn a_rule_that_merely_looks_like_front_matter_keeps_what_is_under_it() {
    let said = "---

Un parrafo de verdad.

---

Otro despues del corte.

<div>x</div>
";

    let out = tidied(said);

    assert!(out.body.contains("Un parrafo de verdad"), "{:?}", out.body);
    assert!(!out.changed.contains(&"front matter"));
}

#[test]
fn a_type_with_angle_brackets_is_not_a_tag() {
    let out = tidied(
        "Vec<String> es un tipo

<div>x</div>
",
    );

    assert!(out.body.contains("Vec<String>"), "{:?}", out.body);
}

#[test]
fn a_tag_after_a_wide_space_is_fenced_rather_than_splitting_a_letter() {
    let said = tisty_core::arriving::tidied("uno\u{a0}<div> dos");

    assert!(said.body.contains("div"), "{}", said.body);
}

#[test]
fn a_colour_or_an_icon_tisty_has_not_got_comes_in_as_what_it_can_say() {
    let made = tidied("un ==dato=={silver} y ::nope:: con ::dog plaid:: y `::nope::`\n");
    assert_eq!(made.body, "un ==dato== y nope con ::dog:: y `::nope::`\n");
    assert!(
        made.changed
            .contains(&"a colour or an icon Tisty does not have")
    );
    assert!(tisty_core::docs::survives(&made.body).is_ok());
}
