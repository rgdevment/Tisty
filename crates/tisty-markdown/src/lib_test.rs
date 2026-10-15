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
        vec![Kind::Heading(1), Kind::Paragraph, Kind::Rule, Kind::Table]
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
