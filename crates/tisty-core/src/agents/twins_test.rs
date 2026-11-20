use super::*;
use crate::agents::{Kind, doc_of};

fn global(name: &str, said: &str) -> String {
    doc_of(GLOBAL, Kind::Rule, name, said.as_bytes()).unwrap()
}

fn address(body: &str) -> String {
    Card::of(body).unwrap().address()
}

#[test]
fn the_newer_of_two_global_rules_sharing_a_name_takes_the_next_free_one() {
    let older = ulid::Ulid::from_parts(1, 1);
    let newer = ulid::Ulid::from_parts(2, 2);
    let docs = vec![
        (newer, "laptop".to_string(), global("rule.md", "Spanish.\n")),
        (older, "desk".to_string(), global("rule.md", "Short.\n")),
        (
            ulid::Ulid::from_parts(3, 3),
            "taken".to_string(),
            global("rule-2.md", "Tests.\n"),
        ),
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
        (
            ulid::Ulid::from_parts(1, 1),
            "desk".to_string(),
            global("style.md", "A.\n"),
        ),
        (
            ulid::Ulid::from_parts(2, 2),
            "laptop".to_string(),
            global("style.md", "B.\n"),
        ),
    ];
    let mut reversed = docs.clone();
    reversed.reverse();
    let renamed = renamed_twins(&docs);
    assert_eq!(renamed, renamed_twins(&reversed));

    let after: Vec<_> = docs
        .into_iter()
        .map(|(id, file, body)| {
            let body = renamed
                .iter()
                .find(|(one, _)| *one == file)
                .map_or(body, |(_, new)| new.clone());
            (id, file, body)
        })
        .collect();
    assert_eq!(address(&after[1].2), "global/rule/style-2.md");
    assert!(renamed_twins(&after).is_empty());
}

#[test]
fn twins_of_an_assistants_own_file_are_one_file_and_keep_their_name() {
    let claude = doc_of("claude", Kind::Rule, "CLAUDE.md", b"Mine.\n").unwrap();
    let docs = vec![
        (
            ulid::Ulid::from_parts(1, 1),
            "desk".to_string(),
            claude.clone(),
        ),
        (ulid::Ulid::from_parts(2, 2), "laptop".to_string(), claude),
    ];
    assert!(renamed_twins(&docs).is_empty());
}
