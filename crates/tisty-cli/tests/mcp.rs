use std::io::Write;
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        served.cli(&["algo mio"]);
        served
    }

    fn command(&self) -> Command {
        let root = self.home.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_tisty"));
        command
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL");
        command
    }

    fn cli(&self, args: &[&str]) -> String {
        let out = self
            .command()
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn talk(&self, said: &[&str]) -> Vec<serde_json::Value> {
        let mut child = self
            .command()
            .arg("mcp")
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

    fn call(&self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let asked = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
        self.talk(&[&asked]).remove(0)
    }
}

#[test]
fn nothing_can_be_filed_until_a_person_turns_an_agent_on() {
    let served = Served::new();

    let said = served.call("propose", serde_json::json!({ "title": "not yet" }));
    assert_eq!(said["result"]["isError"], true);
    assert!(
        said["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("no agent is registered"),
        "{said}"
    );
    assert!(!served.cli(&["ls", "all"]).contains("not yet"));
}

#[test]
fn what_the_agent_files_is_written_by_the_agent_and_not_by_the_person() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call(
        "propose",
        serde_json::json!({ "title": "buy pink card stock" }),
    );

    let mine = std::fs::read_to_string(served.home.path().join("config/config.toml")).unwrap();
    let agent = mine
        .lines()
        .find_map(|line| line.strip_prefix("agent_id = "))
        .map(|said| said.trim_matches('"').to_string())
        .expect("the agent has an identity of its own");

    // Counting directories would pass even if it filed under the person's id: registering
    // creates the second directory before anything is filed.
    let wrote = std::fs::read_to_string(
        served
            .home
            .path()
            .join("data/store")
            .join(&agent)
            .join("active.tisty"),
    )
    .expect("the agent's own directory holds what it filed");
    assert!(wrote.contains("buy pink card stock"), "{wrote}");
}

#[test]
fn undo_belongs_to_the_person_and_never_reaches_what_the_agent_filed() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.cli(&["something I typed myself"]);
    served.call("propose", serde_json::json!({ "title": "what it filed" }));

    served.cli(&["undo"]);

    let left = served.cli(&["ls", "all"]);
    assert!(
        left.contains("what it filed"),
        "the agent's work is not the person's to undo: {left}"
    );
    assert!(!left.contains("something I typed myself"), "{left}");
}

#[test]
fn the_same_source_is_never_filed_twice() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let args = serde_json::json!({ "title": "card stock", "source": "wa:msg-991" });

    let first = served.call("propose", args.clone());
    let again = served.call(
        "propose",
        serde_json::json!({ "title": "same thing, read again", "source": "wa:msg-991" }),
    );

    assert_eq!(first["result"]["structuredContent"]["filed"], true);
    assert_eq!(again["result"]["structuredContent"]["filed"], false);
    assert_eq!(
        again["result"]["structuredContent"]["id"],
        first["result"]["structuredContent"]["id"]
    );
    assert!(!served.cli(&["ls", "all"]).contains("read again"));
}

#[test]
fn what_it_files_lands_in_the_inbox_and_asking_for_a_list_is_refused() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.cli(&["list", "add", "Work"]);

    let asked = served.call(
        "propose",
        serde_json::json!({ "title": "somewhere else", "list": "Work" }),
    );
    served.call(
        "propose",
        serde_json::json!({ "title": "card stock", "tags": ["school"] }),
    );

    assert_eq!(
        asked["result"]["isError"], true,
        "choosing a list is not on offer"
    );
    assert!(!served.cli(&["ls", "all"]).contains("somewhere else"));

    let inbox = served.cli(&["ls", "inbox"]);
    assert!(inbox.contains("card stock"), "{inbox}");
    assert!(served.cli(&["ls", "all"]).contains("agent"));
}

#[test]
fn a_date_in_words_is_refused_with_the_shape_it_wanted() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);

    let said = served.call(
        "propose",
        serde_json::json!({ "title": "card stock", "deadline": "next monday" }),
    );

    let why = said["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(said["result"]["isError"], true);
    assert!(
        why.contains("2026-08-31"),
        "the refusal has to teach: {why}"
    );
    assert!(!served.cli(&["ls", "all"]).contains("card stock"));
}

#[test]
fn there_is_no_tool_for_closing_dropping_or_deleting() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);

    let listed = served.talk(&[r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#]);
    let names: Vec<&str> = listed[0]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["name"].as_str().unwrap())
        .collect();

    assert_eq!(names, ["propose", "note", "attach", "read", "find"]);
    for barred in ["done", "drop", "rm", "undo", "sync", "set"] {
        let said = served.call(barred, serde_json::json!({}));
        assert_eq!(said["error"]["code"], -32602, "{barred} answered: {said}");
    }
}

#[test]
fn a_note_reaches_a_task_the_person_wrote() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.cli(&["ls", "all"]);
    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let id = found["result"]["structuredContent"]["matches"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let said = served.call(
        "note",
        serde_json::json!({ "task": id, "body": "they mentioned this on slack" }),
    );

    assert!(said["result"]["isError"].is_null(), "{said}");
    assert!(
        served.cli(&["show", &id]).contains("slack"),
        "the journal has to carry it"
    );
}

#[test]
fn what_it_attaches_says_where_it_came_from() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let filed = served.call(
        "propose",
        serde_json::json!({ "title": "the slack thread" }),
    );
    let id = filed["result"]["structuredContent"]["id"].as_str().unwrap();

    let loose = served.home.path().join("evidence.txt");
    std::fs::write(&loose, "a photo of the card stock").unwrap();
    let said = served.call(
        "attach",
        serde_json::json!({ "task": id, "path": loose.to_string_lossy() }),
    );

    assert!(said["result"]["isError"].is_null(), "{said}");
    let card = served.cli(&["show", id]);
    assert!(card.contains("evidence.txt"), "{card}");
    assert!(
        card.contains("kept from"),
        "attaching moves a local file into the folder that syncs, so it says which one: {card}"
    );

    let copies: Vec<_> = walked(&served.home.path().join("data/attachments")).collect();
    assert_eq!(copies.len(), 1, "the file is copied, never linked");
}

#[test]
fn a_file_over_the_limit_is_refused_with_the_size_it_copies() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let filed = served.call(
        "propose",
        serde_json::json!({ "title": "somewhere to put it" }),
    );
    let id = filed["result"]["structuredContent"]["id"].as_str().unwrap();

    // A fresh install copies up to five megabytes; a holiday video from a chat is over it.
    let heavy = served.home.path().join("holiday.mp4");
    std::fs::write(&heavy, vec![0u8; 6_000_000]).unwrap();
    let said = served.call(
        "attach",
        serde_json::json!({ "task": id, "path": heavy.to_string_lossy() }),
    );

    let why = said["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(said["result"]["isError"], true, "{said}");
    assert!(
        why.contains("MB"),
        "the refusal has to say the size it copies, or the model cannot adjust: {why}"
    );
    assert!(
        walked(&served.home.path().join("data/attachments"))
            .next()
            .is_none(),
        "nothing over the limit reaches the store"
    );
}

fn walked(at: &std::path::Path) -> Box<dyn Iterator<Item = std::path::PathBuf>> {
    let Ok(entries) = std::fs::read_dir(at) else {
        return Box::new(std::iter::empty());
    };
    Box::new(entries.filter_map(Result::ok).flat_map(|one| {
        let path = one.path();
        if path.is_dir() {
            walked(&path)
        } else {
            Box::new(std::iter::once(path))
        }
    }))
}

#[test]
fn a_misspelt_argument_is_refused_instead_of_dropped() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);

    let said = served.call(
        "propose",
        serde_json::json!({ "title": "pay the deposit", "due": "2026-09-01" }),
    );

    let why = said["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(said["result"]["isError"], true, "{said}");
    assert!(
        why.contains("deadline"),
        "the refusal names what it does take: {why}"
    );
    assert!(
        !served.cli(&["ls", "all"]).contains("pay the deposit"),
        "a task filed with a field silently dropped is worse than none"
    );
}

#[test]
fn what_the_person_wrote_is_not_reported_as_the_agents() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call("propose", serde_json::json!({ "title": "what it filed" }));

    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let mine = &found["result"]["structuredContent"]["matches"][0];
    let theirs = served.call("find", serde_json::json!({ "query": "what it filed" }));

    assert_eq!(mine["by_agent"], false, "{mine}");
    assert_eq!(
        theirs["result"]["structuredContent"]["matches"][0]["by_agent"],
        true
    );
}

#[test]
fn the_model_is_told_what_day_it_is_before_being_asked_for_dates() {
    let served = Served::new();

    let said = served.talk(&[r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{}}"#]);
    let taught = said[0]["result"]["instructions"].as_str().unwrap();

    assert!(
        taught.starts_with("Today is 20"),
        "it is asked for ISO dates, so it has to know today: {taught}"
    );
}

#[test]
fn an_id_reaches_a_client_that_only_shows_the_text() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);

    let filed = served.call("propose", serde_json::json!({ "title": "card stock" }));
    let id = filed["result"]["structuredContent"]["id"].as_str().unwrap();

    assert!(
        filed["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(id),
        "some clients show the model only the text, and without the id it cannot note or attach"
    );
}

#[test]
fn turning_the_agent_off_takes_its_voice_away() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    served.call("propose", serde_json::json!({ "title": "while it could" }));

    served.cli(&["agent", "--off"]);
    let said = served.call(
        "propose",
        serde_json::json!({ "title": "after it could not" }),
    );

    assert_eq!(said["result"]["isError"], true);
    let left = served.cli(&["ls", "all"]);
    assert!(
        left.contains("while it could"),
        "what it filed stays: {left}"
    );
    assert!(!left.contains("after it could not"), "{left}");
}

#[test]
fn an_agent_may_only_take_files_from_where_a_download_lands() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let filed = served.call(
        "propose",
        serde_json::json!({ "title": "somewhere to put it" }),
    );
    let id = filed["result"]["structuredContent"]["id"].as_str().unwrap();

    for barred in ["config/config.toml", "data/store"] {
        let said = served.call(
            "attach",
            serde_json::json!({ "task": id, "path": served.home.path().join(barred) }),
        );
        assert_eq!(
            said["result"]["isError"], true,
            "attachments reach the shared folder, so {barred} is not the agent's to send"
        );
    }

    let allowed = std::env::temp_dir().join("tisty-test-evidence.txt");
    std::fs::write(&allowed, "a photo from the chat").unwrap();
    let said = served.call(
        "attach",
        serde_json::json!({ "task": id, "path": allowed.to_string_lossy() }),
    );
    assert!(
        said["result"]["isError"].is_null(),
        "what it downloaded is exactly what it should be able to keep: {said}"
    );
    let _ = std::fs::remove_file(&allowed);
}

#[test]
fn two_callers_racing_on_one_source_file_it_once() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);
    let call = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "propose", "arguments": {
            "title": "call the bank", "source": "wa:msg-991"
        }},
    })
    .to_string();

    let racing: Vec<_> = (0..8)
        .map(|_| {
            let mut child = served
                .command()
                .arg("mcp")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            writeln!(child.stdin.take().unwrap(), "{call}").unwrap();
            child
        })
        .collect();
    for one in racing {
        let _ = one.wait_with_output().unwrap();
    }

    let listed = served.cli(&["ls", "all"]);
    assert_eq!(
        listed.matches("call the bank").count(),
        1,
        "reading a message twice must not file it twice, however many ask at once: {listed}"
    );
}

#[test]
fn a_line_that_is_not_json_does_not_take_the_session_with_it() {
    let served = Served::new();

    let said = served.talk(&[
        "this is not json at all",
        r#"{"jsonrpc":"2.0","id":2,"method":"ping","params":{}}"#,
    ]);

    assert_eq!(
        said.len(),
        2,
        "the good request behind it still gets an answer"
    );
    assert_eq!(said[0]["error"]["code"], -32700);
    assert!(said[1]["result"].is_object());
}

#[test]
fn a_method_nobody_here_speaks_says_so_and_carries_on() {
    let served = Served::new();

    let said = served.talk(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"ping","params":{}}"#,
    ]);

    assert_eq!(said[0]["error"]["code"], -32601);
    assert!(said[1]["result"].is_object());
}

#[test]
fn a_title_that_would_outlive_its_worth_is_refused() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);

    let huge = served.call(
        "propose",
        serde_json::json!({ "title": "x".repeat(10_000) }),
    );
    let sneaky = served.call(
        "propose",
        serde_json::json!({ "title": "innocent\u{1b}[2K\u{1b}[1ANOT WHAT IT SAYS" }),
    );

    assert_eq!(
        huge["result"]["isError"], true,
        "an append-only log rereads it forever"
    );
    assert_eq!(
        sneaky["result"]["isError"], true,
        "control characters would let a title rewrite the terminal it is printed on"
    );
    assert_eq!(served.cli(&["ls", "all"]).matches('\u{1b}').count(), 0);
}

#[test]
fn reading_one_task_gives_the_journal_that_searching_leaves_out() {
    let served = Served::new();
    served.cli(&["ls", "all"]);
    served.cli(&["desc", "1", "what the thread said"]);
    served.cli(&["log", "1", "support promised to reply"]);
    served.cli(&["step", "1", "add", "gather the screenshots"]);
    served.cli(&["agent", "--on"]);

    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let hit = &found["result"]["structuredContent"]["matches"][0];
    let id = hit["id"].as_str().unwrap();
    let whole = served.call("read", serde_json::json!({ "task": id }));
    let held = &whole["result"]["structuredContent"];

    assert!(
        hit.get("journal").is_none(),
        "searching stays a summary; a list of twenty would drag every journal with it"
    );
    assert_eq!(held["journal"][0]["body"], "support promised to reply");
    assert_eq!(held["description"], "what the thread said");
    assert_eq!(held["steps"][0]["text"], "gather the screenshots");
    assert!(
        whole["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("support promised to reply"),
        "a client that shows only the text has to see it too"
    );
}

#[test]
fn reading_a_task_that_is_not_there_says_where_to_look() {
    let served = Served::new();
    served.cli(&["agent", "--on"]);

    let said = served.call(
        "read",
        serde_json::json!({ "task": "01M14RFT9ECC2B6E4CX4P59XPH" }),
    );

    assert_eq!(said["result"]["isError"], true);
    assert!(
        said["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("find"),
        "the refusal points at the next move"
    );
}

#[test]
fn a_client_of_either_era_gets_an_answer_it_understands() {
    let served = Served::new();

    let said = served.talk(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"old","version":"1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    ]);

    assert_eq!(said.len(), 2, "a notification takes no answer: {said:?}");
    assert!(
        said[0]["result"]["supportedVersions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "2026-07-28")
    );
    assert_eq!(said[1]["result"]["protocolVersion"], "2025-06-18");
    assert!(said[1]["result"]["instructions"].is_string());
}
