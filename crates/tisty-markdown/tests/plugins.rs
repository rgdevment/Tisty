use tisty_markdown::{Mark, read};

fn marks(text: &str) -> Vec<(Mark, String)> {
    let doc = read(text);
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
        marks(said)[1],
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
    let plain = marks(r#"<span data-ico="flag">:flag:</span>"#);
    assert_eq!(
        plain[0].0,
        Mark::Icon {
            name: "flag".into(),
            hue: None
        }
    );
    let quoted = marks(r#"<span data-ico="a&amp;b">x</span>"#);
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
    marks(text)
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
        pens(said),
        vec![(
            Some("green".into()),
            r#"<mark data-pen="green">*green*</mark>"#.to_string()
        )]
    );
    assert!(marks(said).iter().any(|(mark, _)| *mark == Mark::Emphasis));
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
}
