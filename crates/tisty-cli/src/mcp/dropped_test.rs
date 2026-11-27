use super::*;
use tisty_core::event::{DeviceId, Event, Op, TaskAdd};
use tisty_core::model::{LogEntry, Resolved, Status, TaskId};
use ulid::Ulid;

const DAY: i64 = 86_400;

fn at(second: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + second).unwrap()
}

fn noted(second: i64, body: &str) -> LogEntry {
    LogEntry {
        id: Ulid::generate(),
        at: at(second),
        tz: None,
        body: body.into(),
        by: None,
        via: None,
        replaces: None,
    }
}

fn open_task(state: &mut State, title: &str) -> TaskId {
    let id = Ulid::generate();
    state.apply(&Event::new(
        DeviceId("dev_person".into()),
        at(0),
        Op::TaskAdd {
            id,
            d: TaskAdd::new(title.to_string(), "a0"),
        },
    ));
    id
}

fn dropped_at(state: &mut State, title: &str, second: Option<i64>) -> TaskId {
    let id = open_task(state, title);
    let task = state.tasks.get_mut(&id).unwrap();
    task.status = Status::Dropped;
    task.completed_at = second.map(at);
    id
}

fn with_log(state: &mut State, id: TaskId, entries: Vec<LogEntry>) {
    state.tasks.get_mut(&id).unwrap().log = entries;
}

fn resolved(entry: Ulid, drop: bool) -> Resolved {
    Resolved {
        at: at(2),
        by: DeviceId("dev_agent".into()),
        entry,
        via: None,
        branch: None,
        pr: None,
        drop,
    }
}

fn reason_of(state: &State, id: TaskId) -> Option<String> {
    reason(state, &state.tasks[&id])
}

#[test]
fn only_a_dropped_task_not_hidden_is_decided_not_to_do() {
    let mut state = State::default();
    let open = open_task(&mut state, "open one");
    let dropped = dropped_at(&mut state, "dropped one", Some(10));
    let secret = dropped_at(&mut state, "secret one", Some(10));
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    assert!(!decided_not_to_do(&state.tasks[&open]));
    assert!(decided_not_to_do(&state.tasks[&dropped]));
    assert!(!decided_not_to_do(&state.tasks[&secret]));
}

#[test]
fn the_reason_is_the_entry_the_agent_agreed_when_it_said_not_doing() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(10 * DAY));
    let agreed = noted(1, "the vendor shut down");
    let newer = noted(10 * DAY - 5, "an unrelated later note");
    let entry = agreed.id;
    with_log(&mut state, id, vec![agreed, newer]);
    state.tasks.get_mut(&id).unwrap().resolved = Some(resolved(entry, true));
    assert_eq!(
        reason_of(&state, id).as_deref(),
        Some("the vendor shut down")
    );
}

#[test]
fn a_resolution_that_was_not_a_drop_does_not_set_the_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    let done = noted(1, "finished it");
    let recent = noted(90, "changed my mind");
    let entry = done.id;
    with_log(&mut state, id, vec![done, recent]);
    state.tasks.get_mut(&id).unwrap().resolved = Some(resolved(entry, false));
    assert_eq!(reason_of(&state, id).as_deref(), Some("changed my mind"));
}

#[test]
fn an_agreed_entry_missing_from_the_log_falls_back_to_the_last_note() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(&mut state, id, vec![noted(50, "still here")]);
    state.tasks.get_mut(&id).unwrap().resolved = Some(resolved(Ulid::generate(), true));
    assert_eq!(reason_of(&state, id).as_deref(), Some("still here"));
}

#[test]
fn without_an_agreed_word_the_newest_note_of_the_last_day_is_the_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(5 * DAY));
    with_log(
        &mut state,
        id,
        vec![
            noted(5 * DAY - 3600, "earlier that day"),
            noted(5 * DAY - 60, "no longer needed"),
            noted(5 * DAY - DAY, "exactly a day before"),
        ],
    );
    assert_eq!(reason_of(&state, id).as_deref(), Some("no longer needed"));
}

#[test]
fn a_note_exactly_a_day_before_the_drop_still_counts() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(5 * DAY));
    with_log(&mut state, id, vec![noted(4 * DAY, "the day before")]);
    assert_eq!(reason_of(&state, id).as_deref(), Some("the day before"));
}

#[test]
fn a_note_older_than_a_day_before_the_drop_is_no_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(5 * DAY));
    with_log(&mut state, id, vec![noted(4 * DAY - 1, "long ago")]);
    assert_eq!(reason_of(&state, id), None);
}

#[test]
fn a_note_written_after_the_drop_is_ignored() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(&mut state, id, vec![noted(101, "written afterwards")]);
    assert_eq!(reason_of(&state, id), None);
}

#[test]
fn a_note_an_assistant_wrote_is_not_the_persons_reason() {
    let mut state = State::default();
    let agent = DeviceId("dev_agent".into());
    state.assistants.insert(agent.clone());
    let id = dropped_at(&mut state, "old plan", Some(1000));
    let mut theirs = noted(900, "an agent's note");
    theirs.by = Some(agent);
    with_log(&mut state, id, vec![theirs]);
    assert_eq!(reason_of(&state, id), None);
}

#[test]
fn an_older_note_of_the_person_wins_over_a_newer_one_of_an_assistant() {
    let mut state = State::default();
    let agent = DeviceId("dev_agent".into());
    state.assistants.insert(agent.clone());
    let id = dropped_at(&mut state, "old plan", Some(1000));
    let mut theirs = noted(900, "an agent's note");
    theirs.by = Some(agent);
    let mut mine = noted(100, "my own reason");
    mine.by = Some(DeviceId("dev_person".into()));
    with_log(&mut state, id, vec![theirs, mine, noted(50, "no device")]);
    assert_eq!(reason_of(&state, id).as_deref(), Some("my own reason"));
}

#[test]
fn a_task_without_a_drop_time_has_no_fallback_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", None);
    with_log(&mut state, id, vec![noted(1, "a note")]);
    assert_eq!(reason_of(&state, id), None);
}

#[test]
fn a_task_with_no_notes_has_no_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    assert_eq!(reason_of(&state, id), None);
}

#[test]
fn an_empty_body_is_no_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(&mut state, id, vec![noted(50, "  \n \t ")]);
    assert_eq!(reason_of(&state, id), None);
}

#[test]
fn the_reason_is_one_line() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(&mut state, id, vec![noted(50, "first\n\n  second\tthird")]);
    assert_eq!(reason_of(&state, id).as_deref(), Some("first second third"));
}

#[test]
fn a_long_reason_is_cut_at_three_hundred_characters_with_an_ellipsis() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(&mut state, id, vec![noted(50, &"é".repeat(301))]);
    let why = reason_of(&state, id).unwrap();
    assert_eq!(why.chars().count(), 301);
    assert!(why.ends_with('…'));
    assert!(why.starts_with(&"é".repeat(300)));
}

#[test]
fn a_reason_of_exactly_three_hundred_characters_is_kept_whole() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    let body = "a".repeat(300);
    with_log(&mut state, id, vec![noted(50, &body)]);
    assert_eq!(reason_of(&state, id), Some(body));
}

#[test]
fn a_path_in_the_reason_is_replaced() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(
        &mut state,
        id,
        vec![noted(50, "see /home/mario/secret/notes.txt for why")],
    );
    let why = reason_of(&state, id).unwrap();
    assert!(why.contains("[path]"), "{why}");
    assert!(!why.contains("mario"), "{why}");
}

#[test]
fn a_link_to_a_task_out_of_sight_is_masked_in_the_reason() {
    let mut state = State::default();
    let secret = dropped_at(&mut state, "secret title", Some(100));
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    let id = dropped_at(&mut state, "old plan", Some(100));
    let body = format!("blocked by [secret title](tisty:task/{secret})");
    with_log(&mut state, id, vec![noted(50, &body)]);
    let why = reason_of(&state, id).unwrap();
    assert!(!why.contains("secret title"), "{why}");
    assert!(why.contains("a task out of sight"), "{why}");
}

#[test]
fn a_card_has_id_title_status_day_and_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", Some(100));
    with_log(&mut state, id, vec![noted(50, "not needed")]);
    let card = card(&state, &state.tasks[&id]);
    assert_eq!(card["id"], id.to_string());
    assert_eq!(card["title"], "old plan");
    assert_eq!(card["status"], "dropped");
    assert_eq!(card["dropped"], when(at(100)));
    assert_eq!(card["reason"], "not needed");
}

#[test]
fn a_card_leaves_out_the_day_and_reason_when_there_are_none() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "old plan", None);
    let card = card(&state, &state.tasks[&id]);
    assert_eq!(card["status"], "dropped");
    assert!(card.get("dropped").is_none());
    assert!(card.get("reason").is_none());
}

#[test]
fn told_says_the_day_and_the_reason() {
    let said =
        told(&json!({"title": "old plan", "dropped": "2027-01-01 10:00", "reason": "not needed"}));
    assert_eq!(
        said,
        "\"old plan\" was dropped on 2027-01-01 10:00 — the person decided not to do it; their reason: not needed."
    );
}

#[test]
fn told_says_when_no_reason_was_written() {
    let said = told(&json!({"title": "old plan"}));
    assert_eq!(
        said,
        "\"old plan\" was dropped — the person decided not to do it; they wrote no reason."
    );
}

fn titles(found: &[&Task]) -> Vec<String> {
    found.iter().map(|task| task.title.clone()).collect()
}

#[test]
fn an_empty_or_too_short_query_matches_nothing() {
    let mut state = State::default();
    dropped_at(&mut state, "migrate the database", Some(10));
    assert!(matching(&state, "").is_empty());
    assert!(matching(&state, "  ").is_empty());
    assert!(matching(&state, "to a").is_empty());
}

#[test]
fn a_query_matches_by_word_prefix_on_title() {
    let mut state = State::default();
    dropped_at(&mut state, "migrate the database", Some(10));
    assert_eq!(
        titles(&matching(&state, "migrat")),
        ["migrate the database"]
    );
    assert!(matching(&state, "grate").is_empty());
}

#[test]
fn a_query_also_matches_the_reason() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "migrate the database", Some(100));
    with_log(&mut state, id, vec![noted(50, "vendor discontinued")]);
    assert_eq!(
        titles(&matching(&state, "vendor")),
        ["migrate the database"]
    );
}

#[test]
fn at_least_half_of_the_query_words_must_match() {
    let mut state = State::default();
    dropped_at(&mut state, "migrate the database", Some(10));
    assert_eq!(
        titles(&matching(&state, "migrate database kubernetes cluster")),
        ["migrate the database"]
    );
    assert!(matching(&state, "migrate kubernetes cluster").is_empty());
}

#[test]
fn the_best_match_comes_first_then_the_newest_drop() {
    let mut state = State::default();
    dropped_at(&mut state, "database backup old", Some(10));
    dropped_at(&mut state, "database backup newer", Some(30));
    dropped_at(&mut state, "database only", Some(50));
    assert_eq!(
        titles(&matching(&state, "database backup")),
        [
            "database backup newer",
            "database backup old",
            "database only"
        ]
    );
}

#[test]
fn a_hidden_dropped_task_and_open_tasks_are_never_matched() {
    let mut state = State::default();
    open_task(&mut state, "migrate the database");
    let secret = dropped_at(&mut state, "migrate the database too", Some(10));
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    assert!(matching(&state, "migrate database").is_empty());
}

#[test]
fn alike_finds_a_dropped_task_with_a_near_title() {
    let mut state = State::default();
    let id = dropped_at(&mut state, "Migrate the billing database", Some(10));
    let found = alike(&state, "migrate billing database now").unwrap();
    assert_eq!(found.id, id);
}

#[test]
fn alike_ignores_unrelated_titles_and_empty_ones() {
    let mut state = State::default();
    dropped_at(&mut state, "Migrate the billing database", Some(10));
    assert!(alike(&state, "paint the garden fence").is_none());
    assert!(alike(&state, "").is_none());
    assert!(alike(&state, "  --  ").is_none());
}

#[test]
fn alike_skips_hidden_and_open_tasks() {
    let mut state = State::default();
    open_task(&mut state, "Migrate the billing database");
    let secret = dropped_at(&mut state, "Migrate the billing database", Some(10));
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    assert!(alike(&state, "Migrate the billing database").is_none());
}

#[test]
fn alike_answers_with_the_newest_drop() {
    let mut state = State::default();
    dropped_at(&mut state, "Migrate the billing database", Some(10));
    let newest = dropped_at(&mut state, "Migrate billing database", Some(90));
    dropped_at(&mut state, "Migrate the billing database again", Some(50));
    assert_eq!(
        alike(&state, "Migrate the billing database").unwrap().id,
        newest
    );
}

#[test]
fn alike_is_not_fooled_by_a_shared_prefix() {
    let mut state = State::default();
    dropped_at(&mut state, "Tisty MCP: import", Some(10));
    assert!(alike(&state, "Tisty MCP: export").is_none());
    assert!(alike(&state, "Tisty MCP: import").is_some());
}

#[test]
fn alike_needs_most_of_the_words_in_common() {
    let mut state = State::default();
    dropped_at(
        &mut state,
        "migrate billing database cluster nightly",
        Some(10),
    );
    assert!(alike(&state, "migrate billing database").is_none());
    assert!(alike(&state, "migrate billing database cluster nightly").is_some());
}

#[test]
fn refused_answers_only_for_a_dropped_task_in_sight() {
    let mut state = State::default();
    let open = open_task(&mut state, "open one");
    let dropped = dropped_at(&mut state, "dropped one", Some(10));
    let secret = dropped_at(&mut state, "secret one", Some(10));
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    assert!(refused(&state, open).is_none());
    assert!(refused(&state, secret).is_none());
    assert!(refused(&state, Ulid::generate()).is_none());
    let Some(Refused::Tool(said)) = refused(&state, dropped) else {
        panic!("a dropped task is refused");
    };
    assert!(said.contains("dropped one"), "{said}");
    assert!(said.contains("was dropped"), "{said}");
}

#[test]
fn told_adds_no_second_stop_after_a_reason_that_ends_in_one() {
    for ending in [".", "!", "?", "…"] {
        let reason = format!("not needed{ending}");
        let said = told(&json!({"title": "old plan", "reason": reason}));
        assert!(said.ends_with(&reason), "{said}");
        assert!(!said.ends_with(&format!("{ending}.")), "{said}");
    }
    let said = told(&json!({"title": "old plan", "reason": "not needed"}));
    assert!(said.ends_with("not needed."), "{said}");
}
