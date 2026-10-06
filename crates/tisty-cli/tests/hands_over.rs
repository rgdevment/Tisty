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
                                            "clientInfo": { "name": "test", "version": "1" } }),
        _ => serde_json::json!({}),
    };
    let said =
        serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
    writeln!(into, "{said}").unwrap();
    into.flush().unwrap();
    let mut line = String::new();
    from.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn an_update_laid_in_place_takes_the_door_without_the_client_noticing() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    ask(&mut into, &mut from, 1, "initialize");

    door.laid_anew(Path::new(env!("CARGO_BIN_EXE_tisty")));
    let said = ask(&mut into, &mut from, 2, "tools/list");

    assert_eq!(said["id"], 2);
    assert!(
        said["result"]["tools"]
            .as_array()
            .is_some_and(|all| !all.is_empty()),
        "{said}"
    );
    assert!(
        door.log().contains("carries on through it"),
        "the old door answered instead of the new one: {}",
        door.log()
    );
    drop(into);
    assert!(child.wait().unwrap().success());
}

#[test]
fn a_replacement_that_will_not_run_leaves_the_old_door_serving() {
    let door = Door::new();
    let (mut child, mut into, mut from) = door.open();
    ask(&mut into, &mut from, 1, "initialize");
    let broken = door.home.path().join("broken");
    std::fs::write(&broken, b"not a program").unwrap();

    door.laid_anew(&broken);
    let said = ask(&mut into, &mut from, 2, "tools/list");

    assert!(
        said["result"]["tools"]
            .as_array()
            .is_some_and(|all| !all.is_empty()),
        "{said}"
    );
    assert!(door.log().contains("serves on"), "{}", door.log());
    assert_eq!(ask(&mut into, &mut from, 3, "ping")["id"], 3);
    drop(into);
    assert!(child.wait().unwrap().success());
}
