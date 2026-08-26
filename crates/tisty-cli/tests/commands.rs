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

#[test]
fn a_bare_list_with_only_future_work_says_where_it_is() {
    let cli = Cli::new();
    cli.ok(&["buy lemons tomorrow at 8am"]);

    let out = cli.ok(&["ls"]);

    assert!(out.contains("further on"), "{out}");
    assert!(out.contains("1 task"), "{out}");
    assert!(out.contains("tisty ls week"), "{out}");
    assert!(!out.contains("nothing here"), "{out}");
}

#[test]
fn the_count_is_of_what_is_actually_ahead() {
    let cli = Cli::new();
    cli.ok(&["buy lemons tomorrow"]);
    cli.ok(&["renew the certificate tomorrow"]);

    let out = cli.ok(&["ls"]);

    assert!(out.contains("2 tasks"), "{out}");
}

#[test]
fn asking_for_today_on_purpose_still_gets_the_plain_answer() {
    let cli = Cli::new();
    cli.ok(&["buy lemons tomorrow at 8am"]);

    let out = cli.ok(&["ls", "today"]);

    assert!(out.contains("nothing here"), "{out}");
    assert!(!out.contains("further on"), "{out}");
}

#[test]
fn work_with_no_date_is_listed_rather_than_pointed_at() {
    let cli = Cli::new();
    cli.ok(&["buy lemons tomorrow"]);
    cli.ok(&["book a haircut"]);

    let out = cli.ok(&["ls"]);

    assert!(out.contains("book a haircut"), "{out}");
    assert!(!out.contains("further on"), "{out}");
}

#[test]
fn an_empty_store_is_not_told_about_work_it_does_not_have() {
    let cli = Cli::new();

    let out = cli.ok(&["ls"]);

    assert!(out.contains("nothing here"), "{out}");
    assert!(!out.contains("further on"), "{out}");
}

#[test]
fn what_json_answers_does_not_change() {
    let cli = Cli::new();
    cli.ok(&["buy lemons tomorrow at 8am"]);

    let out = cli.ok(&["ls", "--json"]);

    assert_eq!(out.trim(), "[]", "{out}");
}

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
fn an_at_marker_files_the_task_and_creates_the_list() {
    let cli = Cli::new();
    cli.ok(&["water the plants tomorrow @home"]);

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
fn a_bare_number_after_the_marker_stays_in_the_title() {
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
    assert!(
        cli.ok(&["lists"]).contains("archived"),
        "it was not put away"
    );

    cli.ok(&["undo"]);

    assert!(
        !cli.ok(&["lists"]).contains("archived"),
        "the list stayed archived"
    );
    assert!(cli.ok(&["lists"]).contains("errands"));
}

#[test]
fn a_name_another_list_already_uses_is_refused() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "Garden"]);

    for args in [
        ["list", "add", "garden"].as_slice(),
        ["list", "add", "Garden"].as_slice(),
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
    cli.ok(&["call the plumber @errands"]);
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
    cli.ok(&["set", "water the plants", "--priority", "do"]);
    cli.ok(&["done", "water the plants"]);

    cli.ok(&["undo"]);
    cli.ok(&["undo"]);
    assert!(!cli.ok(&["ls", "all"]).contains("!do"));

    cli.ok(&["redo"]);
    assert!(cli.ok(&["ls", "all"]).contains("!do"));
    cli.ok(&["redo"]);
    assert!(cli.ok(&["ls", "archive"]).contains("water the plants"));
}

#[test]
fn redoing_an_undone_creation_brings_it_back_under_a_fresh_name() {
    let cli = Cli::new();
    cli.ok(&["a task"]);
    cli.ok(&["undo"]);
    assert!(!cli.ok(&["ls", "all"]).contains("a task"));

    let run = cli.run(&["redo"]);

    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        cli.ok(&["ls", "all"]).contains("a task"),
        "undoing by mistake would lose the task for ever"
    );
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

#[test]
fn every_read_says_the_same_with_the_cache_and_without_it() {
    let cli = Cli::new();
    cli.ok(&["write the report tomorrow !do @work"]);
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

#[test]
fn doctor_catches_a_cache_that_disagrees_and_repair_clears_it() {
    let cli = Cli::new();
    cli.ok(&["first task"]);
    cli.ok(&["second task"]);
    cli.ok(&["ls", "all"]);

    let db = cli.home.path().join("cache").join("read.db");
    let taken = rusqlite::Connection::open(&db).unwrap();
    let gone = taken
        .execute("DELETE FROM task WHERE rowid = 1", [])
        .unwrap();
    assert_eq!(gone, 1, "the cache held nothing to take away");
    drop(taken);

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
        ["set", "1", "--priority", "urgent"].as_slice(),
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
    assert_eq!(cli.ok(&["ls", "all"]).matches("#active").count(), 3);

    cli.ok(&["undo"]);
    let out = cli.ok(&["ls", "all"]);

    assert_eq!(out.matches("#wip").count(), 3, "{out}");
    assert_eq!(out.matches("#active").count(), 0, "{out}");
}

#[test]
fn undo_on_an_empty_store_says_so_instead_of_failing() {
    let cli = Cli::new();
    cli.ok(&["undo"]);
    let run = cli.run(&["undo"]);

    assert_eq!(run.code, 0, "{}", run.err);
    assert!(run.out.contains("nothing to undo"), "{}", run.out);
}

#[test]
fn the_lists_a_store_starts_with_are_undone_in_one_step() {
    let cli = Cli::new();
    assert!(cli.ok(&["lists"]).contains("Work"));

    cli.ok(&["undo"]);

    assert!(!cli.ok(&["lists"]).contains("Work"));
}

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

    let later = far_ahead();
    let day = later.day().to_string();
    cli.ok(&["set", "1", "--date", &later.to_string(), "--priority", "do"]);
    let out = cli.ok(&["ls", "all"]);
    assert!(out.contains(&day), "{out}");
    assert!(out.contains("!do"), "{out}");

    cli.ok(&["set", "1", "--no-date"]);
    let out = cli.ok(&["ls", "all"]);
    assert!(!out.contains(&day), "{out}");
    assert!(
        out.contains("!do"),
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
    assert!(cli.ok(&["tag", "ls"]).contains("#security"));

    cli.ok(&["set", "1", "--untag", "ops"]);
    let out = cli.ok(&["tag", "ls"]);
    assert!(out.contains("#security"), "{out}");
    assert!(!out.contains("#ops"), "{out}");
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

    assert!(out.contains("#active"), "{out}");
    assert!(!out.contains("#wip"), "{out}");
    assert!(cli.ok(&["ls", "all"]).matches("#active").count() == 2);
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

#[test]
fn tomorrow_is_tomorrow_where_the_user_is() {
    for zone in ["Pacific/Kiritimati", "Pacific/Niue", "UTC"] {
        let cli = Cli::in_zone(zone);
        cli.ok(&["thing tomorrow"]);

        let stored = cli.ok(&["ls", "all", "--json"]);
        let stored: serde_json::Value = serde_json::from_str(stored.trim()).unwrap();
        let at = stored[0]["date"]["at"].as_str().unwrap();

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

#[test]
fn an_exported_journal_entry_says_which_zone_it_was_written_in() {
    let cli = Cli::in_zone("Asia/Kolkata");
    cli.ok(&["investigate the outage"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["log", "1", "the pool never refilled"]);

    let md = cli.ok(&["export", "all", "--markdown"]);
    assert!(md.contains("+05:30"), "{md}");
}

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
    cli.ok(&["rotate the keys tomorrow #security !do"]);
    cli.ok(&["read the access logs #security"]);
    cli.ok(&["update the runbook #docs"]);

    let out = cli.ok(&["ls", "#security"]);
    assert_eq!(out.matches('○').count(), 2, "{out}");

    let out = cli.ok(&["ls", "#security", "!do"]);
    assert!(out.contains("rotate the keys"), "{out}");
    assert!(!out.contains("access logs"), "{out}");
}

fn far_ahead() -> jiff::civil::Date {
    jiff::Zoned::now()
        .date()
        .checked_add(jiff::Span::new().days(400))
        .unwrap()
}

#[test]
fn naming_a_filter_widens_the_scope_past_today() {
    let cli = Cli::new();
    let later = far_ahead().to_string();
    cli.ok(&["add", "deal with this much later @slow", "--date", &later]);

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

    let out = cli.ok(&["ls", "@client-work"]);
    assert!(out.contains("draft the proposal"), "{out}");
    assert!(!out.contains("something else"), "{out}");
}

#[test]
fn what_the_export_writes_the_capture_reads_back() {
    let cli = Cli::new();
    cli.ok(&["ship the release !delegate"]);

    let out = cli.ok(&["export", "--markdown"]);
    let marker = out
        .lines()
        .find_map(|line| line.split_whitespace().find(|word| word.starts_with('!')))
        .expect("the export names the quadrant");

    let fresh = Cli::new();
    fresh.ok(&[&format!("ship it again {marker}")]);
    let seen = fresh.ok(&["ls", "!delegate"]);
    assert!(seen.contains("ship it again"), "{seen}");
}

#[test]
fn the_terminal_can_take_a_quadrant_back_off_a_task() {
    let cli = Cli::new();
    cli.ok(&["water the plants !do"]);
    assert!(cli.ok(&["ls", "all"]).contains("!do"));

    cli.ok(&["set", "1", "--priority", "unclassified"]);

    assert!(!cli.ok(&["ls", "all"]).contains("!do"));
    assert!(cli.ok(&["ls", "!none"]).contains("water the plants"));
}

#[test]
fn a_filter_takes_the_priority_by_name_too() {
    let cli = Cli::new();
    cli.ok(&["ship the release !do"]);
    cli.ok(&["water the plants"]);

    let out = cli.ok(&["ls", "!do"]);
    assert!(out.contains("ship the release"), "{out}");
    assert!(!out.contains("water the plants"), "{out}");

    let run = cli.run(&["ls", "!nonsense"]);
    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
    assert!(run.err.contains("nonsense"), "{}", run.err);
}

#[test]
fn the_marker_the_listing_prints_is_accepted_wherever_a_list_is_named() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "garden"]);
    cli.ok(&["draft the proposal"]);
    cli.ok(&["ls", "all"]);

    cli.ok(&["mv", "1", "@garden"]);
    assert!(cli.ok(&["ls", "@garden"]).contains("draft the proposal"));
}

#[test]
fn two_time_filters_are_refused_rather_than_one_being_dropped() {
    let cli = Cli::new();
    let run = cli.run(&["ls", "today", "tomorrow"]);

    assert_eq!(run.code, 1, "{}{}", run.out, run.err);
}

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

    let json = cli.ok(&["export"]);
    serde_json::from_str::<serde_json::Value>(json.trim()).expect("export is not json");

    let md = cli.ok(&["export", "all", "--markdown"]);
    assert!(md.contains("## [ ] write the postmortem"), "{md}");
    assert!(md.contains("- [ ] collect the timeline"), "{md}");
    assert!(md.contains("the cache never expired"), "{md}");
}

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
    cli.ok(&["kept #keep"]);
    cli.ok(&["dropped #other"]);

    let md = cli.ok(&["export", "#keep", "--markdown"]);
    assert!(md.contains("kept"), "{md}");
    assert!(!md.contains("dropped"), "{md}");
}

#[test]
fn syncing_without_a_folder_says_which_command_sets_one() {
    let cli = Cli::new();
    let run = cli.run(&["sync"]);

    assert_ne!(run.code, 0);
    assert!(run.err.contains("config set remote"), "{}", run.err);
}

#[test]
fn a_machine_with_nothing_of_its_own_adopts_what_the_folder_holds() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();

    let first = Cli::new();
    first.ok(&["config", "set", "remote", &met]);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["config", "set", "remote", &met]);
    second.ok(&["sync"]);

    let out = second.ok(&["ls", "all"]);
    assert!(out.contains("buy bread"), "{out}");
}

#[test]
fn a_machine_that_already_has_a_history_is_refused_instead_of_merged() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();

    let first = Cli::new();
    first.ok(&["config", "set", "remote", &met]);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["config", "set", "remote", &met]);
    second.ok(&["call the bank"]);

    let asked = second.run(&["sync"]);

    assert_ne!(asked.code, 0, "{}", asked.out);
    let mine = second.ok(&["ls", "all"]);
    assert!(mine.contains("call the bank"), "{mine}");
    assert!(!mine.contains("buy bread"), "it merged anyway: {mine}");

    let theirs = first.ok(&["ls", "all"]);
    assert!(
        !theirs.contains("call the bank"),
        "it pushed anyway: {theirs}"
    );
}

#[test]
fn joining_backs_the_machine_up_and_then_takes_what_the_folder_holds() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();
    let kept = tempfile::tempdir().unwrap();
    let zip = kept.path().join("before-joining.zip");

    let first = Cli::new();
    first.ok(&["config", "set", "remote", &met]);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["config", "set", "remote", &met]);
    second.ok(&["call the bank"]);
    second.ok(&["sync", "--join", &zip.display().to_string()]);

    assert!(zip.exists(), "it emptied the machine without a backup");
    let out = second.ok(&["ls", "all"]);
    assert!(out.contains("buy bread"), "{out}");
    assert!(
        !out.contains("call the bank"),
        "joining kept what it had: {out}"
    );
}

#[test]
fn the_first_machine_to_sync_does_not_shut_the_door_on_the_rest() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();

    let first = Cli::new();
    first.ok(&["config", "set", "remote", &met]);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["config", "set", "remote", &met]);
    second.ok(&["sync"]);
    second.ok(&["call the bank"]);
    second.ok(&["sync"]);
    first.ok(&["sync"]);

    let out = first.ok(&["ls", "all"]);
    assert!(
        out.contains("call the bank"),
        "the first to sync shut the door on the rest: {out}"
    );
}

#[test]
fn a_machine_reaches_the_list_on_its_second_sync_because_taking_comes_before_leaving() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();
    let seat = |at: &str| {
        tisty_core::store::ledger(std::path::Path::new(at).join("store"))
            .map(|said| said.allowed.len())
            .unwrap_or(0)
    };

    let one = Cli::new();
    one.ok(&["config", "set", "remote", &met]);
    one.ok(&["buy bread"]);
    one.ok(&["sync"]);
    assert_eq!(seat(&met), 0, "se dio de alta antes de leer la carpeta");

    one.ok(&["sync"]);
    assert_eq!(seat(&met), 1, "no llego a la lista en la segunda vuelta");
}

#[test]
fn a_document_written_in_both_places_is_named_instead_of_going_quiet() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();
    let body = "# Kit\n\nla introduccion\n\nel cierre\n";

    let one = Cli::new();
    one.ok(&["config", "set", "remote", &met]);
    one.ok(&["buy bread"]);
    one.ok(&["sync"]);

    let here = one.home.path().join("data").join("docs");
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("dev_a-0001.md"), body).unwrap();
    let there = shared.path().join("docs");
    std::fs::create_dir_all(&there).unwrap();
    std::fs::write(there.join("dev_a-0001.md"), body).unwrap();

    let out = one.ok(&["sync"]);

    assert!(
        !out.contains("dev_a-0001"),
        "nombro un documento que nadie discute: {out}"
    );
}

#[test]
fn joining_with_nowhere_to_put_the_backup_keeps_everything() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();

    let first = Cli::new();
    first.ok(&["config", "set", "remote", &met]);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["config", "set", "remote", &met]);
    second.ok(&["call the bank"]);

    let asked = second.run(&["sync", "--join", "Z:/no/such/place/before.zip"]);

    assert_ne!(asked.code, 0);
    let out = second.ok(&["ls", "all"]);
    assert!(out.contains("call the bank"), "it emptied anyway: {out}");
}

#[test]
fn a_folder_that_is_not_there_says_so_instead_of_failing_quietly() {
    let cli = Cli::new();
    cli.ok(&["config", "set", "remote", "Z:/no/such/place"]);

    let run = cli.run(&["sync"]);
    assert_ne!(run.code, 0);
    assert!(!run.err.trim().is_empty(), "{}", run.err);
}

#[test]
fn the_remote_is_remembered_and_can_be_taken_back() {
    let cli = Cli::new();

    assert!(cli.ok(&["config"]).contains("remote"));
    cli.ok(&["config", "set", "remote", "drive:tisty"]);
    assert_eq!(cli.ok(&["config", "get", "remote"]).trim(), "drive:tisty");

    cli.ok(&["config", "unset", "remote"]);
    let run = cli.run(&["config", "get", "remote"]);
    assert_ne!(
        run.code, 0,
        "unset means «only on this machine», not «unanswered»"
    );
}

#[test]
fn an_export_carries_what_the_views_fold_away() {
    let cli = Cli::new();
    cli.ok(&["kept #keep"]);
    cli.ok(&["dropped #keep"]);
    cli.ok(&["ls"]);
    cli.ok(&["drop", "dropped"]);

    let out = cli.ok(&["ls", "archive"]);
    assert!(
        !out.contains("dropped"),
        "the archive shows what you did: {out}"
    );

    let json = cli.ok(&["export"]);
    assert!(json.contains("dropped"), "the export lost it: {json}");
    assert!(json.contains("kept"), "{json}");

    let drawer = cli.ok(&["ls", "folded"]);
    assert!(
        drawer.contains("dropped"),
        "no way out of the drawer: {drawer}"
    );
}

fn shared() -> TempDir {
    tempfile::tempdir().unwrap()
}

impl Cli {
    fn store(&self) -> std::path::PathBuf {
        self.home.path().join("data").join("store")
    }

    fn sends(&self, remote: &TempDir) {
        copy_dirs(&self.store(), remote.path());
    }

    fn receives(&self, remote: &TempDir) {
        copy_dirs(remote.path(), &self.store());
    }
}

fn copy_dirs(from: &std::path::Path, to: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(from) else {
        return;
    };
    for device in entries.filter_map(|e| e.ok()) {
        let target = to.join(device.file_name());
        std::fs::create_dir_all(&target).unwrap();
        for file in std::fs::read_dir(device.path())
            .unwrap()
            .filter_map(|e| e.ok())
        {
            std::fs::copy(file.path(), target.join(file.file_name())).unwrap();
        }
    }
}

#[test]
fn a_second_machine_joins_an_existing_remote_on_the_first_try() {
    let remote = shared();

    let first = Cli::new();
    first.ok(&["buy bread"]);
    first.sends(&remote);

    let second = Cli::new();
    second.ok(&["call the bank"]);
    second.receives(&remote);

    let out = second.ok(&["ls", "all"]);
    assert!(out.contains("buy bread"), "{out}");
    assert!(out.contains("call the bank"), "{out}");
}

#[test]
fn what_one_machine_writes_the_other_reads_back() {
    let remote = shared();

    let first = Cli::new();
    first.ok(&["buy bread"]);
    first.sends(&remote);

    let second = Cli::new();
    second.receives(&remote);
    second.ok(&["call the bank"]);
    second.sends(&remote);

    first.receives(&remote);
    let out = first.ok(&["ls", "all"]);
    assert!(out.contains("call the bank"), "{out}");
    assert!(out.contains("buy bread"), "{out}");
}

#[test]
fn completing_a_repeating_task_leaves_the_next_one_waiting() {
    let cli = Cli::new();
    cli.ok(&["take out the bins every tuesday"]);

    let before = cli.ok(&["ls", "all"]);
    assert_eq!(before.matches("take out the bins").count(), 1, "{before}");

    cli.ok(&["done", "1"]);

    let after = cli.ok(&["ls", "all"]);
    assert_eq!(
        after.matches("take out the bins").count(),
        1,
        "the next one did not arrive, or two did
{after}"
    );
    let archive = cli.ok(&["ls", "archive"]);
    assert!(archive.contains("take out the bins"), "{archive}");
}

#[test]
fn undoing_a_repeat_takes_the_next_one_with_it() {
    let cli = Cli::new();
    cli.ok(&["water the plants every 3 days"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);
    cli.ok(&["undo"]);

    let open = cli.ok(&["ls", "all"]);
    assert_eq!(
        open.matches("water the plants").count(),
        1,
        "undo left a copy behind
{open}"
    );
    let archive = cli.ok(&["ls", "archive"]);
    assert!(!archive.contains("water the plants"), "{archive}");
}

#[test]
fn a_repeat_written_in_spanish_works_the_same() {
    let cli = Cli::new();
    cli.ok(&["config", "set", "locale", "es"]);
    cli.ok(&["sacar la basura cada martes"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);

    let open = cli.ok(&["ls", "all"]);
    assert_eq!(open.matches("sacar la basura").count(), 1, "{open}");
}

#[test]
fn the_detail_says_the_cadence_out_loud() {
    let cli = Cli::new();
    cli.ok(&["water the plants every 3 days"]);
    cli.ok(&["ls", "all"]);

    let out = cli.ok(&["show", "1"]);
    assert!(out.contains("every 3 days"), "{out}");
}

#[test]
fn the_cadence_is_said_in_the_language_in_use() {
    let cli = Cli::new();
    cli.ok(&["config", "set", "locale", "es"]);
    cli.ok(&["pagar el arriendo cada mes"]);
    cli.ok(&["ls", "all"]);

    let out = cli.ok(&["show", "1"]);
    assert!(out.contains("cada mes"), "{out}");
}

#[test]
fn a_task_without_a_cadence_says_nothing_about_one() {
    let cli = Cli::new();
    cli.ok(&["buy bread"]);
    cli.ok(&["ls", "all"]);

    let out = cli.ok(&["show", "1"]);
    assert!(!out.contains("every"), "{out}");
}

#[test]
fn a_cadence_can_be_set_on_a_task_that_had_none() {
    let cli = Cli::new();
    cli.ok(&["take out the bins"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["set", "1", "--repeat", "every 3 days"]);

    assert!(cli.ok(&["show", "1"]).contains("every 3 days"));
}

#[test]
fn a_cadence_can_be_taken_off_again() {
    let cli = Cli::new();
    cli.ok(&["take out the bins every tuesday"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["set", "1", "--no-repeat"]);

    let out = cli.ok(&["show", "1"]);
    assert!(!out.contains("every"), "{out}");
}

#[test]
fn a_cadence_that_is_not_one_is_refused() {
    let cli = Cli::new();
    cli.ok(&["take out the bins"]);
    cli.ok(&["ls", "all"]);

    let run = cli.run(&["set", "1", "--repeat", "blue"]);
    assert_ne!(run.code, 0, "{}", run.out);
}

#[test]
fn undoing_and_redoing_a_completion_keeps_the_series_alive() {
    let cli = Cli::new();
    cli.ok(&["take out the bins every week"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);
    cli.ok(&["undo"]);
    cli.ok(&["redo"]);

    let out = cli.ok(&["ls", "all"]);
    assert!(
        out.contains("take out the bins"),
        "the next occurrence is gone: {out}"
    );
}

#[test]
fn a_chained_redo_of_completions_stops_and_tells_the_truth() {
    let cli = Cli::new();
    cli.ok(&["take out the bins every week"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);
    cli.ok(&["undo"]);
    cli.ok(&["undo"]);
    cli.ok(&["redo"]);

    let run = cli.run(&["redo"]);

    assert!(
        !run.err.contains("erased"),
        "it claimed a live task was gone: {}",
        run.err
    );
    assert!(
        cli.ok(&["ls", "all"]).contains("take out the bins"),
        "the series was lost"
    );
}

#[test]
fn dropping_a_repeating_task_says_the_series_is_over() {
    let cli = Cli::new();
    cli.ok(&["take out the bins every tuesday"]);
    cli.ok(&["ls", "all"]);

    let out = cli.ok(&["drop", "1"]);
    assert!(out.contains("repeat ends here"), "{out}");
    assert!(
        cli.ok(&["ls", "all"]).contains("nothing"),
        "a next one arrived"
    );
}

#[test]
fn dropping_an_ordinary_task_says_nothing_about_repeats() {
    let cli = Cli::new();
    cli.ok(&["buy bread"]);
    cli.ok(&["ls", "all"]);

    let out = cli.ok(&["drop", "1"]);
    assert!(!out.contains("repeat"), "{out}");
}

#[test]
fn undoing_a_tick_does_not_leave_the_series_running_twice() {
    let cli = Cli::new();
    cli.ok(&["take out the bins every tuesday"]);
    cli.ok(&["ls", "all"]);
    cli.ok(&["done", "1"]);

    cli.ok(&["ls", "archive"]);
    cli.ok(&["undone", "1"]);

    let open = cli.ok(&["ls", "all"]);
    assert_eq!(
        open.matches("take out the bins").count(),
        1,
        "two of the series are running
{open}"
    );
}

fn named(cli: &Cli) -> String {
    let at = cli.ok(&["config", "path"]);
    let body = std::fs::read_to_string(at.trim()).unwrap();
    body.lines()
        .find_map(|line| line.strip_prefix("device_id = "))
        .unwrap()
        .trim_matches('"')
        .to_string()
}

fn dropped(cli: &Cli, who: &str) {
    let by = named(cli);
    let at = cli
        .home
        .path()
        .join("data")
        .join("store")
        .join(&by)
        .join("active.tisty");
    let mut body = std::fs::read_to_string(&at).unwrap();
    body.push_str(&format!(
        "{{\"v\":3,\"ts\":\"2026-08-15T15:53:21.5029266Z\",\"by\":\"{by}\",\"op\":\"device.remove\",\"d\":\"{who}\"}}\n"
    ));
    std::fs::write(&at, body).unwrap();
}

#[test]
fn a_removed_machine_comes_back_under_a_new_name_in_one_go() {
    let shared = tempfile::tempdir().unwrap();
    let met = shared.path().display().to_string();
    let kept = tempfile::tempdir().unwrap();
    let zip = kept.path().join("before-coming-back.zip");

    let first = Cli::new();
    first.ok(&["config", "set", "remote", &met]);
    first.ok(&["buy bread"]);
    first.ok(&["sync"]);

    let second = Cli::new();
    second.ok(&["config", "set", "remote", &met]);
    second.ok(&["sync"]);
    second.ok(&["call the bank"]);
    second.ok(&["sync"]);

    dropped(&first, &named(&second));
    first.ok(&["sync"]);

    let back = second.run(&["sync", "--join", &zip.display().to_string()]);
    assert_eq!(back.code, 0, "out={} err={}", back.out, back.err);

    second.ok(&["water the plants"]);
    second.ok(&["sync"]);
    first.ok(&["sync"]);

    let out = first.ok(&["ls", "all"]);
    assert!(
        out.contains("water the plants"),
        "the machine that came back never reached the folder: {out}"
    );
}

#[test]
fn a_list_put_away_can_be_brought_back() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "Oficina"]);
    cli.ok(&["list", "archive", "Oficina"]);

    cli.ok(&["list", "unarchive", "Oficina"]);

    let run = cli.run(&["a task", "--list", "Oficina"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
}

#[test]
fn a_list_put_away_still_shows_up_so_it_can_be_named() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "Oficina"]);
    cli.ok(&["list", "archive", "Oficina"]);

    let said = cli.ok(&["list", "ls"]);

    assert!(
        said.contains("Oficina"),
        "guardada y ademas invisible, no habria forma de recuperarla: {said}"
    );
}

#[test]
fn a_list_put_away_refuses_new_tasks_instead_of_swallowing_them() {
    let cli = Cli::new();
    cli.ok(&["list", "add", "Oficina"]);
    cli.ok(&["list", "archive", "Oficina"]);

    let run = cli.run(&["a task", "--list", "Oficina"]);

    assert_ne!(run.code, 0, "{}", run.out);
    assert!(
        run.err.contains("unarchive"),
        "el mensaje no dice como salir: {}",
        run.err
    );
}

#[test]
fn a_story_reads_in_the_terminal_what_the_task_no_longer_carries() {
    let cli = Cli::new();
    cli.ok(&["ship the release"]);
    cli.ok(&["ls"]);
    cli.ok(&["set", "1", "--deadline", "2026-09-12"]);
    cli.ok(&["set", "1", "--deadline", "2026-09-19"]);
    cli.ok(&["log", "1", "the certificate took nine days to issue"]);
    cli.ok(&["done", "1"]);

    let out = cli.ok(&["story", "1"]);

    assert!(out.contains("born"), "{out}");
    assert!(
        out.contains("moves to"),
        "the second deadline is the point: {out}"
    );
    assert!(out.contains("the certificate took nine days"), "{out}");
    assert!(out.contains("closed"), "{out}");
}

#[test]
fn a_story_is_a_subcommand_and_not_a_task_called_story() {
    let cli = Cli::new();
    cli.ok(&["ship the release"]);
    cli.ok(&["ls"]);
    cli.ok(&["story", "1"]);

    let out = cli.ok(&["ls", "all"]);

    assert!(
        !out.contains("story 1"),
        "an unlisted subcommand falls back to add: {out}"
    );
}
