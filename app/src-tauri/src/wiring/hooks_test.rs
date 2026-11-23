use super::*;

fn place(home: &Path, os: Os, command: &str) -> Place {
    Place {
        distro: None,
        os,
        home: home.to_path_buf(),
        config: home.join(".config"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: command.to_string(),
        open: true,
        packaged: false,
        mount: String::new(),
    }
}

fn read(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

const WINDOWS: &str = "C:\\Users\\A B\\AppData\\Local\\Microsoft\\WindowsApps\\tisty.exe";

#[test]
fn claude_code_runs_the_binary_itself_with_its_moment() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, WINDOWS);

    let now = read(&with(Kind::Claude, "", &here).unwrap());

    let start = &now["hooks"]["SessionStart"][0]["hooks"][0];
    assert_eq!(start["command"], WINDOWS);
    assert_eq!(start["args"], json!(["hook", "claude-code", "start"]));
    let tools = now["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0]["matcher"], "Edit|Write|MultiEdit|NotebookEdit");
    assert_eq!(tools[1]["hooks"][0]["args"][2], "tool");
    assert_eq!(now["hooks"]["Stop"][0]["hooks"][0]["args"][2], "stop");
}

#[test]
fn what_the_person_had_stays_where_it_was() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(
        tmp.path(),
        Os::Macos,
        "/Applications/Tisty.app/Contents/MacOS/tisty",
    );
    let was = "{\n  \"model\": \"opus\",\n  \"hooks\": {\n    \"Stop\": [\n      { \"hooks\": [ { \"type\": \"command\", \"command\": \"say done\" } ] }\n    ]\n  }\n}\n";

    let now = with(Kind::Claude, was, &here).unwrap();
    let told = read(&now);

    assert!(now.starts_with("{\n  \"model\": \"opus\",\n"), "{now}");
    assert_eq!(told["hooks"]["Stop"][0]["hooks"][0]["command"], "say done");
    assert_eq!(told["hooks"]["Stop"][1]["hooks"][0]["args"][2], "stop");
    assert!(hooked(Kind::Claude, &now));
}

#[test]
fn installing_twice_writes_tistys_hooks_once() {
    let tmp = tempfile::tempdir().unwrap();
    let one = place(tmp.path(), Os::Linux, "/opt/one/tisty");
    let two = place(tmp.path(), Os::Linux, "/opt/two/tisty");

    let first = with(Kind::Claude, "", &one).unwrap();
    let again = with(Kind::Claude, &first, &two).unwrap();
    let told = read(&again);

    assert_eq!(told["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
    assert_eq!(told["hooks"]["PostToolUse"].as_array().unwrap().len(), 2);
    assert!(!again.contains("/opt/one/"), "{again}");
}

#[test]
fn removing_takes_out_only_tistys_hooks() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/tisty");
    let was = "{\n  \"hooks\": {\n    \"Stop\": [\n      { \"hooks\": [ { \"type\": \"command\", \"command\": \"say done\" } ] }\n    ]\n  }\n}\n";
    let installed = with(Kind::Claude, was, &here).unwrap();

    let removed = without(Kind::Claude, &installed).unwrap();
    let told = read(&removed);

    assert!(!hooked(Kind::Claude, &removed));
    assert_eq!(told["hooks"]["Stop"].as_array().unwrap().len(), 1);
    assert_eq!(told["hooks"]["Stop"][0]["hooks"][0]["command"], "say done");
    assert!(told["hooks"].get("SessionStart").is_none(), "{removed}");
    assert!(told["hooks"].get("PostToolUse").is_none(), "{removed}");
}

#[test]
fn a_hook_of_another_tool_that_names_tisty_is_never_taken_for_ours() {
    let was = "{ \"hooks\": { \"Stop\": [ { \"hooks\": [ { \"type\": \"command\", \"command\": \"echo tisty is great\" } ] } ] } }";
    assert!(!hooked(Kind::Claude, was));
    assert_eq!(without(Kind::Claude, was).unwrap(), was);
}

#[test]
fn cursor_gets_its_own_file_through_powershell_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, "C:\\Users\\O'Neil\\tisty.exe");

    let now = read(&with(Kind::Cursor, "", &here).unwrap());

    assert_eq!(now["version"], 1);
    assert_eq!(
        now["hooks"]["stop"][0]["command"],
        "& 'C:\\Users\\O''Neil\\tisty.exe' hook cursor stop"
    );
    assert!(now["hooks"]["stop"][0]["loop_limit"].is_null());
    assert!(now["hooks"]["stop"][0].get("loop_limit").is_some());
    assert_eq!(
        now["hooks"]["afterFileEdit"][0]["command"],
        "& 'C:\\Users\\O''Neil\\tisty.exe' hook cursor edit"
    );
    assert_eq!(
        Kind::Cursor.file(&here).unwrap(),
        tmp.path().join(".cursor").join("hooks.json")
    );
}

#[test]
fn gemini_reads_milliseconds_and_a_posix_shell_off_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Macos, "/Apps/It's Tisty/tisty");

    let now = read(&with(Kind::Gemini, "{\n  \"theme\": \"dark\"\n}\n", &here).unwrap());

    assert_eq!(now["theme"], "dark");
    let start = &now["hooks"]["SessionStart"][0]["hooks"][0];
    assert_eq!(
        start["command"],
        "'/Apps/It'\\''s Tisty/tisty' hook gemini-cli start"
    );
    assert_eq!(start["timeout"], 15_000);
    assert_eq!(now["hooks"]["AfterTool"][1]["matcher"], "^mcp_tisty_");
    assert_eq!(
        now["hooks"]["AfterAgent"][0]["hooks"][0]["name"],
        "tisty-stop"
    );
}

#[test]
fn copilot_gets_a_file_of_its_own_run_without_a_shell() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, WINDOWS);

    let now = read(&with(Kind::Copilot, "", &here).unwrap());

    assert!(alone(Kind::Copilot));
    assert_eq!(now["version"], 1);
    assert_eq!(now["hooks"]["sessionStart"][0]["exec"], WINDOWS);
    assert_eq!(now["hooks"]["postToolUse"][1]["matcher"], "tisty-.*");
    assert_eq!(
        now["hooks"]["agentStop"][0]["args"],
        json!(["hook", "copilot", "stop"])
    );
    assert_eq!(
        Kind::Copilot.file(&here).unwrap(),
        tmp.path().join(".copilot").join("hooks").join("tisty.json")
    );
}

#[test]
fn only_four_assistants_run_hooks() {
    for id in ["claude-code", "cursor", "gemini-cli", "copilot"] {
        assert!(kind(id).is_some(), "{id}");
    }
    for id in ["codex", "claude-desktop", "vscode", "windsurf"] {
        assert!(kind(id).is_none(), "{id}");
    }
}

#[test]
fn a_file_tisty_cannot_read_is_left_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/tisty");
    assert_eq!(with(Kind::Claude, "[1, 2]", &here), None);
    assert_eq!(with(Kind::Claude, "{ \"hooks\": [] }", &here), None);
}

const ALL: [Kind; 4] = [Kind::Claude, Kind::Cursor, Kind::Gemini, Kind::Copilot];

fn written(kind: Kind, home: &Path, text: &str) {
    let at = kind.file(&place(home, Os::Linux, "")).unwrap();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, text).unwrap();
}

#[test]
fn what_tisty_writes_is_read_back_as_this_tisty_whatever_its_path() {
    for command in [
        WINDOWS,
        "C:\\Users\\O'Neil & Co\\tisty.exe",
        "/Applications/Tisty.app/Contents/MacOS/tisty",
        "/Apps/It's Tisty/tisty",
        "/mnt/c/Program Files/Tisty/tisty.exe",
        "tisty",
    ] {
        for os in [Os::Windows, Os::Macos, Os::Linux] {
            for kind in ALL {
                let tmp = tempfile::tempdir().unwrap();
                let here = place(tmp.path(), os, command);
                written(kind, tmp.path(), &with(kind, "", &here).unwrap());

                let seen = seen(kind, &here).unwrap();

                assert!(seen.on, "{kind:?} {os:?} {command}");
                assert!(
                    !seen.astray && !seen.gone && seen.points.is_none(),
                    "{kind:?} {os:?} {command}: {seen:?}"
                );
            }
        }
    }
}

#[test]
fn hooks_of_a_tisty_that_is_no_longer_there_are_told_apart_from_another_one() {
    for kind in ALL {
        let tmp = tempfile::tempdir().unwrap();
        let os = match cfg!(windows) {
            true => Os::Windows,
            false => Os::Linux,
        };
        let there = tmp.path().join("other's").join("tisty.exe");
        std::fs::create_dir_all(there.parent().unwrap()).unwrap();
        std::fs::write(&there, b"").unwrap();
        let there = there.display().to_string();
        let gone = tmp.path().join("no such").join("tisty.exe");
        let gone = gone.display().to_string();
        let here = place(tmp.path(), os, "/opt/new/tisty");

        let another = with(kind, "", &place(tmp.path(), os, &there)).unwrap();
        written(kind, tmp.path(), &another);
        let found = seen(kind, &here).unwrap();
        assert!(
            found.on && found.astray && !found.gone,
            "{kind:?} {found:?}"
        );
        assert_eq!(found.points.as_deref(), Some(there.as_str()), "{kind:?}");

        let left = with(kind, "", &place(tmp.path(), os, &gone)).unwrap();
        written(kind, tmp.path(), &left);
        let found = seen(kind, &here).unwrap();
        assert!(found.astray && found.gone, "{kind:?} {found:?}");
        assert_eq!(found.points.as_deref(), Some(gone.as_str()), "{kind:?}");
    }
}

#[test]
fn one_hook_left_behind_is_enough_to_say_so() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/new/tisty");
    let mixed =
        with(Kind::Claude, "", &here)
            .unwrap()
            .replacen("/opt/new/tisty", "/opt/old/tisty", 1);
    written(Kind::Claude, tmp.path(), &mixed);

    let found = seen(Kind::Claude, &here).unwrap();

    assert!(found.astray);
    assert_eq!(found.points.as_deref(), Some("/opt/old/tisty"));
}

#[test]
fn what_cannot_be_read_for_certain_is_never_called_broken() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/new/tisty");
    for said in [
        r#"{ "command": "tisty hook claude-code start" }"#,
        r#"{ "command": "tisty", "args": ["hook", "claude-code", "start"] }"#,
        r#"{ "command": "/bin/sh", "args": ["-c", "/old/tisty hook claude-code start"] }"#,
        r#"{ "command": "/usr/local/bin/my-tisty-wrapper.sh hook claude-code start" }"#,
        r#"{ "command": "TISTY=1 /old/tisty hook claude-code start" }"#,
    ] {
        written(
            Kind::Claude,
            tmp.path(),
            &format!(r#"{{ "hooks": {{ "SessionStart": [ {{ "hooks": [ {said} ] }} ] }} }}"#),
        );

        let found = seen(Kind::Claude, &here).unwrap();

        assert!(found.on, "{said}");
        assert!(!found.astray && !found.gone, "{said}: {found:?}");
    }
}

#[test]
fn a_path_written_by_hand_is_read_with_and_without_quotes() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, "C:\\New\\tisty.exe");
    for said in [
        "C:\\\\Old Tisty\\\\tisty.exe hook cursor stop",
        "\\\"C:\\\\Old Tisty\\\\tisty.exe\\\" hook cursor stop",
        "& \\\"C:\\\\Old Tisty\\\\tisty.exe\\\" hook cursor stop",
    ] {
        written(
            Kind::Cursor,
            tmp.path(),
            &format!(r#"{{ "hooks": {{ "stop": [ {{ "command": "{said}" }} ] }} }}"#),
        );

        let found = seen(Kind::Cursor, &here).unwrap();

        assert_eq!(
            found.points.as_deref(),
            Some("C:\\Old Tisty\\tisty.exe"),
            "{said}"
        );
    }
}
