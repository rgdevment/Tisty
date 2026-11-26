use super::*;

const MARK: Spoken = Spoken {
    mark: "hook cursor",
    root: "hooks",
    posix: false,
};

fn new<'a>(command: &'a str, spoken: &'a str) -> New<'a> {
    New {
        command,
        spoken,
        windows: false,
    }
}

fn one(command: &str) -> String {
    let said = serde_json::to_string(command).unwrap();
    format!("{{ \"hooks\": {{ \"stop\": [ {{ \"command\": {said} }} ] }} }}")
}

fn programs(text: &str) -> Vec<String> {
    found(text, &["stop"], &MARK)
        .into_iter()
        .flat_map(|run| run.programs)
        .map(|it| it.said)
        .collect()
}

#[test]
fn only_a_program_run_plainly_is_read_out_of_a_shell_line() {
    for (said, program) in [
        ("/opt/tisty hook cursor stop", Some("/opt/tisty")),
        ("'/o''p' hook cursor stop tisty", None),
        ("'/It'\\''s/tisty' hook cursor stop", Some("/It's/tisty")),
        (
            "& 'C:\\It''s\\tisty.exe' hook cursor stop",
            Some("C:\\It's\\tisty.exe"),
        ),
        (
            "\"C:\\A B\\tisty.exe\" hook cursor stop",
            Some("C:\\A B\\tisty.exe"),
        ),
        (
            "& \"C:\\A B\\tisty.exe\" hook cursor stop",
            Some("C:\\A B\\tisty.exe"),
        ),
        ("C:\\A B\\tisty.exe hook cursor stop", None),
        ("/usr/bin/nice /opt/tisty hook cursor stop", None),
        ("/bin/sh -c '/opt/tisty hook cursor stop'", None),
        ("C:/Tools/quiet.exe bin/tisty hook cursor stop", None),
        ("TISTY=1 /old/tisty hook cursor stop", None),
        ("' hook cursor stop tisty", None),
    ] {
        let found: Vec<String> = programs(&one(said));
        assert_eq!(found.first().map(String::as_str), program, "{said}");
    }
}

#[test]
fn only_the_program_changes_in_the_quotes_it_had() {
    for (was, now) in [
        (
            "/opt/tisty hook cursor stop --mine",
            "/New/tisty hook cursor stop --mine",
        ),
        (
            "'/opt/tisty' hook cursor stop",
            "'/New/tisty' hook cursor stop",
        ),
        (
            "& '/opt/tisty' hook cursor stop",
            "& '/New/tisty' hook cursor stop",
        ),
        (
            "\"/opt/tisty\" hook cursor stop",
            "\"/New/tisty\" hook cursor stop",
        ),
    ] {
        let text = one(was);
        let runs = found(&text, &["stop"], &MARK);

        let said = repointed(&text, &runs, |_| true, &new("/New/tisty", "spoken")).unwrap();

        assert_eq!(said, one(now));
    }
    let text = one("/opt/tisty hook cursor stop");
    let runs = found(&text, &["stop"], &MARK);
    let spaced = repointed(
        &text,
        &runs,
        |_| true,
        &new("/New one/tisty", "'/New one/tisty'"),
    )
    .unwrap();
    assert_eq!(spaced, one("'/New one/tisty' hook cursor stop"));
    let text = one("\"/opt/tisty\" hook cursor stop");
    let runs = found(&text, &["stop"], &MARK);
    assert_eq!(
        repointed(&text, &runs, |_| true, &new("/a\"b/tisty", "")),
        None
    );
}

#[test]
fn every_way_one_hook_is_run_is_read_and_changed() {
    let text = r#"{ "hooks": { "stop": [ { "bash": "/old/tisty hook cursor stop", "powershell": "& 'C:\\Old\\tisty.exe' hook cursor stop" } ] } }"#;
    assert_eq!(programs(text), ["/old/tisty", "C:\\Old\\tisty.exe"]);
    let runs = found(text, &["stop"], &MARK);

    let now = repointed(text, &runs, |_| true, &new("/New/tisty", "")).unwrap();

    assert_eq!(programs(&now), ["/New/tisty", "/New/tisty"]);
}

#[test]
fn a_group_that_is_not_an_object_or_a_list_is_passed_over() {
    assert!(
        programs(r#"{ "hooks": { "stop": [ "x", 3, { "hooks": "y" }, { "hooks": [ 4 ] } ] } }"#)
            .is_empty()
    );
    assert!(programs(r#"{ "hooks": { "stop": "tisty hook cursor stop" } }"#).is_empty());
    assert!(programs(r#"{ "hooks": [] }"#).is_empty());
}

fn moments(text: &str, mark: &str, events: &[&'static str]) -> Vec<String> {
    let spoken = Spoken {
        mark,
        root: "hooks",
        posix: false,
    };
    found(text, events, &spoken)
        .into_iter()
        .map(|run| run.moment)
        .collect()
}

#[test]
fn the_moment_a_hook_runs_for_is_read_in_every_form_it_is_written() {
    for (entry, mark, moment) in [
        (
            r#"{ "command": "/opt/tisty", "args": ["hook", "claude-code", "before"] }"#,
            "hook claude-code",
            "before",
        ),
        (
            r#"{ "exec": "/opt/tisty", "args": ["hook", "copilot", "stop"] }"#,
            "hook copilot",
            "stop",
        ),
        (
            r#"{ "command": "'/opt/tisty' hook cursor stop" }"#,
            "hook cursor",
            "stop",
        ),
        (
            r#"{ "powershell": "& 'C:\\x\\tisty.exe' hook copilot before; exit 0" }"#,
            "hook copilot",
            "before",
        ),
        (
            r#"{ "bash": "'/opt/tisty' hook copilot before || true" }"#,
            "hook copilot",
            "before",
        ),
        (
            r#"{ "bash": "'/opt/tisty' hook copilot start" }"#,
            "hook copilot",
            "start",
        ),
    ] {
        let text = format!(r#"{{ "hooks": {{ "ev": [ {entry} ] }} }}"#);
        assert_eq!(moments(&text, mark, &["ev"]), [moment], "{entry}");
    }
}

#[test]
fn a_hook_with_no_moment_after_its_program_reads_an_empty_one() {
    let text = r#"{ "hooks": { "ev": [ { "command": "'/opt/tisty' hook cursor" } ] } }"#;
    assert_eq!(moments(text, "hook cursor", &["ev"]), [""]);
}

const NAMED: Spoken = Spoken {
    mark: "hook antigravity",
    root: "tisty",
    posix: false,
};

fn named_one(command: &str) -> String {
    let said = serde_json::to_string(command).unwrap();
    format!(
        "{{ \"hooks\": {{ \"Stop\": [ {{ \"command\": \"x\" }} ] }}, \"tisty\": {{ \"Stop\": [ {{ \"command\": {said} }} ] }} }}"
    )
}

#[test]
fn a_root_other_than_hooks_is_the_one_read() {
    let text = named_one("'/opt/tisty' hook antigravity stop");

    let runs = found(&text, &["Stop"], &NAMED);

    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].moment, "stop");
    assert_eq!(runs[0].programs[0].said, "/opt/tisty");
    assert!(found(&text, &["Stop"], &MARK).is_empty());
}

#[test]
fn the_list_and_whether_it_fits_are_looked_for_under_the_root_given() {
    let text = named_one("'/opt/tisty' hook antigravity stop");

    assert!(list(&text, "tisty", "Stop").is_some());
    assert!(list(&text, "mine", "Stop").is_none());
    assert!(list_fits(&text, "tisty", "Stop"));
    assert!(!list_fits(r#"{ "tisty": { "Stop": 3 } }"#, "tisty", "Stop"));
    assert!(list_fits(r#"{ "tisty": { "Stop": 3 } }"#, "hooks", "Stop"));
}

#[test]
fn taking_out_the_only_hook_of_an_event_unsets_it_under_its_own_root() {
    let text = named_one("'/opt/tisty' hook antigravity stop");
    let run = found(&text, &["Stop"], &NAMED).pop().unwrap();

    let now = taken_out(&text, &run).unwrap();

    let told: serde_json::Value = serde_json::from_str(&now).unwrap();
    assert!(told["tisty"].get("Stop").is_none(), "{now}");
    assert_eq!(told["hooks"]["Stop"][0]["command"], "x");
}

#[test]
fn a_command_windows_is_read_as_one_more_way_to_run_the_hook() {
    let text = r#"{ "hooks": { "stop": [ { "command": "'/old/tisty' hook cursor stop", "commandWindows": "\"C:\\Old\\tisty.exe\" hook cursor stop" } ] } }"#;

    assert_eq!(programs(text), ["/old/tisty", "C:\\Old\\tisty.exe"]);
    let runs = found(text, &["stop"], &MARK);
    assert!(runs[0].all_read);

    let now = repointed(text, &runs, |_| true, &new("/New/tisty", "")).unwrap();

    assert_eq!(programs(&now), ["/New/tisty", "/New/tisty"]);
}

#[test]
fn a_command_windows_run_in_a_wrapper_leaves_the_hook_not_wholly_read() {
    let text = r#"{ "hooks": { "stop": [ { "command": "'/old/tisty' hook cursor stop", "commandWindows": "quiet.exe tisty.exe hook cursor stop" } ] } }"#;

    let runs = found(text, &["stop"], &MARK);

    assert_eq!(runs.len(), 1);
    assert!(!runs[0].all_read);
    assert_eq!(programs(text), ["/old/tisty"]);
}
