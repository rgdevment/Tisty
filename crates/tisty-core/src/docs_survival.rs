// Every case here was put through the real editor first: what it refuses is what came back
// damaged, and what it accepts is what came back byte for byte.
#[test]
fn what_the_editor_would_eat_is_refused_before_it_is_written() {
    for (body, why) in [
        ("---\ntitle: notes\n---\n\n<div>x</div>", "HTML"),
        ("<div class=\"warn\">careful</div>", "HTML"),
        (
            "# Note\n\nSomething <div style=\"x\">hidden</div> here.",
            "HTML",
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
        "# t\n\nSomething <!-- hidden --> more",
        "# t\n\nfoo &amp; bar",
        "see [the thread][one]\n\n[one]: https://example.com",
        "$$\nx^2\n$$",
        "```rust title=\"main.rs\" showLineNumbers\nfn main() {}\n```",
        "- > una cita\n- ```\n  x\n  ```",
        "Antes\n\n  ```sh\ngit cherry-pick <uno> <dos>\n```",
        "# t\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n",
        "---\n\nUna raya que nadie cierra no es un front matter.",
        "---\ntitle: notes\ntags: [a, b]\n---\n\nhello",
        "\u{feff}---\ntitle: notes\n---\n\nhello",
        "---\nnota: <b>x</b> y ::nope::\n---\n\nhello",
        "> ```html\n> <div>x</div>\n> ```\n",
        "# t\n\n````\ncode\n```\naun es codigo <div>x</div>\n````\n",
        "# t\n\n```\ncode\n~~~\naun es codigo <div>x</div>\n```\n",
        "",
    ] {
        assert_eq!(super::survives(body), Ok(()), "{body:?}");
    }
}
#[test]
fn the_corpus_is_read_the_way_it_says() {
    let raw = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/frail.json"),
    )
    .expect("the corpus of what survives a save");
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
            "HTML" => "html",
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

#[test]
fn a_colour_or_an_icon_the_language_has_not_got_is_refused() {
    for (body, why) in [
        ("un ==dato=={silver} ya", "a colour outside the palette"),
        ("llamar ::nope:: hoy", "an icon name it does not know"),
        ("el ::dog plaid:: duerme", "a colour outside the palette"),
        ("<mark data-pen=\"rojo\">x</mark>", "HTML"),
        ("<span data-ico=\"nope\"></span>", "HTML"),
    ] {
        assert_eq!(super::survives(body), Err(why), "{body:?}");
    }
}

#[test]
fn colours_and_icons_the_language_has_go_through_and_so_do_double_colons_in_words() {
    for body in [
        "un ==dato=={green} y ==otro==",
        "el ::dog orange:: y ::star::",
        "std::fs::write and a::b in prose",
        "`::nope::` in code",
        "<mark data-pen=\"green\">x</mark>",
    ] {
        assert_eq!(super::survives(body), Ok(()), "{body:?}");
    }
}
