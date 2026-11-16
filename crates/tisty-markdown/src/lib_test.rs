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
    let near = placed_near(&doc, at, at + 5);
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
    let near = placed_near(&doc, at + 2, at + 3);
    let second = &crate::outline(&doc)[1].id;
    assert!(
        near.html.contains(&format!("id=\"{second}\"")),
        "{}",
        near.html
    );
    assert!(html_placed(&doc).contains(&format!("id=\"{second}\"")));
    let words = placed_near(&doc, text.len() - 2, text.len() - 1);
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

fn drawn_names(html: &str) -> Vec<(usize, String)> {
    let between = |tag: &str, from: &str| {
        let at = tag.find(from)? + from.len();
        Some(tag[at..].split('"').next()?.to_string())
    };
    html.split('<')
        .filter(|tag| tag.starts_with('h') && tag[1..].starts_with(|c: char| c.is_ascii_digit()))
        .filter_map(|tag| {
            let tag = tag.split('>').next()?;
            let id = between(tag, " id=\"")?;
            let place = between(tag, "data-at=\"")?;
            Some((place.split('-').next()?.parse().ok()?, id))
        })
        .collect()
}

// What the page shows after a key, as `patched` lays it, or nothing when it draws the whole page.
fn shown_after(was: &Doc, at: Range<usize>, insert: &str) -> Option<(Doc, Vec<(usize, String)>)> {
    let doc = crate::reread_edit(was.clone(), at.clone(), insert).ok()?;
    let by = insert.len() as isize - at.len() as isize;
    let near = placed_near(&doc, at.start, at.start + insert.len());
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
    match near.heads {
        Some(heads) if heads != stays => return None,
        None if !gone.is_empty() => return None,
        _ => {}
    }
    let mut shown: Vec<_> = stay
        .into_iter()
        .map(|(place, name)| (moved(place), name))
        .chain(drawn_names(&near.html))
        .collect();
    shown.sort();
    Some((doc, shown))
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
        if let Some((doc, shown)) = shown_after(&was, at.clone(), insert) {
            assert_eq!(shown, names_of(&doc), "{at:?} → {insert:?}");
        }
    }
    let alone = read("# Uno\n\nA.\n\nB.\n\n# Dos\n\nC.\n");
    assert!(
        shown_after(&alone, 5..5, "s").is_some(),
        "a name no other heading shares is laid without the whole page"
    );
}

#[test]
fn every_key_leaves_the_page_with_the_names_the_whole_page_gives() {
    const TEXTS: [&str; 2] = [
        "# Uno\n\nA.\n\n# Uno\n\nB.\n\n## Uno 1\n\nC.\n\n# Dos\n\n> # Uno\n\nD.\n",
        "Uno\n===\n\nA.\n\n- # Uno\n\nB.\n\n# Uno\n\nÁrbol\n---\n",
    ];
    const PIECES: [&str; 11] = [
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
    ];
    let mut state: u64 = 0x01D5_0F7E;
    let mut next = |below: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % below.max(1) as u64) as usize
    };
    let mut laid = 0;
    let mut all = 0;
    for text in TEXTS {
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
                PIECES[next(PIECES.len())]
            };
            all += 1;
            if let Some((doc, shown)) = shown_after(&was, from..to, insert) {
                laid += 1;
                assert_eq!(
                    shown,
                    names_of(&doc),
                    "{from}..{to} → {insert:?} in {:?}",
                    was.text()
                );
            }
            let now = format!("{}{insert}{}", &was.text()[..from], &was.text()[to..]);
            was = read(&now);
        }
    }
    assert!(
        laid * 2 > all,
        "only {laid} of {all} keys were laid without the whole page"
    );
}

#[test]
fn an_edit_past_the_last_block_draws_the_last_one() {
    let text = "One.\n\nTwo.\n\n";
    let near = placed_near(&read(text), text.len(), text.len());
    assert_eq!(near.drawn, 1..2);
    assert!(near.html.contains("Two."));
    assert_eq!(placed_near(&read(""), 0, 0).drawn, 0..0);
}
