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
