use super::*;
use tisty_core::model::{DateSpec, List, Resolved, Status};
use ulid::Ulid;

const TASKS: &str = "
2026-10-06  2026-12-04  underway
2026-10-06  -           late
-           2026-10-26  overdue
-           2026-10-29  due_soon
2026-10-27  -           today
2026-11-02  -           ahead
-           -           undated
";

fn today() -> jiff::civil::Date {
    "2026-10-27".parse().unwrap()
}

fn day(raw: &str) -> Option<DateSpec> {
    (raw != "-").then(|| DateSpec::all_day(raw.parse().unwrap(), "UTC"))
}

fn dated(state: &mut State, title: &str, on: &str, due: &str) -> Task {
    let mut task = Task::new(Ulid::generate(), title, "a0");
    task.date = day(on);
    task.deadline = day(due);
    state.tasks.insert(task.id, task.clone());
    task
}

fn filed() -> State {
    let mut state = State::default();
    for line in TASKS.lines().filter(|line| !line.trim().is_empty()) {
        let [on, due, title] = line.split_whitespace().collect::<Vec<_>>()[..] else {
            panic!("a task has three columns: {line}");
        };
        dated(&mut state, title, on, due);
    }
    state
}

fn titles(state: &State) -> Vec<String> {
    pressing(state, today())
        .iter()
        .map(|(_, task)| task.title.clone())
        .collect()
}

#[test]
fn catching_up_counts_what_asks_for_attention_today_even_when_it_is_none() {
    let tally = tallied(&pressing(&filed(), today()));

    assert_eq!(
        tally,
        vec![
            ("overdue".to_string(), 1),
            ("due_soon".to_string(), 1),
            ("today".to_string(), 1),
            ("late".to_string(), 1)
        ]
    );
    assert_eq!(pressed(&tally), " (1 overdue, 1 due soon, 1 today, 1 late)");
    let quiet = tallied(&pressing(&State::default(), today()));
    assert!(quiet.iter().all(|(_, many)| *many == 0), "{quiet:?}");
    assert_eq!(quiet.len(), 4);
    assert_eq!(pressed(&quiet), "");
}

#[test]
fn what_presses_comes_with_a_deadline_first_then_the_day_then_what_slipped() {
    assert_eq!(titles(&filed()), ["overdue", "due_soon", "today", "late"]);
}

#[test]
fn a_task_an_agent_said_done_or_the_person_put_away_presses_no_one() {
    let mut state = filed();
    for task in state.tasks.values_mut() {
        match standing_of(task, today()) {
            Some(Standing::Overdue) => {
                task.resolved = Some(Resolved {
                    at: jiff::Timestamp::UNIX_EPOCH,
                    by: tisty_core::DeviceId("agent".into()),
                    entry: Ulid::nil(),
                    via: None,
                    drop: false,
                });
            }
            Some(Standing::Late) => task.hidden = true,
            Some(Standing::Today) => task.status = Status::Dropped,
            Some(Standing::DueSoon) => task.status = Status::Done,
            _ => {}
        }
    }

    assert_eq!(titles(&state), Vec::<String>::new());
}

#[test]
fn catching_up_names_the_five_that_slipped_longest_ago_and_how_to_find_the_rest() {
    let mut state = State::default();
    for on in [
        "2026-10-20",
        "2026-10-03",
        "2026-10-15",
        "2026-10-09",
        "2026-10-01",
        "2026-10-26",
        "2026-10-11",
    ] {
        dated(&mut state, &format!("on {on}"), on, "-");
    }
    let mut kept = serde_json::Map::new();

    let said = name_pressing(&state, &pressing(&state, today()), today(), &mut kept).unwrap();

    let named: Vec<&str> = kept["pressing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["title"].as_str().unwrap())
        .collect();
    assert_eq!(
        named,
        [
            "on 2026-10-01",
            "on 2026-10-03",
            "on 2026-10-09",
            "on 2026-10-11",
            "on 2026-10-15"
        ]
    );
    assert_eq!(kept["pressing"][0]["standing"], json!("late"));
    assert!(kept["pressing"][0].get("list").is_none(), "{kept:?}");
    assert_eq!(kept["pressing_more"], json!(2));
    assert!(
        said.starts_with("Pressing, most first: \"on 2026-10-01\" in the inbox ["),
        "{said}"
    );
    assert!(said.contains("`read` one before you take it up"), "{said}");
    assert!(
        said.contains("2 more: `find` with `scope` «open», `said_done` false and `to` 2026-10-30"),
        "{said}"
    );
}

#[test]
fn a_task_that_presses_is_named_by_its_list_and_alone_says_nothing_more() {
    let mut state = State::default();
    let list = List::new(Ulid::generate(), "Tisty", "a0");
    let mut task = dated(&mut state, "write the news article", "2026-10-27", "-");
    task.list = Some(list.id);
    state.lists.insert(list.id, list);
    state.tasks.insert(task.id, task);
    let mut kept = serde_json::Map::new();

    let said = name_pressing(&state, &pressing(&state, today()), today(), &mut kept).unwrap();

    assert_eq!(kept["pressing"][0]["list"], json!("Tisty"));
    assert_eq!(kept["pressing"][0]["standing"], json!("today"));
    assert!(said.contains("in Tisty ["), "{said}");
    assert!(said.contains("(today)"), "{said}");
    assert!(!said.contains("more"), "{said}");
    assert!(kept.get("pressing_more").is_none(), "{kept:?}");
}

#[test]
fn nothing_pressing_names_nothing() {
    let mut kept = serde_json::Map::new();

    assert!(name_pressing(&State::default(), &[], today(), &mut kept).is_none());
    assert!(kept.is_empty(), "{kept:?}");
}
