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
fn the_four_tags_tisty_writes_itself_are_left_alone() {
    let said = "Un <mark data-pen=\"blue\">azul</mark> y un <u>subrayado</u>.\n";

    let out = tidied(said);

    assert_eq!(out.body, said);
    assert!(out.changed.is_empty(), "{:?}", out.changed);
}

#[test]
fn comments_and_entities_go() {
    let out = tidied("Hola <!-- oculto --> y &amp; y &nbsp;fin\n");

    assert_eq!(out.body, "Hola  y & y  fin\n");
    assert!(out.changed.contains(&"HTML comments"));
    assert!(out.changed.contains(&"HTML entities"));
}

#[test]
fn a_line_break_written_as_a_tag_stops_being_one() {
    assert_eq!(plain("una<br>otra<br />tercera"), "unaotratercera\n");
}

#[test]
fn links_written_by_reference_are_written_inline() {
    let out = tidied(
        "Ver [el sitio][uno] y [otro][dos].\n\n[uno]: https://a.example\n[dos]: <https://b.example>\n",
    );

    assert_eq!(
        out.body,
        "Ver [el sitio](https://a.example) y [otro](https://b.example).\n"
    );
    assert!(out.changed.contains(&"links written by reference"));
    assert!(survives(&out.body).is_ok());
}

#[test]
fn maths_between_dollars_moves_into_a_fence_that_draws() {
    let out = tidied("Antes\n\n$$\n\\int_0^1 x\n$$\n\nDespues\n");

    assert_eq!(out.body, "Antes\n\n```math\n\\int_0^1 x\n```\n\nDespues\n");
    assert!(out.changed.contains(&"maths written between dollars"));
    assert!(survives(&out.body).is_ok());
}

#[test]
fn maths_on_one_line_moves_too() {
    let out = tidied("$$a^2 + b^2$$\n");

    assert_eq!(out.body, "```math\na^2 + b^2\n```\n");
    assert!(survives(&out.body).is_ok());
}

#[test]
fn a_fence_keeps_every_byte_it_holds() {
    let said = "```js\nconst a = \"<b>no</b> &amp; $$\";\n```\n";

    let out = tidied(said);

    assert_eq!(out.body, said, "a fence is not text to be tidied");
    assert!(out.changed.is_empty());
}

#[test]
fn what_a_fence_says_after_its_language_is_dropped_because_the_editor_drops_it() {
    let out = tidied("```js showLineNumbers title=\"a.js\"\nconst a = 1;\n```\n");

    assert_eq!(out.body, "```js\nconst a = 1;\n```\n");
    assert!(survives(&out.body).is_ok());
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
fn a_fence_written_in_from_the_margin_comes_back_to_it() {
    let said = "Antes\n\n  ```sh\ngit cherry-pick <uno> <dos>\n```\n\nDespues\n";
    assert!(
        survives(said).is_err(),
        "a fence Tisty cannot see is why this matters"
    );

    let out = tidied(said);

    assert!(survives(&out.body).is_ok(), "{:?}", out.body);
    assert!(out.body.contains("\n```sh\n"), "{:?}", out.body);
    assert!(
        out.body.contains("<uno> <dos>"),
        "what it holds is untouched"
    );
    assert!(out.changed.contains(&"a fence written in from the margin"));
}

#[test]
fn a_fence_written_in_that_tisty_already_takes_is_left_where_it_was() {
    let said = "Antes\n\n  ```json\n  {\"a\": 1}\n```\n\nDespues\n";

    let out = tidied(said);

    assert_eq!(out.body, said, "nothing to fix is nothing to touch");
    assert!(out.changed.is_empty());
}

#[test]
fn a_bullet_that_opens_on_a_block_keeps_its_words() {
    for (said, want) in [
        (
            "- # Revisar la duda
",
            "- BAR# Revisar la duda
",
        ),
        (
            "- * No deberia mostrar eso
",
            "- BAR* No deberia mostrar eso
",
        ),
        (
            "1. > citado
",
            "1. BAR> citado
",
        ),
    ] {
        let out = tidied(said);
        assert_eq!(out.body, want.replace("BAR", "\\"));
        assert!(survives(&out.body).is_ok(), "{:?}", out.body);
    }
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
fn a_page_named_the_old_way_is_still_found() {
    let one = "[Marzo](tisty:doc/ab12-0002)";
    let old = "![Marzo](tisty:doc/ab12-0002)";

    assert_eq!(
        tisty_core::refs::papers(one),
        vec!["ab12-0002".to_string()],
        "a card written by hand, which the window reads too"
    );
    assert_eq!(
        tisty_core::refs::papers(old),
        vec!["ab12-0002".to_string()],
        "and the shape Tisty writes, which the window draws as a card"
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
        out.body.contains("[texto](https://a.example)"),
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
fn maths_between_dollars_inside_a_line_becomes_something_that_survives() {
    let out = tidied(
        "Ver esta formula $$x=1$$ en el medio.
",
    );

    assert!(survives(&out.body).is_ok(), "{:?}", out.body);
    assert!(
        out.body.contains("x=1"),
        "the maths is still there: {:?}",
        out.body
    );
    assert!(out.changed.contains(&"maths written between dollars"));
}

#[test]
fn maths_left_open_does_not_swallow_the_paragraphs_after_it() {
    let out = tidied(
        "Antes

$$
x = 1

Despues sin cerrar
",
    );

    assert!(survives(&out.body).is_ok(), "{:?}", out.body);
    let after = out.body.split("```").last().unwrap_or_default();
    assert!(
        after.contains("Despues sin cerrar"),
        "the paragraph was eaten: {:?}",
        out.body
    );
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
fn a_comment_that_spans_lines_is_taken_out_whole() {
    let out = tidied(
        "Antes

<!--
oculto
-->

Despues
",
    );

    assert!(!out.body.contains("oculto"), "{:?}", out.body);
    assert!(!out.body.contains("-->"), "{:?}", out.body);
    assert!(survives(&out.body).is_ok());
}

#[test]
fn entities_written_twice_over_are_resolved_all_the_way() {
    let out = tidied(
        "esto es &amp;amp; y listo
",
    );

    assert_eq!(
        out.body,
        "esto es & y listo
"
    );
    assert!(survives(&out.body).is_ok());
    assert_eq!(tidied(&out.body).body, out.body, "and it settles");
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
