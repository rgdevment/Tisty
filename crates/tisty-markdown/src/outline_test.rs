use super::{bare, sections};
use crate::reading::plain;

const ODD: [&str; 16] = [
    "## a  b",
    "# ¡Hola, mundo!",
    "## C# y F#",
    "## foo ##",
    "#  ",
    "#",
    "# x\u{a0}",
    "### 100% (casi) / 2: ya",
    "#### é y ñ, \"citado\" y 'dicho'",
    "## una *cursiva*",
    "## `código`",
    "## a \\* b",
    "## [enlace](x)",
    "## a &amp; b",
    "####### siete",
    "#sin espacio",
];

#[test]
fn a_heading_of_words_alone_reads_as_the_tokenizer_reads_it() {
    let samples = [
        include_str!("../../../app/src-tauri/resources/guide/es/muestra.md"),
        include_str!("../../../app/src-tauri/resources/guide/en/sample.md"),
    ];
    let headings: Vec<&str> = samples
        .iter()
        .flat_map(|one| one.lines())
        .filter(|line| line.starts_with('#'))
        .chain(ODD)
        .collect();
    let mut bared = 0;
    for source in headings {
        if let Some(said) = bare(source) {
            assert_eq!(said, plain(source), "{source:?}");
            bared += 1;
        }
    }
    assert!(bared > 50, "only {bared} headings were read bare");
}

#[test]
fn the_title_is_the_document_and_no_section_of_it() {
    let said = |text: &str| -> Vec<String> {
        sections(&crate::read(text))
            .into_iter()
            .map(|head| head.id)
            .collect()
    };

    assert_eq!(
        said(
            "---
tags: [x]
---
# Trips

## Hotels
"
        ),
        ["hotels"]
    );
    assert_eq!(
        said(
            "Some words first.

# Trips
"
        ),
        ["trips"]
    );
    assert_eq!(
        said(
            "## Hotels

## Hotels
"
        ),
        ["hotels", "hotels-1"]
    );
}
