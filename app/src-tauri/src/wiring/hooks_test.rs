use std::path::Path;

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
                assert!(seen.stray.is_empty(), "{kind:?} {os:?} {command}: {seen:?}");
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
        assert!(found.on, "{kind:?} {found:?}");
        assert_eq!(found.stray, [stray_at(&there, false)], "{kind:?}");

        let left = with(kind, "", &place(tmp.path(), os, &gone)).unwrap();
        written(kind, tmp.path(), &left);
        let found = seen(kind, &here).unwrap();
        assert_eq!(found.stray, [stray_at(&gone, true)], "{kind:?}");
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

    assert_eq!(found.stray, [stray_at("/opt/old/tisty", true)]);
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

        assert!(found.on || found.apart, "{said}");
        assert!(found.stray.is_empty(), "{said}: {found:?}");
    }
}

#[cfg(windows)]
#[test]
fn a_path_written_by_hand_is_read_in_its_quotes_and_never_out_of_a_wrapper() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, "C:\\New\\tisty.exe");
    for (said, read) in [
        (
            "\\\"C:\\\\No such Tisty\\\\tisty.exe\\\" hook cursor stop",
            Some("C:\\No such Tisty\\tisty.exe"),
        ),
        (
            "C:\\\\NoSuchTisty\\\\tisty.exe hook cursor stop",
            Some("C:\\NoSuchTisty\\tisty.exe"),
        ),
        (
            "C:\\\\Tools\\\\quiet.exe C:\\\\New\\\\tisty.exe hook cursor stop",
            None,
        ),
        ("C:/Tools/quiet.exe bin/tisty hook cursor stop", None),
        ("/c/New/tisty.exe hook cursor stop", None),
        ("/usr/bin/nice /opt/new/tisty hook cursor stop", None),
    ] {
        written(
            Kind::Cursor,
            tmp.path(),
            &format!(r#"{{ "hooks": {{ "stop": [ {{ "command": "{said}" }} ] }} }}"#),
        );

        let found = seen(Kind::Cursor, &here).unwrap();

        assert!(found.on || found.apart, "{said}");
        assert_eq!(
            found.stray,
            read.map(|one| stray_at(one, true))
                .into_iter()
                .collect::<Vec<_>>(),
            "{said}"
        );
    }
}

fn stray_at(said: &str, gone: bool) -> Stray {
    Stray {
        said: said.into(),
        gone,
    }
}

const SHARED: &str = r#"{
  "hooks": {
    "SessionStart": [
      { "hooks": [
        { "type": "command", "command": "C:\\mine\\my-own-hook.exe" },
        { "type": "command", "command": "C:\\Old\\tisty.exe", "args": ["hook", "claude-code", "start"] }
      ] }
    ],
    "Stop": [
      { "hooks": [
        { "type": "command", "command": "C:\\Old\\tisty.exe", "args": ["hook", "claude-code", "stop"] }
      ] }
    ]
  }
}
"#;

#[test]
fn a_hook_of_the_persons_in_the_same_group_stays_and_a_group_left_empty_goes() {
    let now = read(&without(Kind::Claude, SHARED).unwrap());

    let start = now["hooks"]["SessionStart"][0]["hooks"].as_array().unwrap();
    assert_eq!(start.len(), 1);
    assert_eq!(start[0]["command"], "C:\\mine\\my-own-hook.exe");
    assert!(now["hooks"].get("Stop").is_none());
}

#[test]
fn a_comment_inside_tistys_hook_does_not_hide_it() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/new/tisty");
    let written_old = with(
        Kind::Claude,
        "",
        &place(tmp.path(), Os::Linux, "/opt/old/tisty"),
    )
    .unwrap();
    let was = written_old.replacen(
        "\"type\": \"command\"",
        "// keep\n \"type\": \"command\"",
        1,
    );
    written(Kind::Claude, tmp.path(), &was);

    let now = with(Kind::Claude, &was, &here).unwrap();

    assert!(!now.contains("/opt/old/"), "{now}");
}

#[cfg(windows)]
#[test]
fn fixing_changes_the_program_alone_and_never_a_wrapper_nor_one_not_named() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, "C:\\New\\tisty.exe");
    let was = r#"{
  "hooks": {
    "Stop": [
      { "matcher": "mine", "hooks": [
        { "type": "command", "command": "C:\\mine\\my-own-hook.exe" },
        { "type": "command", "command": "C:\\NoSuch\\tisty.exe", "args": ["hook", "claude-code", "stop"], "timeout": 99 }
      ] }
    ],
    "SessionStart": [
      { "hooks": [ { "type": "command", "command": "C:\\Tools\\quiet.exe C:\\NoSuch\\tisty.exe hook claude-code start" } ] },
      { "hooks": [ { "type": "command", "command": "C:\\NoSuchEither\\tisty.exe", "args": ["hook", "claude-code", "start"] } ] }
    ]
  }
}
"#;
    written(Kind::Claude, tmp.path(), was);
    assert_eq!(seen(Kind::Claude, &here).unwrap().stray.len(), 2);

    let now = repointed(
        Kind::Claude,
        was,
        &here,
        &["C:\\NoSuch\\tisty.exe".to_string()],
    )
    .unwrap();

    let expected = was.replacen(
        "\"command\": \"C:\\\\NoSuch\\\\tisty.exe\"",
        "\"command\": \"C:\\\\New\\\\tisty.exe\"",
        1,
    );
    assert_eq!(now, expected);
    let out = without_those(
        Kind::Claude,
        was,
        &here,
        &["C:\\NoSuch\\tisty.exe".to_string()],
    )
    .unwrap();
    assert!(
        out.contains("my-own-hook") && out.contains("quiet.exe"),
        "{out}"
    );
    assert!(
        out.contains("NoSuchEither") && !out.contains("\"stop\""),
        "{out}"
    );
}

#[test]
fn hooks_that_cannot_be_read_are_said_so_and_never_to_be_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/tisty");
    let at = Kind::Claude.file(&here).unwrap();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, b"{ \"WHO\": \"Jos\xE9\" }").unwrap();

    let found = seen(Kind::Claude, &here).unwrap();

    assert!(found.unread && !found.on);
}

#[test]
fn a_hook_with_one_way_of_running_it_wrapped_by_the_person_is_never_taken_out() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/new/tisty");
    let was = r#"{ "version": 1, "hooks": { "sessionStart": [ { "bash": "/old/tisty hook copilot start", "powershell": "quiet.exe tisty.exe hook copilot start" } ] } }"#;

    let named = ["/old/tisty".to_string()];
    let out = without_those(Kind::Copilot, was, &here, &named).unwrap();

    assert_eq!(out, was);
    let fixed = repointed(Kind::Copilot, was, &here, &named).unwrap();
    assert_eq!(fixed, was.replace("/old/tisty", "/opt/new/tisty"));
}
