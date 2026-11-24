use super::*;

const MARK: Spoken = Spoken {
    mark: "hook cursor",
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
