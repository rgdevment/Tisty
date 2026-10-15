use tisty_markdown::{Block, Callout, Kind, read};

fn cases() -> Vec<(String, String, Option<String>)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw = std::fs::read_to_string(root.join("fixtures/markdown.json"))
        .expect("the contract the window and the core keep too");
    let cases: Vec<serde_json::Value> = serde_json::from_str(&raw).expect("a list of cases");
    cases
        .into_iter()
        .map(|one| {
            let text = match one["file"].as_str() {
                Some(file) => {
                    std::fs::read_to_string(root.join(file)).expect("a file the contract names")
                }
                None => one["text"].as_str().expect("text").to_string(),
            };
            let canonical = one["canonical"].as_str().map(str::to_string);
            (
                one["name"].as_str().expect("name").to_string(),
                text,
                canonical,
            )
        })
        .collect()
}

fn inside(text: &str, blocks: &[Block], within: std::ops::Range<usize>) -> bool {
    let mut last = within.start;
    blocks.iter().all(|one| {
        let fits =
            one.at.start >= last && one.at.end <= within.end && text.is_char_boundary(one.at.start);
        last = one.at.end;
        fits && inside(text, &one.inner, one.at.clone())
    })
}

#[test]
fn an_unchanged_document_is_written_back_byte_for_byte() {
    let all = cases();
    assert!(all.len() > 50);
    for (name, text, canonical) in all {
        let wanted = canonical.unwrap_or_else(|| text.clone());
        assert_eq!(read(&text).write(), wanted, "{name}");
    }
}

#[test]
fn every_block_sits_in_order_inside_its_parent() {
    for (name, text, _) in cases() {
        let doc = read(&text);
        assert!(inside(&doc.text, &doc.blocks, 0..doc.text.len()), "{name}");
    }
}

#[test]
fn putting_each_block_back_as_it_was_changes_nothing() {
    for (name, text, _) in cases() {
        let doc = read(&text);
        for at in doc.blocks.iter().map(|one| one.at.clone()) {
            let mut again = doc.clone();
            let said = doc.text[at.clone()].to_string();
            again.put(at, &said);
            assert_eq!(again, doc, "{name}");
        }
    }
}

#[test]
fn the_dialect_s_own_blocks_are_read_as_what_they_are() {
    let all = cases();
    let first = |wanted: &str| {
        let (_, text, _) = all
            .iter()
            .find(|(name, _, _)| name == wanted)
            .expect("a case");
        read(text).blocks[0].kind.clone()
    };
    assert_eq!(first("callout-note"), Kind::Callout(Callout::Note));
    assert_eq!(first("callout-bold-first"), Kind::Callout(Callout::Note));
    assert_eq!(
        first("code-titled"),
        Kind::Code {
            tongue: Some("ts".into()),
            title: Some("src/main.ts".into())
        }
    );
    assert_eq!(
        first("code-math"),
        Kind::Code {
            tongue: Some("math".into()),
            title: None
        }
    );
    assert_eq!(first("table-widths"), Kind::Table);
    assert_eq!(first("table-spans"), Kind::Html);
    assert_eq!(first("quote"), Kind::Quote);
    assert_eq!(
        first("ordered-start"),
        Kind::List {
            ordered: true,
            start: 3
        }
    );
}
