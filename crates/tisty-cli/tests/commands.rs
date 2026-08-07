//! End to end: the binary is run the way a person runs it. Unit tests never
//! caught the selector bugs; using the thing did.

use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

struct Cli {
    home: TempDir,
    zone: &'static str,
}

struct Run {
    out: String,
    err: String,
    code: i32,
}

impl Cli {
    /// A fixed zone, or the suite drifts with whoever runs it.
    fn new() -> Self {
        Self::in_zone("America/Santiago")
    }

    fn in_zone(zone: &'static str) -> Self {
        Self {
            home: tempfile::tempdir().unwrap(),
            zone,
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        self.pipe(args, None)
    }

    fn pipe(&self, args: &[&str], stdin: Option<&str>) -> Run {
        self.as_device(self.home.path(), self.zone, args, stdin)
    }

    /// Clears `LC_ALL` too: it outranks `LANG` and would pick the language here.
    fn command(&self, config: &std::path::Path, zone: &str) -> Command {
        let root = self.home.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_tisty"));
        command
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", config.join("config"))
            .env("TISTY_CACHE", config.join("cache"))
            .env("TZ", zone)
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES");
        command
    }

    /// Two configs over one data directory is two devices sharing a store.
    fn as_device(
        &self,
        config: &std::path::Path,
        zone: &str,
        args: &[&str],
        stdin: Option<&str>,
    ) -> Run {
        let mut command = self.command(config, zone);
        command.args(args).stdin(Stdio::piped());

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
fn undo_steps_further_back_instead_of_undoing_itself() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.ok(&["done", "first"]);

    cli.ok(&["undo"]);
    cli.ok(&["undo"]);

    let out = cli.ok(&["ls", "all"]);
    assert!(out.contains("first task"), "{out}");
    assert!(!out.contains("second task"), "{out}");
}

#[test]
fn a_hash_marker_files_the_task_and_creates_the_list() {
    let cli = Cli::new();
    cli.ok(&["water the plants tomorrow #home"]);

    let lists = cli.ok(&["lists"]);
    assert!(lists.contains("home"), "{lists}");

    let out = cli.ok(&["ls", "all"]);
    assert!(
        !out.contains("water the plants tomorrow"),
        "the marker swallowed the date: {out}"
    );
    assert!(out.contains("water the plants"), "{out}");
    assert!(out.contains("tomorrow"), "{out}");
}

#[test]
fn a_bare_number_after_a_hash_stays_in_the_title() {
    let cli = Cli::new();
    cli.ok(&["review PR #42"]);

    assert!(cli.ok(&["ls", "all"]).contains("#42"));
    assert!(!cli.ok(&["lists"]).contains("42"));
}

#[test]
fn a_command_with_nothing_to_act_on_says_so_instead_of_going_quiet() {
    let cli = Cli::new();
    cli.ok(&["only task"]);
    cli.ok(&["ls"]);
    cli.ok(&["done", "1"]);

    let run = cli.run(&["drop", "1"]);

    assert_eq!(run.code, 4, "{}{}", run.out, run.err);
    assert!(!run.err.is_empty(), "failed without a word: {:?}", run.out);
}

#[test]
fn undo_brings_an_archived_list_back() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "errands"]);
    cli.ok(&["list", "archive", "errands"]);
    assert!(!cli.ok(&["lists"]).contains("errands"));

    cli.ok(&["undo"]);

    assert!(
        cli.ok(&["lists"]).contains("errands"),
        "the list stayed archived"
    );
}

#[test]
fn a_name_another_list_already_uses_is_refused() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "Work"]);

    for args in [
        ["list", "add", "work"].as_slice(),
        ["list", "add", "Work"].as_slice(),
    ] {
        let run = cli.run(args);
        assert_ne!(run.code, 0, "{args:?} was allowed: {}", run.out);
        assert!(run.err.contains("already exists"), "{}", run.err);
    }

    cli.ok(&["list", "add", "Home"]);
    let run = cli.run(&["list", "rename", "Home", "work"]);
    assert_ne!(run.code, 0, "{}", run.out);
}

#[test]
fn undoing_a_capture_takes_the_list_it_created_with_it() {
    let cli = Cli::new();
    cli.ok(&["call the plumber #errands"]);
    assert!(cli.ok(&["lists"]).contains("errands"));

    cli.ok(&["undo"]);

    assert!(
        !cli.ok(&["lists"]).contains("errands"),
        "list left orphaned"
    );
    assert!(!cli.ok(&["ls", "all"]).contains("plumber"));
    cli.ok(&["undo"]);
}

#[test]
fn redo_puts_back_what_the_last_undo_took() {
    let cli = Cli::new();
    cli.ok(&["ship the release"]);
    cli.ok(&["done", "ship the release"]);
    assert!(cli.ok(&["ls", "archive"]).contains("ship the release"));

    cli.ok(&["undo"]);
    assert!(cli.ok(&["ls", "all"]).contains("ship the release"));

    cli.ok(&["redo"]);
    assert!(cli.ok(&["ls", "archive"]).contains("ship the release"));
    assert!(!cli.ok(&["ls", "all"]).contains("ship the release"));
}

#[test]
fn redo_walks_the_same_ladder_as_undo() {
    let cli = Cli::new();
    cli.ok(&["water the plants"]);
    cli.ok(&["set", "water the plants", "--priority", "1"]);
    cli.ok(&["done", "water the plants"]);

    cli.ok(&["undo"]);
    cli.ok(&["undo"]);
    assert!(!cli.ok(&["ls", "all"]).contains("!1"));

    cli.ok(&["redo"]);
    assert!(cli.ok(&["ls", "all"]).contains("!1"));
    cli.ok(&["redo"]);
    assert!(cli.ok(&["ls", "archive"]).contains("water the plants"));
}

/// Undoing a creation erases the task, and erasing is the one thing with no way back.
#[test]
fn redoing_an_undone_creation_is_refused_instead_of_pretending() {
    let cli = Cli::new();
    cli.ok(&["a task"]);
    cli.ok(&["undo"]);

    let run = cli.run(&["redo"]);
    assert_ne!(run.code, 0, "{}", run.out);
    assert!(run.err.contains("cannot be redone"), "{}", run.err);
}

#[test]
fn doing_something_new_empties_the_redo_stack() {
    let cli = Cli::new();
    cli.ok(&["a task"]);
    cli.ok(&["done", "a task"]);
    cli.ok(&["undo"]);

    cli.ok(&["another task"]);

    let out = cli.ok(&["redo"]);
    assert!(out.contains("nothing to redo"), "{out}");
}

#[test]
fn redo_on_a_store_nobody_undid_says_so() {
    let cli = Cli::new();
    cli.ok(&["a task"]);

    let out = cli.ok(&["redo"]);
    assert!(out.contains("nothing to redo"), "{out}");
    assert!(cli.ok(&["ls", "all"]).contains("a task"));
}

#[test]
fn a_deadline_before_the_date_is_flagged_without_being_refused() {
    let cli = Cli::new();

    let captured = cli.run(&[
        "ship it",
        "--date",
        "2026-09-10",
        "--deadline",
        "2026-09-01",
    ]);
    assert_eq!(captured.code, 0, "{}", captured.err);
    assert!(captured.err.contains("deadline"), "{}", captured.err);

    cli.ok(&["plan the trip"]);
    cli.ok(&["set", "plan the trip", "--date", "2026-09-10"]);
    let edited = cli.run(&["set", "plan the trip", "--deadline", "2026-09-01"]);
    assert_eq!(edited.code, 0, "{}", edited.err);
    assert!(edited.err.contains("deadline"), "{}", edited.err);
}

#[test]
fn done_with_a_selector_reports_failure_even_with_nothing_open() {
    let cli = Cli::new();
    let run = cli.run(&["done", "5"]);

    assert_eq!(run.code, 4, "{}{}", run.out, run.err);
}

#[test]
fn an_unterminated_fence_cannot_swallow_the_next_task() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.pipe(&["desc", "first"], Some("notes\n```\nunterminated"));

    let out = cli.ok(&["export", "all", "--markdown"]);

    assert!(out.contains("second task"), "{out}");
    assert_eq!(out.matches("```").count() % 2, 0, "fence left open: {out}");
}

#[test]
fn a_heading_of_the_users_never_outranks_the_documents_own() {
    let cli = Cli::new();
    cli.ok(&["the task"]);
    cli.pipe(&["desc", "the task"], Some("# mine\n\nnot a #tag heading"));

    let out = cli.ok(&["export", "all", "--markdown"]);

    assert!(out.contains("#### mine"), "{out}");
    assert!(out.contains("not a #tag heading"), "{out}");
}

#[test]
fn config_tells_an_unset_key_from_an_unknown_one_and_agrees_on_the_code() {
    let cli = Cli::new();

    let unset = cli.run(&["config", "get", "editor"]);
    assert_eq!(unset.code, 4, "{}{}", unset.out, unset.err);
    assert!(!unset.err.is_empty(), "failed without a word");

    let read = cli.run(&["config", "get", "nope"]);
    let write = cli.run(&["config", "set", "nope", "x"]);
    assert_eq!(read.code, write.code, "same error, different exit codes");
}

/// Listing loads no bodies, so the counts have to stand on their own.
#[test]
fn a_listing_reports_its_counts_without_loading_the_bodies() {
    let cli = Cli::new();
    cli.ok(&["write the report"]);
    cli.ok(&["log", "write the report", "spoke to accounting"]);
    cli.ok(&["log", "write the report", "still waiting"]);
    cli.ok(&["step", "write the report", "add", "collect the figures"]);
    cli.ok(&["step", "write the report", "add", "draft it"]);
    cli.ok(&["step", "write the report", "done", "1"]);

    let listed = cli.ok(&["ls", "all"]);
    assert!(listed.contains("✎2"), "journal count missing: {listed}");
    assert!(listed.contains("1/2"), "step count missing: {listed}");

    let shown = cli.ok(&["show", "write the report"]);
    assert!(shown.contains("spoke to accounting"), "{shown}");
    assert!(shown.contains("collect the figures"), "{shown}");
    assert!(
        cli.ok(&["search", "accounting"])
            .contains("write the report")
    );
}

/// Deleting the cache must never change an answer, only how fast it arrives.
#[test]
fn every_read_says_the_same_with_the_cache_and_without_it() {
    let cli = Cli::new();
    cli.ok(&["write the report tomorrow !1 #work"]);
    cli.ok(&["log", "write the report", "spoke to accounting"]);
    cli.ok(&["step", "write the report", "add", "collect the figures"]);
    cli.ok(&["buy milk"]);
    cli.ok(&["done", "buy milk"]);

    let reads: Vec<&[&str]> = vec![
        &["ls", "all"],
        &["ls", "archive"],
        &["lists"],
        &["show", "write the report"],
        &["search", "accounting"],
        &["export", "all", "--markdown"],
        &["ls", "all", "--json"],
    ];
    let cached: Vec<String> = reads.iter().map(|args| cli.ok(args)).collect();

    std::fs::remove_dir_all(cli.home.path().join("cache")).unwrap();

    for (args, before) in reads.iter().zip(cached) {
        assert_eq!(cli.ok(args), before, "`{}` changed", args.join(" "));
    }
}

#[test]
fn the_body_survives_writing_through_the_cache() {
    let cli = Cli::new();
    cli.ok(&["prepare the handover"]);
    cli.ok(&["desc", "prepare the handover", "the keys are in the safe"]);
    cli.ok(&["log", "prepare the handover", "left a note"]);

    let shown = cli.ok(&["show", "prepare the handover"]);
    assert!(shown.contains("keys are in the safe"), "{shown}");
    assert!(shown.contains("left a note"), "{shown}");

    cli.ok(&["undo"]);
    let after = cli.ok(&["show", "prepare the handover"]);
    assert!(after.contains("keys are in the safe"), "{after}");
    assert!(!after.contains("left a note"), "{after}");
}

#[test]
fn doctor_agrees_with_the_log_when_nothing_is_wrong() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.ok(&["ls", "all"]);

    let out = cli.ok(&["doctor"]);
    assert!(out.contains("agrees"), "{out}");
}

/// The cache is a photograph; the log wins. This is what makes that checkable.
#[test]
fn doctor_catches_a_cache_that_disagrees_and_repair_clears_it() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.ok(&["ls", "all"]);

    let db = cli.home.path().join("cache").join("read.db");
    let touched = std::process::Command::new("sqlite3")
        .arg(&db)
        .arg("DELETE FROM task WHERE rowid = 1")
        .status()
        .is_ok_and(|s| s.success());
    if !touched {
        return;
    }

    let run = cli.run(&["doctor"]);
    assert_ne!(run.code, 0, "{}", run.out);
    assert!(run.out.contains("DISAGREES"), "{}", run.out);

    cli.ok(&["doctor", "--repair"]);
    assert_eq!(cli.run(&["doctor"]).code, 0);
    assert!(cli.ok(&["ls", "all"]).contains("first task"));
}

#[test]
fn a_broken_store_does_not_lock_the_user_out_of_config() {
    let cli = Cli::new();
    cli.ok(&["a task"]);

    let store = cli.home.path().join("data").join("store");
    let device = std::fs::read_dir(&store).unwrap().next().unwrap().unwrap();
    let active = device.path().join("active.tisty");
    let mut text = std::fs::read_to_string(&active).unwrap();
    text.push_str("{\"v\":1,\"ts\":\"2026-0");
    std::fs::write(&active, text).unwrap();

    assert_ne!(
        cli.run(&["ls", "all"]).code,
        0,
        "a broken store must not read as an empty one"
    );
    assert_eq!(cli.run(&["config", "get", "device_id"]).code, 0);
}

#[test]
fn absurd_input_is_refused_and_never_panics() {
    let cli = Cli::new();
    cli.ok(&["a task"]);
    cli.ok(&["ls", "all"]);

    let long = "x".repeat(4096);
    for args in [
        ["done", "99999999999999999999"].as_slice(),
        ["done", long.as_str()].as_slice(),
        ["set", "1", "--priority", "250"].as_slice(),
        ["set", "1", "--date", "2026-02-30"].as_slice(),
        ["set", "1", "--date", "not a date at all"].as_slice(),
        ["step", "1", "done", "999"].as_slice(),
        ["step", "1", "done", "0"].as_slice(),
    ] {
        let run = cli.run(args);
        assert_ne!(run.code, 0, "{args:?} was accepted: {}", run.out);
        assert!(!run.err.contains("panicked"), "{args:?}: {}", run.err);
    }
}

#[test]
fn search_matches_literally_and_does_not_read_regex() {
    let cli = Cli::new();
    cli.ok(&["deploy the API v2"]);

    assert!(cli.ok(&["search", "API v2"]).contains("deploy"));
    assert!(!cli.ok(&["search", ".*"]).contains("deploy"));
    assert!(!cli.ok(&["search", "AP."]).contains("deploy"));
}

#[test]
fn renaming_a_tag_onto_an_existing_one_merges_instead_of_duplicating() {
    let cli = Cli::new();
    cli.ok(&["one job"]);
    cli.ok(&["another job"]);
    cli.ok(&["set", "one job", "--tag", "wip"]);
    cli.ok(&["set", "another job", "--tag", "active"]);

    cli.ok(&["tag", "rename", "wip", "active"]);

    let json = cli.ok(&["ls", "all", "--json"]);
    assert!(!json.contains("\"wip\""), "{json}");
    assert_eq!(json.matches("\"active\"").count(), 2, "{json}");
    assert_eq!(cli.ok(&["tag", "ls"]).matches("active").count(), 1);
}

#[test]
fn erasing_the_same_number_twice_is_refused_rather_than_crashing() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["rm", "2", "--force"]);
    let run = cli.run(&["rm", "2", "--force"]);

    assert_eq!(run.code, 4, "{}{}", run.out, run.err);
    assert!(!run.err.contains("panicked"), "{}", run.err);
    assert!(cli.ok(&["ls", "all"]).contains("first task"));
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

    let out = cli
        .command(cli.home.path(), cli.zone)
        .args(["ls", "all"])
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

/// Kiritimati is already on the next date while UTC has not turned over.
#[test]
fn tomorrow_is_tomorrow_where_the_user_is() {
    for zone in ["Pacific/Kiritimati", "Pacific/Niue", "UTC"] {
        let cli = Cli::in_zone(zone);
        cli.ok(&["thing tomorrow"]);

        let stored = cli.ok(&["ls", "all", "--json"]);
        let stored: serde_json::Value = serde_json::from_str(stored.trim()).unwrap();
        let at = stored[0]["date"]["at"].as_str().unwrap();

        // Naming the zone keeps this independent of what the binary resolves.
        let today = jiff::Timestamp::now()
            .to_zoned(jiff::tz::TimeZone::get(zone).unwrap())
            .date();
        let expected = today.tomorrow().unwrap().strftime("%Y-%m-%d").to_string();

        assert!(
            at.starts_with(&expected),
            "{zone}: stored {at}, wanted {expected}"
        );
    }
}

/// The sync design rests on this: order is `(ts, by)` in UTC.
#[test]
fn two_devices_in_opposite_zones_agree_on_everything() {
    let cli = Cli::in_zone("UTC");
    let east = cli.home.path().join("east");
    let west = cli.home.path().join("west");

    cli.as_device(&east, "Pacific/Kiritimati", &["work from the east"], None);
    cli.as_device(&west, "Pacific/Niue", &["work from the west"], None);
    cli.as_device(&east, "Pacific/Kiritimati", &["more from the east"], None);

    let from_east = cli.as_device(&east, "Pacific/Kiritimati", &["export", "all"], None);
    let from_west = cli.as_device(&west, "Pacific/Niue", &["export", "all"], None);

    assert_eq!(from_east.code, 0, "{}", from_east.err);
    assert_eq!(from_east.out, from_west.out, "the two devices disagree");
}

/// A bare `05:46` in an archived document cannot be placed on a timeline.
#[test]
fn an_exported_journal_entry_says_which_zone_it_was_written_in() {
    let cli = Cli::in_zone("Asia/Kolkata");
    cli.ok(&["investigate the outage"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["log", "1", "the pool never refilled"]);

    let md = cli.ok(&["export", "all", "--markdown"]);
    assert!(md.contains("+05:30"), "{md}");
}

/// «It was 5am when I wrote this» is part of what the entry says.
#[test]
fn a_journal_entry_keeps_the_hour_its_author_wrote_it_at() {
    let cli = Cli::in_zone("UTC");
    let east = cli.home.path().join("east");
    let west = cli.home.path().join("west");

    cli.as_device(&east, "Asia/Kolkata", &["investigate the outage"], None);
    cli.as_device(&east, "Asia/Kolkata", &["ls", "all"], None);
    cli.as_device(
        &east,
        "Asia/Kolkata",
        &["log", "1", "the pool never refilled"],
        None,
    );

    let from_east = cli.as_device(
        &east,
        "Asia/Kolkata",
        &["export", "all", "--markdown"],
        None,
    );
    let from_west = cli.as_device(
        &west,
        "America/Santiago",
        &["export", "all", "--markdown"],
        None,
    );

    assert_eq!(from_west.code, 0, "{}", from_west.err);
    assert!(from_west.out.contains("+05:30"), "{}", from_west.out);
    assert_eq!(
        from_east.out, from_west.out,
        "the archive reads differently depending on who opens it"
    );
}

#[test]
fn filters_combine_and_each_one_narrows_the_result() {
    let cli = Cli::new();
    cli.ok(&["rotate the keys tomorrow @security !1"]);
    cli.ok(&["read the access logs @security"]);
    cli.ok(&["update the runbook @docs"]);

    let out = cli.ok(&["ls", "@security"]);
    assert_eq!(out.matches('○').count(), 2, "{out}");

    let out = cli.ok(&["ls", "@security", "!1"]);
    assert!(out.contains("rotate the keys"), "{out}");
    assert!(!out.contains("access logs"), "{out}");
}

#[test]
fn naming_a_filter_widens_the_scope_past_today() {
    let cli = Cli::new();
    cli.ok(&[
        "add",
        "deal with this much later @slow",
        "--date",
        "2026-12-24",
    ]);

    assert!(!cli.ok(&["ls"]).contains("much later"));
    assert!(cli.ok(&["ls", "@slow"]).contains("much later"));
}

#[test]
fn a_list_is_filtered_by_the_name_the_listing_prints() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "Client Work"]);
    cli.ok(&["draft the proposal"]);
    cli.ok(&["something else"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["mv", "1", "Client Work"]);

    let out = cli.ok(&["ls", "#client-work"]);
    assert!(out.contains("draft the proposal"), "{out}");
    assert!(!out.contains("something else"), "{out}");
}

#[test]
fn two_time_filters_are_refused_rather_than_one_being_dropped() {
    let cli = Cli::new();
    let run = cli.run(&["ls", "today", "tomorrow"]);

    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
}

/// A written year is a decision, never rolled forward.
#[test]
fn a_date_that_already_passed_stays_where_it_was_written() {
    let cli = Cli::new();
    cli.ok(&["pay the hosting invoice 2026-07-30"]);

    let out = cli.ok(&["ls", "all", "--json"]);
    assert!(out.contains("2026-07-30"), "{out}");
    assert!(!out.contains("2027-07-30"), "{out}");
}

#[test]
fn settings_are_read_written_and_validated() {
    let cli = Cli::new();

    cli.ok(&["config", "set", "locale", "es"]);
    assert_eq!(cli.ok(&["config", "get", "locale"]).trim(), "es");

    let run = cli.run(&["config", "set", "locale", "klingon"]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);

    cli.ok(&["config", "unset", "locale"]);
    assert!(cli.ok(&["config"]).contains("device_id"));
}

/// Changing it by hand orphans the directory this machine already wrote to.
#[test]
fn the_device_id_cannot_be_edited() {
    let cli = Cli::new();
    let run = cli.run(&["config", "set", "device_id", "whatever"]);

    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
}

#[test]
fn export_hands_the_data_back_in_both_shapes() {
    let cli = Cli::new();
    cli.ok(&["write the postmortem"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["step", "1", "add", "collect the timeline"]);
    cli.ok(&["log", "1", "the cache never expired"]);

    let json = cli.ok(&["export", "all"]);
    serde_json::from_str::<serde_json::Value>(json.trim()).expect("export is not json");

    let md = cli.ok(&["export", "all", "--markdown"]);
    assert!(md.contains("## [ ] write the postmortem"), "{md}");
    assert!(md.contains("- [ ] collect the timeline"), "{md}");
    assert!(md.contains("the cache never expired"), "{md}");
}

/// «tomorrow» is meaningless in a document read months later.
#[test]
fn an_exported_document_carries_absolute_dates() {
    let cli = Cli::new();
    cli.ok(&["ship it", "--date", "2026-12-24"]);

    let md = cli.ok(&["export", "all", "--markdown"]);
    assert!(md.contains("2026-12-24"), "{md}");
}

#[test]
fn export_takes_the_same_filters_as_listing() {
    let cli = Cli::new();
    cli.ok(&["kept @keep"]);
    cli.ok(&["dropped @other"]);

    let md = cli.ok(&["export", "@keep", "--markdown"]);
    assert!(md.contains("kept"), "{md}");
    assert!(!md.contains("dropped"), "{md}");
}

fn bare_remote() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let ok = Command::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .arg(dir.path())
        .status()
        .unwrap()
        .success();
    assert!(ok, "could not create the remote");
    dir
}

impl Cli {
    /// An identity of its own, or the suite fails wherever git has none set.
    fn joins(&self, remote: &TempDir) {
        self.ok(&["sync", "--setup", remote.path().to_str().unwrap()]);
        for pair in [
            ["user.email", "suite@tisty.test"],
            ["user.name", "suite"],
            ["commit.gpgsign", "false"],
        ] {
            Command::new("git")
                .current_dir(self.home.path().join("data"))
                .args(["config", pair[0], pair[1]])
                .status()
                .unwrap();
        }
    }
}

/// A machine joining a remote that already has history used to fail on the
/// first sync and only work on the retry.
#[test]
fn a_second_machine_joins_an_existing_remote_on_the_first_try() {
    let remote = bare_remote();

    let first = Cli::new();
    first.ok(&["buy bread"]);
    first.joins(&remote);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["call the bank"]);
    second.joins(&remote);

    let run = second.run(&["sync"]);
    assert_eq!(run.code, 0, "the first sync failed: {}", run.err);

    let out = second.ok(&["ls", "all"]);
    assert!(out.contains("buy bread"), "{out}");
    assert!(out.contains("call the bank"), "{out}");
}

#[test]
fn what_one_machine_writes_the_other_reads_back() {
    let remote = bare_remote();

    let first = Cli::new();
    first.joins(&remote);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.joins(&remote);
    second.ok(&["sync"]);
    second.ok(&["call the bank"]);
    second.ok(&["sync"]);

    first.ok(&["sync"]);
    let out = first.ok(&["ls", "all"]);
    assert!(out.contains("call the bank"), "{out}");
    assert!(out.contains("buy bread"), "{out}");
}

#[test]
fn syncing_outside_a_repository_says_so() {
    let cli = Cli::new();
    cli.ok(&["buy bread"]);

    let run = cli.run(&["sync"]);
    assert_ne!(run.code, 0);
    assert!(!run.err.trim().is_empty(), "it failed without saying why");
}
