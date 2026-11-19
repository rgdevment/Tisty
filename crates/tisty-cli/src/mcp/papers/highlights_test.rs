use super::*;

const BODY: &str = "# Plan\n\nThe walk is **cached**.\n\n## Part one\n\nPlain.\n\n### Deeper\n\nA ==pen== here.\n\n## Part two\n\nAnd **bold** there.\n";

fn marks(args: Value) -> Value {
    answered(BODY, &args, "\"Plan\"", "", Map::new())
        .ok()
        .unwrap()["structuredContent"]
        .clone()
}

#[test]
fn each_mark_comes_under_the_deepest_section_that_holds_it() {
    let shown = marks(json!({ "marks": true }));
    assert_eq!(
        shown["marks"],
        json!([
            { "at": 0, "section": "Plan", "marks": [{ "line": 3, "said": "**cached**" }] },
            { "at": 2, "section": "Deeper", "marks": [{ "line": 11, "said": "==pen==" }] },
            { "at": 3, "section": "Part two", "marks": [{ "line": 15, "said": "**bold**" }] },
        ])
    );
    assert!(shown.get("marks_left").is_none());
}

#[test]
fn a_section_narrows_the_marks_to_its_lines() {
    let shown = marks(json!({ "marks": true, "section": 1 }));
    assert_eq!(
        shown["marks"],
        json!([{ "at": 2, "section": "Deeper", "marks": [{ "line": 11, "said": "==pen==" }] }])
    );
    assert!(
        answered(
            BODY,
            &json!({ "marks": true, "section": 9 }),
            "x",
            "",
            Map::new()
        )
        .is_err()
    );
}

#[test]
fn marks_before_any_heading_carry_no_section() {
    let said = answered(
        "Lead **in**.\n\n# Later\n",
        &json!({ "marks": true }),
        "x",
        "",
        Map::new(),
    )
    .ok()
    .unwrap();
    assert_eq!(
        said["structuredContent"]["marks"],
        json!([{ "marks": [{ "line": 1, "said": "**in**" }] }])
    );
}

#[test]
fn a_document_without_marks_says_so() {
    let said = answered(
        "# Plain\n\nNothing marked.\n",
        &json!({ "marks": true }),
        "\"Plain\"",
        "",
        Map::new(),
    )
    .ok()
    .unwrap();
    assert_eq!(said["structuredContent"]["marks"], json!([]));
    assert!(
        said["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Nothing in \"Plain\" is highlighted or bold.")
    );
}

#[test]
fn past_the_budget_the_rest_is_counted_and_left_for_a_section() {
    let long = "x".repeat(7_000);
    let body = format!("# Long\n\n**{long}**\n\n**{long}**\n\n**short**\n");
    let said = answered(&body, &json!({ "marks": true }), "x", "", Map::new())
        .ok()
        .unwrap();
    assert_eq!(said["structuredContent"]["marks_left"], json!(2));
    assert_eq!(
        said["structuredContent"]["marks"][0]["marks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_mark_on_a_heading_is_under_that_heading() {
    let said = answered(
        "# Plan\n\n## The **key** part\n\nPlain.\n",
        &json!({ "marks": true }),
        "x",
        "",
        Map::new(),
    )
    .ok()
    .unwrap();
    assert_eq!(said["structuredContent"]["marks"][0]["at"], 1);
}

#[test]
fn marks_that_fill_the_budget_exactly_all_come() {
    let half = "z".repeat(WHOLE_UP_TO / 2 - 4);
    let body = format!("# Even\n\n**{half}**\n\n**{half}**\n");
    let said = answered(&body, &json!({ "marks": true }), "x", "", Map::new())
        .ok()
        .unwrap();
    assert!(said["structuredContent"].get("marks_left").is_none());
    assert_eq!(
        said["structuredContent"]["marks"][0]["marks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn a_passage_longer_than_a_read_holds_is_cut_and_said_so() {
    let long = "y".repeat(WHOLE_UP_TO + 500);
    let body = format!("# Long\n\n**{long}**\n");
    let said = answered(&body, &json!({ "marks": true }), "x", "", Map::new())
        .ok()
        .unwrap();
    let mark = &said["structuredContent"]["marks"][0]["marks"][0];
    assert_eq!(mark["cut"], true, "{mark}");
    assert_eq!(
        mark["said"].as_str().unwrap().chars().count(),
        WHOLE_UP_TO + 1
    );
    assert!(said["structuredContent"].get("marks_left").is_none());
}

#[test]
fn marks_is_asked_only_when_true_and_never_with_lines_or_chars() {
    assert_eq!(asked(&json!({})).ok(), Some(false));
    assert_eq!(asked(&json!({ "marks": false })).ok(), Some(false));
    assert_eq!(
        asked(&json!({ "marks": true, "section": 2 })).ok(),
        Some(true)
    );
    for key in NOT_WITH {
        assert!(asked(&json!({ "marks": true, key: 3 })).is_err(), "{key}");
    }
}
