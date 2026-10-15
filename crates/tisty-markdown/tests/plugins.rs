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
