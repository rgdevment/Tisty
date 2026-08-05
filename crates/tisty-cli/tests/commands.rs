//! End to end: the binary is run the way a person runs it. Unit tests never
//! caught the selector bugs; using the thing did.

use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

struct Cli {
    home: TempDir,
}

struct Run {
    out: String,
    err: String,
    code: i32,
}

impl Cli {
    fn new() -> Self {
        Self {
            home: tempfile::tempdir().unwrap(),
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        self.pipe(args, None)
    }

    fn pipe(&self, args: &[&str], stdin: Option<&str>) -> Run {
        let root = self.home.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_tisty"));
        command
            .args(args)
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .stdin(Stdio::piped());

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(text) = stdin {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(text.as_bytes())
                .unwrap();
        }
        drop(child.stdin.take());

        finish(child.wait_with_output().unwrap())
    }

    fn ok(&self, args: &[&str]) -> String {
        let run = self.run(args);
        assert_eq!(run.code, 0, "`{}` failed: {}", args.join(" "), run.err);
        run.out
    }
}

fn finish(output: Output) -> Run {
    Run {
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
        code: output.status.code().unwrap_or(-1),
    }
}

#[test]
fn a_bare_phrase_is_captured_and_listed() {
    let cli = Cli::new();
    cli.ok(&["renew the certificate"]);

    let out = cli.ok(&["ls", "all"]);
    assert!(out.contains("renew the certificate"), "{out}");
}

#[test]
fn a_date_in_the_phrase_leaves_the_title_clean() {
    let cli = Cli::new();
    let out = cli.ok(&["pay the invoice tomorrow"]);

    assert!(out.contains("pay the invoice"), "{out}");
    assert!(!out.contains("pay the invoice tomorrow"), "{out}");
    assert!(out.contains("tomorrow"), "{out}");
}

/// The bug this guards against completed an unrelated task in silence.
#[test]
fn a_number_from_a_listing_that_no_longer_applies_is_refused() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["search", "second"]);
    let run = cli.run(&["done", "2"]);

    assert_eq!(run.code, 4, "{}{}", run.out, run.err);
    assert!(run.err.contains("last listing"), "{}", run.err);
    assert!(cli.ok(&["ls", "all"]).contains("second task"));
}

#[test]
fn completing_moves_a_task_into_the_archive() {
    let cli = Cli::new();
    cli.ok(&["ship the release"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);

    assert!(!cli.ok(&["ls", "all"]).contains("ship the release"));
    assert!(cli.ok(&["ls", "archive"]).contains("ship the release"));
}

#[test]
fn the_archive_stays_searchable() {
    let cli = Cli::new();
    cli.ok(&["investigate the timeout"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["log", "1", "the retry budget was exhausted"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);

    let out = cli.ok(&["search", "retry budget"]);
    assert!(out.contains("investigate the timeout"), "{out}");
}

#[test]
fn search_reaches_past_the_title() {
    let cli = Cli::new();
    cli.ok(&["prepare the handover"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["desc", "1", "the credentials live in the shared vault"]);

    assert!(
        cli.ok(&["search", "vault"])
            .contains("prepare the handover")
    );
    assert!(
        cli.ok(&["search", "nothing at all"])
            .contains("nothing here")
    );
}

#[test]
fn undo_takes_back_the_last_change() {
    let cli = Cli::new();
    cli.ok(&["book the venue"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);
    assert!(cli.ok(&["ls", "archive"]).contains("book the venue"));

    cli.ok(&["undo"]);
    assert!(cli.ok(&["ls", "all"]).contains("book the venue"));
    assert!(!cli.ok(&["ls", "archive"]).contains("book the venue"));
}

/// One user action, however many events it took: undoing a tag rename halfway
/// would leave the tasks disagreeing about what they are tagged.
#[test]
fn undo_takes_back_a_whole_batch_not_one_event_of_it() {
    let cli = Cli::new();
    for title in ["first job", "second job", "third job"] {
        cli.ok(&[title]);
    }
    cli.ok(&["ls", "all"]);
    for n in ["1", "2", "3"] {
        cli.ok(&["set", n, "--tag", "wip"]);
    }

    cli.ok(&["tag", "rename", "wip", "active"]);
    assert_eq!(cli.ok(&["ls", "all"]).matches("@active").count(), 3);

    cli.ok(&["undo"]);
    let out = cli.ok(&["ls", "all"]);

    assert_eq!(out.matches("@wip").count(), 3, "{out}");
    assert_eq!(out.matches("@active").count(), 0, "{out}");
}

#[test]
fn undo_on_an_empty_store_says_so_instead_of_failing() {
    let cli = Cli::new();
    let run = cli.run(&["undo"]);

    assert_eq!(run.code, 0, "{}", run.err);
    assert!(run.out.contains("nothing to undo"), "{}", run.out);
}

/// A journal entry emptied by an undo must not linger as a dated blank.
#[test]
fn undoing_a_journal_entry_leaves_no_trace_in_the_journal() {
    let cli = Cli::new();
    cli.ok(&["chase the invoice"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["log", "1", "left a voicemail"]);
    cli.ok(&["undo"]);

    let out = cli.ok(&["show", "1"]);
    assert!(!out.contains("journal"), "{out}");
    assert!(!out.contains("voicemail"), "{out}");
}

#[test]
fn erasing_refuses_to_guess_when_nobody_can_confirm() {
    let cli = Cli::new();
    cli.ok(&["draft the policy"]);
    cli.ok(&["ls", "all"]);

    let run = cli.run(&["rm", "1"]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
    assert!(cli.ok(&["ls", "all"]).contains("draft the policy"));

    cli.ok(&["rm", "1", "--force"]);
    assert!(!cli.ok(&["ls", "all"]).contains("draft the policy"));
}

#[test]
fn fields_are_set_and_cleared_one_at_a_time() {
    let cli = Cli::new();
    cli.ok(&["audit the permissions"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["set", "1", "--date", "2026-12-24", "--priority", "1"]);
    let out = cli.ok(&["ls", "all"]);
    assert!(out.contains("24"), "{out}");
    assert!(out.contains("!1"), "{out}");

    cli.ok(&["set", "1", "--no-date"]);
    let out = cli.ok(&["ls", "all"]);
    assert!(!out.contains("24"), "{out}");
    assert!(
        out.contains("!1"),
        "the priority was not asked about: {out}"
    );
}

#[test]
fn a_value_that_is_not_a_date_is_rejected_rather_than_guessed() {
    let cli = Cli::new();
    cli.ok(&["review the contract"]);
    cli.ok(&["ls", "all"]);

    let run = cli.run(&["set", "1", "--date", "sometime"]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
    assert!(run.err.contains("sometime"), "{}", run.err);
}

#[test]
fn tags_are_added_and_taken_off() {
    let cli = Cli::new();
    cli.ok(&["rotate the keys"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["set", "1", "--tag", "security", "--tag", "ops"]);
    assert!(cli.ok(&["tag", "ls"]).contains("@security"));

    cli.ok(&["set", "1", "--untag", "ops"]);
    let out = cli.ok(&["tag", "ls"]);
    assert!(out.contains("@security"), "{out}");
    assert!(!out.contains("@ops"), "{out}");
}

#[test]
fn renaming_a_tag_rewrites_every_task_that_carries_it() {
    let cli = Cli::new();
    for title in ["first job", "second job"] {
        cli.ok(&[title]);
    }
    cli.ok(&["ls", "all"]);
    cli.ok(&["set", "1", "--tag", "wip"]);
    cli.ok(&["set", "2", "--tag", "wip"]);

    cli.ok(&["tag", "rename", "wip", "active"]);
    let out = cli.ok(&["tag", "ls"]);

    assert!(out.contains("@active"), "{out}");
    assert!(!out.contains("@wip"), "{out}");
    assert!(cli.ok(&["ls", "all"]).matches("@active").count() == 2);
}

#[test]
fn a_task_files_under_a_list_and_comes_back_when_it_is_erased() {
    let cli = Cli::new();
    cli.ok(&["migrate the database"]);
    cli.ok(&["list", "add", "Platform"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["mv", "1", "Platform"]);
    assert!(!cli.ok(&["ls", "inbox"]).contains("migrate the database"));

    cli.ok(&["list", "rm", "Platform", "--force"]);
    assert!(cli.ok(&["ls", "inbox"]).contains("migrate the database"));
}

#[test]
fn steps_are_addressed_by_the_number_that_is_printed() {
    let cli = Cli::new();
    cli.ok(&["cut the release"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["step", "1", "add", "tag the commit"]);
    cli.ok(&["step", "1", "add", "publish the notes"]);
    cli.ok(&["step", "1", "done", "2"]);

    let out = cli.ok(&["show", "1"]);
    assert!(out.contains("1/2"), "{out}");

    let run = cli.run(&["step", "1", "done", "9"]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
}

#[test]
fn a_body_can_arrive_on_stdin() {
    let cli = Cli::new();
    cli.ok(&["write the postmortem"]);
    cli.ok(&["ls", "all"]);

    let run = cli.pipe(
        &["desc", "1"],
        Some("root cause: the cache never expired\n"),
    );
    assert_eq!(run.code, 0, "{}", run.err);

    assert!(cli.ok(&["show", "1"]).contains("root cause"));
}

#[test]
fn every_read_command_can_speak_json() {
    let cli = Cli::new();
    cli.ok(&["index the archive"]);
    cli.ok(&["list", "add", "Research"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["set", "1", "--tag", "reading"]);

    for args in [
        &["ls", "all", "--json"][..],
        &["show", "1", "--json"][..],
        &["search", "index", "--json"][..],
        &["lists", "--json"][..],
        &["tag", "ls", "--json"][..],
    ] {
        let out = cli.ok(args);
        serde_json::from_str::<serde_json::Value>(out.trim())
            .unwrap_or_else(|e| panic!("`{}` did not produce json: {e}\n{out}", args.join(" ")));
    }
}

#[test]
fn the_interface_follows_the_locale_but_the_data_does_not() {
    let cli = Cli::new();
    cli.ok(&["ship the thing"]);

    let root = cli.home.path();
    let out = Command::new(env!("CARGO_BIN_EXE_tisty"))
        .args(["ls", "all"])
        .env("TISTY_DATA", root.join("data"))
        .env("TISTY_CONFIG", root.join("config"))
        .env("TISTY_CACHE", root.join("cache"))
        .env("NO_COLOR", "1")
        .env("LANG", "es_CL.UTF-8")
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout);

    assert!(out.contains("todas"), "{out}");
    assert!(out.contains("ship the thing"), "{out}");
}

#[test]
fn an_unknown_filter_names_the_ones_that_exist() {
    let cli = Cli::new();
    let run = cli.run(&["ls", "nonsense"]);

    assert_eq!(run.code, 1);
    assert!(run.err.contains("inbox"), "{}", run.err);
}
