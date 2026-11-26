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

fn run_named(name: &str, script: &str, heard: &[u8], timeout: Duration) -> Ran {
    let scratch = tempfile::tempdir().unwrap();
    let ran = run(Asked {
        name,
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

// PowerShell on Windows, and a POSIX shell wherever one is found, so Windows runs the Linux scripts too.
fn each_shell(
    windows: &str,
    posix: &str,
    heard: &[u8],
    timeout: Duration,
) -> Vec<(&'static str, Ran)> {
    let mut scripts = Vec::new();
    if cfg!(windows) {
        scripts.push(("t.ps1", windows));
    }
    if interpreter("sh").is_ok() {
        scripts.push(("t.sh", posix));
    }
    assert!(!scripts.is_empty());
    scripts
        .into_iter()
        .map(|(name, script)| (name, run_named(name, script, heard, timeout)))
        .collect()
}

const LONG: Duration = Duration::from_secs(60);

#[test]
fn a_script_that_exits_two_with_a_message_blocks_and_says_why() {
    for (name, ran) in each_shell(
        "[Console]::Error.WriteLine('no rm here'); exit 2",
        "echo 'no rm here' >&2; exit 2",
        b"{}",
        LONG,
    ) {
        assert_eq!(ran.code, Some(2), "{name}");
        assert_eq!(ran.said, "no rm here", "{name}");
        assert!(ran.blocks(), "{name}");
    }
}

#[test]
fn a_script_that_exits_zero_or_one_does_not_block() {
    for code in [0, 1] {
        let said = format!("exit {code}");
        for (name, ran) in each_shell(&said, &said, b"{}", LONG) {
            assert_eq!((ran.code, ran.blocks()), (Some(code), false), "{name}");
        }
    }
}

#[test]
fn a_script_hears_what_the_agent_said_and_which_event_and_agent_asked() {
    for (name, ran) in each_shell(
        "$said = [Console]::In.ReadToEnd().Trim(); [Console]::Error.WriteLine(\"$said|$env:TISTY_HOOK_EVENT|$env:TISTY_HOOK_AGENT\")",
        "read -r said; echo \"$said|$TISTY_HOOK_EVENT|$TISTY_HOOK_AGENT\" >&2",
        b"{\"a\":1}\n",
        LONG,
    ) {
        assert_eq!(ran.said, "{\"a\":1}|before-tool|codex", "{name}");
    }
}

#[test]
fn a_script_that_outlasts_its_time_is_felled_and_never_blocks() {
    for (name, ran) in each_shell(
        "Start-Sleep -Seconds 60; exit 2",
        "sleep 60; exit 2",
        b"{}",
        Duration::from_millis(500),
    ) {
        assert!(ran.timed_out, "{name}");
        assert!(!ran.blocks(), "{name}");
        assert!(ran.took < Duration::from_secs(30), "{name} {:?}", ran.took);
    }
}

#[test]
fn what_a_script_says_on_stderr_is_cut_at_the_cap() {
    for (name, ran) in each_shell(
        "$text = 'a' * 100000; [Console]::Error.Write($text)",
        "head -c 100000 /dev/zero | tr '\\000' a >&2",
        b"{}",
        LONG,
    ) {
        assert_eq!(ran.said.len(), SAID_AT_MOST, "{name}");
        assert!(ran.said.bytes().all(|one| one == b'a'), "{name}");
    }
}

#[test]
fn a_script_that_does_not_read_a_big_input_still_ends() {
    let heard = vec![b'x'; 4 * 1024 * 1024];
    for (name, ran) in each_shell("exit 0", "exit 0", &heard, LONG) {
        assert_eq!(ran.code, Some(0), "{name}");
    }
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
    for (name, ran) in each_shell(
        "Start-Process -NoNewWindow -FilePath powershell.exe -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 8'; [Console]::Error.WriteLine('early'); exit 0",
        "sleep 8 &\necho early >&2\nexit 0",
        b"{}",
        LONG,
    ) {
        assert_eq!(ran.code, Some(0), "{name}");
        assert_eq!(ran.said, "early", "{name}");
        assert!(ran.took < Duration::from_secs(6), "{name} {:?}", ran.took);
    }
}

#[cfg(windows)]
#[test]
fn a_shell_script_runs_with_gits_own_bash_and_never_with_wsls() {
    let installed = std::env::var_os("ProgramFiles")
        .map(|at| {
            std::path::PathBuf::from(at)
                .join("Git")
                .join("bin")
                .join("bash.exe")
        })
        .is_some_and(|at| at.is_file());
    if let Ok((bash, _)) = interpreter("sh") {
        let said = bash.to_string_lossy().to_lowercase();
        assert!(
            said.ends_with("bash.exe")
                && !said.contains("system32")
                && !said.contains("windowsapps"),
            "{said}"
        );
    } else {
        assert!(!installed);
    }
}
