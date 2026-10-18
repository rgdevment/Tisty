use super::{reconciled, stages, title};
use crate::event::Op;
use crate::model::{STEP_AT_MOST, Step, Task};
use ulid::Ulid;

fn task(steps: &[(&str, bool, &str)]) -> Task {
    let mut task = Task::new(Ulid::generate(), "Rina's booster", "a0");
    task.steps = steps
        .iter()
        .map(|(text, done, order)| Step {
            id: Ulid::generate(),
            text: (*text).into(),
            done: *done,
            order: (*order).into(),
            by_agent: false,
        })
        .collect();
    task
}

fn named(task: &Task, text: &str) -> Ulid {
    task.steps.iter().find(|one| one.text == text).unwrap().id
}

#[test]
fn every_box_is_a_stage_in_reading_order_but_never_one_inside_code() {
    let body = "---\ntask: x\n---\n\nIntro.\n\n- [x] Bring the record book\n\nMore words.\n\n> - [ ] Ask about the tablet\n\n```md\n- [ ] not a stage\n```\n\n- plain item\n";
    let found = stages(body);
    let said: Vec<(&str, bool)> = found
        .iter()
        .map(|one| (one.text.as_str(), one.done))
        .collect();
    assert_eq!(
        said,
        [
            ("Bring the record book", true),
            ("Ask about the tablet", false)
        ]
    );
}

#[test]
fn a_long_stage_keeps_a_title_cut_at_a_word() {
    let long = "word ".repeat(40);
    let cut = title(&long);
    assert!(cut.chars().count() <= STEP_AT_MOST, "{cut}");
    assert!(cut.ends_with("word…"), "{cut}");
    assert_eq!(title("  short   one "), "short one");
}

#[test]
fn a_document_that_says_what_the_steps_say_asks_for_nothing() {
    let task = task(&[("one", false, "a"), ("two", true, "b")]);
    assert_eq!(
        reconciled(&task, "- [ ] one\n\nWords.\n\n- [ ] two\n"),
        Vec::new()
    );
}

#[test]
fn a_stage_renamed_in_place_keeps_its_step() {
    let task = task(&[
        ("one", false, "a"),
        ("two", true, "b"),
        ("three", false, "c"),
    ]);
    let ops = reconciled(&task, "- [ ] one\n- [x] two, said better\n- [ ] three\n");
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(
        matches!(&ops[0], Op::StepText { d, .. } if d.step == named(&task, "two") && d.text == "two, said better"),
        "{ops:?}"
    );
}

#[test]
fn a_stage_moved_with_its_words_is_reordered_not_removed() {
    let task = task(&[
        ("one", false, "a"),
        ("two", false, "b"),
        ("three", false, "c"),
    ]);
    let ops = reconciled(&task, "- [ ] three\n- [ ] one\n- [ ] two\n");
    assert!(
        ops.iter().all(|one| matches!(one, Op::StepReorder { .. })),
        "{ops:?}"
    );
    assert!(!ops.is_empty());
}

#[test]
fn of_two_stages_alike_only_the_one_taken_away_goes() {
    let task = task(&[("same", false, "a"), ("same", true, "b")]);
    let ops = reconciled(&task, "- [ ] same\n");
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(matches!(ops[0], Op::StepRemove { .. }), "{ops:?}");
}

#[test]
fn a_stage_written_in_the_middle_is_added_between_its_neighbours() {
    let task = task(&[("one", false, "a"), ("three", false, "c")]);
    let ops = reconciled(&task, "- [ ] one\n- [x] two\n- [ ] three\n");
    let Op::StepAdd { d, .. } = &ops[0] else {
        panic!("{ops:?}");
    };
    assert_eq!(d.text, "two");
    assert!(
        d.order.as_str() > "a" && d.order.as_str() < "c",
        "{}",
        d.order
    );
    assert!(
        matches!(&ops[1], Op::StepDone { d: done, .. } if done.step == d.step),
        "a new stage written ticked is a step done: {ops:?}"
    );
}

#[test]
fn a_box_left_unticked_in_the_file_never_undoes_a_tick_made_elsewhere() {
    let task = task(&[("one", true, "a")]);
    assert_eq!(reconciled(&task, "- [ ] one\n"), Vec::new());
}

#[test]
fn a_stage_taken_out_removes_its_step() {
    let task = task(&[("one", false, "a"), ("two", false, "b")]);
    let ops = reconciled(&task, "- [ ] one\n\nThe rest is prose now.\n");
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(
        matches!(&ops[0], Op::StepRemove { d, .. } if d.step == named(&task, "two")),
        "{ops:?}"
    );
}

#[test]
fn a_document_going_out_ticks_its_boxes_as_the_log_has_them() {
    let task = task(&[("one", true, "a"), ("two", false, "b")]);
    let out = super::ticked(&task, "- [ ] one\n\nWords.\n\n- [x] two\n");
    assert_eq!(out, "- [x] one\n\nWords.\n\n- [ ] two\n");
}
