use super::paged;
use crate::event::{Op, PageChange};
use ulid::Ulid;

fn changes(before: &str, now: &str) -> Vec<(String, PageChange)> {
    paged(Ulid::generate(), before, now)
        .into_iter()
        .map(|op| match op {
            Op::TaskPaged { d, .. } => (d.section, d.change),
            other => panic!("{other:?}"),
        })
        .collect()
}

const PAGE: &str =
    "## Description\n<!-- why -->\n\nRenew it.\n\n## Steps\n- [ ] Call\n\n## Notes\n";

#[test]
fn words_written_in_a_section_are_told_by_its_name_and_steps_or_hints_are_not() {
    let now = PAGE.replace("Renew it.", "Renew it before Friday.");
    assert_eq!(
        changes(PAGE, &now),
        vec![("Description".into(), PageChange::Written)]
    );
    let ticked = PAGE.replace("- [ ] Call", "- [x] Call\n- [ ] Pay");
    assert_eq!(changes(PAGE, &ticked), Vec::new());
}

#[test]
fn a_section_added_renamed_or_taken_off_is_told_as_such() {
    let added = format!("{PAGE}\n## When it closes\n");
    assert_eq!(
        changes(PAGE, &added),
        vec![("When it closes".into(), PageChange::Added)]
    );
    let renamed = PAGE.replace("## Notes", "## Findings");
    assert_eq!(
        changes(PAGE, &renamed),
        vec![(
            "Findings".into(),
            PageChange::Renamed {
                from: "Notes".into()
            }
        )]
    );
    let gone = PAGE.replace("## Notes\n", "");
    assert_eq!(
        changes(PAGE, &gone),
        vec![("Notes".into(), PageChange::Removed)]
    );
}

#[test]
fn a_heading_inside_code_is_no_section() {
    let now = PAGE.replace("Renew it.", "Renew it.\n\n```\n## not a section\n```");
    assert_eq!(
        changes(PAGE, &now),
        vec![("Description".into(), PageChange::Written)]
    );
}

#[test]
fn a_section_moved_elsewhere_is_neither_added_nor_taken_off() {
    let moved = "## Description\n<!-- why -->\n\nRenew it.\n\n## Notes\n\n## Steps\n- [ ] Call\n";
    assert_eq!(changes(PAGE, moved), Vec::new());
}

#[test]
fn a_mark_stuck_to_a_word_is_part_of_the_section_s_name() {
    let now = PAGE.replace("## Notes", "## Notes on C#");
    assert_eq!(
        changes(PAGE, &now),
        vec![(
            "Notes on C#".into(),
            PageChange::Renamed {
                from: "Notes".into()
            }
        )]
    );
}

#[test]
fn a_section_moved_while_another_is_renamed_is_told_as_the_rename_alone() {
    let before = "## A\n\n## M\n\n## B\n\n## C\n";
    let now = "## A\n\n## B\n\n## M\n\n## X\n";
    assert_eq!(
        changes(before, now),
        vec![("X".into(), PageChange::Renamed { from: "C".into() })]
    );
}

#[test]
fn a_bullet_that_opens_with_a_link_is_words_not_a_step() {
    let before = "## Notes\n- [Invoice](https://x.y)\n";
    let now = "## Notes\n- [Invoice](https://x.y)\n- [Receipt](https://x.y)\n";
    assert_eq!(
        changes(before, now),
        vec![("Notes".into(), PageChange::Written)]
    );
}

#[test]
fn a_section_written_in_for_a_while_is_told_once_and_again_after_a_long_quiet() {
    let task = Ulid::generate();
    let now = PAGE.replace("Renew it.", "Renew it before Friday.");
    let mut told = std::collections::HashMap::new();
    let mut kept = |at: i64| super::quieted(paged(task, PAGE, &now), &mut told, at).len();

    assert_eq!(kept(0), 1);
    assert_eq!(kept(super::WRITING_FOR - 1), 0);
    assert_eq!(kept(2 * super::WRITING_FOR - 2), 0);
    assert_eq!(kept(3 * super::WRITING_FOR), 1);
}
