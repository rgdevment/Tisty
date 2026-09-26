use super::*;
use crate::event::{DeviceId, Event};

fn with_lists(names: &[&str]) -> State {
    let events: Vec<Event> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            Event::new(
                DeviceId("dev_a".into()),
                jiff::Timestamp::from_millisecond(i as i64 + 1).unwrap(),
                Op::ListAdd {
                    id: Ulid::generate(),
                    d: ListAdd {
                        name: (*name).to_string(),
                        order: format!("a{i}"),
                        color: None,
                    },
                },
            )
        })
        .collect();
    State::replay(&events)
}

fn drafted(title: &str, filing: Option<Filing>) -> Draft {
    Draft {
        title: title.to_string(),
        filing,
        ..Draft::default()
    }
}

fn listed(state: &State, plan: &Plan) -> Option<ListId> {
    let applied = replayed(state, plan);
    applied.tasks[&plan.task].list
}

fn replayed(state: &State, plan: &Plan) -> State {
    let mut applied = state.clone();
    for (i, op) in plan.ops.iter().enumerate() {
        applied.apply(&Event::new(
            DeviceId("dev_a".into()),
            jiff::Timestamp::from_millisecond(1_000 + i as i64).unwrap(),
            op.clone(),
        ));
    }
    applied
}

#[test]
fn a_marked_list_that_is_missing_is_created_alongside_the_task() {
    let state = with_lists(&[]);
    let plan = plan(
        &state,
        drafted("book a haircut", Some(Filing::Marked("home".into()))),
    )
    .unwrap();

    assert_eq!(plan.ops.len(), 2, "the list and the task travel together");
    let applied = replayed(&state, &plan);
    assert_eq!(applied.lists.len(), 1);
    assert_eq!(
        applied.tasks[&plan.task].list,
        applied.lists.keys().next().copied()
    );
}

#[test]
fn a_marked_list_that_exists_is_reused() {
    let state = with_lists(&["work"]);
    let plan = plan(
        &state,
        drafted("fix the checkout", Some(Filing::Marked("work".into()))),
    )
    .unwrap();

    assert_eq!(plan.ops.len(), 1);
    assert_eq!(listed(&state, &plan), state.lists.keys().next().copied());
}

#[test]
fn a_named_list_that_is_missing_is_refused_instead_of_created() {
    let state = with_lists(&["work"]);
    let outcome = plan(
        &state,
        drafted("fix the checkout", Some(Filing::Named("home".into()))),
    );

    assert!(matches!(outcome, Err(Rejected::NoSuchList(_))));
}

#[test]
fn an_ambiguous_marker_is_refused_rather_than_creating_another() {
    let state = with_lists(&["work trip", "work notes"]);
    let outcome = plan(
        &state,
        drafted("book a flight", Some(Filing::Marked("work".into()))),
    );

    assert!(matches!(outcome, Err(Rejected::AmbiguousList(_))));
}

#[test]
fn an_exact_name_wins_over_the_lists_that_merely_contain_it() {
    let state = with_lists(&["work", "work notes"]);
    let plan = plan(
        &state,
        drafted("fix the checkout", Some(Filing::Marked("work".into()))),
    )
    .unwrap();

    assert_eq!(plan.ops.len(), 1);
    let applied = replayed(&state, &plan);
    assert_eq!(
        applied.lists[&applied.tasks[&plan.task].list.unwrap()].name,
        "work"
    );
}

#[test]
fn a_capture_that_is_all_markers_has_no_title_and_is_refused() {
    let state = with_lists(&[]);
    let outcome = plan(&state, drafted("   ", Some(Filing::Marked("home".into()))));

    assert!(matches!(outcome, Err(Rejected::Untitled)));
}

#[test]
fn a_task_is_ordered_after_the_ones_already_there() {
    let state = with_lists(&[]);
    let first = plan(&state, drafted("one", None)).unwrap();
    let after = replayed(&state, &first);
    let second = plan(&after, drafted("two", None)).unwrap();

    let ordered = replayed(&after, &second);
    assert!(ordered.tasks[&first.task].order < ordered.tasks[&second.task].order);
}

#[test]
fn everything_the_phrase_carried_reaches_the_task_it_becomes() {
    let state = with_lists(&["work"]);
    let when = DateSpec::all_day("2026-09-20".parse().unwrap(), "UTC");
    let by = DateSpec::all_day("2026-09-30".parse().unwrap(), "UTC");
    let every = crate::model::Repeat::due(crate::model::Cadence {
        every: 1,
        unit: crate::model::Unit::Week,
    });

    let told = plan(
        &state,
        Draft {
            title: "reunion de equipo".into(),
            date: Some(when.clone()),
            deadline: Some(by.clone()),
            priority: Some(Priority::Do),
            tags: vec![Tag::new("work").unwrap()],
            filing: None,
            repeat: Some(every),
            source: Some("sereno#1".into()),
        },
    )
    .unwrap();

    let one = &replayed(&state, &told).tasks[&told.task];

    assert_eq!(one.title, "reunion de equipo");
    assert_eq!(one.date.as_ref(), Some(&when), "the day was left behind");
    assert_eq!(
        one.deadline.as_ref(),
        Some(&by),
        "the deadline was left behind"
    );
    assert_eq!(one.priority, Priority::Do, "the quadrant was left behind");
    assert_eq!(
        one.tags,
        vec![Tag::new("work").unwrap()],
        "the tags were left behind"
    );
    assert_eq!(one.repeat, Some(every), "the cadence was left behind");
    assert_eq!(
        one.source.as_deref(),
        Some("sereno#1"),
        "the source was left behind"
    );
}
