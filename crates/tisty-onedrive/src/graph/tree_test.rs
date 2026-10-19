use super::{DriveItem, Tree, moved, within};

fn item(json: &str) -> DriveItem {
    serde_json::from_str(json).unwrap()
}

fn folder(id: &str, name: &str, parent: &str) -> DriveItem {
    item(&format!(
        r#"{{"id":"{id}","name":"{name}","folder":{{}},"parentReference":{{"id":"{parent}"}}}}"#
    ))
}

fn file(id: &str, name: &str, parent: &str, etag: &str) -> DriveItem {
    item(&format!(
        r#"{{"id":"{id}","name":"{name}","eTag":"{etag}","size":3,"file":{{"hashes":{{"quickXorHash":"Q{id}"}}}},"parentReference":{{"id":"{parent}"}}}}"#
    ))
}

#[test]
fn a_file_is_named_by_the_folders_above_it_up_to_the_app_s_own() {
    let mut tree = Tree::under("R");
    tree.applied(vec![
        item(r#"{"id":"R","name":"Tisty","folder":{},"parentReference":{"id":"APPS"}}"#),
        folder("S", "store", "R"),
        folder("D", "dev_a", "S"),
        file("F", "active.tisty", "D", "e1"),
        file("T", "tisty.toml", "R", "e2"),
    ]);

    let files = tree.files();
    let names: Vec<&str> = files.keys().map(String::as_str).collect();
    assert_eq!(names, ["store/dev_a/active.tisty", "tisty.toml"]);
    let one = &files["store/dev_a/active.tisty"];
    assert_eq!(one.hash, "quickxor:QF");
    assert_eq!(one.revision, "e1");
    assert_eq!(one.bytes, 3);
}

#[test]
fn a_renamed_folder_moves_every_file_under_it() {
    let mut tree = Tree::under("R");
    tree.applied(vec![
        folder("S", "store", "R"),
        file("F", "a.tisty", "S", "e1"),
    ]);
    let before = tree.files();

    tree.applied(vec![folder("S", "kept", "R")]);
    let (changed, gone) = moved(&before, &tree.files());

    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].name, "kept/a.tisty");
    assert_eq!(gone, ["store/a.tisty"]);
}

#[test]
fn a_deleted_item_leaves_and_a_file_cut_off_from_the_root_is_not_shown() {
    let mut tree = Tree::under("R");
    tree.applied(vec![
        folder("S", "store", "R"),
        file("F", "a.tisty", "S", "e1"),
        file("G", "b.tisty", "R", "e1"),
    ]);

    tree.applied(vec![
        item(r#"{"id":"S","deleted":{}}"#),
        item(r#"{"id":"G","name":"b.tisty","deleted":{}}"#),
    ]);

    assert!(tree.files().is_empty());
}

#[test]
fn the_last_word_on_an_item_in_one_feed_is_the_one_kept() {
    let mut tree = Tree::under("R");
    tree.applied(vec![
        file("F", "a.tisty", "R", "e1"),
        file("F", "a.tisty", "R", "e2"),
    ]);

    assert_eq!(tree.files()["a.tisty"].revision, "e2");
}

#[test]
fn a_loop_in_the_feed_is_never_followed_forever() {
    let mut tree = Tree::under("R");
    tree.applied(vec![
        folder("A", "a", "B"),
        folder("B", "b", "A"),
        file("F", "f", "A", "e1"),
    ]);

    assert!(tree.files().is_empty());
}

#[test]
fn a_path_is_within_a_folder_only_by_whole_names() {
    assert!(within("store/a", ""));
    assert!(within("store/a", "store"));
    assert!(within("store", "store"));
    assert!(!within("stores/a", "store"));
}

#[test]
fn a_file_s_revision_is_its_content_tag_and_its_etag_only_where_onedrive_gives_none() {
    let tagged = item(r#"{"id":"F","name":"a.tisty","eTag":"e1","cTag":"c1","file":{}}"#);
    let untagged = item(r#"{"id":"G","name":"b.tisty","eTag":"e2","file":{}}"#);

    assert_eq!(tagged.seen("a.tisty").revision, "c1");
    assert_eq!(untagged.seen("b.tisty").revision, "e2");
}
