use super::*;

#[test]
fn only_known_extensions_have_an_interpreter() {
    assert_eq!(
        interpreter("exe").map(|_| ()),
        Err(NotRun::NoInterpreter("none"))
    );
}

#[test]
fn a_ran_script_blocks_only_on_two_within_its_time() {
    let ran = |code, timed_out| Ran {
        code: Some(code),
        said: String::new(),
        timed_out,
        took: Duration::ZERO,
    };
    assert!(ran(2, false).blocks());
    assert!(!ran(2, true).blocks());
    assert!(!ran(1, false).blocks());
}

fn name() -> &'static str {
    match cfg!(windows) {
        true => "t.ps1",
        false => "t.sh",
    }
}

fn run_script(windows: &str, unix: &str, heard: &[u8], timeout: Duration) -> Ran {
    let scratch = tempfile::tempdir().unwrap();
    let script = match cfg!(windows) {
        true => windows,
        false => unix,
    };
    let ran = run(Asked {
        name: name(),
        script: script.as_bytes(),
        scratch: scratch.path(),
        heard: heard.to_vec(),
        event: "before-tool",
        agent: "codex",
        timeout,
    })
    .unwrap();
    assert_eq!(std::fs::read_dir(scratch.path()).unwrap().count(), 0);
    ran
}

const LONG: Duration = Duration::from_secs(60);

#[test]
fn a_script_that_exits_two_with_a_message_blocks_and_says_why() {
    let ran = run_script(
        "[Console]::Error.WriteLine('no rm here'); exit 2",
        "echo 'no rm here' >&2; exit 2",
        b"{}",
        LONG,
    );
    assert_eq!(ran.code, Some(2));
    assert_eq!(ran.said, "no rm here");
    assert!(ran.blocks());
}

#[test]
fn a_script_that_exits_zero_or_one_does_not_block() {
    let zero = run_script("exit 0", "exit 0", b"{}", LONG);
    let one = run_script("exit 1", "exit 1", b"{}", LONG);
    assert_eq!((zero.code, zero.blocks()), (Some(0), false));
    assert_eq!((one.code, one.blocks()), (Some(1), false));
}

#[test]
fn a_script_hears_what_the_agent_said_and_which_event_and_agent_asked() {
    let ran = run_script(
        "$said = [Console]::In.ReadToEnd().Trim(); [Console]::Error.WriteLine(\"$said|$env:TISTY_HOOK_EVENT|$env:TISTY_HOOK_AGENT\")",
        "read -r said; echo \"$said|$TISTY_HOOK_EVENT|$TISTY_HOOK_AGENT\" >&2",
        b"{\"a\":1}\n",
        LONG,
    );
    assert_eq!(ran.said, "{\"a\":1}|before-tool|codex");
}

#[test]
fn a_script_that_outlasts_its_time_is_felled_and_never_blocks() {
    let ran = run_script(
        "Start-Sleep -Seconds 60; exit 2",
        "sleep 60; exit 2",
        b"{}",
        Duration::from_millis(500),
    );
    assert!(ran.timed_out);
    assert!(!ran.blocks());
    assert!(ran.took < Duration::from_secs(30), "{:?}", ran.took);
}

#[test]
fn what_a_script_says_on_stderr_is_cut_at_the_cap() {
    let ran = run_script(
        "$text = 'a' * 100000; [Console]::Error.Write($text)",
        "head -c 100000 /dev/zero | tr '\0' a >&2",
        b"{}",
        LONG,
    );
    assert_eq!(ran.said.len(), SAID_AT_MOST);
}

#[test]
fn a_script_that_does_not_read_a_big_input_still_ends() {
    let heard = vec![b'x'; 4 * 1024 * 1024];
    let ran = run_script("exit 0", "exit 0", &heard, LONG);
    assert_eq!(ran.code, Some(0));
}

#[test]
fn a_name_with_no_known_extension_is_not_run() {
    let scratch = tempfile::tempdir().unwrap();
    let asked = Asked {
        name: "t.exe",
        script: b"",
        scratch: scratch.path(),
        heard: Vec::new(),
        event: "stop",
        agent: "codex",
        timeout: LONG,
    };
    assert_eq!(run(asked), Err(NotRun::NoInterpreter("none")));
}

#[test]
fn on_path_finds_a_program_only_in_an_absolute_directory() {
    assert!(on_path("definitely-not-a-program-tisty").is_none());
    let shell = interpreter(if cfg!(windows) { "ps1" } else { "sh" }).unwrap();
    assert!(shell.0.is_absolute());
}

#[test]
fn a_script_leaving_a_background_process_holding_stderr_neither_hangs_nor_loses_what_it_said() {
    let ran = run_script(
        "Start-Process -NoNewWindow -FilePath powershell.exe -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 8'; [Console]::Error.WriteLine('early'); exit 0",
        "sleep 8 &\necho early >&2\nexit 0",
        b"{}",
        LONG,
    );
    assert_eq!(ran.code, Some(0));
    assert_eq!(ran.said, "early");
    assert!(ran.took < Duration::from_secs(6), "{:?}", ran.took);
}
