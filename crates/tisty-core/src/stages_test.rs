use super::{reconciled, stages, ticked, title};
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
fn every_box_is_a_stage_in_reading_order_but_never_one_inside_code_nor_an_empty_one() {
    let body = "---\ntask: x\n---\n\nIntro.\n\n- [x] Bring the record book\n  and the card\n- [ ] \n\nMore words.\n\n> - [ ] Ask about the tablet\n\n```md\n- [ ] not a stage\n```\n\n- plain item\n";
    let said: Vec<(String, bool)> = stages(body)
        .into_iter()
        .map(|one| (one.text, one.done))
        .collect();
    assert_eq!(
        said,
        [
            ("Bring the record book and the card".to_string(), true),
            ("Ask about the tablet".to_string(), false)
        ]
    );
}

#[test]
fn a_long_stage_keeps_a_title_cut_at_a_word() {
    let cut = title(&"word ".repeat(40));
    assert!(cut.chars().count() <= STEP_AT_MOST, "{cut}");
    assert!(cut.ends_with("word…"), "{cut}");
    assert_eq!(title("  short   one "), "short one");
}

#[test]
fn a_document_saved_again_as_it_was_asks_for_nothing() {
    let task = task(&[("one", false, "a"), ("two", true, "b")]);
    let body = "- [ ] one\n\nWords.\n\n- [ ] two\n";
    assert_eq!(reconciled(&task, Some(body), body), Vec::new());
}

#[test]
fn a_document_with_no_boxes_never_takes_the_steps_away() {
    let task = task(&[("one", false, "a"), ("two", false, "b")]);
    assert_eq!(
        reconciled(&task, Some("Words.\n"), "Other words.\n"),
        Vec::new()
    );
}

#[test]
fn a_step_added_in_the_app_survives_a_document_that_does_not_have_it_yet() {
    let task = task(&[("one", false, "a"), ("added elsewhere", false, "b")]);
    let body = "- [ ] one\n";
    assert_eq!(
        reconciled(&task, Some(body), "- [ ] one\n\nMore.\n"),
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
    let before = "- [ ] one\n- [x] two\n- [ ] three\n";
    let ops = reconciled(
        &task,
        Some(before),
        "- [ ] one\n- [x] two, said better\n- [ ] three\n",
    );
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(
        matches!(&ops[0], Op::StepText { d, .. } if d.step == named(&task, "two") && d.text == "two, said better"),
        "{ops:?}"
    );
}

#[test]
fn two_stages_rewritten_with_nothing_to_hold_them_are_new_ones() {
    let task = task(&[("Buy milk", true, "a"), ("Pay rent", false, "b")]);
    let before = "- [x] Buy milk\n- [ ] Pay rent\n";
    let ops = reconciled(
        &task,
        Some(before),
        "- [ ] Call the vet\n- [ ] Book a flight\n",
    );
    let adds = ops
        .iter()
        .filter(|one| matches!(one, Op::StepAdd { .. }))
        .count();
    let gone = ops
        .iter()
        .filter(|one| matches!(one, Op::StepRemove { .. }))
        .count();
    assert_eq!((adds, gone), (2, 2), "{ops:?}");
}

#[test]
fn a_stage_moved_with_its_words_is_reordered_not_removed() {
    let task = task(&[
        ("one", false, "a"),
        ("two", false, "b"),
        ("three", false, "c"),
    ]);
    let before = "- [ ] one\n- [ ] two\n- [ ] three\n";
    let ops = reconciled(&task, Some(before), "- [ ] three\n- [ ] one\n- [ ] two\n");
    assert!(!ops.is_empty());
    assert!(
        ops.iter().all(|one| matches!(one, Op::StepReorder { .. })),
        "{ops:?}"
    );
}

#[test]
fn of_two_stages_alike_only_the_one_taken_away_goes() {
    let task = task(&[("same", false, "a"), ("same", true, "b")]);
    let ops = reconciled(&task, Some("- [ ] same\n- [x] same\n"), "- [ ] same\n");
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(matches!(ops[0], Op::StepRemove { .. }), "{ops:?}");
}

#[test]
fn a_stage_written_in_the_middle_is_added_between_its_neighbours_ticked_as_written() {
    let task = task(&[("one", false, "a"), ("three", false, "c")]);
    let ops = reconciled(
        &task,
        Some("- [ ] one\n- [ ] three\n"),
        "- [ ] one\n- [x] two\n- [ ] three\n",
    );
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
        "{ops:?}"
    );
}

#[test]
fn a_box_ticked_in_the_document_is_learnt_and_a_stale_one_is_not() {
    let task = task(&[("one", false, "a"), ("two", true, "b")]);
    let before = "- [ ] one\n- [ ] two\n";
    let ops = reconciled(&task, Some(before), "- [x] one\n- [ ] two\n");
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(
        matches!(&ops[0], Op::StepDone { d, .. } if d.step == named(&task, "one")),
        "the stale box of two, unticked before and after, undid nothing: {ops:?}"
    );
}

#[test]
fn a_stage_taken_out_removes_its_step() {
    let task = task(&[("one", false, "a"), ("two", false, "b")]);
    let ops = reconciled(
        &task,
        Some("- [ ] one\n- [ ] two\n"),
        "- [ ] one\n\nThe rest is prose now.\n",
    );
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(
        matches!(&ops[0], Op::StepRemove { d, .. } if d.step == named(&task, "two")),
        "{ops:?}"
    );
}

#[test]
fn a_long_step_kept_from_before_is_not_cut_by_a_document_that_repeats_it() {
    let long = "word ".repeat(30);
    let task = task(&[(long.trim(), false, "a")]);
    let body = format!("- [ ] {long}\n");
    assert_eq!(reconciled(&task, None, &body), Vec::new());
}

#[test]
fn a_document_going_out_ticks_its_boxes_as_the_log_has_them() {
    let task = task(&[("one", true, "a"), ("two", false, "b")]);
    let out = ticked(&task, "- [ ] one\n\nWords.\n\n- [x] two\n");
    assert_eq!(out, "- [x] one\n\nWords.\n\n- [ ] two\n");
}
