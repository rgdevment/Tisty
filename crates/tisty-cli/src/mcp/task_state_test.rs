use super::*;
use serde_json::json;
use tisty_core::event::{DeviceId, DocAdd, Event};
use tisty_core::model::{Resolved, Step};
use ulid::Ulid;

use super::super::task_reading::window;

const AGENT: &str = "dev_agent";
const PERSON: &str = "dev_person";

fn at(second: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + second).unwrap()
}

fn noted(second: i64, body: &str, by: &str) -> LogEntry {
    LogEntry {
        id: Ulid::generate(),
        at: at(second),
        tz: None,
        body: body.into(),
        by: Some(DeviceId(by.into())),
        via: None,
        replaces: None,
    }
}

fn step(text: &str, done: bool) -> Step {
    Step {
        id: Ulid::generate(),
        text: text.into(),
        done,
        order: "a0".into(),
        by_agent: false,
        said: false,
    }
}

fn task() -> Task {
    let mut task = Task::new(Ulid::generate(), "Move payments to v2", "a0");
    task.steps = vec![step("Sign", true), step("Upload", false)];
    task.retally();
    task
}

fn state() -> State {
    let mut state = State::default();
    state.assistants.insert(DeviceId(AGENT.into()));
    state
}

fn home() -> (tempfile::TempDir, Paths) {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    (room, paths)
}

const SECTIONS: &str = "---\ntask: x\n---\n\n## Pasos\n\n- [x] Sign\n- [ ] Upload\n\n## Notas\n";

fn page_with(state: &str, long: bool) -> Page {
    let mut body = tisty_core::task_state::stated(SECTIONS, state, true);
    if long {
        body.push_str(&format!("\n## Análisis\n\n{}\n", "x".repeat(4_500)));
    }
    Page::new("t_x-0001".into(), body, Vec::new(), true, None)
}

fn seen<'a>(task: &'a Task, stated: &Stated<'a>) -> Window<'a> {
    let journal: Vec<&LogEntry> = task.journal().collect();
    window(
        &journal,
        None,
        stated.after(),
        |said| said.to_string(),
        |_| None,
    )
}

#[test]
fn a_written_state_on_a_whole_page_is_named_not_repeated() {
    let (_room, paths) = home();
    let mut task = task();
    task.stated_at = Some(at(10));
    task.steps_moved_at = Some(at(5));
    task.log = vec![noted(1, "before", AGENT), noted(20, "after", AGENT)];
    let page = page_with("**hoy** · Claude\nSigned; upload is next.", false);

    let stated = Stated::of(&paths, &state(), &task, Some(&page));
    let notes = seen(&task, &stated);
    let mut whole = json!({});
    stated.structured(&mut whole, &notes);

    assert_eq!(stated.after(), Some(at(10)));
    assert!(whole["state"]["written"].is_string(), "{whole}");
    assert!(whole["state"].get("body").is_none(), "{whole}");
    assert_eq!(whole["state"]["old"], "older than 1 later note(s)");
    let told = stated.told(&notes);
    assert!(told.contains("on its page, under «Estado»"), "{told}");
    assert!(told.contains("It is older than 1 later note(s)."), "{told}");
}

#[test]
fn a_state_older_than_its_steps_and_notes_says_both() {
    let (_room, paths) = home();
    let mut task = task();
    task.stated_at = Some(at(10));
    task.steps_moved_at = Some(at(11));
    let page = page_with("Signed.", false);

    let fresh = Stated::of(&paths, &state(), &task, Some(&page));
    assert_eq!(
        fresh.old().as_deref(),
        Some("older than the last change to its steps")
    );

    task.log = vec![noted(12, "a decision", AGENT)];
    let both = Stated::of(&paths, &state(), &task, Some(&page));
    assert_eq!(
        both.old().as_deref(),
        Some("older than the last change to its steps and 1 later note(s)")
    );

    task.steps_moved_at = Some(at(9));
    task.log.clear();
    assert_eq!(Stated::of(&paths, &state(), &task, Some(&page)).old(), None);
}

#[test]
fn an_outlined_page_brings_its_state_and_cuts_one_written_long_by_other_means() {
    let (_room, paths) = home();
    let mut task = task();
    task.stated_at = Some(at(10));
    let short = page_with("Signed; upload is next.", true);

    let stated = Stated::of(&paths, &state(), &task, Some(&short));
    let notes = seen(&task, &stated);
    let mut whole = json!({});
    stated.structured(&mut whole, &notes);
    assert_eq!(whole["state"]["body"], "Signed; upload is next.");
    assert!(whole["state"].get("cut").is_none(), "{whole}");
    assert!(stated.told(&notes).contains("\nSigned; upload is next."));

    let long = page_with(&"The queue kept its own clock. ".repeat(150), true);
    let stated = Stated::of(&paths, &state(), &task, Some(&long));
    let mut whole = json!({});
    stated.structured(&mut whole, &notes);
    assert_eq!(whole["state"]["cut"], true, "{whole}");
    assert!(whole["state"]["body"].as_str().unwrap().chars().count() <= STATE_SHOWN_UP_TO + 1);
    assert!(
        stated
            .told(&notes)
            .contains("(cut; its section on the page brings it whole)")
    );
}

#[test]
fn a_page_read_apart_still_gives_its_state() {
    let (_room, paths) = home();
    let mut task = task();
    task.stated_at = Some(at(10));
    let file = tisty_core::task_docs::file_at(task.id, 1);
    let body = tisty_core::task_state::stated(SECTIONS, "Signed.", true);
    tisty_core::docs::write(&paths.docs(), &file, &body).unwrap();
    let mut held = state();
    held.apply(&Event::new(
        DeviceId(PERSON.into()),
        at(0),
        Op::DocAdd {
            id: tisty_core::task_docs::id_at(task.id, 1),
            d: DocAdd {
                file,
                order: "a0".into(),
                task_of: Some(task.id),
                ..DocAdd::default()
            },
        },
    ));

    let stated = Stated::of(&paths, &held, &task, None);
    let mut whole = json!({});
    stated.structured(&mut whole, &seen(&task, &stated));

    assert_eq!(whole["state"]["body"], "Signed.", "{whole}");
}

#[test]
fn a_built_state_counts_a_step_an_agent_said_done_apart() {
    let (_room, paths) = home();
    let mut task = task();
    task.steps[1].said = true;
    task.retally();

    let stated = Stated::of(&paths, &state(), &task, None);
    let notes = seen(&task, &stated);
    let mut whole = json!({});
    stated.structured(&mut whole, &notes);
    assert_eq!(whole["state"]["done"], 1);
    assert_eq!(whole["state"]["said_done"], 1);
    assert!(
        stated
            .told(&notes)
            .contains("Built from it: 1 of 2 steps done, 1 said done and waiting on the person."),
        "{}",
        stated.told(&notes)
    );
}

#[test]
fn without_a_written_state_one_is_built_and_the_closing_note_is_pinned_once() {
    let (_room, paths) = home();
    let mut task = task();
    let closing = noted(1, "Paid, receipt 4471.", AGENT);
    task.resolved = Some(Resolved {
        at: at(1),
        by: DeviceId(AGENT.into()),
        entry: closing.id,
        via: None,
        branch: None,
        pr: None,
        drop: false,
    });
    task.log = vec![closing];
    task.log
        .extend((2..14).map(|n| noted(n, &"later ".repeat(80), AGENT)));

    let stated = Stated::of(&paths, &state(), &task, None);
    assert_eq!(stated.after(), None);
    let notes = seen(&task, &stated);
    let mut whole = json!({});
    stated.structured(&mut whole, &notes);
    assert_eq!(whole["state"]["built"], true);
    assert_eq!(whole["state"]["done"], 1);
    assert_eq!(whole["state"]["of"], 2);
    assert_eq!(whole["state"]["closing"]["body"], "Paid, receipt 4471.");
    let told = stated.told(&notes);
    assert!(
        told.contains("No state is written yet. Built from it: 1 of 2 steps done."),
        "{told}"
    );
    assert!(told.contains("The note that said it settled"), "{told}");

    task.log.truncate(1);
    let near = Stated::of(&paths, &state(), &task, None);
    let shown = seen(&task, &near);
    let mut whole = json!({});
    near.structured(&mut whole, &shown);
    assert!(
        whole["state"].get("closing").is_none(),
        "the window already shows it: {whole}"
    );
}

#[test]
fn a_task_with_nothing_to_build_from_says_nothing_of_a_state() {
    let (_room, paths) = home();
    let mut task = task();
    task.steps.clear();
    task.retally();

    let stated = Stated::of(&paths, &state(), &task, None);
    let mut whole = json!({});
    stated.structured(&mut whole, &seen(&task, &stated));

    assert!(whole.get("state").is_none(), "{whole}");
    assert_eq!(stated.told(&seen(&task, &stated)), "");
}

#[test]
fn a_note_replaces_only_an_agents_earlier_note_still_in_force() {
    let mut task = task();
    let theirs = noted(1, "Book it for March.", PERSON);
    let mine = noted(2, "Decided: CSV.", AGENT);
    let closing = noted(3, "Done, receipt 4471.", AGENT);
    let mut newer = noted(4, "Decided: XLSX.", AGENT);
    let gone = noted(5, "Decided: TSV.", AGENT);
    newer.replaces = Some(gone.id);
    task.resolved = Some(Resolved {
        at: at(3),
        by: DeviceId(AGENT.into()),
        entry: closing.id,
        via: None,
        branch: None,
        pr: None,
        drop: false,
    });
    task.log = vec![
        theirs.clone(),
        mine.clone(),
        closing.clone(),
        gone.clone(),
        newer,
    ];
    let held = state();
    let asked = |id: &str| replacing(&held, &task, &json!({ "replaces": id }));

    assert_eq!(granted(replacing(&held, &task, &json!({}))), None);
    assert_eq!(granted(asked(&mine.id.to_string())), Some(mine.id));
    let refused = |id: String| refusal(asked(&id));
    assert!(refused(theirs.id.to_string()).contains("is the person's"));
    assert!(refused(closing.id.to_string()).contains("said \"Move payments to v2\" is settled"));
    assert!(refused(gone.id.to_string()).contains("was replaced already"));
    assert!(refused("not-an-id".into()).contains("no note of"));
    assert!(refused(Ulid::generate().to_string()).contains("no note of"));
}

fn refusal<T>(said: Result<T, Refused>) -> String {
    match said {
        Err(Refused::Tool(said)) => said,
        _ => panic!("a tool refusal was expected"),
    }
}

fn granted<T>(said: Result<T, Refused>) -> T {
    match said {
        Ok(kept) => kept,
        Err(_) => panic!("it was refused"),
    }
}

#[test]
fn a_state_is_refused_before_any_step_on_a_task_said_done_a_routine_or_past_its_size() {
    let (_room, paths) = home();
    let held = state();

    let mut said = task();
    said.resolved = Some(Resolved {
        at: at(1),
        by: DeviceId(AGENT.into()),
        entry: Ulid::generate(),
        via: None,
        branch: None,
        pr: None,
        drop: false,
    });
    assert!(refusal(writable(&paths, &held, &said, "Done.")).contains("already been said done"));

    let mut round = task();
    round.after = Some(Ulid::generate());
    assert!(refusal(writable(&paths, &held, &round, "Done.")).contains("routine"));

    let long = "x".repeat(STATE_AT_MOST + 1);
    assert!(refusal(writable(&paths, &held, &task(), &long)).contains("at most 3000"));
}

fn with_its_page(paths: &Paths, task: &Task, body: &str, locked: bool) -> (State, String) {
    let file = tisty_core::task_docs::file_at(task.id, 1);
    tisty_core::docs::write(&paths.docs(), &file, body).unwrap();
    let doc = tisty_core::task_docs::id_at(task.id, 1);
    let mut held = state();
    held.apply(&Event::new(
        DeviceId(PERSON.into()),
        at(0),
        Op::TaskAdd {
            id: task.id,
            d: tisty_core::event::TaskAdd::new(task.title.clone(), "a0"),
        },
    ));
    held.apply(&Event::new(
        DeviceId(PERSON.into()),
        at(0),
        Op::DocAdd {
            id: doc,
            d: DocAdd {
                file: file.clone(),
                order: "a0".into(),
                task_of: Some(task.id),
                ..DocAdd::default()
            },
        },
    ));
    if locked {
        held.apply(&Event::new(
            DeviceId(PERSON.into()),
            at(1),
            Op::DocLock { id: doc },
        ));
    }
    (held, file)
}

#[test]
fn a_locked_page_takes_no_state() {
    let (_room, paths) = home();
    let task = task();
    let (held, _) = with_its_page(&paths, &task, SECTIONS, true);

    assert!(refusal(writable(&paths, &held, &task, "Signed.")).contains("locked"));
}

#[test]
fn the_persons_lines_in_a_state_are_refused_unless_carried_whole() {
    let (_room, paths) = home();
    let task = task();
    let body = tisty_core::task_state::stated(SECTIONS, "Ask Rina first.\nOK", true);
    let (held, file) = with_its_page(&paths, &task, &body, false);

    let said = refusal(writable(&paths, &held, &task, "OK so far: signed."));
    assert!(
        said.contains("«Ask Rina first.»") && said.contains("«OK»"),
        "{said}"
    );

    let (spanish, page) = granted(writable(
        &paths,
        &held,
        &task,
        "Signed.\nAsk Rina first.\nOK",
    ));
    assert!(spanish, "the page already speaks Spanish");
    assert_eq!(page.as_deref(), Some(file.as_str()));
    assert!(left_out(&paths, &file, &body, "Ask Rina first.\nOK").is_empty());
}

#[test]
fn what_the_assistant_wrote_in_the_state_is_its_own_and_the_persons_lines_stay_theirs() {
    let (_room, paths) = home();
    let was = tisty_core::task_state::stated(SECTIONS, "Mine.\nTheirs.", true);
    left_as_assistant(&paths, "t_x-0001", SECTIONS, &was);
    assert!(left_out(&paths, "t_x-0001", &was, "Replaced whole.").is_empty());

    let edited = was.replace("Theirs.", "Theirs.\nA line of the person's.");
    let now =
        tisty_core::task_state::stated(&edited, "Mine, mended.\nA line of the person's.", true);
    left_as_assistant(&paths, "t_x-0001", &edited, &now);

    assert_eq!(
        left_out(&paths, "t_x-0001", &now, "Only mine."),
        vec!["A line of the person's.".to_string()]
    );
}

#[test]
fn a_tick_answer_says_its_state_was_left_in_text_and_structure() {
    let answer = json!({
        "content": [{ "type": "text", "text": "Ticked 1 step(s)." }],
        "structuredContent": { "said": "Ticked 1 step(s).", "ticked": 1 }
    });

    let told = stated_too(answer, "Left its state on its page.");

    assert_eq!(
        told["content"][0]["text"],
        "Ticked 1 step(s). Left its state on its page."
    );
    assert_eq!(
        told["structuredContent"]["said"],
        "Ticked 1 step(s). Left its state on its page."
    );
    assert_eq!(told["structuredContent"]["stated"], true);
    assert_eq!(heading(true), "Estado");
    assert_eq!(heading(false), "State");
}
