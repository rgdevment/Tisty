use serde_json::json;

use super::*;

fn at(stage: Stage, (done, all): (usize, usize), next: Option<&str>) -> Progress {
    Progress {
        stage,
        count: StepCount { done, said: 0, all },
        next: next.map(str::to_string),
        state: None,
        old: false,
    }
}

#[test]
fn progress_says_the_stage_the_steps_and_the_next_one() {
    assert_eq!(at(Stage::NotStarted, (0, 0), None).told(), "not started");
    assert_eq!(
        at(Stage::UnderWay, (1, 3), Some("two")).told(),
        "under way, 1 of 3 steps ticked, next \"two\""
    );
    assert_eq!(
        at(Stage::Waiting, (2, 2), None).told(),
        "waiting on the person, all 2 steps ticked"
    );
}

#[test]
fn steps_an_agent_said_done_are_counted_apart_from_the_ticked_ones() {
    let mut one = at(Stage::UnderWay, (1, 3), Some("three"));
    one.count.said = 1;
    assert_eq!(
        one.told(),
        "under way, 1 of 3 steps ticked, 1 said done and waiting on the person, next \"three\""
    );
    let mut into = json!({});
    one.put(&mut into);
    assert_eq!(into["steps"], json!([1, 3]));
    assert_eq!(into["steps_said"], json!(1));

    let mut all_said = at(Stage::UnderWay, (1, 2), None);
    all_said.count.said = 1;
    assert_eq!(
        all_said.told(),
        "under way, 1 of 2 steps ticked, 1 said done and waiting on the person",
        "a step only said done is not ticked"
    );
}

#[test]
fn a_state_comes_after_the_steps_and_says_when_it_is_old() {
    let mut held = at(Stage::Blocked, (0, 1), Some("one"));
    held.state = Some("Blocked: no key.".into());
    assert_eq!(
        held.told(),
        "blocked, 0 of 1 steps ticked, next \"one\"; state: Blocked: no key."
    );
    held.old = true;
    assert!(held.told().ends_with("(older than its steps or notes)"));
}

#[test]
fn progress_puts_only_what_it_knows() {
    let mut bare = json!({});
    at(Stage::NotStarted, (0, 0), None).put(&mut bare);
    assert_eq!(bare, json!({ "stage": "not_started" }));

    let mut full = json!({});
    let mut held = at(Stage::UnderWay, (1, 2), Some("two"));
    held.state = Some("Halfway.".into());
    held.old = true;
    held.put(&mut full);
    assert_eq!(
        full,
        json!({ "stage": "under_way", "steps": [1, 2], "state": "Halfway.", "state_old": true })
    );
}

#[test]
fn a_long_step_is_cut_on_a_character() {
    let step = "ñ".repeat(NEXT_SHOWN + 5);
    let said = cut(&step);
    assert_eq!(said.chars().count(), NEXT_SHOWN + 1);
    assert!(said.ends_with('…'));
    assert_eq!(cut("short"), "short");
}

#[test]
fn a_step_given_an_agents_word_is_counted_apart_and_is_not_next() {
    let mut task = Task::new(ulid::Ulid::generate(), "A part", "a0");
    let step = |text: &str, said: bool| tisty_core::model::Step {
        id: ulid::Ulid::generate(),
        text: text.into(),
        done: false,
        order: "a0".into(),
        by_agent: false,
        said,
    };
    task.steps = vec![step("first", true), step("second", false)];
    task.retally();
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));

    let held = progress(&paths, &State::default(), &task);
    assert_eq!(
        held.told(),
        "under way, 0 of 2 steps ticked, 1 said done and waiting on the person, next \"second\""
    );
}

#[test]
fn a_projects_row_counts_steps_as_progress_does() {
    let mut task = Task::new(ulid::Ulid::generate(), "A part", "a0");
    task.steps = vec![tisty_core::model::Step {
        id: ulid::Ulid::generate(),
        text: "only".into(),
        done: false,
        order: "a0".into(),
        by_agent: false,
        said: true,
    }];
    task.retally();
    assert_eq!(
        stage(&task, None),
        Stage::UnderWay,
        "an agent's word is work under way"
    );
    let one = row(Stage::UnderWay, jiff::Timestamp::UNIX_EPOCH, &task, None);
    assert_eq!(one["steps"], json!([0, 1]));
    assert_eq!(one["steps_said"], json!(1));
}
