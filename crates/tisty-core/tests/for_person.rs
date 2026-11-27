use tisty_core::event::{Resolve, StepAdd, StepRef, TaskAdd, TaskPatch};
use tisty_core::model::Task;
use tisty_core::{DeviceId, DeviceKind, Event, Op, State, inverse};
use ulid::Ulid;

const START: i64 = 1_800_000_000;

fn at(seconds: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(START + seconds).unwrap()
}

fn ev(seconds: i64, who: &str, op: Op) -> Event {
    Event::new(DeviceId(who.into()), at(seconds), op)
}

fn joined() -> Event {
    ev(
        0,
        "dev_agent",
        Op::DeviceJoin {
            d: DeviceId("dev_agent".into()),
            k: Some(DeviceKind::Agent),
            p: None,
        },
    )
}

fn with_an_agent() -> State {
    let mut state = State::default();
    state.apply(&joined());
    state
}

fn filed(
    state: &mut State,
    seconds: i64,
    who: &str,
    title: &str,
    part_of: Option<Ulid>,
    for_person: bool,
) -> Ulid {
    let id = Ulid::generate();
    let mut d = TaskAdd::new(title, "a0");
    d.part_of = part_of;
    d.for_person = for_person;
    state.apply(&ev(seconds, who, Op::TaskAdd { id, d }));
    id
}

fn stepped(state: &mut State, seconds: i64, who: &str, task: Ulid, text: &str) -> Ulid {
    let step = Ulid::generate();
    state.apply(&ev(
        seconds,
        who,
        Op::StepAdd {
            id: task,
            d: StepAdd {
                step,
                text: text.into(),
                order: "a0".into(),
            },
        },
    ));
    step
}

fn tick(state: &mut State, seconds: i64, who: &str, task: Ulid, step: Ulid) {
    state.apply(&ev(
        seconds,
        who,
        Op::StepDone {
            id: task,
            d: StepRef { step },
        },
    ));
}

fn say(state: &mut State, seconds: i64, who: &str, task: Ulid, step: Ulid) {
    state.apply(&ev(
        seconds,
        who,
        Op::StepSaid {
            id: task,
            d: StepRef { step },
        },
    ));
}

fn resolve(state: &mut State, seconds: i64, who: &str, task: Ulid, word: bool) {
    let mut d = Resolve::new(Ulid::generate());
    d.word = word;
    state.apply(&ev(seconds, who, Op::TaskResolve { id: task, d }));
}

fn the_task(state: &State, id: Ulid) -> &Task {
    &state.tasks[&id]
}

fn landed() -> Resolve {
    Resolve::new(Ulid::generate()).landing(Some("feat/x".into()), Some("#12".into()))
}

#[test]
fn a_task_add_without_the_flag_reads_as_it_always_did() {
    let id = Ulid::generate();
    let written = serde_json::to_value(TaskAdd::new("old", "a0")).unwrap();
    assert!(written.get("for_person").is_none(), "{written}");
    let read: TaskAdd = serde_json::from_value(written).unwrap();
    assert!(!read.for_person);
    let mut state = with_an_agent();
    state.apply(&ev(1, "dev_agent", Op::TaskAdd { id, d: read }));
    assert!(!the_task(&state, id).for_person);
    assert!(state.agents_own(the_task(&state, id)));
}

#[test]
fn the_flag_is_written_when_it_is_set_and_comes_back() {
    let mut d = TaskAdd::new("theirs", "a0");
    d.for_person = true;
    let written = serde_json::to_value(&d).unwrap();
    assert_eq!(written["for_person"], serde_json::json!(true));
    let back: TaskAdd = serde_json::from_value(written).unwrap();
    assert!(back.for_person);
}

#[test]
fn the_flag_survives_an_event_line_round_trip() {
    let mut d = TaskAdd::new("theirs", "a0");
    d.for_person = true;
    let event = ev(
        1,
        "dev_agent",
        Op::TaskAdd {
            id: Ulid::generate(),
            d,
        },
    );
    let line = serde_json::to_string(&event).unwrap();
    assert!(line.contains("\"for_person\":true"), "{line}");
    let back: Event = serde_json::from_str(&line).unwrap();
    assert_eq!(back, event);
}

#[test]
fn a_task_without_the_flag_serializes_without_it_and_reads_without_it() {
    let task = Task::new(Ulid::generate(), "plain", "a0");
    let written = serde_json::to_value(&task).unwrap();
    assert!(written.get("for_person").is_none(), "{written}");
    let back: Task = serde_json::from_value(written).unwrap();
    assert!(!back.for_person);

    let mut theirs = task.clone();
    theirs.for_person = true;
    let written = serde_json::to_value(&theirs).unwrap();
    assert_eq!(written["for_person"], serde_json::json!(true));
    let back: Task = serde_json::from_value(written).unwrap();
    assert!(back.for_person);
}

#[test]
fn an_agents_task_is_the_agents_unless_it_is_flagged_for_the_person() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    let theirs = filed(&mut state, 2, "dev_agent", "theirs", None, true);
    let own = filed(&mut state, 3, "dev_a", "written by the person", None, false);
    assert!(state.agents_own(the_task(&state, mine)));
    assert!(!state.agents_own(the_task(&state, theirs)));
    assert!(state.filed_by_agents(the_task(&state, theirs)));
    assert!(!state.agents_own(the_task(&state, own)));
    assert!(the_task(&state, theirs).for_person);
}

#[test]
fn the_flag_on_a_task_the_person_filed_changes_nothing_for_agents() {
    let mut state = with_an_agent();
    let flagged = filed(&mut state, 1, "dev_a", "flagged by hand", None, true);
    assert!(!state.agents_own(the_task(&state, flagged)));
    let step = stepped(&mut state, 2, "dev_a", flagged, "do it");
    tick(&mut state, 3, "dev_agent", flagged, step);
    assert!(!the_task(&state, flagged).steps[0].done);
}

#[test]
fn an_agent_ticks_a_step_of_its_own_task() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    let step = stepped(&mut state, 2, "dev_agent", mine, "do it");
    tick(&mut state, 3, "dev_agent", mine, step);
    assert!(the_task(&state, mine).steps[0].done);
}

#[test]
fn an_agents_tick_on_a_step_of_a_task_it_filed_for_the_person_is_refused() {
    let mut state = with_an_agent();
    let theirs = filed(&mut state, 1, "dev_agent", "try it by hand", None, true);
    let step = stepped(&mut state, 2, "dev_agent", theirs, "open the app");
    tick(&mut state, 3, "dev_agent", theirs, step);
    assert!(!the_task(&state, theirs).steps[0].done);
}

#[test]
fn the_person_ticks_a_step_of_the_task_an_agent_filed_for_them() {
    let mut state = with_an_agent();
    let theirs = filed(&mut state, 1, "dev_agent", "try it by hand", None, true);
    let step = stepped(&mut state, 2, "dev_agent", theirs, "open the app");
    tick(&mut state, 3, "dev_a", theirs, step);
    assert!(the_task(&state, theirs).steps[0].done);
}

#[test]
fn an_agents_word_on_a_step_of_a_task_for_the_person_still_lands() {
    let mut state = with_an_agent();
    let theirs = filed(&mut state, 1, "dev_agent", "try it by hand", None, true);
    let step = stepped(&mut state, 2, "dev_agent", theirs, "open the app");
    say(&mut state, 3, "dev_agent", theirs, step);
    let kept = &the_task(&state, theirs).steps[0];
    assert!(kept.said);
    assert!(!kept.done);
}

#[test]
fn an_agents_resolving_of_a_task_for_the_person_is_refused_unless_it_is_a_word() {
    let mut state = with_an_agent();
    let theirs = filed(&mut state, 1, "dev_agent", "try it by hand", None, true);
    resolve(&mut state, 2, "dev_agent", theirs, false);
    assert!(the_task(&state, theirs).resolved.is_none());
    resolve(&mut state, 3, "dev_agent", theirs, true);
    assert!(the_task(&state, theirs).resolved.is_some());
}

#[test]
fn an_agents_resolving_of_its_own_task_lands_whether_or_not_it_is_a_word() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    resolve(&mut state, 2, "dev_agent", mine, false);
    assert!(the_task(&state, mine).resolved.is_some());
}

#[test]
fn an_agent_still_updates_the_title_of_a_task_it_filed_for_the_person() {
    let mut state = with_an_agent();
    let theirs = filed(&mut state, 1, "dev_agent", "try it by hand", None, true);
    state.apply(&ev(
        2,
        "dev_agent",
        Op::TaskUpdate {
            id: theirs,
            d: TaskPatch {
                title: Some("try it on the device".into()),
                ..Default::default()
            },
        },
    ));
    assert_eq!(the_task(&state, theirs).title, "try it on the device");
}

#[test]
fn an_agent_adds_a_step_to_a_task_it_filed_for_the_person() {
    let mut state = with_an_agent();
    let theirs = filed(&mut state, 1, "dev_agent", "try it by hand", None, true);
    stepped(&mut state, 2, "dev_agent", theirs, "open the app");
    assert_eq!(the_task(&state, theirs).steps.len(), 1);
}

#[test]
fn completing_an_agents_whole_keeps_the_open_part_of_the_person() {
    let mut state = with_an_agent();
    let whole = filed(&mut state, 1, "dev_agent", "ship it", None, false);
    let mine = filed(&mut state, 2, "dev_agent", "write it", Some(whole), false);
    let theirs = filed(&mut state, 3, "dev_agent", "try it", Some(whole), true);

    let ops = state.completing(whole, jiff::ZonedDateTime::now());

    assert!(ops.contains(&Op::TaskDrop { id: mine }));
    assert!(!ops.contains(&Op::TaskDrop { id: theirs }));
    assert!(ops.contains(&Op::TaskDone {
        id: whole,
        filled: false
    }));
}

#[test]
fn completing_the_persons_whole_still_lets_every_open_part_go() {
    let mut state = with_an_agent();
    let whole = filed(&mut state, 1, "dev_a", "move house", None, false);
    let mine = filed(&mut state, 2, "dev_agent", "list", Some(whole), false);
    let theirs = filed(&mut state, 3, "dev_agent", "pack", Some(whole), true);

    let ops = state.completing(whole, jiff::ZonedDateTime::now());

    assert!(ops.contains(&Op::TaskDrop { id: mine }));
    assert!(ops.contains(&Op::TaskDrop { id: theirs }));
}

#[test]
fn completing_a_whole_flagged_for_the_person_lets_its_parts_go() {
    let mut state = with_an_agent();
    let whole = filed(&mut state, 1, "dev_agent", "try it all", None, true);
    let theirs = filed(&mut state, 2, "dev_agent", "one", Some(whole), true);

    let ops = state.completing(whole, jiff::ZonedDateTime::now());

    assert!(ops.contains(&Op::TaskDrop { id: theirs }));
}

#[test]
fn a_part_of_the_person_that_is_closed_is_not_dropped_either() {
    let mut state = with_an_agent();
    let whole = filed(&mut state, 1, "dev_agent", "ship it", None, false);
    let theirs = filed(&mut state, 2, "dev_agent", "try it", Some(whole), true);
    state.apply(&ev(
        3,
        "dev_a",
        Op::TaskDone {
            id: theirs,
            filled: false,
        },
    ));
    let ops = state.completing(whole, jiff::ZonedDateTime::now());
    assert!(!ops.contains(&Op::TaskDrop { id: theirs }));
}

#[test]
fn a_resolve_without_branch_or_pr_reads_as_it_always_did() {
    let written = serde_json::to_value(Resolve::new(Ulid::generate())).unwrap();
    assert!(written.get("branch").is_none(), "{written}");
    assert!(written.get("pr").is_none(), "{written}");
    let back: Resolve = serde_json::from_value(written).unwrap();
    assert_eq!(back.branch, None);
    assert_eq!(back.pr, None);
}

#[test]
fn a_resolve_carries_its_branch_and_pr_through_serde() {
    let d = landed();
    let written = serde_json::to_value(&d).unwrap();
    assert_eq!(written["branch"], serde_json::json!("feat/x"));
    assert_eq!(written["pr"], serde_json::json!("#12"));
    let back: Resolve = serde_json::from_value(written).unwrap();
    assert_eq!(back, d);
}

#[test]
fn a_task_keeps_the_branch_and_pr_it_was_said_done_with() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    state.apply(&ev(
        2,
        "dev_agent",
        Op::TaskResolve {
            id: mine,
            d: landed(),
        },
    ));
    let kept = the_task(&state, mine).resolved.clone().unwrap();
    assert_eq!(kept.branch.as_deref(), Some("feat/x"));
    assert_eq!(kept.pr.as_deref(), Some("#12"));
}

#[test]
fn a_resolved_without_branch_or_pr_serializes_without_them() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    resolve(&mut state, 2, "dev_agent", mine, false);
    let written = serde_json::to_value(the_task(&state, mine)).unwrap();
    assert!(written["resolved"].get("branch").is_none(), "{written}");
    assert!(written["resolved"].get("pr").is_none(), "{written}");
}

#[test]
fn undoing_a_taking_back_of_a_word_keeps_the_branch_and_pr() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    state.apply(&ev(
        2,
        "dev_agent",
        Op::TaskResolve {
            id: mine,
            d: landed(),
        },
    ));

    let before = state.clone();
    let taken_back = ev(3, "dev_a", Op::TaskUnresolve { id: mine });
    let back = inverse(&taken_back, &before).expect("taking back has an inverse");
    state.apply(&taken_back);
    assert!(the_task(&state, mine).resolved.is_none());
    for (n, op) in back.into_iter().enumerate() {
        state.apply(&ev(4 + n as i64, "dev_a", op));
    }

    let kept = the_task(&state, mine).resolved.clone().unwrap();
    assert_eq!(kept.branch.as_deref(), Some("feat/x"));
    assert_eq!(kept.pr.as_deref(), Some("#12"));
}

#[test]
fn a_later_resolve_does_not_take_the_landed_branch_and_pr_away() {
    let mut state = with_an_agent();
    let mine = filed(&mut state, 1, "dev_agent", "mine", None, false);
    state.apply(&ev(
        2,
        "dev_agent",
        Op::TaskResolve {
            id: mine,
            d: landed(),
        },
    ));
    state.apply(&ev(
        3,
        "dev_agent",
        Op::TaskResolve {
            id: mine,
            d: Resolve::new(Ulid::generate()).landing(Some("feat/y".into()), None),
        },
    ));

    let kept = the_task(&state, mine).resolved.clone().unwrap();
    assert_eq!(kept.branch.as_deref(), Some("feat/x"));
    assert_eq!(kept.pr.as_deref(), Some("#12"));
}

#[test]
fn the_flag_replays_the_same_from_a_serialized_log() {
    let theirs = Ulid::generate();
    let step = Ulid::generate();
    let mut d = TaskAdd::new("try it", "a0");
    d.for_person = true;
    let log = [
        joined(),
        ev(1, "dev_agent", Op::TaskAdd { id: theirs, d }),
        ev(
            2,
            "dev_agent",
            Op::StepAdd {
                id: theirs,
                d: StepAdd {
                    step,
                    text: "open it".into(),
                    order: "a0".into(),
                },
            },
        ),
        ev(
            3,
            "dev_agent",
            Op::StepDone {
                id: theirs,
                d: StepRef { step },
            },
        ),
    ];
    let read: Vec<Event> = log
        .iter()
        .map(|one| serde_json::from_str(&serde_json::to_string(one).unwrap()).unwrap())
        .collect();
    let replayed = State::replay(&read);
    assert!(replayed.tasks[&theirs].for_person);
    assert!(!replayed.tasks[&theirs].steps[0].done);
}

#[test]
fn after_completing_an_agents_whole_the_open_view_keeps_the_persons_part_and_drops_the_agents() {
    let mut state = with_an_agent();
    let whole = filed(&mut state, 1, "dev_agent", "ship it", None, false);
    let mine = filed(&mut state, 2, "dev_agent", "write it", Some(whole), false);
    let theirs = filed(&mut state, 3, "dev_agent", "try it", Some(whole), true);

    let ops = state.completing(whole, jiff::ZonedDateTime::now());
    for (n, op) in ops.into_iter().enumerate() {
        state.apply(&ev(10 + n as i64, "dev_a", op));
    }

    let today = jiff::civil::date(2027, 1, 20);
    let shown: Vec<Ulid> = state
        .matching(&tisty_core::view::Filter::default(), today)
        .into_iter()
        .map(|one| one.id)
        .collect();
    assert!(shown.contains(&theirs), "{shown:?}");
    assert!(!shown.contains(&mine), "{shown:?}");
    assert!(!shown.contains(&whole), "{shown:?}");
}
