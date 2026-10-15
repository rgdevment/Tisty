use super::*;

fn kinds(text: &str) -> Vec<Kind> {
    read(text).blocks.into_iter().map(|one| one.kind).collect()
}

#[test]
fn a_document_reads_as_its_blocks_in_order() {
    assert_eq!(
        kinds("# Title\n\nWords.\n\n---\n\n| a |\n| --- |\n| 1 |"),
        vec![Kind::Heading(1), Kind::Paragraph, Kind::Rule, Kind::Table]
    );
}

#[test]
fn every_block_knows_the_bytes_it_came_from() {
    let doc = read("# Title\n\nWords here.\n");
    let said: Vec<&str> = doc.blocks.iter().map(|one| doc.said(one)).collect();
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
        vec![Kind::Code {
            tongue: Some("ts".into()),
            title: Some("src/main.ts".into())
        }]
    );
    assert_eq!(
        kinds("```rust title=\"say \\\"hi\\\"\"\nfn main() {}\n```"),
        vec![Kind::Code {
            tongue: Some("rust".into()),
            title: Some("say \"hi\"".into())
        }]
    );
    assert_eq!(
        kinds("~~~\nplain\n~~~"),
        vec![Kind::Code {
            tongue: None,
            title: None
        }]
    );
}

#[test]
fn a_task_item_says_whether_it_is_done() {
    let doc = read("- [ ] open\n- [x] done\n- plain");
    let items: Vec<Kind> = doc.blocks[0]
        .inner
        .iter()
        .map(|one| one.kind.clone())
        .collect();
    assert_eq!(
        items,
        vec![
            Kind::Item { done: Some(false) },
            Kind::Item { done: Some(true) },
            Kind::Item { done: None },
        ]
    );
}

#[test]
fn an_ordered_list_keeps_where_it_starts() {
    assert_eq!(
        kinds("3. three\n4. four"),
        vec![Kind::List {
            ordered: true,
            start: 3
        }]
    );
}

#[test]
fn nested_blocks_sit_inside_their_parent() {
    let doc = read("> - one\n>   - two");
    let quote = &doc.blocks[0];
    assert_eq!(quote.kind, Kind::Quote);
    assert_eq!(
        quote.inner[0].kind,
        Kind::List {
            ordered: false,
            start: 1
        }
    );
    assert!(quote.inner[0].at.start >= quote.at.start && quote.inner[0].at.end <= quote.at.end);
}

#[test]
fn putting_a_block_back_changes_only_its_bytes() {
    let mut doc = read("# Title\n\nOld words.\n\nAfter.\n");
    let at = doc.blocks[1].at.clone();
    doc.put(at, "New words, *longer* now.\n");
    assert_eq!(doc.text, "# Title\n\nNew words, *longer* now.\n\nAfter.\n");
    assert_eq!(doc.blocks.len(), 3);
}

#[test]
fn writing_keeps_one_line_ending_and_no_byte_order_mark() {
    assert_eq!(settled("\u{feff}a\r\nb"), "a\nb");
    assert!(matches!(
        settled("already plain\n"),
        std::borrow::Cow::Borrowed(_)
    ));
}
