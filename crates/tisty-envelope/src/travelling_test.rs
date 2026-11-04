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
fn a_quote_keeps_its_source_only_when_it_leads_somewhere() {
    assert_eq!(ready(r#"<q cite="x.html">a</q>"#, &places()), "<q>a</q>");
    assert_eq!(
        ready(r#"<q cite="https://a.example">a</q>"#, &places()),
        r#"<q cite="https://a.example">a</q>"#
    );
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
