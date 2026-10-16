use tisty_markdown::{Doc, Mark, read};

fn marks(text: &str) -> Vec<(Mark, String)> {
    spans_of(read(text))
}

// The old tags are read only on their way into Tisty's own words, by the transformer.
#[cfg(any(feature = "icons", feature = "pens"))]
fn old_marks(text: &str) -> Vec<(Mark, String)> {
    spans_of(tisty_markdown::read_old(text))
}

fn spans_of(doc: Doc) -> Vec<(Mark, String)> {
    doc.blocks()[0]
        .spans
        .iter()
        .map(|one| (one.mark.clone(), doc.text()[one.at.clone()].to_string()))
        .collect()
}

#[cfg(feature = "icons")]
#[test]
fn an_icon_is_one_mark_with_its_name_and_its_hue() {
    let said = r#"An <span data-ico="star" data-hue="blue">⭐</span> here."#;
    assert_eq!(
        old_marks(said)[1],
        (
            Mark::Icon {
                name: "star".into(),
                hue: Some("blue".into())
            },
            r#"<span data-ico="star" data-hue="blue">⭐</span>"#.to_string()
        )
    );
}

#[cfg(feature = "icons")]
#[test]
fn an_icon_without_a_hue_or_with_quoted_marks_keeps_its_name() {
    let plain = old_marks(r#"<span data-ico="flag">:flag:</span>"#);
    assert_eq!(
        plain[0].0,
        Mark::Icon {
            name: "flag".into(),
            hue: None
        }
    );
    let quoted = old_marks(r#"<span data-ico="a&amp;b">x</span>"#);
    assert_eq!(
        quoted[0].0,
        Mark::Icon {
            name: "a&b".into(),
            hue: None
        }
    );
}

#[cfg(feature = "icons")]
#[test]
fn a_span_that_is_not_an_icon_is_left_as_it_was() {
    let found = marks(r#"<span class="x">y</span>"#);
    assert!(
        found
            .iter()
            .all(|(mark, _)| !matches!(mark, Mark::Icon { .. }))
    );
}

#[test]
fn plain_words_are_only_text() {
    assert_eq!(
        marks("Just words."),
        vec![(Mark::Text, "Just words.".to_string())]
    );
}

#[cfg(feature = "pens")]
fn pens(text: &str) -> Vec<(Option<String>, String)> {
    pens_in(marks(text))
}

#[cfg(feature = "pens")]
fn old_pens(text: &str) -> Vec<(Option<String>, String)> {
    pens_in(old_marks(text))
}

#[cfg(feature = "pens")]
fn pens_in(found: Vec<(Mark, String)>) -> Vec<(Option<String>, String)> {
    found
        .into_iter()
        .filter_map(|(mark, said)| match mark {
            Mark::Pen { color } => Some((color, said)),
            _ => None,
        })
        .collect()
}

#[cfg(feature = "pens")]
#[test]
fn a_double_equals_pair_is_the_yellow_pen() {
    assert_eq!(
        pens("Read ==this== now"),
        vec![(None, "==this==".to_string())]
    );
    assert_eq!(
        pens("Read ==**bold** too== now"),
        vec![(None, "==**bold** too==".to_string())]
    );
}

#[cfg(feature = "pens")]
#[test]
fn a_coloured_pen_wraps_its_words_and_keeps_their_marks() {
    let said = r#"A <mark data-pen="green">*green*</mark> word"#;
    assert_eq!(
        old_pens(said),
        vec![(
            Some("green".into()),
            r#"<mark data-pen="green">*green*</mark>"#.to_string()
        )]
    );
    assert!(
        old_marks(said)
            .iter()
            .any(|(mark, _)| *mark == Mark::Emphasis)
    );
}

#[cfg(feature = "pens")]
#[test]
fn what_only_looks_like_a_pen_is_left_alone() {
    assert!(pens("a == b == c").is_empty());
    assert!(pens("a === b === c").is_empty());
    assert!(pens("`==code==` stays").is_empty());
    assert!(pens("====").is_empty());
}

#[cfg(feature = "math")]
fn formulas(text: &str) -> Vec<String> {
    marks(text)
        .into_iter()
        .filter(|(mark, _)| *mark == Mark::Math)
        .map(|(_, said)| said)
        .collect()
}

#[cfg(feature = "math")]
#[test]
fn a_formula_inside_a_line_is_read_between_single_dollars() {
    assert_eq!(formulas("Energy $E=mc^2$ here"), vec!["$E=mc^2$"]);
    assert_eq!(formulas("$a_b$ and $c$"), vec!["$a_b$", "$c$"]);
}

#[cfg(feature = "math")]
#[test]
fn prices_and_escaped_dollars_never_become_formulas() {
    assert!(formulas("Cuesta $5 y $10 en total.").is_empty());
    assert!(formulas("From $ 3 to $ 4").is_empty());
    assert!(formulas("Pay $20, then $30").is_empty());
    assert!(formulas(r"A \$x\$ literal").is_empty());
    assert!(formulas("`$x$` is code").is_empty());
}

#[cfg(feature = "emoji")]
fn glyphs(text: &str) -> Vec<(String, String)> {
    marks(text)
        .into_iter()
        .filter_map(|(mark, said)| match mark {
            Mark::Emoji { glyph } => Some((glyph, said)),
            _ => None,
        })
        .collect()
}

#[cfg(feature = "emoji")]
#[test]
fn a_known_shortcode_reads_as_its_emoji() {
    assert_eq!(
        glyphs("Done :tada: and :+1:!"),
        vec![("🎉".into(), ":tada:".into()), ("👍".into(), ":+1:".into())]
    );
}

#[cfg(feature = "emoji")]
#[test]
fn what_only_looks_like_a_shortcode_is_left_alone() {
    assert!(glyphs("At 10:30:00 sharp").is_empty());
    assert!(glyphs("a:tada:b and :nothing_here:").is_empty());
    assert!(glyphs("`:tada:` is code").is_empty());
    assert!(glyphs("fix it ::wrench:: today").is_empty());
}

#[cfg(feature = "tables")]
fn columns(text: &str) -> Vec<tisty_markdown::Column> {
    match &read(text).blocks()[0].kind {
        tisty_markdown::Kind::Table { columns } => columns.clone(),
        other => panic!("not a table: {other:?}"),
    }
}

#[cfg(feature = "tables")]
#[test]
fn a_column_keeps_its_width_in_its_dashes_and_its_lean_in_its_colons() {
    use tisty_markdown::{Column, Lean};
    assert_eq!(
        columns("| wide | narrow | right |\n| :---------------- | --- | ------: |\n| a | b | c |"),
        vec![
            Column {
                lean: Some(Lean::Left),
                wide: Some(160)
            },
            Column {
                lean: None,
                wide: None
            },
            Column {
                lean: Some(Lean::Right),
                wide: Some(60)
            },
        ]
    );
}

#[cfg(feature = "tables")]
#[test]
fn a_table_written_without_a_leading_pipe_is_measured_the_same() {
    let found = columns("a | b\n--------- | :---:\n1 | 2");
    assert_eq!(found[0].wide, Some(90));
    assert_eq!(found[1].lean, Some(tisty_markdown::Lean::Center));
}

#[cfg(feature = "details")]
fn tops(text: &str) -> Vec<tisty_markdown::Kind> {
    read(text)
        .blocks()
        .iter()
        .map(|one| one.kind.clone())
        .collect()
}

#[cfg(feature = "details")]
#[test]
fn what_sits_between_details_tags_folds_under_its_summary() {
    use tisty_markdown::Kind;
    let doc = read("<details><summary>More</summary>\n\nHidden.\n\n</details>\n\nAfter.");
    assert_eq!(
        doc.blocks()[0].kind,
        Kind::Details {
            summary: "More".into(),
            open: false
        }
    );
    assert_eq!(doc.blocks()[0].inner[0].kind, Kind::Paragraph);
    assert_eq!(doc.blocks()[1].kind, Kind::Paragraph);
    assert_eq!(
        tops("<details open>\n<summary>Open</summary>\n\nIn.\n\n</details>"),
        vec![Kind::Details {
            summary: "Open".into(),
            open: true
        }]
    );
}

#[cfg(feature = "details")]
#[test]
fn the_colon_fence_is_read_as_a_details_block_too() {
    assert_eq!(
        tops(":::details Old style\n\nInside.\n\n:::"),
        vec![tisty_markdown::Kind::Details {
            summary: "Old style".into(),
            open: false
        }]
    );
}

#[cfg(feature = "details")]
#[test]
fn a_details_block_never_closed_is_left_as_it_was_written() {
    assert_eq!(
        tops("<details><summary>Open</summary>\n\nWords."),
        vec![tisty_markdown::Kind::Html, tisty_markdown::Kind::Paragraph]
    );
}

#[cfg(feature = "outline")]
#[test]
fn every_heading_is_listed_with_its_plain_words_and_an_anchor() {
    let doc = read(
        "# The **plan**\n\nText.\n\n## Step `one`\n\n> ### Quoted\n\n## Step one\n\n## ¿Qué sigue?",
    );
    let found: Vec<(u8, String, String)> = tisty_markdown::outline(&doc)
        .into_iter()
        .map(|one| (one.level, one.said, one.id))
        .collect();
    let wanted: Vec<(u8, &str, &str)> = vec![
        (1, "The plan", "the-plan"),
        (2, "Step one", "step-one"),
        (3, "Quoted", "quoted"),
        (2, "Step one", "step-one-1"),
        (2, "¿Qué sigue?", "qué-sigue"),
    ];
    let wanted: Vec<(u8, String, String)> = wanted
        .into_iter()
        .map(|(level, said, id)| (level, said.to_string(), id.to_string()))
        .collect();
    assert_eq!(found, wanted);
}

#[cfg(feature = "tables")]
#[test]
fn a_table_inside_a_quote_keeps_each_column_s_own_width() {
    let doc = read("> | a | b |\n> | ---------- | ---- |\n> | 1 | 2 |");
    let tisty_markdown::Kind::Table { columns } = &doc.blocks()[0].inner[0].kind else {
        panic!("not a table");
    };
    assert_eq!(columns[0].wide, Some(100));
    assert_eq!(columns[1].wide, Some(40));
}

#[cfg(feature = "pens")]
#[test]
fn a_mark_never_reaches_across_two_cells() {
    let doc = read("| ==a | b== |\n| --- | --- |\n| c | d |");
    let mut found = Vec::new();
    for row in &doc.blocks()[0].inner {
        for cell in &row.inner {
            found.extend(
                cell.spans
                    .iter()
                    .filter(|one| matches!(one.mark, Mark::Pen { .. })),
            );
        }
    }
    assert!(found.is_empty());
}

#[cfg(feature = "math")]
#[test]
fn a_formula_keeps_its_stars_and_a_digit_after_it_makes_it_a_price() {
    let found = marks("$a*b$ and c*d");
    assert_eq!(found[0], (Mark::Math, "$a*b$".to_string()));
    assert!(!found.iter().any(|(mark, _)| *mark == Mark::Emphasis));
    assert!(formulas("Pay $x$5 now").is_empty());
}

#[cfg(feature = "details")]
#[test]
fn a_details_block_written_on_one_line_holds_nothing_to_fold() {
    use tisty_markdown::Kind;
    let doc = read(
        "<details><summary>Outer</summary>\n\n<details><summary>A</summary>Body</details>\n\n</details>",
    );
    assert_eq!(
        doc.blocks()[0].kind,
        Kind::Details {
            summary: "Outer".into(),
            open: false
        }
    );
    assert_eq!(doc.blocks()[0].inner[0].kind, Kind::Html);
}

#[cfg(feature = "details")]
#[test]
fn a_colon_fence_closes_the_container_that_opened_last() {
    let doc = read(":::details More\n\n:::tip\n\nHi\n\n:::\n\nStill hidden\n\n:::");
    assert_eq!(doc.blocks().len(), 1);
    assert_eq!(doc.blocks()[0].inner.len(), 4);
}

#[cfg(feature = "outline")]
#[test]
fn an_anchor_reads_entities_and_broken_lines_and_is_never_taken_twice() {
    let doc = read("# Q&amp;A\n\nLine one\nLine two\n===\n\n# A\n\n# A\n\n# A 1");
    let ids: Vec<(String, String)> = tisty_markdown::outline(&doc)
        .into_iter()
        .map(|one| (one.said, one.id))
        .collect();
    let wanted: Vec<(&str, &str)> = vec![
        ("Q&A", "qa"),
        ("Line one Line two", "line-one-line-two"),
        ("A", "a"),
        ("A", "a-1"),
        ("A 1", "a-1-1"),
    ];
    let wanted: Vec<(String, String)> = wanted
        .into_iter()
        .map(|(said, id)| (said.to_string(), id.to_string()))
        .collect();
    assert_eq!(ids, wanted);
}

#[cfg(feature = "icons")]
#[test]
fn an_icon_never_closed_leaves_the_words_after_it_alone() {
    let found = old_marks(r#"<span data-ico="star"> text <span class="x">hi</span>"#);
    assert!(
        found
            .iter()
            .all(|(mark, _)| !matches!(mark, Mark::Icon { .. }))
    );
    assert!(found.contains(&(Mark::Text, "hi".to_string())));
}

#[cfg(feature = "pens")]
#[test]
fn a_pen_never_cuts_across_another_mark_and_reads_its_colour_anywhere_in_the_tag() {
    assert!(pens("**==a** b==").is_empty());
    assert_eq!(
        old_pens(r#"<mark class="x" data-pen="a&amp;b">w</mark>"#)[0].0,
        Some("a&b".to_string())
    );
}

#[cfg(feature = "details")]
#[test]
fn a_details_block_reads_every_way_of_writing_open_and_its_summary() {
    use tisty_markdown::Kind;
    let doc =
        read("<details open=\"\">\n\n<summary class=\"s\">More</summary>\n\nBody\n\n</details>");
    assert_eq!(
        doc.blocks()[0].kind,
        Kind::Details {
            summary: "More".into(),
            open: true
        }
    );
    assert_eq!(doc.blocks()[0].inner.len(), 1);
}

#[cfg(feature = "outline")]
#[test]
fn a_heading_keeps_its_formula_and_one_inside_a_quote_reads_clean() {
    let doc = read("# Energy $E=mc^2$\n\n> Title\n> ===");
    let said: Vec<String> = tisty_markdown::outline(&doc)
        .into_iter()
        .map(|one| one.said)
        .collect();
    assert_eq!(said, vec!["Energy E=mc^2", "Title"]);
}

#[cfg(feature = "mermaid")]
#[test]
fn a_diagram_with_too_many_parts_is_shown_as_code() {
    let links: String = (0..600)
        .map(|n| format!("  N{n} --> N{}\n", n + 1))
        .collect();
    assert!(tisty_markdown::diagram::drawn(&format!("graph TD\n{links}")).is_none());
}

#[cfg(feature = "tags")]
#[test]
fn a_tag_is_marked_where_the_core_reads_it_and_never_inside_code() {
    let found: Vec<(Mark, String)> = marks("Notes #plan and #mi_etiqueta, `#no` and bg-#fff")
        .into_iter()
        .filter(|(mark, _)| matches!(mark, Mark::Tag { .. }))
        .collect();
    assert_eq!(
        found,
        vec![
            (
                Mark::Tag {
                    name: "plan".into()
                },
                "#plan".to_string()
            ),
            (
                Mark::Tag {
                    name: "mi-etiqueta".into()
                },
                "#mi_etiqueta".to_string()
            ),
        ]
    );
}

#[cfg(feature = "refs")]
#[test]
fn a_reference_between_double_brackets_is_one_mark_outside_code() {
    let found: Vec<(Mark, String)> = marks("See [[My doc]] and step [[#2]], not `[[code]]`.")
        .into_iter()
        .filter(|(mark, _)| matches!(mark, Mark::Ref { .. }))
        .collect();
    assert_eq!(
        found,
        vec![
            (
                Mark::Ref {
                    name: "My doc".into()
                },
                "[[My doc]]".to_string()
            ),
            (Mark::Ref { name: "#2".into() }, "[[#2]]".to_string()),
        ]
    );
}

#[cfg(feature = "tags")]
fn tag_names(text: &str) -> Vec<String> {
    let doc = read(text);
    let mut found = Vec::new();
    for block in doc.blocks() {
        for span in &block.spans {
            if let Mark::Tag { name } = &span.mark {
                found.push(name.clone());
            }
        }
    }
    found
}

#[cfg(feature = "tags")]
#[test]
fn a_tag_with_an_accent_apart_from_its_letter_is_marked_whole() {
    assert_eq!(tag_names("Hecho #disen\u{303}o hoy"), vec!["diseno"]);
}

#[cfg(feature = "tags")]
#[test]
fn only_the_tags_the_core_files_are_marked() {
    let many: String = (0..70).map(|n| format!("#tag{n}x ")).collect();
    let found = tag_names(&many);
    assert_eq!(found.len(), tisty_markdown::tags::AT_MOST);
    assert_eq!(found, tisty_markdown::tags::tags_in(&many));
}

#[cfg(all(feature = "tags", feature = "refs"))]
#[test]
fn a_tag_inside_a_reference_is_part_of_the_reference() {
    assert!(tag_names("See [[#plan]] now").is_empty());
    assert_eq!(
        marks("See [[#plan]] now")
            .into_iter()
            .filter(|(mark, _)| matches!(mark, Mark::Ref { .. }))
            .count(),
        1
    );
}

#[cfg(feature = "details")]
#[test]
fn a_colon_container_named_after_a_callout_reads_as_that_callout() {
    use tisty_markdown::{Callout, Kind};
    let doc = read(":::tip\n\nWater the plants.\n\n:::\n\nAfter.");
    assert_eq!(doc.blocks()[0].kind, Kind::Callout(Callout::Tip));
    assert_eq!(doc.blocks()[0].inner[0].kind, Kind::Paragraph);
    assert_eq!(doc.blocks()[1].kind, Kind::Paragraph);
    assert_eq!(
        tops(":::WARNING\n\nCareful.\n\n:::"),
        vec![Kind::Callout(Callout::Warning)]
    );
    assert_eq!(
        tops(":::recipe\n\nNot a callout.\n\n:::")[0],
        Kind::Paragraph
    );
}
