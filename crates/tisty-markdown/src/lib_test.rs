use super::*;

fn kinds(text: &str) -> Vec<Kind> {
    read(text)
        .blocks()
        .iter()
        .map(|one| one.kind.clone())
        .collect()
}

fn items(text: &str) -> Vec<Kind> {
    read(text).blocks()[0]
        .inner
        .iter()
        .map(|one| one.kind.clone())
        .collect()
}

fn fenced(tongue: Option<&str>, title: Option<&str>) -> Kind {
    Kind::Code {
        tongue: tongue.map(str::to_string),
        title: title.map(str::to_string),
    }
}

#[test]
fn a_document_reads_as_its_blocks_in_order() {
    assert_eq!(
        kinds("# Title\n\nWords.\n\n---\n\n| a |\n| --- |\n| 1 |"),
        vec![
            Kind::Heading(1),
            Kind::Paragraph,
            Kind::Rule,
            Kind::Table {
                columns: vec![Column::default()]
            }
        ]
    );
}

#[test]
fn every_block_knows_the_bytes_it_came_from() {
    let doc = read("# Title\n\nWords here.\n");
    let said: Vec<&str> = doc.blocks().iter().map(|one| doc.said(one)).collect();
    assert_eq!(said, vec!["# Title\n", "Words here.\n"]);
}

#[test]
fn a_callout_is_told_from_a_plain_quote() {
    assert_eq!(
        kinds("> [!WARNING]\n> Careful."),
        vec![Kind::Callout(Callout::Warning)]
    );
    assert_eq!(kinds("> Just a quote."), vec![Kind::Quote]);
}

#[test]
fn a_fence_gives_its_tongue_and_its_title() {
    assert_eq!(
        kinds("```ts title=\"src/main.ts\"\nconst a = 1;\n```"),
        vec![fenced(Some("ts"), Some("src/main.ts"))]
    );
    assert_eq!(kinds("~~~\nplain\n~~~"), vec![fenced(None, None)]);
}

#[test]
fn a_title_keeps_the_quotes_and_backslashes_it_escaped() {
    assert_eq!(
        kinds("```rust title=\"say \\\"hi\\\"\"\nfn main() {}\n```"),
        vec![fenced(Some("rust"), Some("say \"hi\""))]
    );
    assert_eq!(
        kinds("```text title=\"C:\\\\dir\"\nx\n```"),
        vec![fenced(Some("text"), Some("C:\\dir"))]
    );
}

#[test]
fn only_a_word_of_its_own_names_the_title() {
    assert_eq!(
        kinds("```ts subtitle=\"x\"\ncode\n```"),
        vec![fenced(Some("ts"), None)]
    );
}

#[test]
fn a_task_item_says_whether_it_is_done() {
    assert_eq!(
        items("- [ ] open\n- [x] done\n- plain"),
        vec![
            Kind::Item { done: Some(false) },
            Kind::Item { done: Some(true) },
            Kind::Item { done: None },
        ]
    );
}

#[test]
fn a_task_item_in_a_loose_list_still_says_whether_it_is_done() {
    assert_eq!(
        items("- [ ] open\n\n- [x] done\n"),
        vec![
            Kind::Item { done: Some(false) },
            Kind::Item { done: Some(true) }
        ]
    );
}

#[test]
fn an_ordered_list_keeps_where_it_starts() {
    assert_eq!(
        kinds("3. three\n4. four"),
        vec![Kind::List { start: Some(3) }]
    );
}

#[test]
fn a_byte_order_mark_does_not_hide_the_first_block() {
    let doc = read("\u{feff}# Title\n\nWords\n");
    assert_eq!(doc.blocks()[0].kind, Kind::Heading(1));
    assert_eq!(doc.said(&doc.blocks()[0]), "# Title\n");
}

#[test]
fn a_callout_with_words_after_its_marker_is_still_a_callout() {
    assert_eq!(
        kinds("> [!WARNING] Careful here"),
        vec![Kind::Callout(Callout::Warning)]
    );
    assert_eq!(kinds("> [!note]"), vec![Kind::Callout(Callout::Note)]);
    assert_eq!(kinds("> [!WARNING]x"), vec![Kind::Quote]);
    assert_eq!(kinds("> [!ODD] words"), vec![Kind::Quote]);
}

#[test]
fn a_tongue_drops_its_escapes_and_a_wall_keeps_only_its_own_marks() {
    assert_eq!(
        kinds("```c\\+\\+\nint x;\n```"),
        vec![fenced(Some("c++"), None)]
    );
    assert_eq!(
        kinds("~~~`odd` title=\"x\"\ncode\n~~~"),
        vec![fenced(Some("`odd`"), Some("x"))]
    );
}

#[test]
fn a_title_counts_after_any_mark_that_is_not_part_of_a_word() {
    assert_eq!(
        kinds("```ts x-title=\"a\"\ncode\n```"),
        vec![fenced(Some("ts"), Some("a"))]
    );
}

#[test]
fn nested_blocks_sit_inside_their_parent() {
    let doc = read("> - one\n>   - two");
    let quote = &doc.blocks()[0];
    assert_eq!(quote.kind, Kind::Quote);
    assert_eq!(quote.inner[0].kind, Kind::List { start: None });
    assert!(quote.inner[0].at.start >= quote.at.start && quote.inner[0].at.end <= quote.at.end);
}

#[test]
fn putting_a_block_back_changes_only_its_bytes() {
    let mut doc = read("# Title\n\nOld words.\n\nAfter.\n");
    let at = doc.blocks()[1].at.clone();
    assert!(doc.put(at, "New words, *longer* now.\n"));
    assert_eq!(
        doc.text(),
        "# Title\n\nNew words, *longer* now.\n\nAfter.\n"
    );
    assert_eq!(doc.blocks().len(), 3);
}

#[test]
fn a_range_off_the_text_is_refused_and_changes_nothing() {
    let mut doc = read("ñandú\n");
    let was = doc.clone();
    assert!(!doc.put(1..2, "x"));
    assert!(!doc.put(3..99, "x"));
    let (from, to) = (4, 2);
    assert!(!doc.put(from..to, "x"));
    assert_eq!(doc, was);
}

#[test]
fn a_body_goes_to_disk_the_way_the_core_keeps_it() {
    assert_eq!(on_disk("\u{feff}a\r\nb"), "\u{feff}a\r\nb\n");
    assert_eq!(on_disk("already plain\n"), "already plain\n");
    assert_eq!(on_disk(""), "");
}

fn marks(text: &str) -> Vec<(Mark, String)> {
    let doc = read(text);
    doc.blocks()[0]
        .spans
        .iter()
        .map(|one| (one.mark.clone(), doc.text()[one.at.clone()].to_string()))
        .collect()
}

#[test]
fn a_paragraph_knows_every_mark_in_it_and_where_it_sits() {
    let found = marks("A **b**, *c*, ~~d~~, `e` and [f](https://x.y).");
    let wanted: Vec<(Mark, &str)> = vec![
        (Mark::Text, "A "),
        (Mark::Strong, "**b**"),
        (Mark::Text, "b"),
        (Mark::Text, ", "),
        (Mark::Emphasis, "*c*"),
        (Mark::Text, "c"),
        (Mark::Text, ", "),
        (Mark::Strike, "~~d~~"),
        (Mark::Text, "d"),
        (Mark::Text, ", "),
        (Mark::Code, "`e`"),
        (Mark::Text, " and "),
        (
            Mark::Link {
                to: "https://x.y".into(),
            },
            "[f](https://x.y)",
        ),
        (Mark::Text, "f"),
        (Mark::Text, "."),
    ];
    let wanted: Vec<(Mark, String)> = wanted
        .into_iter()
        .map(|(m, s)| (m, s.to_string()))
        .collect();
    assert_eq!(found, wanted);
}

#[test]
fn an_address_written_bare_is_a_link_without_the_full_stop_after_it() {
    let link = |to: &str| Mark::Link { to: to.into() };
    let found = marks("Ver https://x.y/a_(b) y (https://x.y/c).");
    assert!(found.contains(&(link("https://x.y/a_(b)"), "https://x.y/a_(b)".into())));
    assert!(found.contains(&(link("https://x.y/c"), "https://x.y/c".into())));

    let named = marks("[https://x.y](https://otro.z) y `https://z.w` y xhttps://q.r");
    let links: Vec<&Mark> = named
        .iter()
        .map(|(mark, _)| mark)
        .filter(|mark| matches!(mark, Mark::Link { .. }))
        .collect();
    assert_eq!(links, [&link("https://otro.z")]);

    for (said, to) in [
        ("==https://a.com==", "https://a.com"),
        ("==https://a.com=={green}", "https://a.com"),
        ("++https://a.com++", "https://a.com"),
        (
            "ver https://x.com/?a=1&amp;b=2 ya",
            "https://x.com/?a=1&b=2",
        ),
        ("ver https://a.com/a\\_b ya", "https://a.com/a_b"),
    ] {
        let found = marks(said);
        assert!(
            found.iter().any(|(mark, _)| *mark == link(to)),
            "{said:?} gave {found:?}"
        );
    }
}

#[test]
fn a_single_newline_breaks_the_line_and_html_is_kept_as_it_was_written() {
    let found = marks("One\ntwo <kbd>K</kbd>");
    assert_eq!(found[1], (Mark::Break, "\n".to_string()));
    assert!(found.contains(&(Mark::Html, "<kbd>".to_string())));
}

#[test]
fn code_holds_only_its_words_as_written() {
    let doc = read("> ```rust\n> fn *main*() {} #tag\n> ```");
    let code = &doc.blocks()[0].inner[0];
    let said: String = code
        .spans
        .iter()
        .map(|one| {
            assert_eq!(one.mark, Mark::Text);
            &doc.text()[one.at.clone()]
        })
        .collect();
    assert_eq!(said, "fn *main*() {} #tag\n");
}

#[test]
fn front_matter_is_one_block_of_its_own_that_names_and_files_the_document() {
    let said = "---\ntitle: \"Plan de marzo\"\ntags: [rina, Vet]\nfuente: notas\n---\n\n# Otro\n";
    let doc = read(said);
    assert_eq!(doc.blocks()[0].kind, Kind::Front);
    let held = &said[doc.blocks()[0].at.clone()];
    assert!(
        held.starts_with("---\ntitle:") && held.trim_end().ends_with("---"),
        "{held:?}"
    );
    assert_eq!(crate::front::title(said).as_deref(), Some("Plan de marzo"));
    assert_eq!(crate::front::tags(said), ["rina", "Vet"]);
    assert_eq!(
        crate::tags::tags_in(&format!("{said}y #perro")),
        ["rina", "vet", "perro"]
    );
}

#[test]
fn front_matter_lists_are_read_however_a_note_writes_them() {
    for said in [
        "---\ntags:\n  - uno\n  - dos\n---\n",
        "---\ntags:\n- uno\n- dos\n---\n",
        "---\ntags: uno, dos\n---\n",
        "---\ntags: \"#uno #dos\"\n---\n",
        "---\ntag: [uno, 'dos']\n...\n",
    ] {
        assert_eq!(crate::front::tags(said), ["uno", "dos"], "{said:?}");
    }
}

#[test]
fn only_a_closed_block_at_the_very_top_is_front_matter() {
    for said in [
        "---\n\ntitle: x\n---\n",
        "texto\n\n---\ntitle: x\n---\n",
        "---\ntitle: x\n",
        "# Uno\n\n---\n\nmas\n\n---\n",
    ] {
        assert_eq!(crate::front::span(said), None, "{said:?}");
        assert!(crate::front::title(said).is_none(), "{said:?}");
    }
    assert_eq!(crate::front::span("\u{feff}---\na: b\n---"), Some(3..15));
}

#[test]
fn front_matter_past_its_caps_is_kept_but_never_read() {
    let keys: String = (0..33).map(|n| format!("k{n}: v\n")).collect();
    let long = format!("title: {}\n", "a".repeat(501));
    let big = format!("nota: {}\n", "b ".repeat(2100));
    for inner in [keys, long, big] {
        let said = format!("---\ntitle: x\n{inner}tags: [a]\n---\n\n# Otro\n");
        assert!(crate::front::span(&said).is_some());
        assert!(crate::front::fields(&said).is_empty(), "{said:?}");
        assert!(crate::front::title(&said).is_none());
        assert!(crate::front::tags(&said).is_empty());
    }
}

#[test]
fn lines_fenced_by_rules_in_the_middle_of_a_page_stay_on_the_page() {
    for said in [
        "# Notas\n\nintro\n\n---\nReunión del lunes\n---\n\nmas\n",
        "> ---\n> dentro\n> ---\n",
        "- uno\n\n  ---\n  dos\n  ---\n",
    ] {
        let doc = read(said);
        let mut kinds = Vec::new();
        let mut walk = doc.blocks().iter().collect::<Vec<_>>();
        while let Some(one) = walk.pop() {
            kinds.push(one.kind.clone());
            walk.extend(one.inner.iter());
        }
        assert!(!kinds.contains(&Kind::Front), "{said:?}");
        assert!(html(&doc).contains("Reunión del lunes") || !said.contains("Reunión"));
    }
    let doc = read("---\ntitle: x\n---\n\n---\ny\n---\n");
    assert_eq!(doc.blocks()[0].kind, Kind::Front);
    assert_eq!(
        doc.blocks()
            .iter()
            .filter(|one| one.kind == Kind::Front)
            .count(),
        1
    );
}

#[test]
fn a_link_definition_stands_on_the_page_where_it_was_written() {
    let said = "Ver [el mapa][uno].\n\n[uno]: https://x.y/parque \"Abre el sábado\"\n\nfin\n";
    let doc = read(said);
    let kinds: Vec<&Kind> = doc.blocks().iter().map(|one| &one.kind).collect();
    assert_eq!(
        kinds,
        [
            &Kind::Paragraph,
            &Kind::Definition {
                label: "uno".into(),
                to: "https://x.y/parque".into()
            },
            &Kind::Paragraph
        ]
    );
    assert!(said[doc.blocks()[1].at.clone()].starts_with("[uno]: https://x.y/parque"));
    assert!(html(&doc).contains("class=\"refdef\""));

    let twice = read("[vet]: https://a.y\n\n[VET]: https://b.y\n");
    let defined: Vec<&Kind> = twice
        .blocks()
        .iter()
        .map(|one| &one.kind)
        .filter(|kind| matches!(kind, Kind::Definition { .. }))
        .collect();
    assert_eq!(
        defined.len(),
        2,
        "a label written twice keeps both lines on the page"
    );

    let held = read("- uno\n\n  [dos]: https://x.y\n");
    assert!(
        !held
            .blocks()
            .iter()
            .any(|one| matches!(one.kind, Kind::Definition { .. })),
        "one held inside a list is that list's to edit"
    );
}

#[test]
fn every_link_definition_after_a_list_stands_on_the_page() {
    let said = "- uno\n- dos [a][p] y [b][v].\n\n[p]: https://example.com/p\n[v]: https://example.com/v \"T\"\n\nFin.\n";
    let doc = read(said);
    let written: Vec<&str> = doc
        .blocks()
        .iter()
        .map(|one| &said[one.at.clone()])
        .collect();
    assert_eq!(
        written,
        [
            "- uno\n- dos [a][p] y [b][v].\n\n",
            "[p]: https://example.com/p",
            "[v]: https://example.com/v \"T\"",
            "Fin.\n"
        ]
    );
    assert_eq!(html(&doc).matches("class=\"refdef\"").count(), 2);

    let defined = |said: &str| {
        read(said)
            .blocks()
            .iter()
            .filter(|one| matches!(one.kind, Kind::Definition { .. }))
            .count()
    };
    assert_eq!(
        defined("- uno\n  - dos\n\n[p]: https://a.y\n[v]: https://b.y\n[w]: https://c.y\n"),
        3,
        "a nested list stretches no further than a flat one"
    );
    assert_eq!(
        defined("- uno\n\n  [dos]: https://x.y\n  [tres]: https://x.z\n\nFin\n"),
        0,
        "two held inside an item are still that list's to edit"
    );
}

#[test]
fn long_blocks_of_every_kind_read_draw_and_print_in_proportion() {
    let many = |piece: &dyn Fn(usize) -> String| (0..20_000).map(piece).collect::<String>();
    let cases = [
        many(&|n| {
            format!(
                "> line {n} **b**
"
            )
        }),
        many(&|n| {
            format!(
                "see [a{n}]

[a{n}]: https://x/{n}

"
            )
        }),
        many(&|n| format!("word {n} [l](https://x/{n}) ")),
        many(&|n| format!("w ==x{n}== ")),
        many(&|n| format!("::star:: :tada: {n} ")),
        many(&|n| format!("![a{n}](x{n}.png) ")),
        many(&|n| format!("see [[note {n}]] ")),
        many(&|n| format!("at https://x.example/{n} ")),
        format!(
            "![{}](x.png)",
            many(&|n| format!(
                "alt {n} `c`
"
            ))
        ),
    ];

    let started = std::time::Instant::now();
    for text in &cases {
        let doc = read(text);
        assert!(!html(&doc).is_empty());
        assert!(!format!("{:?}", printed(&doc)).is_empty());
    }
    // In proportion all of them take a few seconds even with coverage; asking every piece of every other one took many minutes.
    assert!(started.elapsed() < std::time::Duration::from_secs(60));
}

fn placed(html: &str, text: &str) -> Vec<String> {
    html.split("data-src=\"")
        .skip(1)
        .filter_map(|rest| {
            let (from, to) = rest.split_once('"')?.0.split_once('-')?;
            text.get(from.parse::<usize>().ok()?..to.parse::<usize>().ok()?)
                .map(str::to_string)
        })
        .collect()
}

#[test]
fn a_page_to_write_on_names_the_bytes_of_each_piece_of_words() {
    let text =
        "# Hi **you**\n\nA [link](https://x.y) and &copy; café\n\n- one\n\n```js\nlet a;\n```";
    let drawn = html_placed(&read(text));
    let pieces = placed(&drawn, text);
    for one in ["Hi ", "you", "A ", "link", "&copy;", "one", "let a;\n"] {
        assert!(
            pieces.iter().any(|piece| piece == one),
            "{one} in {pieces:?}"
        );
    }
    assert!(pieces.iter().any(|piece| piece.ends_with("café")));
    assert!(
        pieces
            .iter()
            .all(|piece| !piece.contains("**") && !piece.contains("]("))
    );
    assert!(drawn.contains("contenteditable=\"false\">&copy;"));
}

#[test]
fn a_piece_drawn_as_one_element_carries_its_own_place() {
    let text = "![Rina](rina.png)\n\nA `code` and &copy; here";
    let drawn = html_placed(&read(text));
    assert!(!drawn.contains("contenteditable=\"false\"><img"), "{drawn}");
    assert!(
        !drawn.contains("contenteditable=\"false\"><code>"),
        "{drawn}"
    );
    assert!(
        drawn.contains("contenteditable=\"false\">code</code>"),
        "{drawn}"
    );
    let pieces = placed(&drawn, text);
    assert!(pieces.iter().any(|piece| piece == "`code`"), "{pieces:?}");
    assert!(
        pieces.iter().any(|piece| piece == "![Rina](rina.png)"),
        "{pieces:?}"
    );
}

#[cfg(feature = "math")]
#[test]
fn a_formula_among_words_is_held_whole_by_a_span_as_mathml_takes_no_contenteditable() {
    let text = "Antes $x^2$ y $$a$$ fin";
    let drawn = html_placed(&read(text));
    let maths: Vec<&str> = drawn
        .split("<math")
        .skip(1)
        .filter_map(|rest| rest.split_once('>').map(|(opened, _)| opened))
        .collect();
    assert_eq!(maths.len(), 2, "{drawn}");
    assert!(
        maths.iter().all(|opened| !opened.contains("data-src")),
        "{drawn}"
    );
    assert_eq!(
        drawn.matches("contenteditable=\"false\"><math").count(),
        2,
        "{drawn}"
    );
    let pieces = placed(&drawn, text);
    for one in ["$x^2$", "$$a$$"] {
        assert!(pieces.iter().any(|piece| piece == one), "{pieces:?}");
    }
}

#[test]
fn the_page_to_read_carries_no_bytes() {
    assert!(!html(&read("# Hi **you**\n\n```js\nlet a;\n```")).contains("data-src"));
}

#[test]
fn a_page_to_write_on_looks_as_the_page_to_read() {
    let text = "# Hi **you**\n\nA [link](https://x.y) #tag\n\n> quoted";
    let bare = |html: String| {
        let mut out = String::new();
        let mut inside = false;
        for c in html.chars() {
            match c {
                '<' => inside = true,
                '>' => inside = false,
                _ if !inside => out.push(c),
                _ => {}
            }
        }
        out
    };
    assert_eq!(bare(html_placed(&read(text))), bare(html(&read(text))));
}

#[test]
fn an_edit_draws_again_only_the_blocks_around_it() {
    let text = "# One\n\nTwo words.\n\nThree.\n\nFour.\n\nFive.";
    let at = text.find("words").unwrap_or(0);
    let doc = read(text);
    let near = placed_near(&doc, at, at + 5, Some(doc.labels()), true);
    assert_eq!(near.count, 5);
    assert_eq!(near.drawn, 0..3);
    let spans = doc.blocks().iter().map(|one| [one.at.start, one.at.end]);
    assert_eq!(near.sum, spans_sum(spans.skip(3)));
    assert!(near.html.contains("Two words.") && near.html.contains("Three."));
    assert!(!near.html.contains("Four."));
    assert!(near.html.contains("data-src"));
}

#[test]
fn the_page_and_the_core_sum_the_places_of_blocks_alike() {
    assert_eq!(
        spans_sum([[0, 4], [6, 13], [15, 21]]),
        [0xc795_6160, 0x9bfc_67d1]
    );
    assert_eq!(spans_sum([]), [0x811c_9dc5, 0x9747_b28c]);
}

#[test]
fn a_heading_drawn_again_keeps_the_name_the_whole_page_gives_it() {
    let text = "# Uno\n\nA.\n\n# Uno\n\nB.\n\nC.\n\nD.\n";
    let doc = read(text);
    let at = text.rfind("# Uno").unwrap_or_default();
    let near = placed_near(&doc, at + 2, at + 3, Some(doc.labels()), true);
    let second = &crate::outline(&doc)[1].id;
    assert!(
        near.html.contains(&format!("id=\"{second}\"")),
        "{}",
        near.html
    );
    assert!(html_placed(&doc).contains(&format!("id=\"{second}\"")));
    let words = placed_near(
        &doc,
        text.len() - 2,
        text.len() - 1,
        Some(doc.labels()),
        true,
    );
    assert!(!words.html.contains("<h1"));
}

#[test]
fn the_page_and_the_core_sum_heading_names_alike() {
    assert_eq!(names_sum(["uno", "título"]), [0x950d_02f5, 0x5078_96ce]);
    assert_eq!(names_sum([]), [0x811c_9dc5, 0x9747_b28c]);
}

fn names_of(doc: &Doc) -> Vec<(usize, String)> {
    crate::outline(doc)
        .into_iter()
        .map(|head| (head.at.start, head.id))
        .collect()
}

type Drawn = (Range<usize>, u8, Option<String>);

fn drawn_heads(html: &str) -> Vec<Drawn> {
    let between = |tag: &str, from: &str| {
        let at = tag.find(from)? + from.len();
        Some(tag[at..].split('"').next()?.to_string())
    };
    html.split('<')
        .filter(|tag| tag.starts_with('h') && tag[1..].starts_with(|c: char| c.is_ascii_digit()))
        .filter_map(|tag| {
            let tag = tag.split('>').next()?;
            let level = tag[1..2].parse().ok()?;
            let place = between(tag, "data-at=\"")?;
            let (start, end) = place.split_once('-')?;
            let at = start.parse().ok()?..end.parse().ok()?;
            Some((at, level, between(tag, " id=\"")))
        })
        .collect()
}

fn drawn_names(html: &str) -> Vec<(usize, String)> {
    drawn_heads(html)
        .into_iter()
        .filter_map(|(at, _, id)| Some((at.start, id?)))
        .collect()
}

fn headings_within(blocks: &[Block], within: &Range<usize>, out: &mut Vec<(Range<usize>, u8)>) {
    for one in blocks {
        if let Kind::Heading(level) = one.kind
            && within.contains(&one.at.start)
        {
            out.push((one.at.clone(), level));
        }
        headings_within(&one.inner, within, out);
    }
}

struct Shown {
    doc: Doc,
    names: Vec<(usize, String)>,
    blocks: Vec<Block>,
    titles: Vec<bool>,
}

fn marked(one: &Block, title: Option<usize>) -> bool {
    title == Some(one.at.start)
}

// What the page shows after a key, as `patched` lays it, or nothing when it draws the whole page.
fn shown_after(was: &Doc, at: Range<usize>, insert: &str) -> Option<Shown> {
    let reached = crate::reaches_heading(was, at.clone());
    let doc = crate::reread_edit(was.clone(), at.clone(), insert).ok()?;
    let by = insert.len() as isize - at.len() as isize;
    let near = placed_near(
        &doc,
        at.start,
        at.start + insert.len(),
        Some(was.labels()),
        reached,
    );
    if near.whole {
        return None;
    }
    let old = was.blocks();
    let after = near.count.checked_sub(near.drawn.end)?;
    let kept = old.len().checked_sub(after)?;
    if kept < near.drawn.start {
        return None;
    }
    let edge = old
        .get(near.drawn.start)
        .map_or(was.text().len(), |one| one.at.start);
    let moved = |place: usize| {
        if place < edge {
            place
        } else {
            place.wrapping_add_signed(by)
        }
    };
    let standing = old[..near.drawn.start].iter().chain(&old[kept..]);
    if spans_sum(standing.map(|one| [moved(one.at.start), moved(one.at.end)])) != near.sum {
        return None;
    }
    let redrawn = old[near.drawn.start..kept]
        .first()
        .zip(old[near.drawn.start..kept].last())
        .map_or(0..0, |(first, last)| first.at.start..last.at.end);
    let (gone, stay): (Vec<_>, Vec<_>) = names_of(was)
        .into_iter()
        .partition(|(place, _)| redrawn.contains(place));
    let stays = names_sum(stay.iter().map(|(_, name)| name.as_str()));
    let carried = match near.heads {
        Some(heads) if heads != stays => return None,
        Some(_) => Vec::new(),
        None => {
            let mut heads = Vec::new();
            headings_within(old, &redrawn, &mut heads);
            let laid = drawn_heads(&near.html);
            let same = |(now, level, _): &Drawn, (at, was): &(Range<usize>, u8)| {
                level == was
                    && now.len() == at.len()
                    && (now.start == at.start || now.start == at.start.wrapping_add_signed(by))
            };
            if laid.len() != heads.len()
                || !laid.iter().zip(&heads).all(|(now, was)| same(now, was))
            {
                return None;
            }
            let named = |place: usize| {
                gone.iter()
                    .find(|one| one.0 == place)
                    .map(|one| one.1.clone())
            };
            laid.iter()
                .zip(heads)
                .filter_map(|((now, _, _), (at, _))| Some((now.start, named(at.start)?)))
                .collect()
        }
    };
    let mut names: Vec<_> = stay
        .into_iter()
        .map(|(place, name)| (moved(place), name))
        .chain(drawn_names(&near.html))
        .chain(carried)
        .collect();
    names.sort();
    let blocks = old[..near.drawn.start]
        .iter()
        .chain(&doc.blocks()[near.drawn.clone()])
        .chain(&old[kept..])
        .cloned()
        .collect();
    let (was_titled, now_titled) = (page::titled(old), page::titled(doc.blocks()));
    let redrawn = &doc.blocks()[near.drawn.clone()];
    assert_eq!(
        near.html.matches(" class=\"title\"").count(),
        redrawn.iter().filter(|one| marked(one, now_titled)).count()
    );
    let titles = old[..near.drawn.start]
        .iter()
        .map(|one| marked(one, was_titled))
        .chain(redrawn.iter().map(|one| marked(one, now_titled)))
        .chain(old[kept..].iter().map(|one| marked(one, was_titled)))
        .collect();
    Some(Shown {
        doc,
        names,
        blocks,
        titles,
    })
}

fn titled_as_whole(shown: &Shown) -> bool {
    let title = page::titled(shown.doc.blocks());
    let whole: Vec<bool> = shown
        .doc
        .blocks()
        .iter()
        .map(|one| marked(one, title))
        .collect();
    shown.titles == whole
}

// Blocks the page keeps only move, so what each holds is compared with its places set aside.
fn unplaced(blocks: &[Block]) -> Vec<Block> {
    blocks
        .iter()
        .map(|one| {
            let mut one = one.clone();
            one.at = 0..0;
            if let Kind::Table { columns } = &mut one.kind {
                for typed in columns
                    .iter_mut()
                    .filter_map(|column| column.typed.as_mut())
                {
                    *typed = 0..0;
                }
            }
            for span in &mut one.spans {
                span.at = 0..0;
            }
            one.inner = unplaced(&one.inner);
            one
        })
        .collect()
}

fn reads_as_whole(shown: &Shown) -> bool {
    unplaced(&shown.blocks) == unplaced(shown.doc.blocks())
}

#[test]
fn every_heading_named_is_drawn_where_the_page_looks_for_it() {
    for text in [
        include_str!("../../../app/src-tauri/resources/guide/es/muestra.md"),
        include_str!("../../../app/src-tauri/resources/guide/en/sample.md"),
        "# Uno\n\n> # Uno\n\n- # Uno\n\n::: note\n# Uno\n:::\n\nUno\n===\n\n#\n",
    ] {
        let doc = read(text);
        let mut drawn = drawn_names(&html_placed(&doc));
        drawn.sort();
        assert_eq!(drawn, names_of(&doc));
    }
}

#[test]
fn a_heading_renamed_put_in_or_taken_out_never_leaves_a_stale_name() {
    let twice = "# Uno\n\nA.\n\nB.\n\n# Uno\n\nC.\n";
    let after = "A.\n\nB.\n\n# Uno\n\nC.\n";
    for (text, at, insert) in [
        (twice, 2..5, "Dos"),
        (after, 0..0, "# Uno\n\n"),
        (twice, 0..2, ""),
        (twice, 0..7, ""),
    ] {
        let was = read(text);
        if let Some(shown) = shown_after(&was, at.clone(), insert) {
            assert_eq!(shown.names, names_of(&shown.doc), "{at:?} → {insert:?}");
        }
    }
    let alone = read("# Uno\n\nA.\n\nB.\n\n# Dos\n\nC.\n");
    assert!(
        shown_after(&alone, 5..5, "s").is_some(),
        "a name no other heading shares is laid without the whole page"
    );
}

#[test]
fn a_heading_that_comes_as_another_goes_beside_a_key_is_never_shown_with_the_other_name() {
    for (text, at, insert) in [
        ("> ~~~\n> # a\n> ~~~\n> # b\n", 2..3, ""),
        ("> ~~\n> # a\n> ~~~\n> # b\n", 2..2, "~"),
        ("# a\n\n```\n# b\n```\n\nUno.\n", 5..6, ""),
    ] {
        if let Some(shown) = shown_after(&read(text), at.clone(), insert) {
            assert_eq!(
                shown.names,
                names_of(&shown.doc),
                "{at:?} → {insert:?} in {text:?}"
            );
        }
    }
}

#[test]
fn a_fence_mark_typed_or_taken_beside_headings_never_leaves_a_stale_name() {
    const TEXTS: [&str; 3] = [
        "> ~~~\n> # a\n> ~~~\n> # b\n\n# a\n",
        "- ```\n  # a\n  ```\n  # b\n\n# b\n",
        "> ```\n> # a\n> # b\n> ```\n> # a\n",
    ];
    const MARKS: [&str; 5] = ["~", "`", "", "> ", "\n"];
    let mut state: u64 = 0xFE4C_E5ED;
    let mut next = |below: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % below.max(1) as u64) as usize
    };
    for text in TEXTS {
        for _ in 0..200 {
            let from = next(text.len());
            let to = (from + next(2)).min(text.len());
            let insert = MARKS[next(MARKS.len())];
            if let Some(shown) = shown_after(&read(text), from..to, insert) {
                let key = format!("{from}..{to} → {insert:?} in {text:?}");
                assert_eq!(shown.names, names_of(&shown.doc), "{key}");
            }
        }
    }
}

#[test]
fn a_key_beside_a_heading_keeps_the_name_the_page_shows_without_the_outline() {
    let was = read("# Uno\n\nA.\n\n# Uno\n\nB.\n");
    let at = was.text().find("B.").unwrap_or_default();
    let doc = crate::reread_edit(was.clone(), at..at, "x").unwrap_or_else(|doc| doc);
    let reached = reaches_heading(&was, at..at);
    let near = placed_near(&doc, at, at + 1, Some(was.labels()), reached);
    assert!(near.html.contains("<h1 data-at"));
    assert_eq!(near.heads, None);
    let shown = shown_after(&was, at..at, "x");
    assert!(shown.is_some_and(|one| one.names == names_of(&one.doc)));
}

#[test]
fn a_tag_that_pushes_another_past_the_cap_never_leaves_a_far_block_marked() {
    let many: String = (0..64).map(|n| format!("#n{n} ")).collect();
    let text = format!("Uno.\n\n{many}\n\nA.\n\nB.\n\nLejos #n63.\n");
    let shown = shown_after(&read(&text), 0..0, "#nuevo ");
    assert!(shown.is_none_or(|one| reads_as_whole(&one)));
    let far = read(&format!("Uno.\n\n```\n#n1\n```\n\n{many}\n"));
    assert!(far.labels() != read("Uno.\n").labels());
}

#[test]
fn a_link_or_footnote_defined_changed_or_taken_out_never_leaves_a_block_as_it_read_before() {
    let words = "Ver [x] y[^1].\n\nA.\n\nB.\n\nC.\n\n";
    let end = words.len();
    let linked = format!("{words}[x]: https://a.b\n");
    let noted = format!("{words}[^1]: nota\n");
    let to = linked.find("a.b").unwrap_or_default();
    let keys: [(&str, Range<usize>, &str); 8] = [
        (words, end..end, "[x]: https://a.b\n"),
        (&linked, end..linked.len(), ""),
        (&linked, to..to + 3, "c.d"),
        (&linked, end + 1..end + 2, "z"),
        (words, end..end, "> [x]: https://a.b\n"),
        (words, end..end, "[^1]: nota\n"),
        (&noted, end..noted.len(), ""),
        (&noted, end + 2..end + 3, "2"),
    ];
    for (text, at, insert) in keys {
        let shown = shown_after(&read(text), at.clone(), insert);
        assert!(
            shown.is_none_or(|one| reads_as_whole(&one)),
            "{at:?} → {insert:?} in {text:?}"
        );
    }
    let far = linked.find("B.").unwrap_or_default();
    assert!(
        shown_after(&read(&linked), far..far, "s").is_some(),
        "a key away from every definition is laid without the whole page"
    );
    let said = noted.find("nota").unwrap_or_default() + 4;
    assert!(
        shown_after(&read(&noted), said..said, "s").is_some(),
        "the words of a footnote are laid without the whole page"
    );
}

fn keyed(texts: &[&str], pieces: &[&str], seed: u64) -> (usize, usize) {
    let mut state = seed;
    let mut next = |below: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % below.max(1) as u64) as usize
    };
    let mut laid = 0;
    let mut all = 0;
    for text in texts {
        let mut was = read(text);
        for _ in 0..300 {
            let len = was.text().len();
            let mut from = next(len + 1);
            while !was.text().is_char_boundary(from) {
                from -= 1;
            }
            let mut to = (from + next(8)).min(len);
            while !was.text().is_char_boundary(to) {
                to -= 1;
            }
            let insert = if next(3) == 0 {
                ""
            } else {
                pieces[next(pieces.len())]
            };
            all += 1;
            if let Some(shown) = shown_after(&was, from..to, insert) {
                laid += 1;
                let key = format!("{from}..{to} → {insert:?} in {:?}", was.text());
                assert_eq!(shown.names, names_of(&shown.doc), "{key}");
                assert!(reads_as_whole(&shown), "{key}");
                assert!(titled_as_whole(&shown), "{key}");
            }
            let now = format!("{}{insert}{}", &was.text()[..from], &was.text()[to..]);
            was = read(&now);
        }
    }
    (laid, all)
}

#[test]
fn every_key_leaves_the_page_with_the_names_the_whole_page_gives() {
    let texts = [
        "# Uno\n\nA.\n\n# Uno\n\nB.\n\n## Uno 1\n\nC.\n\n# Dos\n\n> # Uno\n\nD.\n",
        "Uno\n===\n\nA.\n\n- # Uno\n\nB.\n\n# Uno\n\nÁrbol\n---\n",
        "> ~~~\n> # Uno\n> ~~~\n> # Dos\n> # Uno\n\nA.\n\n# Uno\n",
        "# Uno\n\n```\n# Dos\n```\n\n<div>\n# Uno\n</div>\n\n# Dos\n\nB.\n",
    ];
    let pieces = [
        "# ",
        "Uno",
        "Dos",
        "\n\n",
        "#",
        "===",
        "1",
        " ",
        "a",
        "> ",
        "# Uno\n\n",
        "~",
        "~~~\n",
        "```",
        "<div>\n",
    ];
    let (laid, all) = keyed(&texts, &pieces, 0x01D5_0F7E);
    assert!(
        laid * 2 > all,
        "only {laid} of {all} keys were laid without the whole page"
    );
}

#[test]
fn every_key_leaves_each_link_and_footnote_as_the_whole_page_reads_it() {
    let texts = [
        "Ver [x] y[^1].\n\nA [x][y].\n\n> B[^2].\n\n[x]: https://a.b\n\n- C [y].\n\n[^1]: nota\n",
        "Uno.\n\n::: note\nDos [x].\n:::\n\nTres[^1].\n\n> [y]: https://c.d\n\nCuatro [y].\n",
    ];
    let pieces = [
        "[x]",
        "[y]",
        "[x]: https://a.b\n",
        "[y]: u\n",
        "[^1]",
        "[^1]: n\n",
        "[^2]: m\n",
        ":::",
        "::: note\n",
        ":::\n",
        "\n\n",
        "> ",
        "a",
        ":",
        "]",
        "\n",
    ];
    let (laid, all) = keyed(&texts, &pieces, 0x00DE_F1AE);
    assert!(
        laid * 2 > all,
        "only {laid} of {all} keys were laid without the whole page"
    );
}

#[test]
fn the_block_right_after_the_properties_is_drawn_as_the_title() {
    for (text, tag) in [
        ("---\ntitle: x\n---\n\nHola.\n\nMás.\n", "p"),
        ("---\ntitle: x\n---\n\n### Hola.\n\nMás.\n", "h3"),
    ] {
        let doc = read(text);
        for drawn in [html(&doc), html_placed(&doc), pieces(&doc, 8).concat()] {
            assert_eq!(drawn.matches("class=\"title\"").count(), 1, "{drawn}");
            let at = drawn.find(&format!("<{tag}")).unwrap_or_default();
            let title = drawn[at..]
                .split(&format!("</{tag}>"))
                .next()
                .unwrap_or_default();
            assert!(
                title.contains("class=\"title\"") && title.contains("Hola."),
                "{drawn}"
            );
        }
    }
    for text in [
        "Hola.\n\nMás.\n",
        "---\ntitle: x\n---\n\n#### Hola\n\nMás.\n",
        "---\ntitle: x\n---\n\n- Hola.\n\nMás.\n",
        "Hola.\n\n---\ntitle: x\n---\n\nMás.\n",
        "> ---\n> title: x\n> ---\n>\n> Hola.\n",
    ] {
        assert!(!html(&read(text)).contains("class=\"title\""), "{text}");
    }
}

#[test]
fn a_key_near_the_properties_leaves_the_title_where_the_whole_page_draws_it() {
    let text = "---\ntitle: x\n---\n\nHola.\n\nMás.\n\nY más.\n";
    let hola = text.find("Hola.").unwrap_or_default();
    let closed = text.find("---\n\n").unwrap_or_default();
    let listed = "---\ntitle: x\n---\n\n- uno\n\nHola.\n\nMás.\n";
    let list = listed.find("- uno").unwrap_or_default();
    let made = shown_after(&read("title: x\n---\n\nHola.\n\nMás.\n"), 0..0, "---\n");
    assert!(made.is_none_or(|one| titled_as_whole(&one) && reads_as_whole(&one)));
    let keys: [(&str, Range<usize>, &str); 10] = [
        (text, hola..hola + 7, ""),
        (text, hola..hola, "Nuevo.\n\n"),
        (text, hola..hola, "- uno\n\n"),
        (text, hola + 2..hola + 2, "\n\n"),
        (text, hola + 5..hola + 7, ""),
        (text, hola..hola + 5, ""),
        (text, 0..1, ""),
        (text, closed..closed + 1, ""),
        (text, 0..hola, ""),
        (listed, list..list + 7, ""),
    ];
    for (text, at, insert) in keys {
        let shown = shown_after(&read(text), at.clone(), insert);
        let key = format!("{at:?} → {insert:?} in {text:?}");
        let shown = shown.unwrap_or_else(|| panic!("drawn whole instead of laid: {key}"));
        assert!(titled_as_whole(&shown) && reads_as_whole(&shown), "{key}");
    }
}

#[test]
fn every_key_near_the_properties_leaves_the_title_the_whole_page_draws() {
    let texts = [
        "---\ntitle: x\n---\n\nHola.\n\nMás.\n\n# Uno\n\nY más.\n",
        "---\ntags: [a]\n---\n\n# Hola\n\nMás.\n\n- uno\n\nFin.\n",
        "Hola.\n\n---\n\nMás.\n\n---\na: b\n---\n",
    ];
    let pieces = [
        "---\n",
        "---",
        "-",
        "title: x\n",
        "\n\n",
        "\n",
        "Hola",
        "# ",
        "- ",
        "a",
        "Nuevo.\n\n",
    ];
    let (laid, all) = keyed(&texts, &pieces, 0x7171_E5ED);
    assert!(
        laid * 2 > all,
        "only {laid} of {all} keys were laid without the whole page"
    );
}

#[test]
fn an_edit_past_the_last_block_draws_the_last_one() {
    let text = "One.\n\nTwo.\n\n";
    let doc = read(text);
    let near = placed_near(&doc, text.len(), text.len(), Some(doc.labels()), true);
    assert_eq!(near.drawn, 1..2);
    assert!(near.html.contains("Two."));
    assert_eq!(placed_near(&read(""), 0, 0, None, true).drawn, 0..0);
}

fn drawn_at(text: &str, from: usize, to: usize) -> Range<usize> {
    placed_near(&read(text), from, to, None, true).drawn
}

fn spans_of(text: &str) -> Vec<Range<usize>> {
    read(text)
        .blocks()
        .iter()
        .map(|one| one.at.clone())
        .collect()
}

#[test]
fn a_key_draws_the_block_it_touches_whichever_side_of_the_key_the_block_lies() {
    let middle = "One.\n\nTwo.\n\nThree.\n\n four.\n\nFive.\n\nSix.\n";
    assert_eq!(
        spans_of(middle),
        [0..5, 6..11, 12..19, 21..27, 28..34, 35..40]
    );
    assert_eq!(drawn_at(middle, 20, 20), 2..4);
    assert_eq!(drawn_at(middle, 19, 19), 1..4);
    assert_eq!(drawn_at(middle, 21, 21), 2..5);
    assert_eq!(drawn_at(middle, 20, 21), 2..5);
    assert_eq!(drawn_at(middle, 27, 27), 2..5);
    assert_eq!(drawn_at(middle, 28, 28), 3..6);
    let first = " One.\n\nTwo.\n\nThree.\n\nFour.\n";
    assert_eq!(spans_of(first)[0], 1..6);
    assert_eq!(drawn_at(first, 0, 0), 0..2);
    assert_eq!(drawn_at(first, 1, 1), 0..2);
    let last = "One.\n\nTwo.\n\nThree.\n\n four.\n";
    assert_eq!(spans_of(last)[3], 21..27);
    assert_eq!(drawn_at(last, 20, 20), 2..4);
    assert_eq!(drawn_at(last, 27, 27), 2..4);
    assert_eq!(drawn_at(&last[..20], 20, 20), 2..3);
    let titled = "---\ntitle: x\n---\n\n Hola.\n\n más.\n\nY más.\n";
    assert_eq!(spans_of(titled), [0..17, 19..25, 27..33, 34..42]);
    assert_eq!(drawn_at(titled, 18, 18), 0..2);
    assert_eq!(drawn_at(titled, 26, 26), 1..3);
    let blank = "One.\n\nTwo.\n\n  \n\nThree.\n\nFour.\n";
    assert_eq!(spans_of(blank), [0..5, 6..11, 16..23, 24..30]);
    assert_eq!(drawn_at(blank, 13, 13), 1..3);
    assert_eq!(drawn_at(blank, 15, 15), 1..3);
    let headed = "Uno.\n\nDos.\n\n Tres\n===\n";
    let heads = |from, to, reached| placed_near(&read(headed), from, to, None, reached).heads;
    assert!(heads(12, 12, true).is_some());
    assert_eq!(heads(12, 12, false), None);
    assert_eq!(heads(6, 7, false), None);
    assert!(heads(14, 15, false).is_some());
}

#[test]
fn a_block_left_starting_with_a_space_is_laid_without_the_whole_page() {
    let keys = [
        ("# Uno\n\nA.\n\nDos palabras.\n\nTres.\n", "Dos"),
        ("Uno.\n\nA palabras.\n\nTres.\n", "A"),
        ("Dos palabras.\n\nTres.\n\nCuatro.\n", "Dos"),
        ("Uno.\n\nTres.\n\nDos palabras.\n", "Dos"),
        ("---\ntitle: x\n---\n\nDos palabras.\n\nTres.\n", "Dos"),
        ("# Uno\n\nA.\n\nDos palabras\n===\n\nTres.\n", "Dos"),
        ("# Uno\n\nA.\n\nDos\npalabras.\n\nTres.\n", "Dos"),
        ("# Uno\n\nA.\n\nDos \npalabras.\n\nTres.\n", "Dos"),
    ];
    for (text, word) in keys {
        let key = format!("{word:?} taken out of {text:?}");
        let at = text.find(word).unwrap_or_else(|| panic!("no {key}"));
        let shown = shown_after(&read(text), at..at + word.len(), "");
        let shown = shown.unwrap_or_else(|| panic!("drawn whole instead of laid: {key}"));
        assert!(reads_as_whole(&shown) && titled_as_whole(&shown), "{key}");
        assert_eq!(shown.names, names_of(&shown.doc), "{key}");
    }
}

#[test]
fn a_paste_that_leaves_a_heading_starting_with_a_space_keeps_its_name_without_the_whole_page() {
    let text = "# Uno\n\nA.\n\nUno.\n\nDos palabras\n===\n\nTres.\n";
    let from = text
        .find("no.")
        .unwrap_or_else(|| panic!("no \"no.\" in {text:?}"));
    let to = text
        .find(" palabras")
        .unwrap_or_else(|| panic!("no palabras in {text:?}"));
    let shown = shown_after(&read(text), from..to, "X.\n\n");
    let shown = shown.unwrap_or_else(|| panic!("drawn whole instead of laid"));
    assert_eq!(
        shown.doc.text(),
        "# Uno\n\nA.\n\nUX.\n\n palabras\n===\n\nTres.\n"
    );
    assert!(reads_as_whole(&shown) && titled_as_whole(&shown));
    assert_eq!(shown.names, names_of(&shown.doc));
}

#[test]
fn a_key_on_a_blank_line_above_a_heading_reads_no_outline_and_the_heading_keeps_its_name() {
    let atx = "Uno.\n\nDos.\n\n\n# Tres\n\nCuatro.\n";
    let spaced = "Uno.\n\nDos.\n\n \n# Tres\n\nCuatro.\n";
    let setext = "Uno.\n\n\nTres\n===\n\nCuatro.\n";
    let keys: [(&str, Range<usize>, &str); 5] = [
        (atx, 12..12, "\n"),
        (atx, 12..12, " "),
        (spaced, 12..13, ""),
        (setext, 6..6, "\n"),
        (setext, 6..6, " "),
    ];
    for (text, at, insert) in keys {
        let key = format!("{at:?} → {insert:?} in {text:?}");
        let was = read(text);
        let doc = crate::reread_edit(was.clone(), at.clone(), insert).unwrap_or_else(|doc| doc);
        let reached = reaches_heading(&was, at.clone());
        assert!(!reached, "{key}");
        let to = at.start + insert.len();
        let near = placed_near(&doc, at.start, to, Some(was.labels()), reached);
        assert_eq!(near.heads, None, "{key}");
        let shown = shown_after(&was, at, insert);
        let shown = shown.unwrap_or_else(|| panic!("drawn whole instead of laid: {key}"));
        assert_eq!(shown.names, names_of(&shown.doc), "{key}");
    }
}

#[test]
fn a_key_reaches_a_heading_from_its_first_byte_to_the_line_break_after_it() {
    let text = "Uno.\n\n# Dos\n\nTres.\n";
    assert_eq!(spans_of(text), [0..5, 6..12, 13..19]);
    let was = read(text);
    assert!(!reaches_heading(&was, 5..5));
    assert!(reaches_heading(&was, 5..6));
    assert!(reaches_heading(&was, 12..12));
    assert!(!reaches_heading(&was, 13..13));
    assert!(reaches_heading(&was, 0..19));
    let lines = "Uno.\n\nDos\npalabras\n===\n\nTres.\n";
    let at = lines.find("Dos").unwrap_or_default();
    assert!(reaches_heading(&read(lines), at..at + 3));
    let shown = shown_after(&read(lines), at..at + 3, "");
    let shown = shown.unwrap_or_else(|| panic!("drawn whole instead of laid"));
    assert_eq!(shown.names, names_of(&shown.doc));
}

#[test]
fn a_heading_taken_out_whole_is_laid_when_no_other_heading_shares_its_name() {
    let text = "Uno.\n\n# Dos\n\nTres.\n\nCuatro.\n\n# Cinco\n";
    let at = text.find("# Dos").unwrap_or_default();
    let shown = shown_after(&read(text), at..at + 7, "");
    let shown = shown.unwrap_or_else(|| panic!("drawn whole instead of laid"));
    assert_eq!(shown.names, names_of(&shown.doc));
}

const SKILL: &str = "---\nagent: claude/skill/pdf/SKILL.md\ntitle: claude · pdf/SKILL.md\n---\n---\nname: pdf\ndescription: Fill PDF forms.\n---\n\n# PDF\n";

#[test]
fn an_agent_card_is_left_to_the_agent_view_and_a_skills_own_properties_show_as_fields() {
    let doc = read(SKILL);
    assert_eq!(kinds(SKILL)[..2], [Kind::Front, Kind::Front]);
    assert_eq!(
        &SKILL[doc.blocks()[1].at.clone()],
        "---\nname: pdf\ndescription: Fill PDF forms.\n---\n"
    );
    let drawn = html(&doc);
    assert!(drawn.contains("<dl class=\"front agent-card\""), "{drawn}");
    assert!(!drawn.contains("<dt>agent</dt>"), "{drawn}");
    assert!(drawn.contains("<dl class=\"front agent-own\""), "{drawn}");
    assert!(
        drawn.contains("<dt>description</dt><dd>Fill PDF forms.</dd>"),
        "{drawn}"
    );
    assert!(!drawn.contains("<hr"), "{drawn}");
    assert!(drawn.contains("<h1 id=\"pdf\" class=\"title\""), "{drawn}");
    assert_eq!(doc.write(), SKILL);
}

#[test]
fn a_skills_own_properties_too_long_for_fields_are_shown_as_written() {
    let long = "x".repeat(crate::front::VALUE_AT_MOST + 1);
    let skill = format!(
        "---\nagent: claude/skill/pdf/SKILL.md\n---\n---\nname: pdf\ndescription: {long}\n---\n# PDF\n"
    );
    let drawn = html(&read(&skill));
    assert!(drawn.contains("<dl class=\"front agent-own\""), "{drawn}");
    assert!(
        drawn.contains(&format!("<pre>name: pdf\ndescription: {long}</pre>")),
        "{drawn}"
    );
}

#[test]
fn only_an_agent_card_lets_a_second_block_of_properties_through() {
    let fronts = |text: &str| {
        kinds(text)
            .iter()
            .filter(|one| **one == Kind::Front)
            .count()
    };
    assert_eq!(fronts("---\ntitle: x\n---\n---\nname: y\n---\n"), 1);
    let rule = "---\nagent: global/rule/AGENTS.md\n---\n# Rules\n\n---\n\nmore\n";
    assert_eq!(fronts(rule), 1);
    assert!(!html(&read(rule)).contains("<dt>agent</dt>"));
    let stray = "---\nagent: yo\n---\n---\nname: y\n---\n";
    assert_eq!(fronts(stray), 1);
    assert!(html(&read(stray)).contains("<dt>agent</dt><dd>yo</dd>"));
    for odd in [
        "global/note/x.md",
        "Global/rule/x.md",
        "global/rule/",
        "[global/rule/x.md, odd]",
    ] {
        let text = format!("---\nagent: {odd}\n---\n---\nname: y\n---\n");
        assert_eq!(fronts(&text), 1, "{odd}");
    }
    let flagged = "---\nagent: [claude/skill/pdf/SKILL.md, crlf, bom]\n---\n---\nname: y\n---\n";
    assert_eq!(fronts(flagged), 2);
}

#[test]
fn marks_written_inside_an_address_and_again_past_it_still_draw_the_page() {
    for said in [
        "Join https://zoom.us/j/123?pwd=dGVzdA==&uname=x then check a == b",
        "https://example.com/?a=b==c and x == y",
        "Docs https://en.cppreference.com/c++17/x then i ++ j",
        "http://a==b ==",
        "http://a++b ++",
        "see https://x.com/a==b and ==yellow== here",
    ] {
        let drawn = html(&read(said));
        assert!(drawn.contains("<p"), "{said:?} gave {drawn:?}");
    }
    let marked = html(&read("see https://x.com/a==b and ==yellow== here"));
    assert!(marked.contains("<mark"), "{marked:?}");
}

#[test]
fn spaces_typed_in_a_row_are_drawn_each_one_after_the_first_unbreakable() {
    let drawn = html(&read("uno   dos"));
    assert!(drawn.contains("uno \u{a0}\u{a0}dos"), "{drawn:?}");
    let drawn = html(&read("uno dos y tres"));
    assert!(drawn.contains("uno dos y tres"), "{drawn:?}");
    assert!(!drawn.contains('\u{a0}'), "{drawn:?}");
}

#[test]
fn spaces_in_a_row_keep_their_run_in_the_placed_page_and_in_inline_code() {
    let placed_page = html_placed(&read("uno   dos **a  b**"));
    assert!(
        placed_page.contains("uno \u{a0}\u{a0}dos "),
        "{placed_page:?}"
    );
    assert!(placed_page.contains("a \u{a0}b"), "{placed_page:?}");
    let code = html(&read("x `a   b` y"));
    assert!(code.contains("<code>a \u{a0}\u{a0}b</code>"), "{code:?}");
}

#[test]
fn an_entity_among_spaces_is_left_as_written() {
    let drawn = html(&read("a &copy;  b"));
    assert!(drawn.contains("&copy; \u{a0}b"), "{drawn:?}");
    assert!(!drawn.contains("&amp;copy;"), "{drawn:?}");
}

#[test]
fn a_paragraph_holding_only_a_no_break_space_is_drawn_as_a_blank_line_with_its_place() {
    let text = "uno\n\n&nbsp;\n\ndos";
    let drawn = html_placed(&read(text));
    assert!(
        drawn.contains("<p class=\"blank\" data-at=\"5-12\""),
        "{drawn:?}"
    );
    assert_eq!(drawn.matches("class=\"blank\"").count(), 1, "{drawn:?}");
    assert!(!html(&read("uno\n\ndos")).contains("blank"));
    assert!(!html(&read("uno &nbsp; dos")).contains("blank"));
    assert_eq!(BLANK, "&nbsp;");
}
