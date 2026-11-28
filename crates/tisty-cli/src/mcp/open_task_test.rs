use super::*;
use tisty_core::model::Step;
use ulid::Ulid;

fn at(minute: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + minute * 60).unwrap()
}

fn task_with(steps: &[(&str, bool)]) -> Task {
    let mut task = Task::new(Ulid::generate(), "Ship the export", "a0");
    task.steps = steps
        .iter()
        .enumerate()
        .map(|(n, (text, done))| Step {
            id: Ulid::generate(),
            text: (*text).to_string(),
            done: *done,
            order: format!("a{n}"),
            by_agent: true,
            said: false,
        })
        .collect();
    task.retally();
    task
}

fn due(task: TaskId) -> Due {
    Due {
        task,
        minutes: 12,
        elsewhere: 0,
        only_proposed: false,
        others_read: false,
    }
}

#[test]
fn a_turn_left_unwritten_asks_for_one_tick_with_its_state_and_nothing_else() {
    let asked = TURN_LEFT_UNWRITTEN;

    assert!(asked.contains("`task_edit_steps` (`action: \"tick\"` or `\"set_state\"`)"));
    assert!(asked.contains("`state`"));
    for never in ["`task_note`", "`task_resolve`", "progress"] {
        assert!(!asked.contains(never), "{never} in {asked}");
    }
    assert!(asked.contains("end your turn without writing anything"));
}

#[test]
fn a_task_read_and_left_unwritten_is_asked_about_once_after_a_quiet_stretch() {
    let task = Ulid::generate();
    let mut open = OpenTask::default();

    assert_eq!(open.heard(Touch::Read(task), at(0)), None);
    assert_eq!(
        open.heard(Touch::Looked, at(4)),
        None,
        "reading around is no work yet"
    );
    let asked = open.heard(Touch::Looked, at(11)).unwrap();
    assert_eq!((asked.task, asked.minutes), (task, 11));

    open.asked();
    assert_eq!(open.heard(Touch::Looked, at(30)), None, "once per stretch");
}

#[test]
fn writing_on_the_open_task_starts_a_new_stretch() {
    let task = Ulid::generate();
    let mut open = OpenTask::default();
    open.heard(Touch::Read(task), at(0));
    open.heard(Touch::Looked, at(11));
    open.asked();

    assert_eq!(open.heard(Touch::Worked(task), at(12)), None);
    assert_eq!(open.heard(Touch::Looked, at(20)), None);
    assert!(open.heard(Touch::Looked, at(23)).is_some());
}

#[test]
fn writing_on_another_task_moves_the_open_task_there() {
    let (first, second) = (Ulid::generate(), Ulid::generate());
    let mut open = OpenTask::default();
    open.heard(Touch::Read(first), at(0));

    open.heard(Touch::Worked(second), at(9));

    assert_eq!(
        open.heard(Touch::Looked, at(15)),
        None,
        "the agent just wrote"
    );
    assert_eq!(open.heard(Touch::Looked, at(19)).unwrap().task, second);
}

#[test]
fn the_first_task_read_stays_open_while_others_are_only_read() {
    let (first, other) = (Ulid::generate(), Ulid::generate());
    let mut open = OpenTask::default();
    open.heard(Touch::Read(first), at(0));
    open.heard(Touch::Read(other), at(1));

    let asked = open.heard(Touch::Looked, at(10)).unwrap();

    assert_eq!(asked.task, first);
    assert!(asked.others_read);
}

#[test]
fn a_task_asked_about_and_still_unwritten_gives_way_to_the_next_one_read() {
    let (first, next) = (Ulid::generate(), Ulid::generate());
    let mut open = OpenTask::default();
    open.heard(Touch::Read(first), at(0));
    open.heard(Touch::Looked, at(10));
    open.asked();

    assert_eq!(open.heard(Touch::Read(next), at(11)), None);

    assert_eq!(open.heard(Touch::Looked, at(21)).unwrap().task, next);
}

#[test]
fn three_writes_elsewhere_ask_without_waiting() {
    let task = Ulid::generate();
    let mut open = OpenTask::default();
    open.heard(Touch::Read(task), at(0));

    assert_eq!(open.heard(Touch::Elsewhere, at(1)), None);
    assert_eq!(open.heard(Touch::Elsewhere, at(1)), None);
    let asked = open.heard(Touch::Elsewhere, at(2)).unwrap();

    assert_eq!((asked.task, asked.elsewhere), (task, 3));
}

#[test]
fn tasks_filed_one_after_another_leave_the_first_open_and_count_no_writes() {
    let (first, second) = (Ulid::generate(), Ulid::generate());
    let mut open = OpenTask::default();
    open.heard(Touch::Proposed(first), at(0));
    for _ in 0..5 {
        assert_eq!(open.heard(Touch::Proposed(second), at(1)), None);
    }

    let asked = open.heard(Touch::Looked, at(10)).unwrap();

    assert_eq!(asked.task, first);
    assert!(asked.only_proposed);
    assert!(!asked.others_read, "filing a task is not reading one");
}

#[test]
fn reading_a_task_it_proposed_makes_it_work_of_its_own() {
    let task = Ulid::generate();
    let mut open = OpenTask::default();
    open.heard(Touch::Proposed(task), at(0));

    open.heard(Touch::Read(task), at(1));

    assert!(!open.heard(Touch::Looked, at(10)).unwrap().only_proposed);
}

#[test]
fn saying_a_task_done_lets_the_open_task_go() {
    let task = Ulid::generate();
    let mut open = OpenTask::default();
    open.heard(Touch::Read(task), at(0));

    open.heard(Touch::Ended, at(5));

    assert_eq!(open.heard(Touch::Looked, at(30)), None);
}

#[test]
fn nothing_is_open_until_a_task_is_read_proposed_or_written() {
    let mut open = OpenTask::default();
    for minute in [0, 15, 60] {
        assert_eq!(open.heard(Touch::Elsewhere, at(minute)), None);
        assert_eq!(open.heard(Touch::Looked, at(minute)), None);
    }
}

#[test]
fn the_question_names_the_task_and_the_next_unticked_step() {
    let task = task_with(&[("Write the parser", true), ("Wire the menu", false)]);

    let asked = question(&task, &due(task.id)).unwrap();

    assert_eq!(asked["task"], json!(task.id.to_string()));
    assert_eq!(asked["left"], json!(1));
    let said = asked["said"].as_str().unwrap();
    insta::assert_snapshot!(said);
    assert!(
        !said.contains("progress"),
        "Tisty asks to tick and say done, never to note progress"
    );
}

#[test]
fn the_question_says_when_its_state_is_older_than_its_steps() {
    let mut task = task_with(&[("Write the parser", true), ("Wire the menu", false)]);
    let said = |task: &Task| {
        question(task, &due(task.id)).unwrap()["said"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert!(!said(&task).contains("older"), "{}", said(&task));

    task.stated_at = Some(at(5));
    task.steps_moved_at = Some(at(4));
    assert!(!said(&task).contains("older"), "{}", said(&task));

    task.steps_moved_at = Some(at(6));
    assert!(
        said(&task).contains("; its state is older than its steps."),
        "{}",
        said(&task)
    );
}

#[test]
fn every_step_ticked_asks_to_say_it_done() {
    let task = task_with(&[("Write the parser", true)]);

    let said = question(&task, &due(task.id)).unwrap()["said"]
        .as_str()
        .unwrap()
        .to_string();

    insta::assert_snapshot!(said);
}

#[test]
fn steps_said_done_on_the_persons_task_are_counted_as_everywhere_else() {
    let said = |steps: &[(&str, bool)], words: usize| {
        let mut task = task_with(steps);
        for step in task.steps.iter_mut().filter(|one| !one.done).take(words) {
            step.said = true;
        }
        task.retally();
        question(&task, &due(task.id)).unwrap()["said"]
            .as_str()
            .unwrap()
            .to_string()
    };

    let all_said = said(&[("Write the parser", false), ("Ship it", false)], 2);
    assert!(
        all_said.contains(", and 0 of 2 steps are ticked, 2 said done and waiting on the person, but it is not said done."),
        "{all_said}"
    );
    let one_said = said(&[("Write the parser", false), ("Ship it", false)], 1);
    assert!(
        one_said.contains(
            ", and 1 of 2 steps is still unticked, the next \"Ship it\", 1 said done and waiting on the person."
        ),
        "{one_said}"
    );
}

#[test]
fn writes_elsewhere_and_another_task_read_are_said_as_such() {
    let task = task_with(&[]);
    let due = Due {
        minutes: 2,
        elsewhere: 3,
        others_read: true,
        ..due(task.id)
    };

    insta::assert_snapshot!(question(&task, &due).unwrap()["said"].as_str().unwrap());
}

#[test]
fn a_task_only_filed_with_nothing_to_tick_is_not_asked_about() {
    let task = task_with(&[]);
    let due = Due {
        only_proposed: true,
        ..due(task.id)
    };

    assert_eq!(question(&task, &due), None);
}

#[test]
fn a_task_said_done_closed_or_hidden_since_is_not_asked_about() {
    let mut hidden = task_with(&[("Wire the menu", false)]);
    hidden.hidden = true;
    let mut closed = task_with(&[("Wire the menu", false)]);
    closed.status = tisty_core::model::Status::Done;

    assert_eq!(question(&hidden, &due(hidden.id)), None);
    assert_eq!(question(&closed, &due(closed.id)), None);
}

#[test]
fn a_call_is_judged_by_what_it_named_and_what_it_answered() {
    let task = Ulid::generate();
    let answered = |shown: Value| json!({ "structuredContent": shown });
    let id = task.to_string();

    assert_eq!(
        touch(
            "workspace_read",
            Some(task),
            false,
            &answered(json!({ "id": id, "status": "open" }))
        ),
        Touch::Read(task)
    );
    assert_eq!(
        touch(
            "workspace_read",
            Some(task),
            false,
            &answered(json!({ "doc": format!("t_{}-0001", id.to_lowercase()), "body": "" }))
        ),
        Touch::Looked,
        "its page read as a document is only a look"
    );
    assert_eq!(
        touch(
            "workspace_read",
            Some(task),
            false,
            &answered(json!({ "id": id, "status": "done" }))
        ),
        Touch::Looked
    );
    assert_eq!(
        touch(
            "workspace_read",
            Some(task),
            false,
            &answered(json!({ "id": id, "said_done": "2026-11-17 10:00" }))
        ),
        Touch::Looked
    );
    assert_eq!(
        touch(
            "task_create",
            None,
            true,
            &answered(json!({ "id": task.to_string(), "proposed": true, "steps": 2 }))
        ),
        Touch::Proposed(task)
    );
    assert_eq!(
        touch(
            "task_create",
            None,
            true,
            &answered(json!({ "id": task.to_string(), "proposed": true }))
        ),
        Touch::Looked,
        "a task filed with no steps is no work of the agent's"
    );
    assert_eq!(
        touch(
            "task_create",
            None,
            true,
            &answered(json!({ "id": task.to_string(), "proposed": false, "steps": 2 }))
        ),
        Touch::Looked
    );
    assert_eq!(
        touch("task_note", Some(task), true, &json!({})),
        Touch::Worked(task)
    );
    assert_eq!(
        touch("doc_create", None, true, &json!({})),
        Touch::Elsewhere
    );
    assert_eq!(
        touch("workspace_search", None, false, &json!({})),
        Touch::Looked
    );
    assert_eq!(
        touch("task_resolve", Some(task), true, &json!({})),
        Touch::Ended
    );
}

#[test]
fn a_task_is_named_by_its_id_or_by_its_page() {
    let task = Ulid::generate();
    let page = tisty_core::task_docs::file_at(task, 1);

    assert_eq!(aimed(&json!({ "task": task.to_string() })), Some(task));
    assert_eq!(aimed(&json!({ "doc": task.to_string() })), Some(task));
    assert_eq!(aimed(&json!({ "doc": page })), Some(task));
    assert_eq!(aimed(&json!({ "doc": "4nnctnme-0032" })), None);
}

#[test]
fn a_question_goes_in_the_structured_answer_and_after_the_text() {
    let mut answer = json!({
        "content": [{ "type": "text", "text": "Found 2." }],
        "structuredContent": { "total": 2 },
    });

    assert!(with_ask(
        &mut answer,
        json!({ "said": "Is it up to date?" })
    ));

    assert_eq!(
        answer["structuredContent"]["ask"]["said"],
        json!("Is it up to date?")
    );
    assert_eq!(
        answer["content"][0]["text"],
        json!("Found 2.\n\nIs it up to date?")
    );
}

#[test]
fn a_question_due_on_an_answer_that_is_only_text_waits_for_the_next() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().join("data"), home.path().join("config"));
    let task = Ulid::generate();
    let mut open = OpenTask::default();
    open.heard(Touch::Read(task), at(0));
    let text_only = json!({ "content": [{ "type": "text", "text": "The rules." }] });

    let said = open.answered(&paths, Touch::Looked, text_only.clone(), at(11));

    assert_eq!(said, text_only);
    assert_eq!(open.heard(Touch::Looked, at(12)).unwrap().task, task);
}

#[test]
fn a_connection_keeps_one_stretch_however_long_it_talks() {
    let mut open = OpenTask::default();
    let mut last = Ulid::nil();
    for n in 0..10_000 {
        last = Ulid::generate();
        open.heard(Touch::Worked(last), at(n));
        open.heard(Touch::Read(Ulid::generate()), at(n));
    }

    assert_eq!(open.heard(Touch::Looked, at(10_010)).unwrap().task, last);
    assert!(
        std::mem::size_of::<OpenTask>() <= 64,
        "a connection holds one stretch, never a list that grows with the tasks it touches"
    );
}

#[test]
fn an_answer_with_nothing_structured_takes_no_question() {
    let mut answer = json!({ "content": [{ "type": "text", "text": "The rules." }] });

    assert!(!with_ask(
        &mut answer,
        json!({ "said": "Is it up to date?" })
    ));
    assert_eq!(answer["content"][0]["text"], json!("The rules."));
}

#[test]
fn a_task_filed_for_the_person_is_never_asked_about() {
    let mut task = task_with(&[("Write the parser", true), ("Wire the menu", false)]);
    assert!(question(&task, &due(task.id)).is_some());

    task.for_person = true;

    assert_eq!(question(&task, &due(task.id)), None);
}

#[test]
fn a_task_filed_for_the_person_with_nothing_ticked_is_not_asked_about_either() {
    let mut task = task_with(&[("Try it by hand", false)]);
    task.for_person = true;

    assert_eq!(question(&task, &due(task.id)), None);
}
