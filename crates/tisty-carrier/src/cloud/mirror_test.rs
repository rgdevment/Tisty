use super::*;

#[test]
fn a_full_drive_gets_the_history_then_the_documents_and_the_attachments_last() {
    let mut names = [
        "attachments/ab/plano.pdf",
        "docs/notas.md",
        "machines/dev_a.json",
        "store/dev_a.jsonl",
    ];

    names.sort_by_key(|name| (pushed_in_turn(name), name.to_string()));

    assert_eq!(
        names,
        [
            "store/dev_a.jsonl",
            "docs/notas.md",
            "machines/dev_a.json",
            "attachments/ab/plano.pdf",
        ]
    );
}
