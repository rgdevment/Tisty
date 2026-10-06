use super::*;
use crate::event::{DeviceId, Event, LogAdd, Resolve, StepAdd, StepRef, TaskPatch};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn ev(ms: i64, who: &str, op: Op) -> Event {
    Event::new(DeviceId(who.into()), at(ms), op)
}

fn opened(id: TaskId, open: bool) -> Op {
    Op::TaskUpdate {
        id,
        d: TaskPatch {
            open_to_agents: Some(open),
            ..Default::default()
        },
    }
}

fn with_an_agent() -> State {
    let mut state = State::default();
    state.apply(&ev(
        1,
        "dev_agent",
        Op::DeviceJoin {
            d: DeviceId("dev_agent".into()),
            k: Some(crate::event::DeviceKind::Agent),
            p: None,
        },
    ));
    state
}

fn written(state: &mut State, ms: i64, who: &str, title: &str) -> TaskId {
    let id = Ulid::generate();
    state.apply(&ev(
        ms,
        who,
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new(title, "a0"),
        },
    ));
    id
}

/// Everything an assistant may fill in, written in one go.
fn filled_in(state: &mut State, ms: i64, id: TaskId) -> StepId {
    let step = Ulid::generate();
    let entry = Ulid::generate();
    state.apply(&ev(
        ms,
        "dev_agent",
        Op::TaskDescribe {
            id,
            d: crate::event::Body {
                body: Some("the yearly one".into()),
            },
        },
    ));
    state.apply(&ev(
        ms + 1,
        "dev_agent",
        Op::StepAdd {
            id,
            d: StepAdd {
                step,
                text: "pay".into(),
                order: "a0".into(),
            },
        },
    ));
    state.apply(&ev(
        ms + 2,
        "dev_agent",
        Op::StepDone {
            id,
            d: StepRef { step },
        },
    ));
    state.apply(&ev(
        ms + 3,
        "dev_agent",
        Op::TaskLog {
            id,
            d: LogAdd::new(entry, "paid"),
        },
    ));
    state.apply(&ev(
        ms + 4,
        "dev_agent",
        Op::TaskResolve {
            id,
            d: Resolve::new(entry),
        },
    ));
    step
}

#[test]
fn only_the_person_opens_a_task_and_shuts_it_again() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");

    state.apply(&ev(3, "dev_agent", opened(id, true)));
    assert!(
        !state.tasks[&id].open_to_agents,
        "an assistant cannot let itself in"
    );
    assert!(!state.attended_by_agents(&state.tasks[&id]));

    state.apply(&ev(4, "dev_laptop", opened(id, true)));
    assert!(state.tasks[&id].open_to_agents);
    assert!(state.attended_by_agents(&state.tasks[&id]));

    state.apply(&ev(5, "dev_agent", opened(id, false)));
    assert!(
        state.tasks[&id].open_to_agents,
        "nor shut the door behind itself"
    );

    state.apply(&ev(6, "dev_laptop", opened(id, false)));
    assert!(!state.tasks[&id].open_to_agents);
}

#[test]
fn a_fill_in_on_a_task_the_person_kept_is_let_go_at_replay() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");

    filled_in(&mut state, 10, id);

    let task = &state.tasks[&id];
    assert_eq!(task.description, None);
    assert!(task.steps.is_empty());
    assert!(task.resolved.is_none(), "no mark on what was never theirs");
    assert_eq!(
        task.log.len(),
        1,
        "the journal line stays: a note is not a fill-in"
    );
}

#[test]
fn a_fill_in_lands_once_the_person_opened_the_task() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");
    state.apply(&ev(3, "dev_laptop", opened(id, true)));

    let step = filled_in(&mut state, 10, id);

    let task = &state.tasks[&id];
    assert_eq!(task.description.as_deref(), Some("the yearly one"));
    assert!(task.steps.iter().any(|s| s.id == step && s.done));
    assert!(task.resolved.is_some());
    assert_eq!(
        task.status,
        Status::Open,
        "saying it is done is still not closing it"
    );
}

#[test]
fn shutting_the_task_keeps_what_was_filled_in_and_refuses_what_comes_after() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");
    state.apply(&ev(3, "dev_laptop", opened(id, true)));
    filled_in(&mut state, 10, id);
    state.apply(&ev(20, "dev_laptop", opened(id, false)));

    let step = Ulid::generate();
    state.apply(&ev(
        21,
        "dev_agent",
        Op::StepAdd {
            id,
            d: StepAdd {
                step,
                text: "one more".into(),
                order: "a1".into(),
            },
        },
    ));

    let task = &state.tasks[&id];
    assert_eq!(task.description.as_deref(), Some("the yearly one"));
    assert_eq!(task.steps.len(), 1, "the later step never lands");
    assert!(task.resolved.is_some(), "shutting is not unsaying");
}

#[test]
fn what_an_assistant_filed_needs_no_opening_and_stays_its_own_past_the_badge() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_agent", "pasar biome sobre el front");
    assert!(state.attended_by_agents(&state.tasks[&id]));

    state.apply(&ev(
        3,
        "dev_laptop",
        Op::DeviceRemove {
            d: DeviceId("dev_agent".into()),
        },
    ));
    filled_in(&mut state, 10, id);

    let task = &state.tasks[&id];
    assert!(state.attended_by_agents(task));
    assert_eq!(task.description.as_deref(), Some("the yearly one"));
    assert!(task.resolved.is_some());
}

#[test]
fn a_task_the_person_wrote_is_theirs_however_much_an_assistant_wrote_on_it() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");
    state.apply(&ev(
        3,
        "dev_agent",
        Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), "I looked into it"),
        },
    ));

    assert!(!state.attended_by_agents(&state.tasks[&id]));
    assert_eq!(
        state.tasks[&id].created_by,
        Some(DeviceId("dev_laptop".into()))
    );
}

#[test]
fn an_assistant_patch_on_the_persons_task_is_let_go_whole_but_for_a_bell() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");

    state.apply(&ev(
        3,
        "dev_agent",
        Op::TaskUpdate {
            id,
            d: TaskPatch {
                open_to_agents: Some(true),
                title: Some("renew the TLS certificate".into()),
                ..Default::default()
            },
        },
    ));
    let task = &state.tasks[&id];
    assert!(!task.open_to_agents);
    assert_eq!(task.title, "renew the certificate", "nothing of it lands");

    let bell = crate::model::DateSpec::fixed("2026-10-01T09:00".parse().unwrap(), "UTC");
    state.apply(&ev(
        4,
        "dev_agent",
        Op::TaskUpdate {
            id,
            d: TaskPatch {
                reminders: Some(vec![bell.clone()]),
                ..Default::default()
            },
        },
    ));
    assert_eq!(
        state.tasks[&id].reminders,
        vec![bell.clone()],
        "a bell only ever adds"
    );

    state.apply(&ev(
        5,
        "dev_agent",
        Op::TaskUpdate {
            id,
            d: TaskPatch {
                reminders: Some(vec![bell]),
                title: Some("renew the TLS certificate".into()),
                ..Default::default()
            },
        },
    ));
    assert_eq!(
        state.tasks[&id].title, "renew the certificate",
        "a bell smuggling a title in is let go with it"
    );
}

/// Everything else an assistant could write on the person's task — a close, a drop, a hide,
/// a move, a mark taken off, a tick the person gave taken back — is let go, on an opened task too.
#[test]
fn what_no_door_ever_lets_an_assistant_do_is_let_go_on_every_task() {
    let mut state = with_an_agent();
    let mine = written(&mut state, 2, "dev_laptop", "renew the certificate");
    state.apply(&ev(3, "dev_laptop", opened(mine, true)));
    let step = filled_in(&mut state, 10, mine);
    let theirs = written(&mut state, 20, "dev_agent", "pasar biome sobre el front");

    for id in [mine, theirs] {
        state.apply(&ev(30, "dev_agent", Op::TaskDone { id, filled: false }));
        assert_eq!(state.tasks[&id].status, Status::Open, "no close");
        state.apply(&ev(31, "dev_agent", Op::TaskDrop { id }));
        assert_eq!(state.tasks[&id].status, Status::Open, "no drop");
        state.apply(&ev(32, "dev_agent", Op::TaskHide { id }));
        assert!(!state.tasks[&id].hidden, "no hide");
        state.apply(&ev(33, "dev_agent", Op::TaskUnresolve { id }));
    }
    assert!(state.tasks[&mine].resolved.is_some(), "no unsaying");
    state.apply(&ev(
        34,
        "dev_laptop",
        Op::StepDone {
            id: mine,
            d: StepRef { step },
        },
    ));
    state.apply(&ev(
        34,
        "dev_agent",
        Op::StepUndone {
            id: mine,
            d: StepRef { step },
        },
    ));
    assert!(
        state.tasks[&mine].steps[0].done,
        "no taking back a tick the person gave"
    );
    state.apply(&ev(
        35,
        "dev_agent",
        Op::TaskUpdate {
            id: mine,
            d: TaskPatch {
                title: Some("renamed".into()),
                ..Default::default()
            },
        },
    ));
    assert_eq!(
        state.tasks[&mine].title, "renew the certificate",
        "opened is not filed: the title stays the person's"
    );
    state.apply(&ev(
        36,
        "dev_agent",
        Op::TaskLog {
            id: mine,
            d: LogAdd::new(Ulid::generate(), "a note lands anywhere"),
        },
    ));
    assert_eq!(state.tasks[&mine].log.len(), 2);
}

fn step_written(state: &mut State, ms: i64, who: &str, id: TaskId, text: &str) -> StepId {
    let step = Ulid::generate();
    state.apply(&ev(
        ms,
        who,
        Op::StepAdd {
            id,
            d: StepAdd {
                step,
                text: text.into(),
                order: format!("a{ms}"),
            },
        },
    ));
    step
}

fn reworded(id: TaskId, step: StepId, text: &str) -> Op {
    Op::StepText {
        id,
        d: crate::event::StepText {
            step,
            text: text.into(),
        },
    }
}

fn note_edited(id: TaskId, entry: crate::model::LogId, body: &str) -> Op {
    Op::TaskLogEdit {
        id,
        d: crate::event::LogEdit {
            entry,
            body: body.into(),
        },
    }
}

#[test]
fn an_opened_task_lets_an_assistant_mend_its_open_steps_and_its_journal() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");
    state.apply(&ev(3, "dev_laptop", opened(id, true)));
    let pay = step_written(&mut state, 4, "dev_laptop", id, "pay");
    let keep = step_written(&mut state, 5, "dev_laptop", id, "keep the receipt");
    let entry = Ulid::generate();
    state.apply(&ev(
        6,
        "dev_laptop",
        Op::TaskLog {
            id,
            d: LogAdd::new(entry, "asked for the quote"),
        },
    ));

    state.apply(&ev(10, "dev_agent", reworded(id, pay, "pay the renewal")));
    state.apply(&ev(
        11,
        "dev_agent",
        Op::StepRemove {
            id,
            d: StepRef { step: keep },
        },
    ));
    state.apply(&ev(
        12,
        "dev_agent",
        note_edited(id, entry, "asked for the yearly quote"),
    ));

    let task = &state.tasks[&id];
    assert_eq!(task.step(pay).unwrap().text, "pay the renewal");
    assert!(task.step(keep).is_none());
    assert_eq!(task.log[0].body, "asked for the yearly quote");
}

#[test]
fn a_closed_step_an_emptied_note_and_a_kept_task_are_never_the_assistants_to_mend() {
    let mut state = with_an_agent();
    let id = written(&mut state, 2, "dev_laptop", "renew the certificate");
    state.apply(&ev(3, "dev_laptop", opened(id, true)));
    let pay = step_written(&mut state, 4, "dev_laptop", id, "pay");
    state.apply(&ev(
        5,
        "dev_laptop",
        Op::StepDone {
            id,
            d: StepRef { step: pay },
        },
    ));
    let entry = Ulid::generate();
    state.apply(&ev(
        6,
        "dev_laptop",
        Op::TaskLog {
            id,
            d: LogAdd::new(entry, "asked for the quote"),
        },
    ));
    let kept = written(&mut state, 7, "dev_laptop", "call the bank");
    let call = step_written(&mut state, 8, "dev_laptop", kept, "call");

    state.apply(&ev(10, "dev_agent", reworded(id, pay, "paid twice")));
    state.apply(&ev(
        11,
        "dev_agent",
        Op::StepRemove {
            id,
            d: StepRef { step: pay },
        },
    ));
    state.apply(&ev(12, "dev_agent", note_edited(id, entry, " ")));
    state.apply(&ev(13, "dev_agent", reworded(kept, call, "call twice")));

    let task = &state.tasks[&id];
    assert_eq!(task.step(pay).unwrap().text, "pay", "a closed step stays");
    assert_eq!(
        task.log[0].body, "asked for the quote",
        "emptying is erasing"
    );
    assert_eq!(state.tasks[&kept].step(call).unwrap().text, "call");
}
