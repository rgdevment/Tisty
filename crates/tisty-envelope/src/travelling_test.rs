use super::*;

fn places() -> Places {
    let mut out = Places::default();
    out.pictures.insert(
        "rina.png".into(),
        Placed {
            at: 0,
            width: 640,
            height: 480,
        },
    );
    out.files.insert("cartilla.pdf".into(), 1);
    out.documents.insert("enero".into(), Reached::Page(2));
    out.documents.insert("otro".into(), Reached::Cited(0));
    out
}

#[test]
fn a_formula_keeps_its_empty_parts_closed() {
    let html = tisty_markdown::html(&tisty_markdown::read(r"texto $a\,b$ fin"));
    assert!(ready(&html, &places()).contains(r#"<mspace width="0.16666667em"/><mi>b</mi>"#));
    assert_eq!(
        ready(r#"<span title=a/>x</span>"#, &places()),
        r#"<span title="a/">x</span>"#
    );
}

#[test]
fn an_address_is_told_from_a_title_that_repeats_it() {
    for (said, goes) in [
        (
            "[R](cartilla.pdf \"cartilla.pdf\")\n",
            "[R](tisty:file/1 \"cartilla.pdf\")\n",
        ),
        (
            "![a](rina.png \"rina.png\")\n",
            "![a](tisty:picture/0 \"rina.png\")\n",
        ),
        ("![rina.png](rina.png)\n", "![rina.png](tisty:picture/0)\n"),
        (
            "[![rina.png](rina.png)](cartilla.pdf \"rina.png\")\n",
            "[![rina.png](tisty:picture/0)](tisty:file/1 \"rina.png\")\n",
        ),
        ("[R](C:/m/s.pdf \"C:/m/s.pdf\")\n", "R\n"),
    ] {
        assert_eq!(body_ready(said, &places()), goes);
    }
}

#[test]
fn an_address_is_found_however_it_was_spelled() {
    let mut places = places();
    places.files.insert("un_archivo (1).pdf".into(), 3);
    places.files.insert("un_archivo.pdf".into(), 4);
    for (said, goes) in [
        ("[R](un\\_archivo.pdf)\n", "[R](tisty:file/4)\n"),
        ("[R](<un_archivo (1).pdf>)\n", "[R](<tisty:file/3>)\n"),
        ("[R](un\\_archivo.pdf \"t\")\n", "[R](tisty:file/4 \"t\")\n"),
    ] {
        assert_eq!(body_ready(said, &places), goes);
    }
}

#[test]
fn a_definition_leads_where_its_links_would() {
    for (said, goes) in [
        (
            "[R][r]\n\n[r]: cartilla.pdf\n",
            "[R][r]\n\n[r]: tisty:file/1\n",
        ),
        (
            "[R][r]\n\n[r]: cartilla.pdf \"cartilla.pdf\"\n",
            "[R][r]\n\n[r]: tisty:file/1 \"cartilla.pdf\"\n",
        ),
        ("[R][r]\n\n[r]: #titulo\n", "[R][r]\n\n[r]: #titulo\n"),
        (
            "[R][r]\n\n[r]: https://a.example\n",
            "[R][r]\n\n[r]: https://a.example\n",
        ),
    ] {
        assert_eq!(body_ready(said, &places()), goes);
    }
}

#[test]
fn a_definition_that_leads_nowhere_leaves_its_links_as_words() {
    for said in [
        "[R][r]\n\n[r]: C:/m/s.pdf\n",
        "[R][r]\n\n[r]: C:/m/s.pdf \"C:/m/s.pdf\"\n",
        "[R][r]\n\n[r]: un\\_secreto.pdf\n",
    ] {
        let goes = body_ready(said, &places());
        assert_eq!(goes.trim_end(), "R");
    }
    assert_eq!(body_ready("[r]\n\n[r]: s.pdf\n", &places()).trim_end(), "r");
}

#[test]
fn the_html_draws_no_address_that_leads_nowhere() {
    let drawn = |said: &str| {
        ready(
            &tisty_markdown::html(&tisty_markdown::read(said)),
            &places(),
        )
    };
    let gone = drawn("[R][r]\n\n[r]: C:/Users/m/secreto.pdf\n");
    assert!(!gone.contains("secreto"), "{gone}");
    assert!(!gone.contains("refdef"), "{gone}");
    let kept = drawn("[R][r]\n\n[r]: https://a.example\n");
    assert!(kept.contains("refdef-to"), "{kept}");
}

#[test]
fn a_picture_says_its_place_and_its_size() {
    assert_eq!(
        ready(
            r#"<img src="rina.png" alt="Rina" loading="lazy">"#,
            &places()
        ),
        r#"<img src="tisty:picture/0" alt="Rina" width="640" height="480" loading="lazy">"#
    );
}

#[test]
fn a_picture_that_cannot_go_travels_as_its_words() {
    assert_eq!(
        ready(
            r#"<p><img src="no.png" alt="Rina" loading="lazy"></p>"#,
            &places()
        ),
        "<p>Rina</p>"
    );
}

#[test]
fn a_picture_on_the_web_is_never_asked_for() {
    assert_eq!(
        ready(
            r#"<img src="https://a.example/x.png" alt="x" loading="lazy">"#,
            &places()
        ),
        r#"<a href="https://a.example/x.png">x</a>"#
    );
}

#[test]
fn a_file_is_a_card_with_its_place() {
    assert_eq!(
        ready(
            r#"<img src="cartilla.pdf" alt="Cartilla" loading="lazy" data-as="file">"#,
            &places()
        ),
        r#"<a class="card" href="tisty:file/1">Cartilla</a>"#
    );
    assert_eq!(
        ready(r#"<a href="cartilla.pdf">la cartilla</a>"#, &places()),
        r#"<a class="card" href="tisty:file/1">la cartilla</a>"#
    );
}

#[test]
fn a_document_is_reached_only_when_it_travels_too() {
    assert_eq!(
        ready(r#"<a class="card" data-doc="enero">Enero</a>"#, &places()),
        r#"<a class="card" href="tisty:page/2">Enero</a>"#
    );
    assert_eq!(
        ready(r#"<a href="tisty:doc/otro">Otro</a>"#, &places()),
        r#"<a class="card" href="tisty:cited/0">Otro</a>"#
    );
    assert_eq!(
        ready(r#"<a class="card" data-doc="nada">Nada</a>"#, &places()),
        "Nada"
    );
}

#[test]
fn a_task_leaves_no_trace_but_its_words() {
    assert_eq!(
        ready(r#"<a class="card" data-task="01M6">Vacunar</a>"#, &places()),
        "Vacunar"
    );
}

#[test]
fn a_reference_is_a_card_or_its_words() {
    assert_eq!(
        ready(
            r#"<span class="ref" data-ref="enero">enero</span>"#,
            &places()
        ),
        r#"<a class="card" href="tisty:page/2">enero</a>"#
    );
    assert_eq!(
        ready(
            r#"<span class="ref" data-ref="nada">nada</span> <span class="tag" data-tag="a">#a</span>"#,
            &places()
        ),
        r##"nada <span class="tag" data-tag="a">#a</span>"##
    );
}

#[test]
fn a_link_stays_only_where_it_still_leads() {
    assert_eq!(
        ready(
            r##"<a href="https://a.example/?a=1&amp;b=2">a</a><a href="#enero">b</a>"##,
            &places()
        ),
        r##"<a href="https://a.example/?a=1&amp;b=2">a</a><a href="#enero">b</a>"##
    );
    for said in [
        "otro.html",
        "C:\\Users\\a.md",
        "#",
        "//a.example",
        "tisty:task/1",
    ] {
        assert_eq!(
            ready(&format!("<a href=\"{said}\">a <em>b</em></a>"), &places()),
            "a <em>b</em>"
        );
    }
}

#[test]
fn nobody_reads_where_a_block_sat_in_the_words() {
    assert_eq!(
        ready(r#"<p data-at="1-2">a</p><hr data-at="3-4">"#, &places()),
        "<p>a</p><hr>"
    );
}

#[test]
fn what_is_kept_to_itself_stays_home() {
    let front = r#"<dl class="front"><div><dt>title</dt><dd>a</dd></div><div><dt>task</dt><dd>01M6</dd></div><div><dt>translationKey</dt><dd>x</dd></div></dl>"#;
    assert_eq!(
        ready(front, &places()),
        r#"<dl class="front"><div><dt>title</dt><dd>a</dd></div></dl>"#
    );
}

#[test]
fn what_stays_home_stays_in_any_capitals() {
    let said = "---\ntitle: a\nTask: 01M6\nTRANSLATIONKEY: k\n---\n# a\n";
    let html = ready(
        &tisty_markdown::html(&tisty_markdown::read(said)),
        &places(),
    );
    assert!(!html.contains("01M6"), "{html}");
    assert!(
        !html.to_ascii_lowercase().contains("translationkey"),
        "{html}"
    );
    assert!(html.contains("<dt>title</dt>"), "{html}");
    assert!(!body_ready(said, &places()).contains("01M6"));
}

#[test]
fn how_the_person_plans_their_work_stays_home() {
    let said = "---\ntitle: a\nauthor: Rina\nlist: Trabajo\nDue: 2026-12-01\nscheduled: 2026-11-30\nremind: [2026-11-29]\npriority: do\nrepeat: every week from due\n---\n# a\n";
    let html = ready(
        &tisty_markdown::html(&tisty_markdown::read(said)),
        &places(),
    );
    let body = body_ready(said, &places());
    for text in [&html, &body] {
        for gone in ["Trabajo", "2026", "every week", "priority"] {
            assert!(!text.contains(gone), "{gone} went along in {text}");
        }
        assert!(
            text.contains("Rina"),
            "what the person wrote for others stays: {text}"
        );
    }
}

#[test]
fn html_typed_by_hand_goes_as_the_drawn_html_does() {
    let said = "<!-- nota C:/secreto -->\n\n<img src=\"C:/secreto.png\" alt=\"foto\">\n\n<!-- otra -->\n<div><a href=\"C:/secreto.pdf\">R</a></div>\n\n<img src=\"rina.png\" alt=\"Rina\">\n";
    let goes = body_ready(said, &places());
    assert!(
        !goes.contains("secreto") && !goes.contains("nota"),
        "{goes}"
    );
    assert!(goes.contains("tisty:picture/0"), "{goes}");
}

#[test]
fn a_file_named_as_a_picture_is_a_card_in_the_words_too() {
    assert_eq!(
        body_ready("![a](cartilla.pdf)\n", &places()),
        "[a](tisty:file/1)\n"
    );
    assert_eq!(
        body_ready("![a](rina.png)\n", &places()),
        "![a](tisty:picture/0)\n"
    );
}

#[test]
fn a_quote_keeps_its_source_only_when_it_leads_somewhere() {
    assert_eq!(ready(r#"<q cite="x.html">a</q>"#, &places()), "<q>a</q>");
    assert_eq!(
        ready(r#"<q cite="https://a.example">a</q>"#, &places()),
        r#"<q cite="https://a.example">a</q>"#
    );
}

#[test]
fn a_path_never_stands_in_for_a_picture_that_could_not_go() {
    let html = r#"<img src="C:\Users\Mario\Fotos\rina.png" alt="" loading="lazy">"#;
    assert_eq!(ready(html, &places()), "");
}

#[test]
fn a_link_that_went_takes_its_address_with_it() {
    let html =
        "<p><a href=\"tisty:task/01M6\">tisty:task/01M6</a> y <a href=\"otro.html\">otro</a></p>";
    assert_eq!(ready(html, &places()), "<p> y otro</p>");
}

#[test]
fn only_the_properties_block_loses_what_stays_home() {
    let html = r#"<div><dt>task</dt><dd>a</dd></div><p>queda</p></dd></div>"#;
    assert_eq!(ready(html, &places()), html);
    let lone = r#"<dl class="front"><div><dt>task</dt><dd>01M6</dd></div></dl><p>a</p>"#;
    assert_eq!(ready(lone, &places()), "<p>a</p>");
}

#[test]
fn a_link_never_ends_up_inside_a_link() {
    let html =
        r#"<a href="https://a.example"><img src="cartilla.pdf" alt="Cartilla" loading="lazy"></a>"#;
    assert_eq!(
        ready(html, &places()),
        r#"<a href="https://a.example">Cartilla</a>"#
    );
}

#[test]
fn html_written_loosely_cannot_hang_it() {
    assert_eq!(ready("<p title=x>a</p>", &places()), "<p title=\"x\">a</p>");
    assert_eq!(ready("<p =>a</p>", &places()), "<p =>a</p>");
    assert_eq!(
        ready("<p title='a'>b</p>", &places()),
        "<p title=\"a\">b</p>"
    );
}

#[test]
fn the_words_keep_nothing_their_html_left_home() {
    let body = "---\ntitle: a\ntask: 01M6\ntranslationKey: x\n---\n![Rina](rina.png) ![Otra](C:\\fotos\\b.png) ![web](https://a.example/x.png)\n\n[la cartilla](cartilla.pdf) [enero](tisty:doc/enero) [nada](tisty:doc/nada) [fuera](https://a.example) [ruta](a/b.md) <tisty:task/01M6>\n\n![tarea](tisty:task/01M6) [[enero]] [[nada]]\n\n[def]: otro.html\n";
    let done = body_ready(body, &places());
    for gone in [
        "01M6",
        "translationKey",
        "C:\\",
        "fotos",
        "rina.png",
        "tisty:doc",
        "a/b.md",
        "otro.html",
        "[[",
    ] {
        assert!(!done.contains(gone), "{gone} in {done}");
    }
    for kept in [
        "title: a",
        "![Rina](tisty:picture/0)",
        "[web](https://a.example/x.png)",
        "[la cartilla](tisty:file/1)",
        "[enero](tisty:page/2)",
        "[fuera](https://a.example)",
    ] {
        assert!(done.contains(kept), "{kept} not in {done}");
    }
    assert!(done.contains("nada"));
    assert!(done.contains("tarea"));
}

#[test]
fn it_says_what_a_page_names_before_anything_is_found() {
    let found = named(
        r##"<img src="rina.png" alt="x"><a href="cartilla.pdf">a</a><a href="#b">b</a><a href="https://a.example">c</a><a class="card" data-doc="enero">d</a><a href="tisty:doc/otro">e</a><span class="ref" data-ref="tercero">f</span>"##,
    );
    assert_eq!(found.pictures, ["rina.png"]);
    assert_eq!(found.links, ["cartilla.pdf"]);
    assert_eq!(found.documents, ["enero", "otro", "tercero"]);
}
