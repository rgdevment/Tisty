use super::*;

fn said(args: Value) -> String {
    help(&args)["structuredContent"]["help"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn left_alone_it_names_every_page_by_its_section() {
    let index = said(json!({}));
    assert!(index.contains("- sync-and-backup: "), "{index}");
    assert!(index.contains("- keyboard-shortcuts: "), "{index}");
    assert_eq!(topics().first().map(String::as_str), Some("index"));
}

#[test]
fn a_topic_brings_its_page_in_the_language_asked_for_whichever_name_it_came_by() {
    let english = said(json!({ "topic": "sync-and-backup" }));
    assert!(english.starts_with("# Sync and backup"), "{english}");
    let spanish = said(json!({ "topic": "sync-and-backup", "lang": "es" }));
    assert!(
        spanish.starts_with("# Sincronizar y respaldar"),
        "{spanish}"
    );
    let named = said(json!({ "topic": "sincronizar-y-respaldar" }));
    assert!(named.starts_with("# Sync and backup"), "{named}");
}

#[test]
fn an_unknown_topic_says_so_and_hands_the_index_back() {
    let read = said(json!({ "topic": "nowhere" }));
    assert!(read.starts_with("No topic is called \"nowhere\""), "{read}");
    assert!(read.contains("- installation: "), "{read}");
}
