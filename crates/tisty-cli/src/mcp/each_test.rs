use serde_json::json;

use super::*;

#[test]
fn a_field_every_row_shares_moves_under_each_by_its_list() {
    let mut whole = json!({
        "matches": [
            { "id": "a", "title": "One", "list": "Brain", "by_agent": true, "status": "open" },
            { "id": "b", "title": "Two", "list": "Brain", "by_agent": true, "status": "done" },
        ],
        "total": 2,
    });
    said_once(&mut whole, &["matches"]);
    assert_eq!(
        whole,
        json!({
            "matches": [
                { "id": "a", "title": "One", "status": "open" },
                { "id": "b", "title": "Two", "status": "done" },
            ],
            "total": 2,
            "each": { "matches": { "list": "Brain", "by_agent": true } },
        })
    );
}

#[test]
fn a_field_one_row_lacks_stays_on_every_row() {
    let mut whole = json!({ "journal": [
        { "id": "1", "at": "10:00", "body": "x", "by": "Claude Code" },
        { "id": "2", "at": "10:00", "body": "y" },
    ]});
    let was = whole.clone();
    said_once(&mut whole, &["journal"]);
    assert_eq!(
        whole, was,
        "nothing shared, and `at` alike is the row's own"
    );
}

#[test]
fn one_row_alone_or_no_list_is_left_as_it_is() {
    let mut whole = json!({ "parts": [{ "id": "a", "list": "Brain" }], "docs": "none" });
    let was = whole.clone();
    said_once(&mut whole, &["parts", "docs", "missing"]);
    assert_eq!(whole, was);
}

#[test]
fn each_list_keeps_its_own_shared_fields() {
    let mut whole = json!({
        "tasks": [{ "id": "a", "list": "Casa" }, { "id": "b", "list": "Casa" }],
        "docs": [{ "doc": "x", "folder": "Recetas" }, { "doc": "y", "folder": "Recetas" }],
    });
    said_once(&mut whole, &["tasks", "docs"]);
    assert_eq!(
        whole["each"],
        json!({ "tasks": { "list": "Casa" }, "docs": { "folder": "Recetas" } })
    );
    assert_eq!(whole["tasks"], json!([{ "id": "a" }, { "id": "b" }]));
}
