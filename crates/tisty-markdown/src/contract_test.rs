use std::collections::BTreeSet;

use crate::{html, read};

const ALLOWED: &str = include_str!("../../../docs/contract/allowed.json");

const SHEETS: &[&str] = &[
    include_str!("../../../app/src-tauri/resources/guide/es/guia.md"),
    include_str!("../../../app/src-tauri/resources/guide/es/muestra.md"),
    include_str!("../../../app/src-tauri/resources/guide/es/codigo.md"),
    include_str!("../../../app/src-tauri/resources/guide/es/pagina.md"),
    include_str!("../../../app/src-tauri/resources/guide/en/guide.md"),
    include_str!("../../../app/src-tauri/resources/guide/en/sample.md"),
    include_str!("../../../app/src-tauri/resources/guide/en/code.md"),
    include_str!("../../../app/src-tauri/resources/guide/en/page.md"),
];

fn listed(field: &str) -> BTreeSet<String> {
    let allowed: serde_json::Value = serde_json::from_str(ALLOWED).unwrap();
    allowed[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one.as_str().unwrap().to_owned())
        .collect()
}

// Each tag and each attribute name of a sheet the translator wrote, whose values never hold an unescaped quote.
fn drawn(sheet: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut tags = BTreeSet::new();
    let mut attributes = BTreeSet::new();
    for piece in sheet.split('<').skip(1) {
        let Some(open) = piece.split('>').next() else {
            continue;
        };
        if !open.starts_with(|one: char| one.is_ascii_alphabetic()) {
            continue;
        }
        let unquoted: String = open.split('"').step_by(2).collect();
        let mut words = unquoted.split_whitespace();
        if let Some(tag) = words.next() {
            tags.insert(tag.trim_end_matches('/').to_ascii_lowercase());
        }
        for word in words {
            let name = word.split('=').next().unwrap_or_default();
            if !name.is_empty() && name != "/" {
                attributes.insert(name.to_ascii_lowercase());
            }
        }
    }
    (tags, attributes)
}

// A translator that writes something new has to take it to the contract first, or every reader would strip it.
#[test]
fn what_the_translator_draws_stays_inside_the_contract() {
    let tags = listed("tags");
    let attributes = listed("attributes");
    for text in SHEETS {
        let (seen_tags, seen_attributes) = drawn(&html(&read(text)));
        assert!(!seen_tags.is_empty());
        let unlisted: Vec<_> = seen_tags.difference(&tags).collect();
        assert!(
            unlisted.is_empty(),
            "tags outside allowed.json: {unlisted:?}"
        );
        let unlisted: Vec<_> = seen_attributes.difference(&attributes).collect();
        assert!(
            unlisted.is_empty(),
            "attributes outside allowed.json: {unlisted:?}"
        );
    }
}
