use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, SystemTime};

use tempfile::TempDir;

struct Door {
    home: TempDir,
    binary: PathBuf,
}

impl Door {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let binary = home
            .path()
            .join("bin")
            .join(Path::new(env!("CARGO_BIN_EXE_tisty")).file_name().unwrap());
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::copy(env!("CARGO_BIN_EXE_tisty"), &binary).unwrap();
        let door = Self { home, binary };
        door.command().args(["algo mio"]).output().unwrap();
        door.command().args(["agent", "--on"]).output().unwrap();
        door
    }

    fn command(&self) -> Command {
        let root = self.home.path();
        let mut command = Command::new(&self.binary);
        command
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .stdin(Stdio::null());
        command
    }

    fn open(&self) -> (Child, ChildStdin, BufReader<ChildStdout>) {
        let mut child = self
            .command()
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let into = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        (child, into, from)
    }

    /// What an installer does on Windows, where a running binary can be moved but not written.
    fn laid_anew(&self, with: &Path) {
        std::fs::rename(&self.binary, self.binary.with_extension("old")).unwrap();
        std::fs::copy(with, &self.binary).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&self.binary)
            .unwrap()
            .set_modified(SystemTime::now() + Duration::from_secs(60))
            .unwrap();
    }

    fn log(&self) -> String {
        std::fs::read_to_string(self.home.path().join("config/private/tisty.log"))
            .unwrap_or_default()
    }
}

fn ask(
    into: &mut ChildStdin,
    from: &mut BufReader<ChildStdout>,
    id: u64,
    method: &str,
) -> serde_json::Value {
    let params = match method {
        "initialize" => serde_json::json!({ "protocolVersion": "2025-06-18", "capabilities": {},
                                            "clientInfo": { "name": "claude-code", "version": "1" } }),
        _ => serde_json::json!({}),
    };
    said(
        into,
        from,
        serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    )
}

fn propose(
    into: &mut ChildStdin,
    from: &mut BufReader<ChildStdout>,
    id: u64,
    title: &str,
) -> serde_json::Value {
    said(
        into,
        from,
        serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call",
                            "params": { "name": "propose", "arguments": { "title": title } } }),
    )
}

fn said(
    into: &mut ChildStdin,
    from: &mut BufReader<ChildStdout>,
    asked: serde_json::Value,
) -> serde_json::Value {
    writeln!(into, "{asked}").unwrap();
    into.flush().unwrap();
    let mut line = String::new();
    from.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn listed(said: &serde_json::Value) -> bool {
    said["result"]["tools"]
        .as_array()
        .is_some_and(|all| !all.is_empty())
}

impl Door {
    fn added(&self) -> Vec<String> {
        std::fs::read_dir(self.home.path().join("data/store"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| std::fs::read_to_string(e.path().join("active.tisty")).ok())
            .flat_map(|held| {
                held.lines()
                    .filter(|line| {
                        line.contains("\"task.add\"") && line.contains("\"tags\":[\"agent\"]")
                    })
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[test]
fn an_update_laid_in_place_takes_the_door_without_the_client_noticing() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    ask(&mut into, &mut from, 1, "initialize");

    door.laid_anew(Path::new(env!("CARGO_BIN_EXE_tisty")));
    let said = ask(&mut into, &mut from, 2, "tools/list");

    assert_eq!(said["id"], 2);
    assert!(listed(&said), "{said}");
    assert!(
        door.log().contains("carries on through it"),
        "the old door answered instead of the new one: {}",
        door.log()
    );
    drop(into);
    assert!(child.wait().unwrap().success());
}

#[test]
fn the_new_door_is_greeted_as_the_client_greeted_the_old_one() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    ask(&mut into, &mut from, 1, "initialize");

    door.laid_anew(Path::new(env!("CARGO_BIN_EXE_tisty")));
    propose(&mut into, &mut from, 2, "after the update");

    let added = door.added();
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(
        added[0].contains("\"via\":\"claude-code\""),
        "the new door wrote without knowing which client spoke: {added:?}"
    );
    drop(into);
    assert!(child.wait().unwrap().success());
}

#[test]
fn a_client_that_never_greeted_has_nothing_said_twice() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    propose(&mut into, &mut from, 1, "before the update");

    door.laid_anew(Path::new(env!("CARGO_BIN_EXE_tisty")));
    let said = ask(&mut into, &mut from, 2, "tools/list");

    assert!(listed(&said), "{said}");
    assert!(
        door.log().contains("carries on through it"),
        "{}",
        door.log()
    );
    assert_eq!(
        door.added().len(),
        1,
        "a write the client made once reached the store twice"
    );
    drop(into);
    assert!(child.wait().unwrap().success());
}

#[test]
fn a_replacement_that_will_not_run_leaves_the_old_door_serving_and_is_tried_once() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    ask(&mut into, &mut from, 1, "initialize");
    let broken = door.home.path().join("broken");
    std::fs::write(&broken, b"not a program").unwrap();

    door.laid_anew(&broken);
    let said = ask(&mut into, &mut from, 2, "tools/list");
    assert_eq!(ask(&mut into, &mut from, 3, "ping")["id"], 3);

    assert!(listed(&said), "{said}");
    assert_eq!(door.log().matches("serves on").count(), 1, "{}", door.log());
    drop(into);
    assert!(child.wait().unwrap().success());
}

#[test]
fn a_replacement_that_starts_but_never_answers_leaves_the_old_door_serving() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    ask(&mut into, &mut from, 1, "initialize");
    let silent = if cfg!(windows) {
        PathBuf::from(std::env::var("SystemRoot").unwrap()).join("System32/whoami.exe")
    } else {
        PathBuf::from("/bin/true")
    };

    door.laid_anew(&silent);
    let said = ask(&mut into, &mut from, 2, "tools/list");

    assert!(listed(&said), "{said}");
    assert!(door.log().contains("serves on"), "{}", door.log());
    drop(into);
    assert!(child.wait().unwrap().success());
}
