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

#[test]
fn a_stage_whose_step_was_taken_away_in_the_app_does_not_bring_it_back() {
    let task = task(&[("one", false, "a")]);
    let ops = reconciled(
        &task,
        Some("- [ ] one\n- [ ] two\n"),
        "- [x] one\n- [ ] two\n",
    );
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert!(matches!(ops[0], Op::StepDone { .. }), "{ops:?}");
}

#[test]
fn a_long_stage_already_kept_is_not_added_again_on_every_save() {
    let long = format!("{} first", "word ".repeat(30));
    let other = format!("{} second", "word ".repeat(30));
    let task = task(&[
        (title(&long).as_str(), false, "a"),
        (title(&other).as_str(), false, "b"),
    ]);
    let body = format!("- [ ] {long}\n- [ ] {other}\n");
    assert_eq!(reconciled(&task, Some(&body), &body), Vec::new());
}

#[test]
fn wrapped_words_stay_with_their_stage_until_a_list_or_code_begins() {
    let body = "- [ ] Buy flour\n  1.5 kg of it\n- [ ] Read\n  *carefully* the docs\n- [ ] Run\n  ```\n  cargo x\n  ```\n";
    let said: Vec<String> = stages(body).into_iter().map(|one| one.text).collect();
    assert_eq!(
        said,
        ["Buy flour 1.5 kg of it", "Read *carefully* the docs", "Run"]
    );
}

#[test]
fn a_step_not_in_the_document_keeps_its_place_when_the_others_move() {
    let task = task(&[
        ("one", false, "a"),
        ("hidden", false, "m"),
        ("two", false, "y"),
    ]);
    let ops = reconciled(
        &task,
        Some(
            "- [ ] one
- [ ] two
",
        ),
        "- [ ] two
- [ ] one
",
    );
    let mut keys: std::collections::BTreeMap<Ulid, String> = task
        .steps
        .iter()
        .map(|one| (one.id, one.order.clone()))
        .collect();
    for one in &ops {
        if let Op::StepReorder { d, .. } = one {
            keys.insert(d.step, d.order.clone());
        }
    }
    let mut read: Vec<(&String, Ulid)> = keys.iter().map(|(id, key)| (key, *id)).collect();
    read.sort();
    let order: Vec<Ulid> = read.into_iter().map(|(_, id)| id).collect();
    assert_eq!(
        order,
        [
            named(&task, "two"),
            named(&task, "hidden"),
            named(&task, "one")
        ],
        "{ops:?}"
    );
}

#[test]
fn a_move_made_in_the_app_is_not_undone_by_a_document_saved_with_its_old_order() {
    let task = task(&[("one", false, "b"), ("two", false, "a")]);
    assert_eq!(
        reconciled(
            &task,
            Some("- [ ] one\n- [ ] two\n"),
            "- [ ] one\n- [ ] two\n\nMore.\n"
        ),
        Vec::new()
    );
}

#[test]
fn a_stage_added_never_shares_its_key_with_a_step_the_document_does_not_have() {
    let cases = [
        (
            task(&[
                ("one", false, "a"),
                ("hidden", false, "m"),
                ("two", false, "y"),
            ]),
            "- [ ] one\n- [ ] two\n",
            "- [ ] one\n- [ ] new\n- [ ] two\n",
            ("a", "m"),
        ),
        (
            task(&[("one", false, "a"), ("added elsewhere", false, "b")]),
            "- [ ] one\n",
            "- [ ] one\n- [ ] new\n",
            ("a", "b"),
        ),
    ];
    for (task, before, now, (low, high)) in cases {
        let ops = reconciled(&task, Some(before), now);
        let Some(Op::StepAdd { d, .. }) = ops.first() else {
            panic!("{ops:?}");
        };
        assert!(
            d.order.as_str() > low && d.order.as_str() < high,
            "{}",
            d.order
        );
    }
}

#[test]
fn wrapped_words_that_only_look_like_a_heading_or_a_rule_stay_with_their_stage() {
    let body = "- [ ] Call the vet\n  #pets before friday\n- [ ] Read\n  ***really*** carefully\n";
    let said: Vec<String> = stages(body).into_iter().map(|one| one.text).collect();
    assert_eq!(
        said,
        [
            "Call the vet #pets before friday",
            "Read ***really*** carefully"
        ]
    );
}

#[test]
fn a_hard_break_at_the_end_of_a_wrapped_line_is_not_part_of_the_step() {
    let body = "- [ ] Bring the book\\\n  and the card\n";
    let said: Vec<String> = stages(body).into_iter().map(|one| one.text).collect();
    assert_eq!(said, ["Bring the book and the card"]);
}

#[test]
fn a_word_that_ends_right_at_the_limit_is_kept_whole() {
    let words = format!("{} {} tail", "a".repeat(49), "b".repeat(49));
    assert_eq!(
        title(&words),
        format!("{} {}…", "a".repeat(49), "b".repeat(49))
    );
    let one = title(&"c".repeat(STEP_AT_MOST + 5));
    assert_eq!(one.chars().count(), STEP_AT_MOST);
}

#[test]
fn a_step_filed_from_elsewhere_joins_the_last_list_written_its_way() {
    let task = task(&[("one", false, "a"), ("from an agent", true, "b")]);
    assert_eq!(
        ticked(&task, "Words.\n\n- [ ] one\n\nMore words.\n"),
        "Words.\n\n- [ ] one\n- [x] from an agent\n\nMore words.\n"
    );
    assert_eq!(
        ticked(&task, "> * [ ] one"),
        "> * [ ] one\n> * [x] from an agent\n"
    );
}

#[test]
fn a_step_with_no_list_yet_goes_under_the_template_s_box_hint() {
    let task = task(&[("from an agent", false, "a")]);
    assert_eq!(
        ticked(
            &task,
            "## Steps\n<!-- - [ ] The first step -->\n\n## Notes\n"
        ),
        "## Steps\n<!-- - [ ] The first step -->\n\n- [ ] from an agent\n\n## Notes\n"
    );
}

#[test]
fn a_step_reworded_elsewhere_rewrites_its_own_line() {
    let task = task(&[("one", true, "a"), ("two", false, "b")]);
    let body = "- [x] one\n- [ ] two\n\nWords.\n";
    assert_eq!(
        super::reworded(body, &task, named(&task, "two"), "two, again"),
        Some("- [x] one\n- [ ] two, again\n\nWords.\n".to_string())
    );
    assert_eq!(
        super::reworded(body, &task, named(&task, "one"), "anything"),
        None
    );
}

#[test]
fn of_two_steps_written_alike_only_the_one_reworded_changes_its_line() {
    let task = task(&[("call", false, "a"), ("call", false, "b")]);
    let second = task.steps[1].id;
    assert_eq!(
        super::reworded("- [ ] call\n- [ ] call\n", &task, second, "email"),
        Some("- [ ] call\n- [ ] email\n".to_string())
    );
}

#[test]
fn a_step_taken_off_leaves_no_box_behind() {
    let task = task(&[
        ("one", false, "a"),
        ("two", true, "b"),
        ("three", false, "c"),
    ]);
    let body = "- [ ] one\n- [x] two\n- [ ] three\n";
    assert_eq!(
        super::unplanned(body, &task, &[named(&task, "one"), named(&task, "two")]),
        "- [x] two\n- [ ] three\n"
    );
}

#[test]
fn of_two_steps_written_alike_only_the_one_taken_off_loses_its_box() {
    let task = task(&[("call", false, "a"), ("call", false, "b")]);
    let first = task.steps[0].id;
    assert_eq!(
        super::unplanned("- [ ] call\n- [ ] call\n", &task, &[first]),
        "- [ ] call\n"
    );
}

#[test]
fn a_step_filed_on_a_page_with_no_box_goes_at_its_end() {
    let task = task(&[("one", false, "a"), ("from an agent", true, "b")]);
    assert_eq!(
        ticked(&task, "---\ntask: x\n---\n"),
        "---\ntask: x\n---\n\n- [ ] one\n- [x] from an agent\n"
    );
}

#[test]
fn a_step_whose_line_cannot_read_back_hides_no_other_step() {
    let task = task(&[("one", false, "a"), ("", false, "b"), ("three", false, "c")]);
    let out = ticked(&task, "Words.\n");
    assert!(out.contains("- [ ] one\n"), "{out}");
    assert!(out.contains("- [ ] three\n"), "{out}");
    assert_eq!(stages(&out).len(), 2, "{out}");
}

#[test]
fn the_boxes_inside_a_span_name_their_own_steps() {
    let task = task(&[("water", false, "a"), ("cake", false, "b")]);
    let body = "## Every round\n- [ ] water\n\n## This time\n- [ ] cake\n";
    let from = body.find("## This time").unwrap();
    assert_eq!(
        super::boxed_in(&task, body, from..body.len()),
        vec![named(&task, "cake")]
    );
}

#[test]
fn steps_are_not_added_where_an_open_fence_would_swallow_them() {
    let task = task(&[("one", false, "a")]);
    let body = "Words.\n\n```\ncode";
    assert_eq!(ticked(&task, body), body);
}

#[test]
fn a_step_taken_off_with_one_under_it_leaves_the_rest_whole_and_moves_it_out() {
    let task = task(&[("A", false, "a"), ("B", false, "b")]);
    let body = "- [ ] A\n  - [ ] B\n\n## Notes\nkeep me\n";
    assert_eq!(
        super::unplanned(body, &task, &[named(&task, "A"), named(&task, "B")]),
        "\n## Notes\nkeep me\n"
    );
    assert_eq!(
        super::unplanned(body, &task, &[named(&task, "A")]),
        "- [ ] B\n\n## Notes\nkeep me\n"
    );
}

#[test]
fn a_step_written_on_two_lines_is_reworded_whole() {
    let task = task(&[
        ("Call the bank about the loan", false, "a"),
        ("Next", false, "b"),
    ]);
    let body = "- [ ] Call the bank\n  about the loan\n- [ ] Next\n";
    assert_eq!(
        super::reworded(
            body,
            &task,
            named(&task, "Call the bank about the loan"),
            "Email the bank"
        ),
        Some("- [ ] Email the bank\n- [ ] Next\n".to_string())
    );
}

#[test]
fn a_new_step_joins_the_outer_list_not_one_nested_under_a_step() {
    let task = task(&[("A", false, "a"), ("A.1", false, "b"), ("B", false, "c")]);
    assert_eq!(
        ticked(&task, "- [ ] A\n  - [ ] A.1\n"),
        "- [ ] A\n  - [ ] A.1\n- [ ] B\n"
    );
}

#[test]
fn two_steps_written_over_side_by_side_keep_who_they_are_when_the_editor_says_so() {
    let task = task(&[("a", false, "a"), ("b", true, "b")]);
    let before = "- [ ] a\n- [x] b\n";
    let now = "- [ ] call the vet\n- [x] bring the card\n";

    let blind = reconciled(&task, Some(before), now);
    assert!(blind.iter().any(|op| matches!(op, Op::StepRemove { .. })));

    let told = super::reconciled_with(
        &task,
        Some(before),
        now,
        &[
            ("a".into(), "call the vet".into()),
            ("b".into(), "bring the card".into()),
        ],
    );
    assert_eq!(told.len(), 2, "{told:?}");
    assert!(
        told.iter().all(|op| matches!(op, Op::StepText { .. })),
        "{told:?}"
    );
}

#[test]
fn a_step_ending_in_a_backslash_is_still_its_own_box() {
    let task = task(&[("one", true, "a"), ("the other aside\\", true, "b")]);
    let body = "- [x] one\n- [x] the other aside\\\n";
    assert_eq!(ticked(&task, body), body);
    assert_eq!(reconciled(&task, None, body), Vec::new());
}
