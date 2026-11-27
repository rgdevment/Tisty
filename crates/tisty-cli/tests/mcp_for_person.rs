use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

struct Served {
    home: TempDir,
    lang: &'static str,
}

impl Served {
    fn new() -> Self {
        Self::speaking("en_US.UTF-8")
    }

    fn speaking(lang: &'static str) -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
            lang,
        };
        served.person().begins("algo mio");
        served.person().lets_the_agent_in();
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn talk(&self, said: &[String]) -> Vec<Value> {
        let root = self.home.path();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .arg("mcp")
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", self.lang)
            .env_remove("LC_ALL")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        {
            let mut pipe = child.stdin.take().unwrap();
            for one in said {
                writeln!(pipe, "{one}").unwrap();
            }
        }
        let out = child.wait_with_output().unwrap();
        assert!(
            out.stderr.is_empty(),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect(line))
            .collect()
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let asked = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
        self.talk(&[asked]).remove(0)
    }

    fn tools(&self) -> Vec<Value> {
        let asked = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }).to_string();
        let said = self.talk(&[asked]).remove(0);
        said["result"]["tools"].as_array().unwrap().clone()
    }

    fn files(&self, args: Value) -> String {
        let said = self.call("propose", args);
        assert_ne!(said["result"]["isError"], json!(true), "{said}");
        said["result"]["structuredContent"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn read(&self, id: &str) -> Value {
        self.call("read", json!({ "task": id }))["result"]["structuredContent"].clone()
    }
}

fn text(said: &Value) -> String {
    said["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn refused(said: &Value) -> String {
    assert_eq!(said["result"]["isError"], json!(true), "{said}");
    text(said)
}

fn answered(said: &Value) -> Value {
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
    said["result"]["structuredContent"].clone()
}

const ONE_OWNER: &str =
    "A task is yours or the person's, never both: what only they can do goes in their own task.";

fn flagged(served: &Served, title: &str) -> bool {
    served.person().state().tasks[&served.person().id_of(title)].for_person
}

fn parts_of(served: &Served, whole: &str) -> Vec<Value> {
    served.read(whole)["parts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn titles(parts: &[Value]) -> Vec<String> {
    parts
        .iter()
        .map(|one| one["title"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn propose_says_in_its_schema_that_a_task_is_for_the_agent_or_the_person() {
    let served = Served::new();
    let tools = served.tools();
    let propose = tools.iter().find(|one| one["name"] == "propose").unwrap();
    let whose = &propose["inputSchema"]["properties"]["for"];
    assert_eq!(whose["type"], json!("string"), "{whose}");
    assert_eq!(whose["enum"], json!(["agent", "person"]), "{whose}");
    assert!(
        whose["description"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("required"),
        "{whose}"
    );
}

#[test]
fn unplan_says_in_its_schema_that_steps_can_go_to_the_person() {
    let served = Served::new();
    let tools = served.tools();
    let unplan = tools.iter().find(|one| one["name"] == "unplan").unwrap();
    let props = &unplan["inputSchema"]["properties"];
    assert_eq!(props["to_person"]["type"], json!("boolean"), "{props}");
    assert_eq!(props["title"]["type"], json!("string"), "{props}");
    assert!(
        unplan["description"]
            .as_str()
            .unwrap()
            .contains("`to_person`")
    );
}

#[test]
fn a_task_without_for_is_refused_and_the_refusal_teaches_both_values() {
    let served = Served::new();
    let said = served.call("propose", json!({ "title": "no owner" }));
    let why = refused(&said);
    assert!(why.contains("needs `for`"), "{why}");
    assert!(
        why.contains("\"agent\"") && why.contains("\"person\""),
        "{why}"
    );
    assert!(why.contains("a manual test"), "{why}");
    assert!(why.contains(ONE_OWNER), "{why}");
    assert!(why.contains("Nothing was written"), "{why}");
    assert!(!served.person().open().contains("no owner"));
}

#[test]
fn a_task_with_for_null_is_refused_like_one_without_it() {
    let served = Served::new();
    let said = served.call("propose", json!({ "title": "null owner", "for": null }));
    assert!(refused(&said).contains("needs `for`"));
    assert!(!served.person().open().contains("null owner"));
}

#[test]
fn a_task_with_an_invalid_for_is_refused_saying_what_works() {
    let served = Served::new();
    for bad in [json!("both"), json!("Agent"), json!("")] {
        let said = served.call("propose", json!({ "title": "bad owner", "for": bad }));
        let why = refused(&said);
        assert!(
            why.contains("`for` is \"agent\" or \"person\""),
            "{bad}: {why}"
        );
        assert!(why.contains("Nothing was written"), "{bad}: {why}");
    }
    assert!(!served.person().open().contains("bad owner"));
}

#[test]
fn a_for_that_is_not_text_is_refused_saying_it_takes_text() {
    let served = Served::new();
    for bad in [json!(3), json!(true), json!(["agent"])] {
        let said = served.call("propose", json!({ "title": "bad owner", "for": bad }));
        let why = refused(&said);
        assert!(why.contains("`for` takes text"), "{bad}: {why}");
        assert!(why.contains("\"agent\""), "{bad}: {why}");
    }
    assert!(!served.person().open().contains("bad owner"));
}

#[test]
fn a_task_for_the_agent_is_filed_unflagged_and_says_so() {
    let served = Served::new();
    let said = served.call(
        "propose",
        json!({ "for": "agent", "title": "mine", "steps": ["a"] }),
    );
    let kept = answered(&said);
    assert_eq!(kept["for"], json!("agent"), "{said}");
    assert_eq!(kept["proposed"], json!(true), "{said}");
    assert!(!flagged(&served, "mine"));
}

#[test]
fn a_task_for_the_person_is_filed_flagged_and_says_so() {
    let served = Served::new();
    let said = served.call(
        "propose",
        json!({ "for": "person", "title": "try it by hand", "steps": ["open the app"] }),
    );
    let kept = answered(&said);
    assert_eq!(kept["for"], json!("person"), "{said}");
    assert!(text(&said).contains("for the person"), "{said}");
    assert!(flagged(&served, "try it by hand"));
}

#[test]
fn the_answer_to_a_task_of_the_agent_ends_with_the_one_owner_sentence() {
    let served = Served::new();
    let plain = text(&served.call("propose", json!({ "for": "agent", "title": "bare" })));
    assert!(plain.contains(ONE_OWNER), "{plain}");
    let stepped = text(&served.call(
        "propose",
        json!({ "for": "agent", "title": "with steps", "steps": ["a", "b"] }),
    ));
    assert!(stepped.contains(ONE_OWNER), "{stepped}");
}

#[test]
fn a_batch_without_for_turns_every_task_away_teaching_why() {
    let served = Served::new();
    let said = served.call(
        "propose",
        json!({ "tasks": [{ "title": "one" }, { "title": "two" }] }),
    );
    let kept = answered(&said);
    assert_eq!(kept["written"], json!(0), "{said}");
    let turned = kept["tasks"].as_array().unwrap();
    assert_eq!(turned.len(), 2, "{said}");
    for one in turned {
        assert_eq!(one["proposed"], json!(false), "{one}");
        assert!(
            one["refused"].as_str().unwrap().contains("needs `for`"),
            "{one}"
        );
    }
    assert!(text(&said).contains("0 written, 0 already there, 2 turned away"));
    assert!(!served.person().open().contains("two"));
}

#[test]
fn a_batch_files_the_tasks_that_say_whose_they_are_and_turns_the_others_away() {
    let served = Served::new();
    let said = served.call(
        "propose",
        json!({ "tasks": [
            { "for": "agent", "title": "mine" },
            { "title": "nobody" },
            { "for": "person", "title": "theirs" },
            { "for": "both", "title": "greedy" }
        ] }),
    );
    let kept = answered(&said);
    assert_eq!(kept["written"], json!(2), "{said}");
    let all = kept["tasks"].as_array().unwrap();
    assert_eq!(all[0]["for"], json!("agent"), "{said}");
    assert_eq!(all[2]["for"], json!("person"), "{said}");
    assert!(all[1]["refused"].as_str().unwrap().contains("needs `for`"));
    assert!(
        all[3]["refused"]
            .as_str()
            .unwrap()
            .contains("\"agent\" or \"person\"")
    );
    assert!(text(&said).ends_with(ONE_OWNER), "{said}");
    assert!(!flagged(&served, "mine"));
    assert!(flagged(&served, "theirs"));
    assert!(!served.person().open().contains("nobody"));
    assert!(!served.person().open().contains("greedy"));
}

#[test]
fn a_batch_that_wrote_nothing_does_not_end_with_the_one_owner_sentence() {
    let served = Served::new();
    let said = served.call("propose", json!({ "tasks": [{ "title": "one" }] }));
    assert!(!text(&said).ends_with(ONE_OWNER), "{said}");
    assert!(text(&said).ends_with("Nothing was written."), "{said}");
}

#[test]
fn parts_take_the_for_of_their_whole_unless_they_say_their_own() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship it",
        "parts": [
            { "title": "write it" },
            { "title": "try it", "for": "person" },
            { "title": "check it", "for": "agent" }
        ]
    }));
    assert_eq!(titles(&parts_of(&served, &whole)).len(), 3);
    assert!(!flagged(&served, "write it"));
    assert!(flagged(&served, "try it"));
    assert!(!flagged(&served, "check it"));
    assert!(!flagged(&served, "ship it"));
}

#[test]
fn the_parts_of_a_whole_for_the_person_are_the_persons_unless_they_say_otherwise() {
    let served = Served::new();
    served.files(json!({
        "for": "person", "title": "try it all",
        "parts": [
            { "title": "first try" },
            { "title": "agent bit", "for": "agent" }
        ]
    }));
    assert!(flagged(&served, "try it all"));
    assert!(flagged(&served, "first try"));
    assert!(!flagged(&served, "agent bit"));
}

#[test]
fn a_part_with_an_invalid_for_turns_the_whole_call_away() {
    let served = Served::new();
    let said = served.call(
        "propose",
        json!({
            "for": "agent", "title": "ship it",
            "parts": [{ "title": "try it", "for": "nobody" }]
        }),
    );
    let why = refused(&said);
    assert!(why.contains("\"agent\" or \"person\""), "{why}");
    assert!(!served.person().open().contains("ship it"));
}

#[test]
fn a_part_filed_under_an_open_whole_with_part_of_needs_its_own_for() {
    let served = Served::new();
    let whole = served.files(json!({ "for": "agent", "title": "ship it" }));
    let missing = served.call("propose", json!({ "title": "try it", "part_of": whole }));
    assert!(refused(&missing).contains("needs `for`"));
    let said = served.call(
        "propose",
        json!({ "title": "try it", "part_of": whole, "for": "person" }),
    );
    assert_eq!(answered(&said)["for"], json!("person"), "{said}");
    assert!(flagged(&served, "try it"));
    assert_eq!(
        titles(&parts_of(&served, &whole)),
        vec!["try it".to_string()]
    );
}

#[test]
fn read_names_who_owns_a_task_an_agent_filed() {
    let served = Served::new();
    let mine = served.files(json!({ "for": "agent", "title": "mine" }));
    let theirs = served.files(json!({ "for": "person", "title": "theirs" }));
    assert_eq!(served.read(&mine)["for"], json!("agent"));
    assert_eq!(served.read(&theirs)["for"], json!("person"));
}

#[test]
fn read_says_nothing_of_who_owns_a_task_the_person_wrote() {
    let served = Served::new();
    let id = served.person().id_of("algo mio").to_string();
    let read = served.read(&id);
    assert!(read.get("for").is_none_or(Value::is_null), "{read}");
}

#[test]
fn a_tick_on_a_task_for_the_person_gives_a_word_and_ticks_nothing() {
    let served = Served::new();
    let id = served.files(json!({
        "for": "person", "title": "try it by hand", "steps": ["open the app", "press go"]
    }));
    let said = served.call("tick", json!({ "task": id, "step": "open the app" }));
    let kept = answered(&said);
    assert_eq!(kept["said"], json!(1), "{said}");
    assert_eq!(kept["ticked"], json!(0), "{said}");
    assert!(text(&said).contains("the person's own task"), "{said}");
    let state = served.person().state();
    let task = &state.tasks[&served.person().id_of("try it by hand")];
    assert!(task.steps[0].said && !task.steps[0].done);
    assert!(!task.steps[1].said);
}

#[test]
fn a_tick_on_a_task_for_the_agent_ticks_for_real() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "mine", "steps": ["a", "b"] }));
    let said = served.call("tick", json!({ "task": id, "step": "a" }));
    let kept = answered(&said);
    assert_eq!(kept["ticked"], json!(1), "{said}");
    let state = served.person().state();
    let task = &state.tasks[&served.person().id_of("mine")];
    assert!(task.steps[0].done && !task.steps[0].said);
}

#[test]
fn the_word_on_a_step_of_a_task_for_the_person_can_be_taken_back() {
    let served = Served::new();
    let id = served.files(json!({
        "for": "person", "title": "try it by hand", "steps": ["open the app"]
    }));
    served.call("tick", json!({ "task": id, "step": "open the app" }));
    let said = served.call("untick", json!({ "task": id, "step": "open the app" }));
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
    let state = served.person().state();
    let task = &state.tasks[&served.person().id_of("try it by hand")];
    assert!(!task.steps[0].said && !task.steps[0].done);
}

#[test]
fn the_persons_own_tick_on_a_task_for_the_person_is_not_taken_back_by_untick() {
    let served = Served::new();
    let id = served.files(json!({
        "for": "person", "title": "try it by hand", "steps": ["open the app"]
    }));
    served
        .person()
        .ticks(served.person().id_of("try it by hand"), 1);
    let said = served.call("untick", json!({ "task": id, "step": "open the app" }));
    assert_eq!(said["result"]["isError"], json!(true), "{said}");
    let state = served.person().state();
    assert!(state.tasks[&served.person().id_of("try it by hand")].steps[0].done);
}

#[test]
fn the_last_tick_says_to_say_done_without_waiting_for_publishing_or_merging() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "mine", "steps": ["a"] }));
    let said = served.call("tick", json!({ "task": id, "step": "a" }));
    let why = text(&said);
    assert!(why.contains("None left: `say_done`"), "{why}");
    assert!(
        why.contains("Publishing and merging are the person's"),
        "{why}"
    );
}

#[test]
fn say_done_with_a_step_unticked_hands_it_over_with_unplan_and_to_person() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "mine", "steps": ["a", "b"] }));
    served.call("tick", json!({ "task": id, "step": "a" }));
    let said = served.call("say_done", json!({ "task": id, "body": "did it" }));
    let why = refused(&said);
    assert!(why.contains("still has 1 step(s) unticked"), "{why}");
    assert!(why.contains("`unplan` and `to_person`"), "{why}");
    assert!(why.contains("`note`"), "{why}");
}

#[test]
fn say_done_with_an_open_part_of_the_agent_names_unplan_to_person() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship it", "parts": [{ "title": "write it" }]
    }));
    let said = served.call("say_done", json!({ "task": whole, "body": "did it" }));
    let why = refused(&said);
    assert!(why.contains("still has 1 part(s) open"), "{why}");
    assert!(why.contains("`unplan` and `to_person`"), "{why}");
}

#[test]
fn an_open_part_for_the_person_does_not_pin_the_whole() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship it",
        "parts": [{ "title": "try it", "for": "person" }]
    }));
    let said = served.call(
        "say_done",
        json!({ "task": whole, "body": "built and proven" }),
    );
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
    assert!(
        served.person().state().tasks[&served.person().id_of("ship it")]
            .resolved
            .is_some()
    );
}

#[test]
fn a_part_of_the_agent_still_pins_the_whole_beside_one_for_the_person() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship it",
        "parts": [{ "title": "try it", "for": "person" }, { "title": "write it" }]
    }));
    let said = served.call("say_done", json!({ "task": whole, "body": "done" }));
    assert!(
        refused(&said).contains("still has 1 part(s) open"),
        "{said}"
    );
}

#[test]
fn say_done_on_a_task_for_the_person_is_only_a_word() {
    let served = Served::new();
    let id = served.files(json!({
        "for": "person", "title": "try it by hand", "steps": ["open the app"]
    }));
    served.call("tick", json!({ "task": id, "step": "open the app" }));
    let said = served.call("say_done", json!({ "task": id, "body": "I tried it" }));
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
    let state = served.person().state();
    let task = &state.tasks[&served.person().id_of("try it by hand")];
    assert!(task.is_open());
    assert!(task.resolved.is_some());
}

#[test]
fn unplan_to_person_makes_one_companion_part_in_the_list_with_the_steps() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship the export", "list": "Personal",
        "steps": ["write it", "try it by hand", "try it on a phone"]
    }));
    let said = served.call(
        "unplan",
        json!({ "task": whole, "steps": ["try it by hand", "try it on a phone"], "to_person": true }),
    );
    let kept = answered(&said);
    assert_eq!(kept["to_person"], json!(true), "{said}");
    assert_eq!(kept["companion_made"], json!(true), "{said}");
    assert_eq!(kept["removed"], json!(2), "{said}");
    assert_eq!(kept["left"], json!(1), "{said}");
    assert!(text(&said).contains("in a new task of theirs"), "{said}");

    let state = served.person().state();
    let title = "ship the export: your part";
    let companion = &state.tasks[&served.person().id_of(title)];
    let anchor = &state.tasks[&served.person().id_of("ship the export")];
    assert!(companion.for_person);
    assert_eq!(companion.part_of, Some(anchor.id));
    assert_eq!(companion.list, anchor.list);
    assert!(companion.list.is_some());
    let words: Vec<&str> = companion
        .steps
        .iter()
        .map(|one| one.text.as_str())
        .collect();
    assert_eq!(words, vec!["try it by hand", "try it on a phone"]);
    assert_eq!(kept["to"], json!(companion.id.to_string()));
    let left: Vec<&str> = anchor.steps.iter().map(|one| one.text.as_str()).collect();
    assert_eq!(left, vec!["write it"]);
}

#[test]
fn unplan_to_person_takes_the_title_it_is_given() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship the export", "steps": ["write it", "try it"]
    }));
    served.call(
        "unplan",
        json!({ "task": whole, "step": "try it", "to_person": true, "title": "Try the export" }),
    );
    let state = served.person().state();
    let companion = &state.tasks[&served.person().id_of("Try the export")];
    assert!(companion.for_person);
    assert!(
        state
            .tasks
            .values()
            .all(|one| one.title != "ship the export: your part")
    );
}

#[test]
fn unplan_to_person_titles_the_companion_in_english_by_default() {
    let served = Served::new();
    let whole = served.files(json!({ "for": "agent", "title": "Export", "steps": ["a", "b"] }));
    served.call(
        "unplan",
        json!({ "task": whole, "step": "b", "to_person": true }),
    );
    assert!(
        served
            .person()
            .state()
            .tasks
            .values()
            .any(|one| one.title == "Export: your part")
    );
}

#[test]
fn unplan_to_person_titles_the_companion_in_the_language_of_the_person() {
    let served = Served::speaking("es_ES.UTF-8");
    let whole = served.files(json!({ "for": "agent", "title": "Exportar", "steps": ["a", "b"] }));
    served.call(
        "unplan",
        json!({ "task": whole, "step": "b", "to_person": true }),
    );
    let made: Vec<String> = served
        .person()
        .state()
        .tasks
        .values()
        .map(|one| one.title.clone())
        .collect();
    assert!(
        made.contains(&"Exportar: lo que te toca".to_string()),
        "{made:?}"
    );
}

#[test]
fn a_second_unplan_to_person_reuses_the_same_companion() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship the export", "steps": ["a", "b", "c"]
    }));
    let first = answered(&served.call(
        "unplan",
        json!({ "task": whole, "step": "b", "to_person": true }),
    ));
    let second = served.call(
        "unplan",
        json!({ "task": whole, "step": "c", "to_person": true }),
    );
    let kept = answered(&second);
    assert_eq!(kept["companion_made"], json!(false), "{second}");
    assert_eq!(kept["to"], first["to"], "{second}");
    assert!(text(&second).contains("at the end of"), "{second}");

    let state = served.person().state();
    let made: Vec<_> = state.tasks.values().filter(|one| one.for_person).collect();
    assert_eq!(made.len(), 1);
    let words: Vec<&str> = made[0].steps.iter().map(|one| one.text.as_str()).collect();
    assert_eq!(words, vec!["b", "c"]);
}

#[test]
fn the_steps_of_a_part_go_to_the_companion_of_its_whole() {
    let served = Served::new();
    served.files(json!({
        "for": "agent", "title": "ship the export",
        "parts": [{ "title": "write it", "steps": ["a", "b"] }]
    }));
    let part = served.person().id_of("write it").to_string();
    let said = served.call(
        "unplan",
        json!({ "task": part, "step": "b", "to_person": true }),
    );
    let kept = answered(&said);
    let state = served.person().state();
    let whole_id = served.person().id_of("ship the export");
    let companion = &state.tasks[&served.person().id_of("ship the export: your part")];
    assert_eq!(companion.part_of, Some(whole_id));
    assert_eq!(kept["to"], json!(companion.id.to_string()), "{said}");
    assert_eq!(companion.steps[0].text, "b");
}

#[test]
fn the_steps_of_a_part_under_a_whole_of_the_person_go_to_that_whole() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "person", "title": "try it all",
        "parts": [{ "title": "write it", "for": "agent", "steps": ["a", "b"] }]
    }));
    let part = served.person().id_of("write it").to_string();
    let said = served.call(
        "unplan",
        json!({ "task": part, "step": "b", "to_person": true }),
    );
    let kept = answered(&said);
    assert_eq!(kept["to"], json!(whole), "{said}");
    assert_eq!(kept["companion_made"], json!(false), "{said}");
    let state = served.person().state();
    let held = &state.tasks[&served.person().id_of("try it all")];
    assert_eq!(held.steps[0].text, "b");
    assert!(
        state
            .tasks
            .values()
            .all(|one| !one.title.ends_with(": your part"))
    );
}

#[test]
fn a_part_of_the_person_already_there_is_the_companion() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship it", "steps": ["a", "b"],
        "parts": [{ "title": "try it", "for": "person" }]
    }));
    let said = served.call(
        "unplan",
        json!({ "task": whole, "step": "b", "to_person": true }),
    );
    let kept = answered(&said);
    assert_eq!(kept["companion_made"], json!(false), "{said}");
    let state = served.person().state();
    let held = &state.tasks[&served.person().id_of("try it")];
    assert_eq!(kept["to"], json!(held.id.to_string()));
    assert_eq!(held.steps[0].text, "b");
}

#[test]
fn unplan_to_person_never_pins_the_whole() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship it", "steps": ["build", "try it by hand"]
    }));
    served.call("tick", json!({ "task": whole, "step": "build" }));
    served.call(
        "unplan",
        json!({ "task": whole, "step": "try it by hand", "to_person": true }),
    );
    let said = served.call(
        "say_done",
        json!({ "task": whole, "body": "built and proven" }),
    );
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
}

#[test]
fn unplan_refuses_to_and_to_person_together() {
    let served = Served::new();
    let one = served.files(json!({ "for": "agent", "title": "one", "steps": ["a"] }));
    let other = served.files(json!({ "for": "agent", "title": "other" }));
    let said = served.call(
        "unplan",
        json!({ "task": one, "step": "a", "to": other, "to_person": true }),
    );
    let why = refused(&said);
    assert!(why.contains("`to`") && why.contains("`to_person`"), "{why}");
    assert!(why.contains("Nothing was moved"), "{why}");
    assert_eq!(served.read(&one)["steps"].as_array().unwrap().len(), 1);
}

#[test]
fn unplan_to_person_from_a_task_that_is_already_the_persons_is_refused() {
    let served = Served::new();
    let theirs = served.files(json!({ "for": "person", "title": "theirs", "steps": ["a"] }));
    let said = served.call(
        "unplan",
        json!({ "task": theirs, "step": "a", "to_person": true }),
    );
    let why = refused(&said);
    assert!(why.contains("is the person's already"), "{why}");
    assert!(why.contains("nothing was moved"), "{why}");
    assert_eq!(served.read(&theirs)["steps"].as_array().unwrap().len(), 1);
}

#[test]
fn unplan_to_person_from_a_task_the_person_wrote_is_refused() {
    let served = Served::new();
    let id = served.person().id_of("algo mio");
    served.person().adds_step(id, "something");
    let said = served.call(
        "unplan",
        json!({ "task": id.to_string(), "step": "something", "to_person": true }),
    );
    assert_eq!(said["result"]["isError"], json!(true), "{said}");
    assert_eq!(served.person().state().tasks[&id].steps.len(), 1);
}

#[test]
fn unplan_to_person_from_a_part_whose_whole_is_closed_is_refused() {
    let served = Served::new();
    served.files(json!({
        "for": "agent", "title": "ship it",
        "parts": [{ "title": "write it", "steps": ["a"] }]
    }));
    let part = served.person().id_of("write it").to_string();
    served.person().closes(served.person().id_of("ship it"));
    let said = served.call(
        "unplan",
        json!({ "task": part, "step": "a", "to_person": true }),
    );
    assert_eq!(said["result"]["isError"], json!(true), "{said}");
}

#[test]
fn unplan_to_person_from_a_turn_of_something_that_repeats_is_refused() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "water", "steps": ["a"] }));
    served
        .person()
        .repeats(served.person().id_of("water"), "every week");
    let said = served.call(
        "unplan",
        json!({ "task": id, "step": "a", "to_person": true }),
    );
    let why = refused(&said);
    assert!(why.contains("repeats"), "{why}");
    assert!(why.contains("`for` \"person\""), "{why}");
}

#[test]
fn unplan_to_person_with_a_step_already_ticked_changes_nothing() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "mine", "steps": ["a", "b"] }));
    served.call("tick", json!({ "task": id, "step": "a" }));
    let said = served.call(
        "unplan",
        json!({ "task": id, "step": "a", "to_person": true }),
    );
    assert_eq!(said["result"]["isError"], json!(true), "{said}");
    assert!(
        served
            .person()
            .state()
            .tasks
            .values()
            .all(|one| !one.for_person)
    );
}

#[test]
fn unplan_without_to_person_still_just_takes_the_steps_off() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "mine", "steps": ["a", "b"] }));
    let said = served.call("unplan", json!({ "task": id, "step": "b" }));
    let kept = answered(&said);
    assert!(kept.get("to_person").is_none(), "{said}");
    assert_eq!(kept["to"], Value::Null, "{said}");
    assert_eq!(served.person().state().tasks.len(), 2);
}

#[test]
fn the_companion_made_by_unplan_is_read_as_for_the_person() {
    let served = Served::new();
    let whole = served.files(json!({ "for": "agent", "title": "ship it", "steps": ["a", "b"] }));
    let kept = answered(&served.call(
        "unplan",
        json!({ "task": whole, "step": "b", "to_person": true }),
    ));
    let companion = kept["to"].as_str().unwrap();
    assert_eq!(served.read(companion)["for"], json!("person"));
    assert_eq!(
        titles(&parts_of(&served, &whole)),
        vec!["ship it: your part".to_string()]
    );
}

fn resolved_of(served: &Served, title: &str) -> Option<tisty_core::model::Resolved> {
    served.person().state().tasks[&served.person().id_of(title)]
        .resolved
        .clone()
}

fn finished(served: &Served, title: &str) -> String {
    let id = served.files(json!({ "for": "agent", "title": title, "steps": ["a"] }));
    served.call("tick", json!({ "task": id, "step": "a" }));
    id
}

#[test]
fn say_done_keeps_the_branch_and_the_pull_request_on_the_task_and_echoes_them() {
    let served = Served::new();
    let id = finished(&served, "ship it");

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "built and proven", "branch": "feat/ship", "pr": "#42" }),
    );

    let kept = answered(&said);
    assert_eq!(kept["branch"], json!("feat/ship"), "{said}");
    assert_eq!(kept["pr"], json!("#42"), "{said}");
    assert!(text(&said).contains(", on #42"), "{said}");
    let resolved = resolved_of(&served, "ship it").unwrap();
    assert_eq!(resolved.branch.as_deref(), Some("feat/ship"));
    assert_eq!(resolved.pr.as_deref(), Some("#42"));
    assert!(!resolved.drop);
}

#[test]
fn say_done_with_only_a_branch_names_the_branch() {
    let served = Served::new();
    let id = finished(&served, "ship it");

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "proven", "branch": "feat/ship" }),
    );

    let kept = answered(&said);
    assert_eq!(kept["branch"], json!("feat/ship"), "{said}");
    assert!(kept.get("pr").is_none(), "{said}");
    assert!(text(&said).contains(", on the branch feat/ship"), "{said}");
    let resolved = resolved_of(&served, "ship it").unwrap();
    assert_eq!(resolved.branch.as_deref(), Some("feat/ship"));
    assert_eq!(resolved.pr, None);
}

#[test]
fn say_done_with_only_a_pull_request_keeps_only_that() {
    let served = Served::new();
    let id = finished(&served, "ship it");

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "proven", "pr": "https://example.com/pull/7" }),
    );

    let kept = answered(&said);
    assert_eq!(kept["pr"], json!("https://example.com/pull/7"), "{said}");
    assert!(kept.get("branch").is_none(), "{said}");
    let resolved = resolved_of(&served, "ship it").unwrap();
    assert_eq!(resolved.branch, None);
    assert_eq!(resolved.pr.as_deref(), Some("https://example.com/pull/7"));
}

#[test]
fn say_done_without_them_keeps_and_echoes_none() {
    let served = Served::new();
    let id = finished(&served, "ship it");

    let said = served.call("say_done", json!({ "task": id, "body": "proven" }));

    let kept = answered(&said);
    assert!(
        kept.get("branch").is_none() && kept.get("pr").is_none(),
        "{said}"
    );
    assert!(!text(&said).contains(", on "), "{said}");
    let resolved = resolved_of(&served, "ship it").unwrap();
    assert_eq!((resolved.branch, resolved.pr), (None, None));
}

#[test]
fn a_branch_or_pr_is_trimmed_and_blank_ones_are_left_out() {
    let served = Served::new();
    let id = finished(&served, "ship it");

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "proven", "branch": "  feat/ship  ", "pr": "   " }),
    );

    let kept = answered(&said);
    assert_eq!(kept["branch"], json!("feat/ship"), "{said}");
    assert!(kept.get("pr").is_none(), "{said}");
    let resolved = resolved_of(&served, "ship it").unwrap();
    assert_eq!(resolved.branch.as_deref(), Some("feat/ship"));
    assert_eq!(resolved.pr, None);
}

#[test]
fn a_branch_of_exactly_two_hundred_characters_is_kept() {
    let served = Served::new();
    let id = finished(&served, "ship it");
    let long = "b".repeat(200);

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "proven", "branch": long }),
    );

    assert_eq!(answered(&said)["branch"], json!(long), "{said}");
}

#[test]
fn a_branch_or_pr_over_two_hundred_characters_is_refused_and_nothing_is_said() {
    let served = Served::new();
    let id = finished(&served, "ship it");
    let long = "b".repeat(201);

    for key in ["branch", "pr"] {
        let said = served.call(
            "say_done",
            json!({ "task": id, "body": "proven", key: long }),
        );
        let why = refused(&said);
        assert!(why.contains(&format!("`{key}`")), "{why}");
        assert!(why.contains("at most 200 characters"), "{why}");
        assert!(why.contains("Nothing was said"), "{why}");
        assert!(resolved_of(&served, "ship it").is_none());
    }
    assert!(
        served.person().state().tasks[&served.person().id_of("ship it")]
            .log
            .is_empty()
    );
}

#[test]
fn a_branch_or_pr_with_a_control_character_is_refused_and_nothing_is_said() {
    let served = Served::new();
    let id = finished(&served, "ship it");

    for (key, bad) in [
        ("branch", "feat/x\nsecond line"),
        ("pr", "#4\tnext"),
        ("branch", "feat/\u{7}x"),
        ("pr", "line\r\nbreak"),
    ] {
        let said = served.call(
            "say_done",
            json!({ "task": id, "body": "proven", key: bad }),
        );
        let why = refused(&said);
        assert!(why.contains("one line"), "{key} {bad:?}: {why}");
        assert!(why.contains("Nothing was said"), "{key} {bad:?}: {why}");
        assert!(resolved_of(&served, "ship it").is_none());
    }
}

#[test]
fn a_refused_branch_does_not_stop_the_same_call_from_working_once_fixed() {
    let served = Served::new();
    let id = finished(&served, "ship it");
    served.call(
        "say_done",
        json!({ "task": id, "body": "proven", "branch": "a\nb" }),
    );

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "proven", "branch": "feat/ship" }),
    );

    assert_eq!(answered(&said)["branch"], json!("feat/ship"));
    assert_eq!(
        resolved_of(&served, "ship it").unwrap().branch.as_deref(),
        Some("feat/ship")
    );
}

#[test]
fn say_not_doing_refuses_a_branch_or_a_pull_request_and_says_nothing() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "drop it", "steps": ["a"] }));

    for (key, value) in [("branch", "feat/moot"), ("pr", "#9")] {
        let said = served.call(
            "say_not_doing",
            json!({ "task": id, "body": "moot", key: value }),
        );
        let why = refused(&said);
        assert!(
            why.contains(&format!("`{key}` is not something `say_not_doing` takes")),
            "{why}"
        );
        assert!(resolved_of(&served, "drop it").is_none());
    }
}

#[test]
fn say_not_doing_without_them_still_works() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "drop it", "steps": ["a"] }));

    let said = served.call("say_not_doing", json!({ "task": id, "body": "moot" }));

    let kept = answered(&said);
    assert!(
        kept.get("branch").is_none() && kept.get("pr").is_none(),
        "{said}"
    );
    let resolved = resolved_of(&served, "drop it").unwrap();
    assert!(resolved.drop);
    assert_eq!((resolved.branch, resolved.pr), (None, None));
}

#[test]
fn a_word_on_a_task_for_the_person_keeps_the_branch_and_pr_as_well() {
    let served = Served::new();
    let id = served.files(json!({
        "for": "person", "title": "try it by hand", "steps": ["open the app"]
    }));
    served.call("tick", json!({ "task": id, "step": "open the app" }));

    let said = served.call(
        "say_done",
        json!({ "task": id, "body": "tried", "branch": "feat/x", "pr": "#5" }),
    );

    assert_eq!(answered(&said)["pr"], json!("#5"), "{said}");
    let resolved = resolved_of(&served, "try it by hand").unwrap();
    assert_eq!(resolved.branch.as_deref(), Some("feat/x"));
}

#[test]
fn say_done_on_a_person_task_with_a_step_unticked_points_to_note_not_to_unplan() {
    let served = Served::new();
    let id = served.files(json!({
        "for": "person", "title": "try it by hand", "steps": ["open the app"]
    }));

    let said = served.call("say_done", json!({ "task": id, "body": "tried" }));

    let why = refused(&said);
    assert!(why.contains("still has 1 step(s) unticked"), "{why}");
    assert!(why.contains("the person's task"), "{why}");
    assert!(why.contains("gives your word"), "{why}");
    assert!(why.contains("`note`"), "{why}");
    assert!(!why.contains("to_person"), "{why}");
    assert!(!why.contains("`unplan`"), "{why}");
}

#[test]
fn the_answer_to_a_task_for_the_person_ends_with_the_one_owner_sentence() {
    let served = Served::new();
    let bare = text(&served.call("propose", json!({ "for": "person", "title": "no steps" })));
    assert!(bare.contains("for the person"), "{bare}");
    assert!(bare.contains(ONE_OWNER), "{bare}");
    assert!(!bare.contains("step(s)"), "{bare}");

    let stepped = text(&served.call(
        "propose",
        json!({ "for": "person", "title": "two steps", "steps": ["a", "b"] }),
    ));
    assert!(stepped.contains("with 2 step(s)"), "{stepped}");
    assert!(stepped.contains(ONE_OWNER), "{stepped}");
}

#[test]
fn reschedule_is_refused_on_a_task_for_the_person_and_moves_its_day_nowhere() {
    let served = Served::new();
    let id = served.files(json!({ "for": "person", "title": "sign the lease" }));

    let said = served.call("reschedule", json!({ "task": id, "date": "2026-09-27" }));
    let why = refused(&said);

    assert!(why.contains("the person's own"), "{why}");
    assert!(why.contains("leave the day alone"), "{why}");
    let state = served.person().state();
    assert!(
        state.tasks[&served.person().id_of("sign the lease")]
            .date
            .is_none()
    );
}

#[test]
fn reschedule_is_refused_on_a_deadline_of_a_task_for_the_person_too() {
    let served = Served::new();
    let id = served.files(json!({ "for": "person", "title": "sign the lease" }));

    let said = served.call(
        "reschedule",
        json!({ "task": id, "deadline": "2026-09-27" }),
    );

    refused(&said);
    let state = served.person().state();
    assert!(
        state.tasks[&served.person().id_of("sign the lease")]
            .deadline
            .is_none()
    );
}

#[test]
fn reschedule_still_moves_the_day_of_a_task_for_the_agent() {
    let served = Served::new();
    let id = served.files(json!({ "for": "agent", "title": "ship the export" }));

    let said = served.call("reschedule", json!({ "task": id, "date": "2026-09-27" }));

    assert_eq!(answered(&said)["date"], json!("2026-09-27"), "{said}");
}

#[test]
fn unplan_to_person_files_the_companion_in_the_anchors_list_even_when_it_is_archived() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship the export", "list": "Personal",
        "steps": ["write it", "try it by hand"]
    }));
    let list = served.person().state().tasks[&served.person().id_of("ship the export")]
        .list
        .unwrap();
    served
        .person()
        .does(tisty_core::Op::ListArchive { id: list });

    let said = served.call(
        "unplan",
        json!({ "task": whole, "step": "try it by hand", "to_person": true }),
    );

    assert_eq!(answered(&said)["companion_made"], json!(true), "{said}");
    let state = served.person().state();
    let anchor = &state.tasks[&served.person().id_of("ship the export")];
    let companion = &state.tasks[&served.person().id_of("ship the export: your part")];
    assert_eq!(companion.list, Some(list));
    assert_eq!(companion.list, anchor.list);
    assert!(companion.for_person);
    assert_eq!(companion.part_of, Some(anchor.id));
}

#[test]
fn unplan_to_person_orders_the_new_companion_right_after_its_anchor() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship the export", "list": "Personal",
        "steps": ["write it", "try it by hand"]
    }));

    answered(&served.call(
        "unplan",
        json!({ "task": whole, "step": "try it by hand", "to_person": true }),
    ));

    let state = served.person().state();
    let anchor = &state.tasks[&served.person().id_of("ship the export")];
    let companion = &state.tasks[&served.person().id_of("ship the export: your part")];
    assert_eq!(companion.order, tisty_core::order::after(&anchor.order));
    assert!(companion.order > anchor.order);
}

#[test]
fn unplan_to_person_from_a_task_in_no_list_files_the_companion_in_no_list() {
    let served = Served::new();
    let whole = served.files(json!({
        "for": "agent", "title": "ship the export", "steps": ["write it", "try it by hand"]
    }));

    answered(&served.call(
        "unplan",
        json!({ "task": whole, "step": "try it by hand", "to_person": true }),
    ));

    let state = served.person().state();
    let companion = &state.tasks[&served.person().id_of("ship the export: your part")];
    assert_eq!(companion.list, None);
}

fn part_under_a_whole_said(served: &Served, tool: &str) -> String {
    let whole = served.files(json!({
        "for": "agent", "title": "ship it",
        "parts": [{ "title": "write it", "steps": ["a"] }]
    }));
    let part_id = served.person().id_of("write it");
    let part = part_id.to_string();
    served.call("tick", json!({ "task": part, "step": "a" }));
    let said = served.call("say_done", json!({ "task": part, "body": "written" }));
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
    let said = served.call(tool, json!({ "task": whole, "body": "settled" }));
    assert_ne!(said["result"]["isError"], json!(true), "{said}");
    served
        .person()
        .does(tisty_core::Op::TaskUnresolve { id: part_id });
    served.person().adds_step(part_id, "b");
    part
}

#[test]
fn unplan_to_person_from_a_part_of_a_whole_said_done_is_refused_naming_the_whole() {
    let served = Served::new();
    let part = part_under_a_whole_said(&served, "say_done");

    let again = served.call(
        "unplan",
        json!({ "task": part, "step": "b", "to_person": true }),
    );

    let why = refused(&again);
    assert!(why.contains("already been said done"), "{why}");
    assert!(why.contains("ship it"), "{why}");
    assert!(
        served
            .person()
            .state()
            .tasks
            .values()
            .all(|one| !one.for_person)
    );
}

#[test]
fn unplan_to_person_from_a_part_of_a_whole_said_not_doing_is_refused_naming_the_whole() {
    let served = Served::new();
    let part = part_under_a_whole_said(&served, "say_not_doing");

    let again = served.call(
        "unplan",
        json!({ "task": part, "step": "b", "to_person": true }),
    );

    let why = refused(&again);
    assert!(why.contains("already been said"), "{why}");
    assert!(why.contains("ship it"), "{why}");
    assert!(
        served
            .person()
            .state()
            .tasks
            .values()
            .all(|one| !one.for_person)
    );
}
