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
fn front_matter_from_an_exported_note_comes_in_as_it_was_written() {
    let said = "---\ntitle: Reunion\ntags: [a, b]\n---\n\n# Reunion\n\nlo dicho\n";
    let out = tidied(said);
    assert_eq!(out.body, said);
    assert!(out.changed.is_empty(), "{:?}", out.changed);

    let out = tidied("---\nnota: <b>x</b>\n---\n\nun <b>fuerte</b>\n");
    assert_eq!(out.body, "---\nnota: <b>x</b>\n---\n\nun **fuerte**\n");
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
        "Antes <!-- <b>borrador</b> --> y\n\n<!--\n<div>viejo</div>\n-->\n",
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

#[test]
fn obsidian_forms_come_in_as_the_language_says_them() {
    let said = "> [!info]- Ojo\n> texto\n\nver [[Plan|el plan]] y ![[foto.png|300]] %%nota%%\n\n%%\nlargo\n%%\n\n`[[x|y]] %%no%%`\n\n```\n[[a|b]] %%c%%\n```\n";

    let out = tidied(said);

    assert_eq!(
        out.body,
        "> [!NOTE] Ojo\n> texto\n\nver [[Plan]] y ![foto.png](<foto.png>) <!--nota-->\n\n<!--\nlargo\n-->\n\n`[[x|y]] %%no%%`\n\n```\n[[a|b]] %%c%%\n```\n"
    );
    for what in [
        "callouts written the Obsidian way",
        "comments written the Obsidian way",
        "links to a note under another name",
        "files embedded the Obsidian way",
    ] {
        assert!(out.changed.contains(&what), "{what}: {:?}", out.changed);
    }
    assert!(survives(&out.body).is_ok(), "{:?}", out.body);
}

#[test]
fn what_is_already_tisty_passes_the_obsidian_step_untouched() {
    for said in [
        "> [!TIP]\n> riega\n",
        "> [!warning] Ojo\n> x\n",
        "ver [[Plan]] y [[#2]]\n",
        "> [!raro] queda como cita\n",
    ] {
        let out = tidied(said);
        assert_eq!(out.body, said);
        assert!(out.changed.is_empty(), "{said:?} {:?}", out.changed);
    }
    assert_eq!(
        tidied("> [!bug]+\n> falla\n").body,
        "> [!CAUTION]\n> falla\n"
    );
}

#[test]
fn a_page_of_html_comes_in_as_the_tisty_language() {
    let html = r#"<html><head><style>p{}</style><script>x()</script></head><body>
<h2>Plan</h2>
<p>Un <strong>dato</strong>, <em>otro</em>, <u>subrayado</u> y <a href="https://x.y">un enlace</a>.</p>
<ul><li>uno</li><li>dos</li></ul>
<p><mark>amarillo</mark> <mark style="background-color:#bbf7d0">verde</mark> <span style="color:#dc2626">rojo</span> <span style="color:#000">negro</span></p>
<details><summary>Más  datos</summary><p>adentro</p></details>
<table><tr><th>a</th><th>b</th></tr><tr><td>1</td><td>2</td></tr></table>
</body></html>"#;

    let said = tidied(&tisty_core::arriving::from_html(html)).body;

    for piece in [
        "## Plan",
        "**dato**",
        "*otro*",
        "++subrayado++",
        "[un enlace](https://x.y)",
        "uno",
        "==amarillo==",
        "==verde=={green}",
        "==rojo=={red}",
        ":::details Más datos",
        "adentro",
        "| a | b |",
    ] {
        assert!(said.contains(piece), "{piece:?} missing from {said}");
    }
    assert!(!said.contains("x()"), "{said}");
    assert!(
        !said.contains("=={black}") && !said.contains("==negro"),
        "{said}"
    );
    assert!(survives(&said).is_ok(), "{:?}", survives(&said));
}

#[test]
fn a_table_drawn_in_a_terminal_comes_in_as_a_table() {
    let said = "Esta semana\n\n┌────────┬──────┐\n│ Tarea  │ Para │\n├────────┼──────┤\n│ Pagar  │ Lunes│\n│ la luz │      │\n├────────┼──────┤\n│ Comprar│ Mié  │\n└────────┴──────┘\n\nfin";

    let out = tisty_core::arriving::from_plain(said).unwrap();

    assert_eq!(
        out,
        "Esta semana\n\n| Tarea | Para |\n| --- | --- |\n| Pagar la luz | Lunes |\n| Comprar | Mié |\n\nfin"
    );
    assert!(survives(&out).is_ok());
}

#[test]
fn lines_that_read_like_code_come_in_fenced_and_prose_is_left_alone() {
    let code =
        "use tisty_core::Paths;\n\nfn kept(body: &str) -> String {\n    retargeted(body)\n}\n";
    assert_eq!(
        tisty_core::arriving::from_plain(code).as_deref(),
        Some(
            "```rust\nuse tisty_core::Paths;\n\nfn kept(body: &str) -> String {\n    retargeted(body)\n}\n```"
        )
    );
    let quoting = "fn a() {\n    let b = \"```x```\";\n}\n";
    let out = tisty_core::arriving::from_plain(quoting).unwrap();
    assert!(
        out.starts_with("````rust\n") && out.ends_with("\n````"),
        "{out}"
    );
    let script = "const a = (b) => {\n  return b + 1;\n};\n";
    assert!(
        tisty_core::arriving::from_plain(script)
            .unwrap()
            .starts_with("```javascript\n")
    );
    let plain = "x = {\n  a;\n  b;\n}\n";
    assert!(
        tisty_core::arriving::from_plain(plain)
            .unwrap()
            .starts_with("```\n")
    );
    for prose in [
        "Hola, ¿cómo estás?\nTodo bien por acá.\nNos vemos mañana.",
        "- leche\n- pan\n- huevos {grandes}",
        "una línea; otra",
    ] {
        assert_eq!(tisty_core::arriving::from_plain(prose), None, "{prose:?}");
    }
}

#[test]
fn what_the_html_review_found_comes_in_right() {
    let from = tisty_core::arriving::from_html;
    assert!(
        from(r#"<p>Paga<span style="color: rgb(220, 38, 38);"> la luz </span>hoy</p>"#)
            .contains("Paga ==la luz=={red} hoy")
    );
    let code = from(
        r#"<pre><code><span style="color:#d73a49">let</span> a = <mark>1</mark>;</code></pre>"#,
    );
    assert!(code.contains("let a = 1;"), "{code}");
    assert!(!code.contains("=="), "{code}");
    let wrapped = from(r#"<form id="aspnetForm"><h1>Intranet</h1><p>texto</p></form>"#);
    assert!(
        wrapped.contains("# Intranet") && wrapped.contains("texto"),
        "{wrapped}"
    );
}

#[test]
fn what_the_plain_text_review_found_comes_in_right() {
    let from = tisty_core::arriving::from_plain;
    assert_eq!(from(".\n├── src\n│   └── main.rs\n└── Cargo.toml"), None);
    assert_eq!(
        from("Querida Ana;\nte escribo desde casa,\nnos vemos el lunes."),
        None
    );
    assert_eq!(
        from("┌───┬───┐\n│ a │ b │\n├───┼───┤\n│ 1 │ 3 │\n│ 2 │ 4 │\n└───┴───┘").as_deref(),
        Some("| a | b |\n| --- | --- |\n| 1 | 3 |\n| 2 | 4 |")
    );
    assert_eq!(
        from("┏━━━┳━━━┓\n┃ a ┃ b ┃\n┣━━━╋━━━┫\n┃ 1 ┃ 2 ┃\n┗━━━┻━━━┛").as_deref(),
        Some("| a | b |\n| --- | --- |\n| 1 | 2 |")
    );
}

#[test]
fn a_letter_a_diagram_and_a_bare_table_come_in_as_what_they_are() {
    let from = tisty_core::arriving::from_plain;
    assert_eq!(from("Hi team,\nlet me know what you think,\nthanks."), None);
    assert_eq!(
        from("┌─────┐   ┌─────┐\n│ API │──▶│ DB  │\n└─────┘   └─────┘"),
        None,
        "boxes drawn side by side are a drawing, not a table"
    );
    assert_eq!(
        from("┌───┬───┐\n│ a │ b │\n│ c │ d │\n└───┴───┘").as_deref(),
        Some("| a | b |\n| --- | --- |\n| c | d |")
    );
}

#[test]
fn a_copy_from_google_docs_keeps_its_bold_and_its_links_plain() {
    let docs = r#"<meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-1"><p dir="ltr"><span style="font-weight:700">Importante</span><span style="font-weight:400"> y </span><a href="https://x.y"><span style="color:#1155cc;text-decoration:underline">aquí</span></a></p></b>"#;

    let said = tisty_core::arriving::from_html(docs);

    assert!(
        said.contains("**Importante** y [aquí](https://x.y)"),
        "{said}"
    );
    assert!(
        !said.starts_with("**Importante** y [aquí](https://x.y)**"),
        "{said}"
    );
    assert!(!said.contains("=={blue}"), "{said}");
}

#[test]
fn a_picture_inside_the_page_and_a_clear_colour_leave_nothing_behind() {
    let said = tisty_core::arriving::from_html(
        r#"<p>Foto <img src="data:image/png;base64,AAAA" alt="logo"> y <span style="background-color: rgba(255, 255, 0, 0)">nada</span> y <span style="color: rgb(100%, 0%, 0%)">rojo</span></p>"#,
    );
    assert!(!said.contains("data:"), "{said}");
    assert!(said.contains("logo"), "{said}");
    assert!(!said.contains("==nada"), "{said}");
    assert!(said.contains("==rojo=={red}"), "{said}");
}

fn titles(all: &[(&str, &str)]) -> tisty_core::arriving::Titles {
    let docs: Vec<tisty_core::docs::Doc> = all
        .iter()
        .map(|(id, title)| tisty_core::docs::Doc {
            id: (*id).to_string(),
            title: (*title).to_string(),
        })
        .collect();
    tisty_core::arriving::Titles::of(&docs)
}

#[test]
fn a_note_named_by_title_links_the_one_document_that_bears_it() {
    let here = titles(&[("D1", "Plan de Rina"), ("D2", "Vacunas"), ("D3", "Vacunas")]);
    let out = tisty_core::arriving::tidied_among(
        "ver [[plan de rina]], [[Notas/Plan de Rina#Hoy|el plan]] y [[Vacunas]]\n\n![[Plan de Rina]]\n\n![[Otra nota]]\n\n![[foto.png]]\n",
        &here,
    );
    assert_eq!(
        out.body,
        "ver [plan de rina](tisty:doc/D1), [el plan](tisty:doc/D1) y [[Vacunas]]\n\n![Plan de Rina](tisty:doc/D1)\n\n[[Otra nota]]\n\n![foto.png](<foto.png>)\n"
    );
    for said in [
        "notes named by title, now linked to the document that bears it",
        "notes named by a title more than one document bears, left as references",
        "notes embedded by a title no document here bears, left as references",
        "files embedded the Obsidian way",
    ] {
        assert!(
            out.changed.contains(&said),
            "{said} was not told: {:?}",
            out.changed
        );
    }
}

#[test]
fn a_title_with_brackets_keeps_them_as_words_and_code_is_never_touched() {
    let here = titles(&[("D1", "Acta [borrador]")]);
    let out = tisty_core::arriving::tidied_among(
        "[[Acta [borrador]]] y [[Acta [borrador]|acta]] `[[Acta [borrador]]]`\n",
        &here,
    );
    assert!(
        out.body.ends_with("`[[Acta [borrador]]]`\n"),
        "{:?}",
        out.body
    );
    let plain = tisty_core::arriving::tidied_among("[[Sin Igual]]\n", &here);
    assert_eq!(plain.body, "[[Sin Igual]]\n");
    let dotted = titles(&[("D9", "Reunión 2024.10")]);
    let met = tisty_core::arriving::tidied_among("[[Reunión 2024.10]]\n", &dotted);
    assert_eq!(met.body, "[Reunión 2024.10](tisty:doc/D9)\n");
}

#[test]
fn a_page_break_written_the_latex_way_becomes_the_rule_that_ends_a_sheet() {
    let out = tisty_core::arriving::tidied("Uno\n\\newpage\nDos\n\n```tex\n\\pagebreak\n```\n");
    assert_eq!(out.body, "Uno\n\n---\n\nDos\n\n```tex\n\\pagebreak\n```\n");
    assert!(out.changed.contains(&"page breaks written the LaTeX way"));
    let indented = tisty_core::arriving::tidied("- Uno\n  \\newpage\n\n    \\pagebreak\n");
    assert!(!indented.body.contains("---"), "{:?}", indented.body);
}

#[test]
fn a_page_ended_by_a_style_in_html_comes_in_as_a_rule() {
    for html in [
        "<p>Uno</p><div style=\"page-break-after: always\"></div><p>Dos</p>",
        "<p>Uno</p><p style=\"break-before: page\">Dos</p>",
        "<p style=\"page-break-after:always\">Uno</p><p>Dos</p>",
    ] {
        let said = tisty_core::arriving::from_html(html);
        let (one, two) = (said.find("Uno").unwrap(), said.find("Dos").unwrap());
        let rule = said.find("---").expect("the sheet's end is kept");
        assert!(one < rule && rule < two, "{said:?}");
    }
    assert!(!tisty_core::arriving::from_html("<p style=\"color:red\">Uno</p>").contains("---"));
}
