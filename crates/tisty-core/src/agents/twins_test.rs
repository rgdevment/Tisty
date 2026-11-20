use super::*;
use crate::agents::{Kind, doc_of};

type Doc = (DocId, String, String, bool);

fn global(name: &str, said: &str) -> String {
    doc_of(GLOBAL, Kind::Rule, name, said.as_bytes()).unwrap()
}

fn one(n: u64, file: &str, body: String, agreed: bool) -> Doc {
    (
        ulid::Ulid::from_parts(n, n.into()),
        file.to_string(),
        body,
        agreed,
    )
}

fn address(body: &str) -> String {
    Card::of(body).unwrap().address()
}

#[test]
fn the_newer_of_two_global_rules_sharing_a_name_takes_the_next_free_one() {
    let docs = vec![
        one(2, "laptop", global("rule.md", "Spanish.\n"), true),
        one(1, "desk", global("rule.md", "Short.\n"), true),
        one(3, "taken", global("rule-2.md", "Tests.\n"), true),
    ];

    let renamed = renamed_twins(&docs);

    assert_eq!(renamed.len(), 1);
    let (file, body) = &renamed[0];
    assert_eq!(file, "laptop");
    assert_eq!(address(body), "global/rule/rule-3.md");
    assert!(
        body.ends_with("Spanish.\n"),
        "only the name changes: {body}"
    );
    assert!(body.contains("title: global · rule-3.md"));
}

#[test]
fn both_computers_rename_alike_and_a_second_pass_finds_nothing() {
    let docs = vec![
        one(1, "desk", global("style.md", "A.\n"), true),
        one(2, "laptop", global("style.md", "B.\n"), true),
    ];
    let mut reversed = docs.clone();
    reversed.reverse();
    let renamed = renamed_twins(&docs);
    assert_eq!(renamed, renamed_twins(&reversed));

    let after: Vec<Doc> = docs
        .into_iter()
        .map(|(id, file, body, agreed)| {
            let body = renamed
                .iter()
                .find(|(named, _)| *named == file)
                .map_or(body, |(_, new)| new.clone());
            (id, file, body, agreed)
        })
        .collect();
    assert_eq!(address(&after[1].2), "global/rule/style-2.md");
    assert!(renamed_twins(&after).is_empty());
}

#[test]
fn twins_of_an_assistants_own_file_are_one_file_and_keep_their_name() {
    let claude = doc_of("claude", Kind::Rule, "CLAUDE.md", b"Mine.\n").unwrap();
    let docs = vec![
        one(1, "desk", claude.clone(), true),
        one(2, "laptop", claude, true),
    ];
    assert!(renamed_twins(&docs).is_empty());
}

#[test]
fn a_global_skill_brought_twice_is_one_skill_and_keeps_its_name() {
    let skill = doc_of(GLOBAL, Kind::Skill, "review/SKILL.md", b"Review.\n").unwrap();
    let docs = vec![
        one(1, "desk", skill.clone(), true),
        one(2, "laptop", skill, true),
    ];
    assert!(renamed_twins(&docs).is_empty());
}

#[test]
fn an_older_twin_waiting_for_approval_still_counts_and_an_unapproved_one_is_never_rewritten() {
    let docs = vec![
        one(1, "desk", global("rule.md", "A.\n"), false),
        one(2, "laptop", global("rule.md", "B.\n"), true),
        one(3, "phone", global("rule.md", "C.\n"), false),
    ];
    let renamed = renamed_twins(&docs);
    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0].0, "laptop");
}
