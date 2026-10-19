use super::{file_at, id_at, made, of, seed, task_in, task_of_file};
use crate::event::{DeviceId, Event, Op, TaskAdd};
use crate::state::State;
use ulid::Ulid;

fn applied(state: &mut State, op: Op) {
    state.apply(&Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(1).unwrap(),
        op,
    ));
}

#[test]
fn a_task_names_its_document_the_same_way_everywhere_and_never_by_its_own_id() {
    let task = Ulid::from_string("01M563T6T541A3Q2PP5YQW5XK1").unwrap();

    assert_eq!(file_at(task, 1), "t_01m563t6t541a3q2pp5yqw5xk1-0001");
    assert!(crate::docs::resolve(std::path::Path::new("docs"), &file_at(task, 1)).is_ok());
    assert_eq!(task_of_file(&file_at(task, 2)), Some(task));
    assert_eq!(task_of_file("dev_a-0001"), None);
    assert_eq!(id_at(task, 1), id_at(task, 1));
    assert_ne!(id_at(task, 1), id_at(task, 2));
    assert_ne!(
        id_at(task, 1),
        task,
        "the document would answer to the task's permissions"
    );
}

#[test]
fn the_seed_names_its_task_and_reads_back() {
    let task = Ulid::generate();
    assert_eq!(task_in(&seed(task)), Some(task));
    assert_eq!(task_in("# Notes\n\nNo properties here.\n"), None);
    assert_eq!(task_in("---\ntask: not-an-id\n---\n"), None);
}

#[test]
fn a_document_made_twice_for_one_task_is_one_file_that_belongs_to_it() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );

    let first = made(room.path(), &state, task, task, "", None).unwrap();
    std::fs::write(
        room.path().join(format!("{}.md", file_at(task, 1))),
        format!("{}\nWords already written.\n", seed(task)),
    )
    .unwrap();
    let again = made(room.path(), &state, task, task, "", None).unwrap();

    let body = crate::docs::read(room.path(), &file_at(task, 1)).unwrap();
    assert!(body.contains("Words already written."), "{body}");
    applied(&mut state, first);
    applied(&mut state, again);
    let kept = of(&state, task).expect("the task's document");
    assert_eq!(kept.task_of, Some(task));
    assert_eq!(kept.file, file_at(task, 1));
    assert_eq!(state.docs.len(), 1);
}

#[test]
fn a_document_made_again_after_its_delete_takes_the_next_turn_and_belongs_to_the_task() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    let first = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, first);
    let gone = of(&state, task).unwrap().id;
    applied(&mut state, Op::DocDelete { id: gone });
    std::fs::remove_file(room.path().join(format!("{}.md", file_at(task, 1)))).unwrap();

    let again = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, again);

    let kept = of(&state, task).expect("the task's document again");
    assert_eq!(kept.file, file_at(task, 2));
    assert_eq!(kept.id, id_at(task, 2));
}

#[test]
fn a_second_making_keeps_what_was_built_on_the_first() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    let first = made(room.path(), &state, task, task, "", None).unwrap();
    let elsewhere = first.clone();
    applied(&mut state, first);
    let id = of(&state, task).unwrap().id;
    applied(&mut state, Op::DocArchive { id });

    applied(&mut state, elsewhere);

    assert!(
        of(&state, task).unwrap().archived,
        "the second making undid the first"
    );
}

#[test]
fn a_file_taken_in_before_its_making_gets_the_id_its_making_would_give() {
    let task = Ulid::generate();
    assert_eq!(
        super::owned(&file_at(task, 3)),
        Some((task, id_at(task, 3)))
    );
}

#[test]
fn only_the_name_a_making_writes_belongs_to_a_task() {
    let task = Ulid::generate();
    let lower = task.to_string().to_ascii_lowercase();
    assert_eq!(super::owned(&format!("t_{lower}-1")), None);
    assert_eq!(super::owned(&format!("t_{lower}-0000")), None);
    for file in [format!("t_{lower}-0001"), format!("T_{task}-0001")] {
        assert_eq!(super::owned(&file), Some((task, id_at(task, 1))), "{file}");
    }
}

#[test]
fn a_task_s_description_and_steps_open_its_document() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );
    applied(
        &mut state,
        Op::TaskDescribe {
            id: task,
            d: crate::event::Body {
                body: Some("Call the vet first.".into()),
            },
        },
    );
    for (step, text, order) in [
        (Ulid::generate(), "book", "b"),
        (Ulid::generate(), "pay", "a"),
    ] {
        applied(
            &mut state,
            Op::StepAdd {
                id: task,
                d: crate::event::StepAdd {
                    step,
                    text: text.into(),
                    order: order.into(),
                },
            },
        );
    }
    let paid = state.tasks[&task]
        .steps
        .iter()
        .find(|one| one.text == "pay")
        .unwrap()
        .id;
    applied(
        &mut state,
        Op::StepDone {
            id: task,
            d: crate::event::StepRef { step: paid },
        },
    );

    let first = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, first);

    let body = crate::docs::read(room.path(), &file_at(task, 1)).unwrap();
    assert_eq!(
        body,
        format!(
            "{}\nCall the vet first.\n\n- [x] pay\n- [ ] book\n",
            seed(task)
        )
    );
    assert!(crate::stages::reconciled(&state.tasks[&task], None, &body).is_empty());
}

#[test]
fn a_task_erased_for_good_takes_its_document_along() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );
    let first = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, first);
    let doc = of(&state, task).unwrap().id;

    assert_eq!(
        super::erased_with(&state, task),
        vec![Op::DocDelete { id: doc }, Op::TaskDelete { id: task }]
    );
    let alone = Ulid::generate();
    assert_eq!(
        super::erased_with(&state, alone),
        vec![Op::TaskDelete { id: alone }]
    );
}

#[test]
fn a_story_that_kept_its_task_keeps_its_words_through_an_erase_from_elsewhere() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );
    let first = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, first);
    let erase = super::erased_with(&state, task);
    applied(
        &mut state,
        Op::TaskUpdate {
            id: task,
            d: crate::event::TaskPatch {
                read_as: Some(Some(crate::Reading::Story)),
                ..Default::default()
            },
        },
    );

    for op in erase {
        applied(&mut state, op);
    }

    assert!(state.tasks.contains_key(&task));
    assert!(of(&state, task).is_some(), "the story lost its words");
}

#[test]
fn words_that_reach_a_task_already_erased_go_with_it() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );
    let words = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, Op::TaskDelete { id: task });

    applied(&mut state, words);

    assert!(state.docs.values().all(|one| one.task_of != Some(task)));
    assert!(state.is_erased(id_at(task, 1)));
}

#[test]
fn a_task_erased_without_naming_its_words_still_takes_them() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );
    let words = made(room.path(), &state, task, task, "", None).unwrap();
    applied(&mut state, words);

    applied(&mut state, Op::TaskDelete { id: task });

    assert!(of(&state, task).is_none());
    assert!(state.is_erased(id_at(task, 1)));
}

#[test]
fn every_turn_of_a_routine_answers_to_its_first() {
    let mut state = State::default();
    let first = Ulid::generate();
    let second = Ulid::generate();
    let third = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: first,
            d: TaskAdd::new("Water the plants", "a0"),
        },
    );
    for (id, before) in [(second, first), (third, second)] {
        let mut d = TaskAdd::new("Water the plants", "a1");
        d.after = Some(before);
        applied(&mut state, Op::TaskAdd { id, d });
    }

    assert_eq!(super::owner(&state, third), first);
    assert_eq!(super::owner(&state, second), first);
    assert_eq!(super::owner(&state, first), first);
    let alone = Ulid::generate();
    assert_eq!(super::owner(&state, alone), alone);
}

#[test]
fn a_routine_s_words_are_found_on_the_turn_still_open() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let first = Ulid::generate();
    let second = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: first,
            d: TaskAdd::new("Water the plants", "a0"),
        },
    );
    let words = made(room.path(), &state, first, first, "", None).unwrap();
    applied(&mut state, words);
    applied(
        &mut state,
        Op::TaskDone {
            id: first,
            filled: false,
        },
    );
    let mut d = TaskAdd::new("Water the plants", "a1");
    d.after = Some(first);
    applied(&mut state, Op::TaskAdd { id: second, d });

    assert_eq!(super::behind(&state).get(&file_at(first, 1)), Some(&second));
}

#[test]
fn a_task_s_document_opens_with_its_template_then_its_own_words() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Ship 4.2", "a0"),
        },
    );
    applied(
        &mut state,
        Op::TaskDescribe {
            id: task,
            d: crate::event::Body {
                body: Some("Friday, after lunch.".into()),
            },
        },
    );

    let first = made(
        room.path(),
        &state,
        task,
        task,
        "## Steps\r\n\r\n- [ ] Back up first\r\n",
        None,
    )
    .unwrap();
    applied(&mut state, first);

    let body = crate::docs::read(room.path(), &file_at(task, 1)).unwrap();
    assert_eq!(
        body,
        format!(
            "{}\n## Steps\n\n- [ ] Back up first\n\nFriday, after lunch.\n",
            seed(task)
        )
    );
    let ops = crate::stages::reconciled(&state.tasks[&task], None, &body);
    assert!(
        matches!(&ops[..], [Op::StepAdd { d, .. }] if d.text == "Back up first"),
        "{ops:?}"
    );
}

#[test]
fn a_step_the_template_already_lists_is_not_written_twice() {
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Ship 4.2", "a0"),
        },
    );
    applied(
        &mut state,
        Op::StepAdd {
            id: task,
            d: crate::event::StepAdd {
                step: Ulid::generate(),
                text: "Back up first".into(),
                order: "a0".into(),
            },
        },
    );

    let body = super::opened(&state, task, "## Steps\n\n- [ ] Back up first\n");

    assert_eq!(body.matches("Back up first").count(), 1, "{body}");
}

#[test]
fn a_task_s_words_go_into_the_template_s_sections_not_after_them() {
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Ship 4.2", "a0"),
        },
    );
    applied(
        &mut state,
        Op::TaskDescribe {
            id: task,
            d: crate::event::Body {
                body: Some("Friday.".into()),
            },
        },
    );
    applied(
        &mut state,
        Op::StepAdd {
            id: task,
            d: crate::event::StepAdd {
                step: Ulid::generate(),
                text: "Back up".into(),
                order: "a0".into(),
            },
        },
    );
    let template = "## Description\n<!-- What for. -->\n\n## Steps\n<!-- - [ ] The first step -->\n\n## Notes\n<!-- Anything. -->\n";

    let body = super::opened(&state, task, template);

    assert_eq!(
        body,
        format!(
            "{}\n## Description\n<!-- What for. -->\n\nFriday.\n\n## Steps\n<!-- - [ ] The first step -->\n\n- [ ] Back up\n\n## Notes\n<!-- Anything. -->\n",
            seed(task)
        )
    );
}

#[test]
fn a_description_fills_the_section_the_template_keeps_for_it() {
    let body = "---\ntask: x\n---\n\n## Description\n<!-- What and why -->\n\nOld words.\n\n## Steps\n<!-- - [ ] The first step -->\n";
    assert_eq!(
        super::described(body, "New words.\n"),
        "---\ntask: x\n---\n\n## Description\n<!-- What and why -->\n\nNew words.\n\n## Steps\n<!-- - [ ] The first step -->\n"
    );
    assert_eq!(
        super::described("---\ntask: x\n---\n\nNotes.\n", "Said."),
        "---\ntask: x\n---\n\nNotes.\n\nSaid.\n"
    );
}

#[test]
fn a_new_description_takes_the_old_one_s_own_headings_with_it() {
    let body = "## Description\n<!-- What and why -->\n";
    let once = super::described(
        body,
        "Intro\n# Context\nold detail\n```\n# not a heading\n```\n",
    );

    let twice = super::described(&format!("{once}\n## Steps\n"), "New text");

    assert_eq!(
        twice,
        "## Description\n<!-- What and why -->\n\nNew text\n\n## Steps\n"
    );
}

#[test]
fn a_routine_s_later_turn_still_reaches_its_page_once_the_first_is_erased() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let first = Ulid::generate();
    let second = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: first,
            d: TaskAdd::new("Water the plants", "a0"),
        },
    );
    let words = made(room.path(), &state, first, first, "", None).unwrap();
    applied(&mut state, words);
    let mut d = TaskAdd::new("Water the plants", "a1");
    d.after = Some(first);
    applied(&mut state, Op::TaskAdd { id: second, d });

    applied(&mut state, Op::TaskDelete { id: first });

    assert_eq!(super::owner(&state, second), first);
    assert!(of(&state, super::owner(&state, second)).is_some());
}

#[test]
fn a_routine_s_page_goes_with_its_last_turn_once_the_first_is_erased() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let first = Ulid::generate();
    let second = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: first,
            d: TaskAdd::new("Water the plants", "a0"),
        },
    );
    let words = made(room.path(), &state, first, first, "", None).unwrap();
    applied(&mut state, words);
    let mut d = TaskAdd::new("Water the plants", "a1");
    d.after = Some(first);
    applied(&mut state, Op::TaskAdd { id: second, d });
    applied(&mut state, Op::TaskDelete { id: first });

    applied(&mut state, Op::TaskDelete { id: second });

    assert!(of(&state, first).is_none());
    assert!(state.is_erased(id_at(first, 1)));
}

#[test]
fn an_open_task_s_journal_comes_to_its_page_as_notes_and_a_closed_one_s_stays_behind() {
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );
    applied(
        &mut state,
        Op::TaskLog {
            id: task,
            d: crate::event::LogAdd::new(Ulid::generate(), "Called the vet.\n- [ ] not a step"),
        },
    );
    let template = "## Notes\n<!-- What you find out -->\n";

    let page = super::opened(&state, task, template);
    let notes = page
        .split_once("<!-- What you find out -->\n\n**")
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    assert!(notes.contains("**\nCalled the vet."), "{page}");
    assert!(page.contains(r"- \[ ] not a step"), "{page}");
    assert!(crate::stages::stages(&page).is_empty(), "{page}");

    applied(
        &mut state,
        Op::TaskDone {
            id: task,
            filled: false,
        },
    );
    assert!(!super::opened(&state, task, template).contains("Called the vet."));
}

#[test]
fn a_routine_s_journal_stays_off_its_page_since_its_close_logs_this_time_again() {
    let mut state = State::default();
    let task = Ulid::generate();
    let mut round = TaskAdd::new("Water the plants", "a0");
    round.repeat = Some(crate::model::Repeat::due(crate::model::Cadence {
        every: 1,
        unit: crate::model::Unit::Week,
    }));
    applied(&mut state, Op::TaskAdd { id: task, d: round });
    applied(
        &mut state,
        Op::TaskLog {
            id: task,
            d: crate::event::LogAdd::new(Ulid::generate(), "The fern looked dry."),
        },
    );

    let page = super::opened(
        &state,
        task,
        "## Every round\n<!-- - [ ] What to do -->\n\n## This time\n<!-- Anything different? -->\n",
    );
    assert!(!page.contains("The fern looked dry."), "{page}");
    assert!(crate::closing::said_in(&page, &crate::closing::THIS_TIME).is_empty());
}

#[test]
fn another_machine_s_version_of_a_page_stays_on_it_under_its_own_heading() {
    let task = Ulid::generate();
    let mine = format!("{}\n## Notes\nMine.\n", seed(task));
    let theirs = format!("{}\n## Notes\nTheirs.\n- [ ] a box of theirs\n", seed(task));

    let kept = super::kept_beside(&mine, &theirs, "Other version");

    assert!(kept.starts_with(mine.trim_end()), "{kept}");
    assert!(
        kept.contains("## Other version\n\n### Notes\nTheirs."),
        "{kept}"
    );
    assert_eq!(kept.matches("task:").count(), 1, "{kept}");
    assert_eq!(crate::stages::stages(&kept).len(), 0, "{kept}");
    assert_eq!(super::kept_beside(&kept, &theirs, "Other version"), kept);
}

#[test]
fn a_description_s_own_headings_sit_one_level_under_its_section() {
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Audit", "a0"),
        },
    );
    applied(
        &mut state,
        Op::TaskDescribe {
            id: task,
            d: crate::event::Body {
                body: Some("## Scope\nAll of it.\n```\n## code\n```".into()),
            },
        },
    );

    let page = super::opened(&state, task, "## Description\n<!-- what -->\n");
    assert!(page.contains("### Scope\nAll of it."), "{page}");
    assert!(page.contains("```\n## code\n```"), "{page}");
}

#[test]
fn a_box_in_old_words_stays_text_but_a_link_in_a_bullet_is_left_alone() {
    assert_eq!(super::unboxed("1. [ ] buy food"), r"1. \[ ] buy food");
    assert_eq!(super::unboxed("> - [x] ask"), r"> - \[x] ask");
    assert_eq!(super::unboxed("- [docs](https://x)"), "- [docs](https://x)");
}

#[test]
fn a_task_s_words_are_put_away_while_no_turn_of_it_is_open() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let first = Ulid::generate();
    let second = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: first,
            d: TaskAdd::new("Water the plants", "a0"),
        },
    );
    let words = made(room.path(), &state, first, first, "", None).unwrap();
    applied(&mut state, words);
    let away = |state: &State| state.held_away(of(state, first).unwrap());
    assert!(!away(&state));

    applied(
        &mut state,
        Op::TaskDone {
            id: first,
            filled: false,
        },
    );
    assert!(away(&state));

    let mut d = TaskAdd::new("Water the plants", "a1");
    d.after = Some(first);
    applied(&mut state, Op::TaskAdd { id: second, d });
    assert!(
        !away(&state),
        "the open turn writes on the first one's words"
    );

    applied(
        &mut state,
        Op::TaskDone {
            id: second,
            filled: false,
        },
    );
    assert!(away(&state));
    applied(&mut state, Op::TaskReopen { id: second });
    assert!(!away(&state));
}
