// Every case here was put through the real editor first: what it refuses is what came back
// damaged, and what it accepts is what came back byte for byte.
#[test]
fn what_the_editor_would_eat_is_refused_before_it_is_written() {
    for (body, why) in [
        ("---\ntitle: notes\n---\n\nhello", "YAML frontmatter"),
        (
            "\u{feff}---\ntitle: notes\n---\n\nhello",
            "YAML frontmatter",
        ),
        ("<div class=\"warn\">careful</div>", "HTML"),
        (
            "# Note\n\nSomething <div style=\"x\">hidden</div> here.",
            "HTML",
        ),
        ("# t\n\nSomething <!-- hidden --> more", "HTML comments"),
        ("# t\n\nfoo &amp; bar", "HTML entities"),
        ("a claim[^1]\n\n[^1]: the source", "footnotes"),
        (
            "see [the thread][one]\n\n[one]: https://example.com",
            "reference links",
        ),
        ("> ```\n> code\n\n<div>real</div>", "HTML"),
        ("```text\n> ```\n```\n\n<div>real</div>", "HTML"),
    ] {
        assert_eq!(super::survives(body), Err(why), "{body:?}");
    }
}

#[test]
fn what_comes_back_untouched_goes_through() {
    for body in [
        "# Cartulinas\n\nRosa y palos de paleta.",
        "- one\n- two\n\n**bold** and `code`",
        "[a link](https://example.com) in a line",
        "# t\n\n<https://example.com> look",
        "# t\n\n```html\n<div class=\"warn\">hi</div>\n```\n",
        "# t\n\n```js\nconst a: Array<T> = []\n```\n",
        "# t\n\n```ini\n[section]: value\n```\n",
        "# t\n\n```\n[^1]: a note\n```\n",
        "# t\n\n    <div>indented is code too</div>\n",
        "# t\n\nuse `<T>` inline, and `&amp;` too",
        "# t\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n",
        "---\n\nUna raya que nadie cierra no es un front matter.",
        "> ```html\n> <div>x</div>\n> ```\n",
        "# t\n\n````\ncode\n```\naun es codigo <div>x</div>\n````\n",
        "# t\n\n```\ncode\n~~~\naun es codigo <div>x</div>\n```\n",
        "",
    ] {
        assert_eq!(super::survives(body), Ok(()), "{body:?}");
    }
}
#[test]
fn both_halves_read_the_same_corpus_the_same_way() {
    let raw = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/frail.json"),
    )
    .expect("the corpus the window reads too");
    let corpus: Vec<serde_json::Value> = serde_json::from_str(&raw).expect("a list of cases");

    for one in corpus {
        let text = one["text"].as_str().expect("text");
        let why: Vec<&str> = one["why"]
            .as_array()
            .expect("why")
            .iter()
            .filter_map(|it| it.as_str())
            .collect();
        let said = super::survives(text).err().map(|it| match it {
            "YAML frontmatter" => "front",
            "HTML" => "html",
            "HTML comments" => "comments",
            "HTML entities" => "entities",
            "maths written between dollars" => "maths",
            "footnotes" => "notes",
            "reference links" => "refs",
            "what a fence says after its language" => "fence",
            "a list item that opens on a block" => "block",
            other => other,
        });

        match said {
            None => assert!(
                why.is_empty(),
                "{text:?} was let through, corpus says {why:?}"
            ),
            Some(found) => assert!(
                why.contains(&found),
                "{text:?} was refused for {found:?}, corpus says {why:?}"
            ),
        }
    }
}
