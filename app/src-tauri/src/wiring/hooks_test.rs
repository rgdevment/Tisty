use std::path::Path;

use super::*;

fn place(home: &Path, os: Os, command: &str) -> Place {
    Place {
        distro: None,
        os,
        home: home.to_path_buf(),
        config: home.join(".config"),
        dsh: home.join(".dsh"),
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
    assert_eq!(tools.len(), 3);
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
    assert_eq!(told["hooks"]["PostToolUse"].as_array().unwrap().len(), 3);
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
fn only_six_assistants_run_hooks() {
    for id in [
        "claude-code",
        "cursor",
        "gemini-cli",
        "copilot",
        "codex",
        "antigravity",
    ] {
        assert!(kind(id).is_some(), "{id}");
    }
    for id in ["claude-desktop", "vscode", "windsurf"] {
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

const ALL: [Kind; 6] = [
    Kind::Claude,
    Kind::Cursor,
    Kind::Gemini,
    Kind::Copilot,
    Kind::Codex,
    Kind::Antigravity,
];

fn root(kind: Kind) -> &'static str {
    match kind {
        Kind::Antigravity => "tisty",
        _ => "hooks",
    }
}

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

const BEFORE_ONLY: [(Kind, &[&str]); 6] = [
    (Kind::Claude, &["PreToolUse"]),
    (Kind::Cursor, &["preToolUse", "beforeShellExecution"]),
    (Kind::Gemini, &["BeforeTool"]),
    (Kind::Copilot, &["preToolUse"]),
    (Kind::Codex, &["PreToolUse"]),
    (Kind::Antigravity, &["PreToolUse"]),
];

fn before_events(kind: Kind) -> &'static [&'static str] {
    BEFORE_ONLY.iter().find(|one| one.0 == kind).unwrap().1
}

fn as_text(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap()
}

fn as_of_1_49(kind: Kind, text: &str) -> String {
    let mut now = read(text);
    for event in before_events(kind) {
        now[root(kind)].as_object_mut().unwrap().remove(*event);
    }
    as_text(&now)
}

fn person_entry(kind: Kind) -> Value {
    match kind {
        Kind::Claude | Kind::Gemini | Kind::Codex | Kind::Antigravity => {
            json!({ "matcher": "Bash", "hooks": [ { "type": "command", "command": "say hi" } ] })
        }
        Kind::Cursor | Kind::Copilot => json!({ "command": "say hi" }),
    }
}

fn linux() -> (tempfile::TempDir, Place) {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/tisty");
    (tmp, here)
}

#[test]
fn every_assistant_is_written_the_hook_that_asks_first_in_its_own_way() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Linux, "/opt/tisty");
    let args = json!(["hook", "claude-code", "before"]);

    let claude = read(&with(Kind::Claude, "", &here).unwrap());
    let pre = &claude["hooks"]["PreToolUse"];
    assert_eq!(pre.as_array().unwrap().len(), 1);
    assert_eq!(
        pre[0]["matcher"],
        "^(Write|Bash|PowerShell|mcp__ccd_session__start_session)$"
    );
    assert_eq!(pre[0]["hooks"][0]["args"], args);
    assert_eq!(pre[0]["hooks"][0]["command"], "/opt/tisty");
    assert_eq!(pre[0]["hooks"][0]["timeout"], 3);

    let cursor = read(&with(Kind::Cursor, "", &here).unwrap());
    assert_eq!(cursor["hooks"]["preToolUse"][0]["matcher"], "Write");
    assert_eq!(
        cursor["hooks"]["preToolUse"][0]["command"],
        "'/opt/tisty' hook cursor before"
    );
    assert_eq!(cursor["hooks"]["preToolUse"][0]["timeout"], 3);
    let shell = &cursor["hooks"]["beforeShellExecution"][0];
    let matcher = shell["matcher"].as_str().unwrap();
    assert_eq!(matcher, PROSE_WRITTEN);
    for expected in [
        "[mM][dD]",
        "[hH][aA][nN][dD][oO][fF][fF]",
        "[nN][oO][tT][eE][sS]",
        "[tT][oO][dD][oO]",
        "[pP][lL][aA][nN]",
        "[pP][rR][oO][mM][pP][tT]",
        "[bB][rR][iI][eE][fF]",
    ] {
        assert!(
            matcher.contains(expected),
            "{expected} missing in {matcher}"
        );
    }
    assert_eq!(shell["command"], "'/opt/tisty' hook cursor before");

    let gemini = read(&with(Kind::Gemini, "", &here).unwrap());
    let before = &gemini["hooks"]["BeforeTool"][0];
    assert_eq!(before["matcher"], "^(write_file|run_shell_command)$");
    assert_eq!(before["hooks"][0]["timeout"], 3_000);
    assert_eq!(before["hooks"][0]["name"], "tisty-before");
    assert_eq!(
        before["hooks"][0]["command"],
        "'/opt/tisty' hook gemini-cli before"
    );
}

#[test]
fn copilot_runs_the_hook_that_asks_first_through_a_shell_that_never_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let windows = place(tmp.path(), Os::Windows, WINDOWS);
    let elsewhere = place(tmp.path(), Os::Linux, "/opt/It's/tisty");

    let on_windows = read(&with(Kind::Copilot, "", &windows).unwrap());
    let before = &on_windows["hooks"]["preToolUse"][0];
    assert_eq!(before["matcher"], "create|bash|powershell|shell");
    assert_eq!(before["timeoutSec"], 3);
    assert_eq!(
        before["powershell"],
        format!("& '{WINDOWS}' hook copilot before; exit 0")
    );
    assert!(before.get("exec").is_none() && before.get("bash").is_none());

    let off_windows = read(&with(Kind::Copilot, "", &elsewhere).unwrap());
    let before = &off_windows["hooks"]["preToolUse"][0];
    assert_eq!(before["matcher"], "create|bash|powershell|shell");
    assert_eq!(
        before["bash"],
        "'/opt/It'\\''s/tisty' hook copilot before || true"
    );
    assert!(before.get("exec").is_none() && before.get("powershell").is_none());
}

#[test]
fn what_tisty_wrote_with_the_hook_that_asks_first_is_read_back_whole() {
    for kind in ALL {
        let (tmp, here) = linux();
        written(kind, tmp.path(), &with(kind, "", &here).unwrap());

        let found = seen(kind, &here).unwrap();

        assert!(found.on && !found.lacks_before && !found.apart, "{kind:?}");
    }
}

#[test]
fn hooks_of_the_release_before_have_no_hook_that_asks_first() {
    for kind in ALL {
        let (tmp, here) = linux();
        let old = as_of_1_49(kind, &with(kind, "", &here).unwrap());
        written(kind, tmp.path(), &old);

        let found = seen(kind, &here).unwrap();

        assert!(found.on && found.lacks_before, "{kind:?}");
    }
}

#[test]
fn turning_hooks_of_the_release_before_off_and_on_brings_the_hook_that_asks_first() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let full = with(kind, "", &here).unwrap();
        let old = as_of_1_49(kind, &full);

        let off = without(kind, &old).unwrap();
        let on = with(kind, &off, &here).unwrap();

        assert!(!hooked(kind, &off), "{kind:?}");
        assert!(!lacks_before(kind, &on), "{kind:?}");
        assert_eq!(read(&on)[root(kind)], read(&full)[root(kind)], "{kind:?}");
    }
}

#[test]
fn taking_hooks_out_takes_the_one_that_asks_first_and_leaves_the_persons() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let mut mixed = read(&with(kind, "", &here).unwrap());
        let event = before_events(kind)[0];
        mixed[root(kind)][event]
            .as_array_mut()
            .unwrap()
            .insert(0, person_entry(kind));

        let now = without(kind, &as_text(&mixed)).unwrap();
        let told = read(&now);

        assert!(!hooked(kind, &now), "{kind:?}");
        assert_eq!(
            told[root(kind)][event],
            json!([person_entry(kind)]),
            "{kind:?}"
        );
        assert!(!now.contains("/opt/tisty"), "{kind:?} {now}");
    }
}

#[test]
fn taking_hooks_out_leaves_no_event_that_held_only_the_one_that_asks_first() {
    for kind in ALL {
        let (_tmp, here) = linux();

        let now = read(&without(kind, &with(kind, "", &here).unwrap()).unwrap());

        for event in before_events(kind) {
            assert!(now[root(kind)].get(*event).is_none(), "{kind:?} {event}");
        }
    }
}

#[test]
fn fixing_the_program_repoints_the_hook_that_asks_first_too() {
    for kind in ALL {
        let tmp = tempfile::tempdir().unwrap();
        let old = place(tmp.path(), Os::Linux, "/opt/old/tisty");
        let here = place(tmp.path(), Os::Linux, "/opt/new/tisty");
        let was = with(kind, "", &old).unwrap();

        let now = repointed(kind, &was, &here, &["/opt/old/tisty".to_string()]).unwrap();

        assert!(!now.contains("/opt/old/"), "{kind:?} {now}");
        assert_eq!(
            now,
            was.replace("/opt/old/tisty", "/opt/new/tisty"),
            "{kind:?}"
        );
        let told = read(&now);
        let event = before_events(kind)[0];
        assert!(as_text(&told[root(kind)][event]).contains("/opt/new/tisty"));
    }
}

#[test]
fn fixing_the_program_keeps_the_tail_of_copilots_shell_form_on_both_systems() {
    let tmp = tempfile::tempdir().unwrap();
    let elsewhere_old = place(tmp.path(), Os::Linux, "/opt/old/tisty");
    let elsewhere = place(tmp.path(), Os::Linux, "/opt/new/tisty");
    let was = with(Kind::Copilot, "", &elsewhere_old).unwrap();
    let now = repointed(
        Kind::Copilot,
        &was,
        &elsewhere,
        &["/opt/old/tisty".to_string()],
    )
    .unwrap();
    let before = &read(&now)["hooks"]["preToolUse"][0];
    assert_eq!(
        before["bash"],
        "'/opt/new/tisty' hook copilot before || true"
    );
}

#[cfg(windows)]
#[test]
fn fixing_the_program_keeps_the_tail_of_copilots_powershell_form_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let old = place(tmp.path(), Os::Windows, "C:\\NoSuch\\Old\\tisty.exe");
    let here = place(tmp.path(), Os::Windows, "C:\\New One\\tisty.exe");
    let was = with(Kind::Copilot, "", &old).unwrap();

    let now = repointed(
        Kind::Copilot,
        &was,
        &here,
        &["C:\\NoSuch\\Old\\tisty.exe".to_string()],
    )
    .unwrap();

    let told = read(&now);
    assert_eq!(
        told["hooks"]["preToolUse"][0]["powershell"],
        "& 'C:\\New One\\tisty.exe' hook copilot before; exit 0"
    );
    assert_eq!(
        told["hooks"]["sessionStart"][0]["exec"],
        "C:\\New One\\tisty.exe"
    );
}

#[test]
fn codex_runs_its_hook_through_cmd_with_double_quotes_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(
        tmp.path(),
        Os::Windows,
        "C:\\Program Files\\O'Neil\\tisty.exe",
    );

    let now = read(&with(Kind::Codex, "", &here).unwrap());

    let start = &now["hooks"]["SessionStart"][0]["hooks"][0];
    assert_eq!(start["type"], "command");
    assert_eq!(
        start["command"],
        "\"C:\\Program Files\\O'Neil\\tisty.exe\" hook codex start"
    );
    assert_eq!(
        now["hooks"]["Stop"][0]["hooks"][0]["command"],
        "\"C:\\Program Files\\O'Neil\\tisty.exe\" hook codex stop"
    );
}

#[test]
fn codex_runs_its_hook_through_a_posix_shell_off_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Macos, "/Apps/It's Tisty/tisty");

    let now = read(&with(Kind::Codex, "", &here).unwrap());

    assert_eq!(
        now["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "'/Apps/It'\\''s Tisty/tisty' hook codex start"
    );
}

#[test]
fn codex_reads_seconds_and_matches_the_tools_that_leave_something() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Codex, "", &here).unwrap());

    assert_eq!(now["hooks"]["SessionStart"][0]["hooks"][0]["timeout"], 15);
    assert_eq!(now["hooks"]["SessionStart"][0].get("matcher"), None);
    assert_eq!(
        now["hooks"]["PreToolUse"][0]["matcher"],
        "^(Bash|apply_patch)$"
    );
    assert_eq!(now["hooks"]["PreToolUse"][0]["hooks"][0]["timeout"], 3);
    assert_eq!(
        now["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "'/opt/tisty' hook codex before"
    );
    let tools = now["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
    assert_eq!(tools[0]["matcher"], "^apply_patch$");
    assert_eq!(tools[1]["matcher"], "^mcp__tisty__");
    assert_eq!(tools[1]["hooks"][0]["timeout"], 5);
    assert_eq!(now["hooks"]["Stop"][0]["hooks"][0]["timeout"], 5);
}

#[test]
fn codex_and_antigravity_each_get_a_file_of_their_own() {
    let (tmp, here) = linux();

    assert_eq!(
        Kind::Codex.file(&here).unwrap(),
        tmp.path().join(".codex").join("hooks.json")
    );
    assert_eq!(
        Kind::Antigravity.file(&here).unwrap(),
        tmp.path().join(".gemini").join("config").join("hooks.json")
    );
    assert!(!alone(Kind::Codex) && !alone(Kind::Antigravity));
}

#[test]
fn antigravity_keeps_its_hooks_under_a_name_and_not_under_hooks() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Antigravity, "", &here).unwrap());

    assert!(now.get("hooks").is_none(), "{now}");
    let ours = &now["tisty"];
    assert_eq!(ours["SessionStart"].as_array().unwrap().len(), 1);
    assert_eq!(ours["SessionStart"][0]["type"], "command");
    assert_eq!(
        ours["SessionStart"][0]["command"],
        "'/opt/tisty' hook antigravity start"
    );
    assert_eq!(ours["SessionStart"][0]["timeout"], 15);
    assert!(ours["SessionStart"][0].get("hooks").is_none());
    assert_eq!(
        ours["Stop"][0]["command"],
        "'/opt/tisty' hook antigravity stop"
    );
    assert!(ours["Stop"][0].get("matcher").is_none());
}

#[test]
fn antigravity_matches_the_tools_that_leave_something_by_their_own_names() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Antigravity, "", &here).unwrap());

    let before = &now["tisty"]["PreToolUse"][0];
    assert_eq!(before["matcher"], "^(write_to_file|run_command)$");
    assert_eq!(before["hooks"][0]["timeout"], 3);
    assert_eq!(
        before["hooks"][0]["command"],
        "'/opt/tisty' hook antigravity before"
    );
    let tools = now["tisty"]["PostToolUse"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
    assert_eq!(
        tools[0]["matcher"],
        "^(write_to_file|replace_file_content|multi_replace_file_content)$"
    );
    assert_eq!(tools[1]["matcher"], "^mcp_tisty_");
    assert_eq!(
        tools[1]["hooks"][0]["command"],
        "'/opt/tisty' hook antigravity tool"
    );
}

#[test]
fn antigravity_runs_its_hook_through_cmd_with_double_quotes_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, "C:\\Users\\O'Neil & Co\\tisty.exe");

    let now = read(&with(Kind::Antigravity, "", &here).unwrap());

    assert_eq!(
        now["tisty"]["Stop"][0]["command"],
        "\"C:\\Users\\O'Neil & Co\\tisty.exe\" hook antigravity stop"
    );
    assert_eq!(
        now["tisty"]["PreToolUse"][0]["hooks"][0]["command"],
        "\"C:\\Users\\O'Neil & Co\\tisty.exe\" hook antigravity before"
    );
}

#[test]
fn antigravity_runs_its_hook_through_posix_shell_on_macos_and_wsl2() {
    let tmp = tempfile::tempdir().unwrap();
    let macos = place(tmp.path(), Os::Macos, "/Apps/It's Tisty/tisty");
    let now = read(&with(Kind::Antigravity, "", &macos).unwrap());
    assert_eq!(
        now["tisty"]["Stop"][0]["command"],
        "'/Apps/It'\\''s Tisty/tisty' hook antigravity stop"
    );

    let mut wsl = place(
        tmp.path(),
        Os::Windows,
        "/mnt/c/Users/O'Neil/AppData/Local/Programs/Tisty/tisty.exe",
    );
    wsl.distro = Some("Ubuntu".to_string());
    let now = read(&with(Kind::Antigravity, "", &wsl).unwrap());
    assert_eq!(
        now["tisty"]["Stop"][0]["command"],
        "'/mnt/c/Users/O'\\''Neil/AppData/Local/Programs/Tisty/tisty.exe' hook antigravity stop"
    );
    assert_eq!(
        now["tisty"]["PreToolUse"][0]["hooks"][0]["command"],
        "'/mnt/c/Users/O'\\''Neil/AppData/Local/Programs/Tisty/tisty.exe' hook antigravity before"
    );
}

const OTHER_HOOK: &str = "{\n  \"my-linter\": {\n    \"Stop\": [\n      { \"type\": \"command\", \"command\": \"lint --all\" }\n    ]\n  }\n}\n";

#[test]
fn another_named_hook_of_antigravity_stays_untouched_when_adding_and_removing() {
    let (tmp, here) = linux();

    let added = with(Kind::Antigravity, OTHER_HOOK, &here).unwrap();
    let told = read(&added);
    assert_eq!(told["my-linter"], read(OTHER_HOOK)["my-linter"]);
    assert!(hooked(Kind::Antigravity, &added));
    written(Kind::Antigravity, tmp.path(), &added);
    assert!(!seen(Kind::Antigravity, &here).unwrap().apart);

    let removed = without(Kind::Antigravity, &added).unwrap();

    assert!(!hooked(Kind::Antigravity, &removed));
    assert_eq!(read(&removed)["my-linter"], read(OTHER_HOOK)["my-linter"]);
    assert!(!removed.contains("/opt/tisty"), "{removed}");
}

#[test]
fn taking_antigravity_out_leaves_no_hook_name_of_ours_behind() {
    let (_tmp, here) = linux();
    let added = with(Kind::Antigravity, "", &here).unwrap();

    let removed = without(Kind::Antigravity, &added).unwrap();

    assert!(!removed.contains("/opt/tisty"), "{removed}");
    assert!(read(&removed).get("tisty").is_none(), "{removed}");
}

#[test]
fn antigravitys_hook_name_stays_when_the_person_left_something_in_it() {
    let (_tmp, here) = linux();
    let mut now = read(&with(Kind::Antigravity, "", &here).unwrap());
    now["tisty"]["enabled"] = json!(false);

    let removed = without(Kind::Antigravity, &as_text(&now)).unwrap();

    assert_eq!(
        read(&removed)["tisty"],
        json!({ "enabled": false }),
        "{removed}"
    );
}

#[test]
fn a_handler_of_the_persons_inside_tistys_stop_stays_when_ours_goes() {
    let (_tmp, here) = linux();
    let mine = json!({ "type": "command", "command": "say bye" });
    let mut now = read(&with(Kind::Antigravity, "", &here).unwrap());
    now["tisty"]["Stop"]
        .as_array_mut()
        .unwrap()
        .insert(0, mine.clone());

    let removed = without(Kind::Antigravity, &as_text(&now)).unwrap();
    let told = read(&removed);

    assert!(!hooked(Kind::Antigravity, &removed));
    assert_eq!(told["tisty"]["Stop"], json!([mine]));
    assert!(told["tisty"].get("SessionStart").is_none(), "{removed}");
}

#[test]
fn tisty_run_from_another_named_hook_of_antigravity_is_the_persons_way() {
    let (_tmp, here) = linux();
    let was = r#"{ "mine": { "Stop": [ { "type": "command", "command": "/opt/tisty hook antigravity stop" } ] } }"#;

    assert!(!hooked(Kind::Antigravity, was));
    assert!(apart(Kind::Antigravity, was));
    assert_eq!(with(Kind::Antigravity, was, &here), None);
    assert_eq!(without(Kind::Antigravity, was).unwrap(), was);
}

#[test]
fn antigravitys_hook_turned_off_by_hand_is_the_persons_way() {
    let (_tmp, here) = linux();
    let mut now = read(&with(Kind::Antigravity, "", &here).unwrap());
    now["tisty"]["enabled"] = json!(false);
    let off = as_text(&now);

    assert!(apart(Kind::Antigravity, &off));
    assert_eq!(with(Kind::Antigravity, &off, &here), None);
}

#[test]
fn another_named_hook_of_antigravity_that_never_runs_tisty_is_not_apart() {
    let was = r#"{ "my-linter": { "PreToolUse": [ { "matcher": "run_command", "hooks": [ { "command": "./lint.sh" } ] } ] } }"#;
    assert!(!apart(Kind::Antigravity, was));
}

#[test]
fn a_file_of_antigravity_tisty_cannot_read_is_left_alone() {
    let (_tmp, here) = linux();
    assert_eq!(with(Kind::Antigravity, "[1, 2]", &here), None);
    assert_eq!(with(Kind::Antigravity, r#"{ "tisty": [] }"#, &here), None);
}

#[test]
fn taking_codex_out_leaves_the_persons_hooks_and_file() {
    let (_tmp, here) = linux();
    let was = "{\n  \"hooks\": {\n    \"Stop\": [\n      { \"hooks\": [ { \"type\": \"command\", \"command\": \"say done\" } ] }\n    ]\n  }\n}\n";

    let added = with(Kind::Codex, was, &here).unwrap();
    let removed = without(Kind::Codex, &added).unwrap();

    assert_eq!(read(&removed), read(was));
}

#[cfg(windows)]
#[test]
fn fixing_the_program_keeps_codexs_double_quoted_form_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let old = place(tmp.path(), Os::Windows, "C:\\NoSuch\\Old\\tisty.exe");
    let here = place(tmp.path(), Os::Windows, "C:\\New One\\tisty.exe");
    let was = with(Kind::Codex, "", &old).unwrap();

    let now = repointed(
        Kind::Codex,
        &was,
        &here,
        &["C:\\NoSuch\\Old\\tisty.exe".to_string()],
    )
    .unwrap();

    let told = read(&now);
    assert_eq!(
        told["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "\"C:\\New One\\tisty.exe\" hook codex start"
    );
    assert!(!now.contains("NoSuch"), "{now}");
}

#[cfg(windows)]
#[test]
fn a_codex_path_written_in_double_quotes_is_read_back_whole() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path(), Os::Windows, "C:\\New\\tisty.exe");
    let old = place(tmp.path(), Os::Windows, "C:\\No such Tisty\\tisty.exe");
    written(
        Kind::Codex,
        tmp.path(),
        &with(Kind::Codex, "", &old).unwrap(),
    );

    let found = seen(Kind::Codex, &here).unwrap();

    assert_eq!(
        found.stray,
        [stray_at("C:\\No such Tisty\\tisty.exe", true)]
    );
}

#[test]
fn a_command_windows_of_the_persons_beside_ours_makes_the_hook_apart() {
    let (tmp, here) = linux();
    for mine in ["quiet.exe tisty.exe hook codex start", "my-own-hook.cmd"] {
        let was = format!(
            r#"{{ "hooks": {{ "SessionStart": [ {{ "hooks": [ {{ "type": "command", "command": "'/old/tisty' hook codex start", "commandWindows": "{mine}" }} ] }} ] }} }}"#
        );
        written(Kind::Codex, tmp.path(), &was);

        let found = seen(Kind::Codex, &here).unwrap();

        assert!(found.apart && !found.on, "{mine}: {found:?}");
        assert_eq!(with(Kind::Codex, &was, &here), None, "{mine}");
    }
}

#[test]
fn a_command_windows_of_the_persons_is_never_taken_out_nor_rewritten() {
    let (_tmp, here) = linux();
    let was = r#"{ "hooks": { "SessionStart": [ { "hooks": [ { "type": "command", "command": "'/old/tisty' hook codex start", "commandWindows": "quiet.exe tisty.exe hook codex start" } ] } ] } }"#;
    let named = ["/old/tisty".to_string()];

    assert_eq!(without_those(Kind::Codex, was, &here, &named).unwrap(), was);
    assert_eq!(without(Kind::Codex, was).unwrap(), was);
    let fixed = repointed(Kind::Codex, was, &here, &named).unwrap();
    assert_eq!(fixed, was.replace("/old/tisty", "/opt/tisty"));
    assert!(fixed.contains("quiet.exe tisty.exe hook codex start"));
}

const SHELL_EVENTS: [(Kind, &str); 6] = [
    (Kind::Claude, "PostToolUse"),
    (Kind::Cursor, "afterShellExecution"),
    (Kind::Gemini, "AfterTool"),
    (Kind::Copilot, "postToolUse"),
    (Kind::Codex, "PostToolUse"),
    (Kind::Antigravity, "PostToolUse"),
];

fn shell_event(kind: Kind) -> &'static str {
    SHELL_EVENTS.iter().find(|one| one.0 == kind).unwrap().1
}

fn is_shell(entry: &Value) -> bool {
    entry.to_string().contains("shell")
        && !entry.to_string().contains("beforeShellExecution")
        && !entry.to_string().contains(" before")
        && !entry.to_string().contains("\"before\"")
}

fn shell_entry(kind: Kind, value: &Value) -> Value {
    value[root(kind)][shell_event(kind)]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| is_shell(one))
        .unwrap_or_else(|| panic!("{kind:?} has no shell hook in {value}"))
        .clone()
}

fn as_of_the_release_without_shell(kind: Kind, text: &str) -> String {
    let mut now = read(text);
    let event = shell_event(kind);
    let entries = now[root(kind)][event].as_array_mut().unwrap();
    entries.retain(|one| !is_shell(one));
    if entries.is_empty() {
        now[root(kind)].as_object_mut().unwrap().remove(event);
    }
    as_text(&now)
}

#[test]
fn every_assistant_is_written_a_hook_after_its_shell_commands() {
    for kind in ALL {
        let (_tmp, here) = linux();

        let now = read(&with(kind, "", &here).unwrap());
        let entry = shell_entry(kind, &now);

        assert!(entry.to_string().contains("/opt/tisty"), "{kind:?} {entry}");
    }
}

#[test]
fn claude_code_matches_the_shells_it_runs() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Claude, "", &here).unwrap());
    let shell = shell_entry(Kind::Claude, &now);

    assert_eq!(shell["matcher"], "^(Bash|PowerShell)$");
    assert_eq!(
        shell["hooks"][0]["args"],
        json!(["hook", "claude-code", "shell"])
    );
    assert_eq!(shell["hooks"][0]["command"], "/opt/tisty");
    assert_eq!(shell["hooks"][0]["timeout"], 5);
    let post = now["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(post.len(), 3);
    assert_eq!(post[0]["matcher"], "Edit|Write|MultiEdit|NotebookEdit");
    assert_eq!(
        post[1]["matcher"],
        "^(mcp__(tisty|plugin_tisty_tisty)__|MCP:)"
    );
    assert_eq!(post[2], shell);
}

#[test]
fn cursor_runs_a_hook_after_each_shell_execution() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Cursor, "", &here).unwrap());
    let after = now["hooks"]["afterShellExecution"].as_array().unwrap();

    assert_eq!(after.len(), 1);
    assert_eq!(after[0]["command"], "'/opt/tisty' hook cursor shell");
    assert_eq!(after[0]["timeout"], 5);
    assert!(after[0].get("matcher").is_none());
    assert!(now["hooks"]["beforeShellExecution"].is_array());
}

#[test]
fn gemini_matches_its_shell_tool_and_reads_milliseconds() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Gemini, "", &here).unwrap());
    let after = now["hooks"]["AfterTool"].as_array().unwrap();
    let shell = shell_entry(Kind::Gemini, &now);

    assert_eq!(after.len(), 3);
    assert_eq!(after[1]["matcher"], "^mcp_tisty_");
    assert_eq!(after[2], shell);
    assert_eq!(shell["matcher"], "^run_shell_command$");
    assert_eq!(shell["hooks"][0]["timeout"], 5_000);
    assert_eq!(
        shell["hooks"][0]["command"],
        "'/opt/tisty' hook gemini-cli shell"
    );
}

#[test]
fn copilot_matches_bash_powershell_and_shell_after_its_own_tools() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Copilot, "", &here).unwrap());
    let post = now["hooks"]["postToolUse"].as_array().unwrap();

    assert_eq!(post.len(), 3);
    assert_eq!(
        post[0]["matcher"],
        "create|edit|apply_patch|str_replace_editor"
    );
    assert_eq!(post[1]["matcher"], "tisty-.*");
    assert_eq!(post[2]["matcher"], "bash|powershell|shell");
    assert_eq!(post[2]["timeoutSec"], 5);
    assert_eq!(post[2]["exec"], "/opt/tisty");
    assert_eq!(post[2]["args"], json!(["hook", "copilot", "shell"]));
}

#[test]
fn copilot_runs_the_shell_hook_itself_with_its_moment_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let windows = place(tmp.path(), Os::Windows, WINDOWS);

    let now = read(&with(Kind::Copilot, "", &windows).unwrap());
    let shell = &now["hooks"]["postToolUse"][2];

    assert_eq!(shell["matcher"], "bash|powershell|shell");
    assert_eq!(shell["exec"], WINDOWS);
    assert_eq!(shell["args"], json!(["hook", "copilot", "shell"]));
    assert!(shell.get("powershell").is_none() && shell.get("bash").is_none());
}

#[test]
fn codex_matches_only_bash_for_its_shell() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Codex, "", &here).unwrap());
    let tools = now["hooks"]["PostToolUse"].as_array().unwrap();

    assert_eq!(tools.len(), 3);
    assert_eq!(tools[0]["matcher"], "^apply_patch$");
    assert_eq!(tools[1]["matcher"], "^mcp__tisty__");
    assert_eq!(tools[2]["matcher"], "^Bash$");
    assert_eq!(tools[2]["hooks"][0]["timeout"], 5);
    assert_eq!(
        tools[2]["hooks"][0]["command"],
        "'/opt/tisty' hook codex shell"
    );
}

#[test]
fn antigravity_matches_run_command_for_its_shell() {
    let (_tmp, here) = linux();

    let now = read(&with(Kind::Antigravity, "", &here).unwrap());
    let tools = now["tisty"]["PostToolUse"].as_array().unwrap();

    assert_eq!(tools.len(), 3);
    assert_eq!(tools[2]["matcher"], "^run_command$");
    assert_eq!(
        tools[2]["hooks"][0]["command"],
        "'/opt/tisty' hook antigravity shell"
    );
}

#[test]
fn what_tisty_wrote_with_the_shell_hook_lacks_nothing() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let text = with(kind, "", &here).unwrap();

        assert!(!lacks_before(kind, &text), "{kind:?}");
    }
}

#[test]
fn hooks_without_the_shell_moment_are_told_apart_from_complete_ones() {
    for kind in ALL {
        let (tmp, here) = linux();
        let old = as_of_the_release_without_shell(kind, &with(kind, "", &here).unwrap());
        written(kind, tmp.path(), &old);

        let found = seen(kind, &here).unwrap();

        assert!(lacks_before(kind, &old), "{kind:?}");
        assert!(found.on && found.lacks_before && !found.apart, "{kind:?}");
    }
}

#[test]
fn hooks_without_the_hook_that_asks_first_still_lack_it() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let old = as_of_1_49(kind, &with(kind, "", &here).unwrap());

        assert!(lacks_before(kind, &old), "{kind:?}");
    }
}

#[test]
fn hooks_lacking_both_moments_are_not_called_complete() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let no_shell = as_of_the_release_without_shell(kind, &with(kind, "", &here).unwrap());
        let neither = as_of_1_49(kind, &no_shell);

        assert!(lacks_before(kind, &neither), "{kind:?}");
    }
}

#[test]
fn turning_hooks_off_and_on_brings_the_shell_hook_back() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let full = with(kind, "", &here).unwrap();
        let old = as_of_the_release_without_shell(kind, &full);

        let off = without(kind, &old).unwrap();
        let on = with(kind, &off, &here).unwrap();

        assert!(!lacks_before(kind, &on), "{kind:?}");
        assert_eq!(read(&on)[root(kind)], read(&full)[root(kind)], "{kind:?}");
    }
}

#[test]
fn installing_twice_writes_the_shell_hook_once() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let once = with(kind, "", &here).unwrap();

        let twice = with(kind, &once, &here).unwrap();

        assert_eq!(read(&twice), read(&once), "{kind:?}");
    }
}

#[test]
fn taking_hooks_out_takes_the_shell_hook_and_leaves_the_persons() {
    for kind in ALL {
        let (_tmp, here) = linux();
        let mut mixed = read(&with(kind, "", &here).unwrap());
        let event = shell_event(kind);
        mixed[root(kind)][event]
            .as_array_mut()
            .unwrap()
            .insert(0, person_entry(kind));

        let now = without(kind, &as_text(&mixed)).unwrap();
        let told = read(&now);

        assert!(!hooked(kind, &now), "{kind:?}");
        assert_eq!(
            told[root(kind)][event],
            json!([person_entry(kind)]),
            "{kind:?}"
        );
        assert!(!now.contains("/opt/tisty"), "{kind:?} {now}");
    }
}

#[test]
fn fixing_the_program_repoints_the_shell_hook_too() {
    for kind in ALL {
        let tmp = tempfile::tempdir().unwrap();
        let old = place(tmp.path(), Os::Linux, "/opt/old/tisty");
        let here = place(tmp.path(), Os::Linux, "/opt/new/tisty");
        let was = with(kind, "", &old).unwrap();

        let now = repointed(kind, &was, &here, &["/opt/old/tisty".to_string()]).unwrap();
        let shell = shell_entry(kind, &read(&now));

        assert!(
            shell.to_string().contains("/opt/new/tisty"),
            "{kind:?} {shell}"
        );
        assert!(!now.contains("/opt/old/"), "{kind:?} {now}");
    }
}
