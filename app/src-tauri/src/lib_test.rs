/// What answers for an attachment is remembered process-wide, so the tests that reach it
/// take turns rather than reading each other's answers.
static ALONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn what_answered_for_its_name_is_remembered_past_this_launch() {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());

    let room = tempfile::tempdir().unwrap();
    let kept = room.path().join("cache").join("vouched.json");
    crate::vouching::vouching_kept_at(kept.clone());

    // The name has to answer for the bytes, or nothing is written down to remember.
    let loose = room.path().join("loose.mp4");
    std::fs::write(&loose, b"lo que pesa").unwrap();
    let (sha256, _) = tisty_core::attach::hashed(&loose).unwrap();
    let shelf = &sha256[..2];
    let leaf = format!("charla-{}.mp4", &sha256[2..10]);
    let at = room.path().join("attachments").join(shelf);
    std::fs::create_dir_all(&at).unwrap();
    let file = at.join(&leaf);
    std::fs::copy(&loose, &file).unwrap();
    let reference = format!("attachments/{shelf}/{leaf}");

    assert_eq!(
        vouching::vouches(&file, &reference),
        Some(true),
        "the name does not answer for it"
    );
    assert!(kept.is_file(), "the answer was not written down");

    let said: std::collections::HashMap<String, bool> =
        serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    let (row, answered) = said.iter().next().expect("one row");
    assert!(*answered, "it was written down as not answering for itself");
    assert!(
        row.starts_with(&file.display().to_string()),
        "the row does not name the file it answered for: {row}"
    );
}

#[test]
fn a_file_icloud_took_away_is_not_read_as_one_that_was_lost() {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());

    let here = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    let reference = "attachments/cd/charla-e5f6a7b8.mp4";
    let shelf = shared.path().join("attachments/cd");
    std::fs::create_dir_all(&shelf).unwrap();
    std::fs::write(shelf.join(".charla-e5f6a7b8.mp4.icloud"), b"a few bytes").unwrap();

    // Off a Mac nothing can be asked back, but it is still told apart from what is gone.
    let told = finding::found_in(reference, here.path(), Some(shared.path()));
    match cfg!(target_os = "macos") {
        true => assert!(matches!(
            told,
            finding::Sought::Coming | finding::Sought::At(_)
        )),
        false => assert!(
            matches!(told, finding::Sought::Away),
            "nobody here to ask iCloud"
        ),
    }
    assert!(matches!(
        finding::found_in(
            "attachments/ab/nope-00000000.txt",
            here.path(),
            Some(shared.path())
        ),
        finding::Sought::No
    ));
}

#[cfg(windows)]
#[test]
fn a_hole_whose_body_cannot_be_read_is_not_accused_of_being_torn() {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    const OFFLINE: u32 = 0x0000_1000;
    const SHARED_WITH_NOBODY: u32 = 0;

    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let here = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    let from = tempfile::tempdir().unwrap();
    let cache = here.path().join("cache").join("vouched.json");
    crate::vouching::vouching_kept_at(cache.clone());

    let loose = from.path().join("charla.mp4");
    std::fs::write(&loose, b"lo grabado").unwrap();
    let reference =
        tisty_core::attach::keep(&loose, shared.path(), tisty_core::attach::COPIED_UP_TO)
            .unwrap()
            .at;
    let at = shared.path().join(&reference);
    let body = std::fs::read(&at).unwrap();

    std::fs::remove_file(&at).unwrap();
    let mut marked = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .attributes(OFFLINE)
        .open(&at)
        .unwrap();
    marked.write_all(&body).unwrap();
    drop(marked);

    let shut = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARED_WITH_NOBODY)
        .open(&at)
        .unwrap();

    let told = finding::found_in(&reference, here.path(), Some(shared.path()));
    drop(shut);

    let said = std::fs::read_to_string(&cache).unwrap_or_default();
    assert!(
        !said.contains("false"),
        "a keeper that would not open it was written down as corruption: {said}"
    );
    assert!(
        matches!(told, finding::Sought::Held),
        "a body nobody could read is not a body that lied"
    );
}

#[test]
fn an_attachment_is_looked_for_here_first_and_then_where_it_is_shared() {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let here = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    let from = tempfile::tempdir().unwrap();
    let kept = |root: &std::path::Path, called: &str, body: &[u8]| {
        let at = from.path().join(called);
        std::fs::write(&at, body).unwrap();
        tisty_core::attach::keep(&at, root, tisty_core::attach::COPIED_UP_TO)
            .unwrap()
            .at
    };
    let mine = kept(here.path(), "nota.txt", b"lo apuntado");
    let theirs = kept(shared.path(), "charla.mp4", b"lo grabado");
    let mine = mine.as_str();
    let theirs = theirs.as_str();

    assert!(matches!(
        finding::found_in(mine, here.path(), Some(shared.path())),
        finding::Sought::At(_)
    ));
    assert!(
        matches!(
            finding::found_in(theirs, here.path(), Some(shared.path())),
            finding::Sought::At(_)
        ),
        "what only the shared folder holds is still reachable"
    );
    assert!(
        matches!(
            finding::found_in(theirs, here.path(), None),
            finding::Sought::No
        ),
        "without a shared folder there is nowhere else to look"
    );
    assert!(matches!(
        finding::found_in("attachments/ab/nope-00000000.txt", here.path(), None),
        finding::Sought::No
    ));
    let lying = shared.path().join(theirs);
    std::fs::write(&lying, b"other bytes entirely").unwrap();
    assert!(
        matches!(
            finding::found_in(theirs, here.path(), Some(shared.path())),
            finding::Sought::Torn
        ),
        "what does not answer for its own name is not handed over"
    );
    assert!(
        matches!(
            finding::found_in("../outside.txt", here.path(), Some(shared.path())),
            finding::Sought::No
        ),
        "the way out is still shut"
    );
}

#[test]
fn a_body_that_changed_underneath_is_the_only_one_held_back() {
    use super::stale;

    assert!(stale(Some("aa"), Some("bb")), "somebody wrote in it");
    assert!(!stale(Some("aa"), Some("aa")), "it is as it was read");
    assert!(!stale(None, Some("bb")), "this window never read it");
    assert!(!stale(Some("aa"), None), "it is not there to compare");
}

#[test]
fn a_folder_name_stops_where_the_agent_and_the_core_stop() {
    use crate::answers::shelves::named_folder;
    let most = tisty_core::model::FOLDER_NAME_AT_MOST;

    assert_eq!(named_folder("  Condominio  ").unwrap(), "Condominio");
    assert_eq!(
        named_folder(&"á".repeat(most)).unwrap().chars().count(),
        most
    );
    assert_eq!(
        named_folder(&"a".repeat(most + 1)).unwrap_err().code,
        "folderNameTooLong"
    );
    assert_eq!(named_folder("   ").unwrap_err().code, "untitled");
}

#[test]
fn a_view_can_ask_for_several_lists_at_once() {
    let a = ulid::Ulid::generate();
    let b = ulid::Ulid::generate();
    let view = View {
        lists: vec![a.to_string(), b.to_string()],
        ..bare()
    };

    assert_eq!(view.resolve().unwrap().lists, vec![a, b]);
}

#[test]
fn the_list_being_read_joins_the_ones_being_filtered() {
    let open = ulid::Ulid::generate();
    let also = ulid::Ulid::generate();
    let view = View {
        list: Some(open.to_string()),
        lists: vec![also.to_string()],
        ..bare()
    };

    assert_eq!(view.resolve().unwrap().lists, vec![open, also]);
}

#[test]
fn a_list_that_is_not_an_id_is_refused_rather_than_ignored() {
    let view = View {
        lists: vec!["not an id".into()],
        ..bare()
    };

    assert!(view.resolve().is_err());
}

fn bare() -> View {
    View {
        archive: false,
        everything: false,
        inbox: false,
        list: None,
        lists: Vec::new(),
        most: None,
        board: false,
        spread: false,
        soonest: false,
        tags: Vec::new(),
        tagged: false,
        hidden: false,
        window: None,
        repeating: false,
        reading: None,
    }
}

#[test]
fn the_folder_the_person_chose_for_syncing_can_be_opened() {
    let shared = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();

    assert!(within(
        shared.path(),
        &[data.path().to_path_buf(), shared.path().to_path_buf()]
    ));
}

#[test]
fn nothing_the_screen_names_reaches_a_folder_we_were_not_given() {
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();

    assert!(!within(elsewhere.path(), &[shared.path().to_path_buf()]));
}

#[test]
fn a_neighbour_whose_name_merely_begins_the_same_is_not_inside() {
    let dir = tempfile::tempdir().unwrap();
    let ours = dir.path().join("drive");
    let theirs = dir.path().join("drive-private");
    std::fs::create_dir_all(&ours).unwrap();
    std::fs::create_dir_all(&theirs).unwrap();

    assert!(!within(&theirs, &[ours]));
}

#[test]
fn the_other_version_lands_in_the_same_folder_as_the_one_it_came_from() {
    let folder = ulid::Ulid::generate();

    let (where_at, _, order) = placed(Some((Some(folder), None, "a0".into())), "dev_a-0009");

    assert_eq!(where_at, Some(folder));
    assert!(order.as_str() > "a0", "no quedo despues del original");
}

#[test]
fn the_other_version_stays_loose_only_when_the_original_is_loose() {
    let (where_at, _, order) = placed(Some((None, None, "a0".into())), "dev_a-0009");

    assert_eq!(where_at, None);
    assert!(order.as_str() > "a0");
}

#[test]
fn an_original_nobody_can_find_does_not_stop_the_other_version_from_landing() {
    let (where_at, _, order) = placed(None, "dev_a-0009");

    assert_eq!(where_at, None);
    assert_eq!(order, "dev_a-0009");
}

#[test]
fn the_other_version_of_a_page_is_a_page_of_the_same_document() {
    let folder = ulid::Ulid::generate();
    let up = ulid::Ulid::generate();

    let (where_at, page_of, _) = placed(Some((Some(folder), Some(up), "a0".into())), "dev_a-0009");

    assert_eq!(
        page_of,
        Some(up),
        "it came back as a document beside the book"
    );
    assert_eq!(where_at, Some(folder));
}
use super::*;
use crate::summing::{AHEAD, BEADS, horizon};

#[test]
fn only_paths_inside_the_store_can_be_shown() {
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    std::fs::create_dir_all(data.join("attachments")).unwrap();
    let mine = data.join("attachments/kept.pdf");
    std::fs::write(&mine, b"x").unwrap();

    let outside = home.path().join("id_rsa");
    std::fs::write(&outside, b"x").unwrap();

    let ours = |at: &std::path::Path| {
        let real = at.canonicalize().unwrap();
        real.starts_with(data.canonicalize().unwrap())
    };

    assert!(ours(&mine));
    assert!(!ours(&outside), "a path outside the store was shown");
}

#[test]
fn nothing_else_in_the_project_is_allowed_to_be_unsafe() {
    fn rust(at: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(at) else {
            return;
        };
        for one in entries.filter_map(|e| e.ok()) {
            let path = one.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                rust(&path, found);
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository");
    let mut files = Vec::new();
    rust(&root.join("crates"), &mut files);
    rust(&root.join("app/src-tauri/src"), &mut files);

    let mut allowed: Vec<String> = files
        .iter()
        .filter(|at| {
            std::fs::read_to_string(at)
                .map(|body| {
                    body.lines()
                        .any(|one| one.trim_start().starts_with("#[allow(unsafe_code)]"))
                })
                .unwrap_or(false)
        })
        .map(|at| at.display().to_string())
        .collect();
    allowed.sort();

    let audited = ["src-tauri/src/desktop.rs", "src-tauri/src/shop.rs"];
    assert_eq!(
        allowed.len(),
        audited.len(),
        "unsafe is allowed outside the audited places: {allowed:?}"
    );
    for (mine, is) in allowed.iter().zip(audited) {
        assert!(std::path::Path::new(mine).ends_with(is), "{allowed:?}");
    }
}

fn now() -> jiff::Zoned {
    "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
}

fn held(title: &str) -> tisty_core::model::Task {
    tisty_core::model::Task::new(ulid::Ulid::generate(), title, "a0")
}

fn away(from: jiff::civil::Date, days: i64) -> jiff::civil::Date {
    from.checked_add(jiff::Span::new().try_days(days).unwrap())
        .unwrap()
}

fn kept(state: &mut State, task: tisty_core::model::Task) {
    state.tasks.insert(task.id, task);
}

#[test]
fn what_comes_reaches_a_week_and_stops() {
    let from = today();
    let mut state = State::default();
    for days in [1_i64, 7, 8] {
        let mut task = held("somewhere ahead");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, days),
            "America/Santiago",
        ));
        kept(&mut state, task);
    }

    assert_eq!(coming(&state, from).len(), 2, "the eighth day is outside");
}

#[test]
fn today_keeps_its_own_place_and_is_not_ahead() {
    let from = today();
    let mut state = State::default();
    let mut task = held("call the bank");
    task.date = Some(tisty_core::model::DateSpec::all_day(
        from,
        "America/Santiago",
    ));
    kept(&mut state, task);

    assert!(coming(&state, from).is_empty());
}

#[test]
fn what_only_falls_due_still_comes() {
    let from = today();
    let mut state = State::default();
    let mut task = held("hand in the report");
    task.deadline = Some(tisty_core::model::DateSpec::all_day(
        away(from, 2),
        "America/Santiago",
    ));
    kept(&mut state, task);

    let out = coming(&state, from);

    assert_eq!(out.len(), 1);
    assert_eq!(out[0].on, away(from, 2));
    assert!(out[0].due);
}

#[test]
fn a_deadline_still_counts_when_the_work_was_meant_for_another_day() {
    let from = today();
    let mut state = State::default();
    let mut task = held("finish the report");
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, -1),
        "America/Santiago",
    ));
    task.deadline = Some(tisty_core::model::DateSpec::all_day(
        away(from, 2),
        "America/Santiago",
    ));
    kept(&mut state, task);

    let out = coming(&state, from);

    assert_eq!(out.len(), 1, "the day it was meant for is behind us");
    assert_eq!(out[0].on, away(from, 2));
    assert!(out[0].due);
}

#[test]
fn one_day_that_is_both_is_said_once() {
    let from = today();
    let mut state = State::default();
    let mut task = held("the interview");
    let on = tisty_core::model::DateSpec::all_day(away(from, 1), "America/Santiago");
    task.date = Some(on.clone());
    task.deadline = Some(on);
    kept(&mut state, task);

    let out = coming(&state, from);

    assert_eq!(
        out.len(),
        1,
        "working on it and owing it is one day, not two"
    );
    assert!(!out[0].due);
}

#[test]
fn a_day_to_work_on_it_and_a_day_it_falls_due_are_both_worth_saying() {
    let from = today();
    let mut state = State::default();
    let mut task = held("finish the report");
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, 1),
        "America/Santiago",
    ));
    task.deadline = Some(tisty_core::model::DateSpec::all_day(
        away(from, 3),
        "America/Santiago",
    ));
    kept(&mut state, task);

    let out = coming(&state, from);

    assert_eq!(out.len(), 2);
    assert!(!out[0].due);
    assert!(out[1].due);
}

fn daily(from: jiff::civil::Date, until: Option<jiff::civil::Date>) -> tisty_core::model::Task {
    let mut task = held("take the pills");
    task.date = Some(tisty_core::model::DateSpec::all_day(
        from,
        "America/Santiago",
    ));
    let mut repeat = tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 1,
        unit: tisty_core::model::Unit::Day,
    });
    repeat.until = until;
    task.repeat = Some(repeat);
    task
}

#[test]
fn a_routine_is_named_once_and_crowds_no_day() {
    let from = today();
    let mut state = State::default();
    kept(&mut state, daily(from, None));

    assert!(
        coming(&state, from).is_empty(),
        "it holds no day of its own"
    );
    assert_eq!(recurring(&state, from).len(), 1);
}

#[test]
fn a_snapshot_carries_only_the_turns_the_strip_draws() {
    let from = today();
    let mut state = State::default();
    let mut last = None;
    for step in 0..12 {
        let mut turn = daily(away(from, -40 + step), None);
        turn.after = last;
        turn.status = tisty_core::model::Status::Done;
        last = Some(turn.id);
        kept(&mut state, turn);
    }
    let mut open = daily(from, None);
    open.after = last;
    let id = open.id;
    kept(&mut state, open);

    let whole = tisty_core::series::series(&state, id).expect("a routine keeps a series");
    let out = recurring(&state, from);
    let told = out[0].series.as_ref().expect("a routine keeps a series");

    assert!(
        whole.turns.len() > BEADS,
        "the chain is longer than the strip draws, or this proves nothing"
    );
    assert_eq!(
        told.turns.len(),
        BEADS,
        "the strip draws {BEADS} beads, so {BEADS} turns cross the bridge"
    );
    assert_eq!(
        told.kept, whole.kept,
        "the counters still see the whole chain"
    );
}

#[test]
fn a_routine_left_unkept_still_falls_on_its_own_weekday() {
    let from = today();
    let mut state = State::default();
    let mut task = daily(from, None);
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, -10),
        "America/Santiago",
    ));
    task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 1,
        unit: tisty_core::model::Unit::Week,
    }));
    kept(&mut state, task);

    let out = recurring(&state, from);

    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].on,
        Some(away(from, 4)),
        "ten days late, its turn is still the weekday it was dealt"
    );
}

#[test]
fn a_monthly_routine_left_unkept_does_not_vanish_from_the_week() {
    let from = today();
    let mut state = State::default();
    let mut task = daily(from, None);
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, -27),
        "America/Santiago",
    ));
    task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 1,
        unit: tisty_core::model::Unit::Month,
    }));
    kept(&mut state, task);

    assert_eq!(
        recurring(&state, from).len(),
        1,
        "its turn falls inside the week ahead, however late it is"
    );
}

#[test]
fn a_routine_falling_once_this_week_says_which_day() {
    let from = today();
    let mut state = State::default();
    let mut task = daily(from, None);
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, 2),
        "America/Santiago",
    ));
    task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 2,
        unit: tisty_core::model::Unit::Month,
    }));
    kept(&mut state, task);

    let out = recurring(&state, from);

    assert_eq!(out.len(), 1, "its own date lands inside the week");
    assert_eq!(out[0].on, Some(away(from, 2)));
    assert!(coming(&state, from).is_empty(), "and it crowds no day");
}

#[test]
fn a_routine_falling_every_day_names_no_day_at_all() {
    let from = today();
    let mut state = State::default();
    kept(&mut state, daily(from, None));

    let out = recurring(&state, from);

    assert_eq!(out.len(), 1);
    assert_eq!(out[0].on, None, "seven turns name no single day");
}

#[test]
fn a_cadence_owing_nothing_this_week_is_no_routine_of_this_week() {
    let from = today();
    let mut state = State::default();
    let mut task = daily(from, None);
    task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 2,
        unit: tisty_core::model::Unit::Month,
    }));
    kept(&mut state, task);

    assert!(recurring(&state, from).is_empty());
}

#[test]
fn a_cadence_that_has_ended_owes_nothing_at_all() {
    let from = today();
    let mut state = State::default();
    kept(&mut state, daily(from, Some(from)));

    assert!(recurring(&state, from).is_empty());
}

#[test]
fn a_cadence_of_zero_still_names_the_single_day_it_falls_on() {
    let from = today();
    let mut state = State::default();
    let mut task = daily(from, None);
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, 2),
        "America/Santiago",
    ));
    task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 0,
        unit: tisty_core::model::Unit::Day,
    }));
    kept(&mut state, task);

    let out = recurring(&state, from);

    assert_eq!(out.len(), 1, "a cadence of zero should still surface once");
    assert_eq!(
        out[0].on,
        Some(away(from, 2)),
        "«every 0 days» stands still on the same date instead of advancing, so the loop \
         pushes that one real day seven more times and the day gets folded away as if the \
         routine crowded the whole week"
    );
}

#[test]
fn a_cadence_of_four_hundred_days_owes_nothing_this_week() {
    let from = today();
    let mut state = State::default();
    let mut task = daily(from, None);
    task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 400,
        unit: tisty_core::model::Unit::Day,
    }));
    kept(&mut state, task);

    assert!(recurring(&state, from).is_empty());
}

#[test]
fn a_horizon_past_the_edge_of_the_calendar_is_none() {
    assert_eq!(horizon(jiff::civil::Date::MAX), None);
}

#[test]
fn nothing_comes_or_recurs_once_the_calendar_runs_out() {
    let from = jiff::civil::Date::MAX;
    let mut state = State::default();

    let mut plain = held("at the edge of time");
    plain.date = Some(tisty_core::model::DateSpec::all_day(from, "UTC"));
    kept(&mut state, plain);

    let mut routine = daily(from, None);
    routine.date = Some(tisty_core::model::DateSpec::all_day(from, "UTC"));
    kept(&mut state, routine);

    assert!(coming(&state, from).is_empty());
    assert!(recurring(&state, from).is_empty());
}

#[test]
fn a_hidden_task_neither_comes_nor_recurs() {
    let from = today();
    let mut state = State::default();
    let mut task = held("folded away");
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, 2),
        "America/Santiago",
    ));
    task.hidden = true;
    kept(&mut state, task);

    let mut routine = daily(from, None);
    routine.hidden = true;
    kept(&mut state, routine);

    assert!(coming(&state, from).is_empty());
    assert!(recurring(&state, from).is_empty());
}

#[test]
fn a_dropped_task_neither_comes_nor_recurs() {
    let from = today();
    let mut state = State::default();
    let mut task = held("let go");
    task.date = Some(tisty_core::model::DateSpec::all_day(
        away(from, 2),
        "America/Santiago",
    ));
    task.status = tisty_core::model::Status::Dropped;
    kept(&mut state, task);

    let mut routine = daily(from, None);
    routine.status = tisty_core::model::Status::Dropped;
    kept(&mut state, routine);

    assert!(coming(&state, from).is_empty());
    assert!(recurring(&state, from).is_empty());
}

#[test]
fn a_deadline_on_the_last_day_of_the_window_still_counts() {
    let from = today();
    let mut state = State::default();
    let mut task = held("submit the form");
    task.deadline = Some(tisty_core::model::DateSpec::all_day(
        away(from, AHEAD),
        "America/Santiago",
    ));
    kept(&mut state, task);

    let out = coming(&state, from);

    assert_eq!(out.len(), 1, "the seventh day still belongs to the window");
    assert_eq!(out[0].on, away(from, AHEAD));
}

#[test]
fn a_capture_inside_a_list_is_filed_by_id() {
    let mut state = State::default();
    let list = ulid::Ulid::generate();
    state.apply(&tisty_core::Event::new(
        tisty_core::event::DeviceId("dev".into()),
        jiff::Timestamp::now(),
        Op::ListAdd {
            id: list,
            d: tisty_core::event::ListAdd {
                name: "unificación de login".into(),
                color: None,
                order: "a0".into(),
            },
        },
    ));

    let mut draft: tisty_core::capture::Draft =
        tisty_nl::parse("revisar el deploy", &now(), "es").into();
    draft.filing = Some(tisty_core::capture::Filing::Kept(list));

    let plan = tisty_core::capture::plan(&state, draft).expect("filed");
    assert!(matches!(plan.ops.first(), Some(Op::TaskAdd { d, .. }) if d.list == Some(list)));
}

#[test]
fn an_accepted_offer_sets_the_date_and_trims_the_title() {
    let text = "revisar el informe del lunes";
    let read = tisty_nl::parse(text, &now(), "es");
    let offer = read.offers.first().cloned().expect("an offer");
    let mut draft: tisty_core::capture::Draft = read.clone().into();
    assert!(draft.date.is_none());

    let edits = answers::tasks::Edits {
        date: Some(offer.date.date().to_string()),
        take_offer: true,
        ..Default::default()
    };
    edits.apply(&mut draft, &now(), "es").unwrap();
    draft.title = edits.retitled(text, &read, "es").expect("a new title");

    assert_eq!(draft.title, "revisar el informe");
    assert_eq!(draft.date.unwrap().date().to_string(), "2026-08-10");
}

#[test]
fn a_removal_leaves_nothing_behind() {
    let mut draft: tisty_core::capture::Draft =
        tisty_nl::parse("comprar pan mañana #casa !hacer", &now(), "es").into();
    assert!(draft.date.is_some());

    answers::tasks::Edits {
        no_date: true,
        no_priority: true,
        no_tags: vec!["casa".to_string()],
        ..Default::default()
    }
    .apply(&mut draft, &now(), "es")
    .unwrap();

    assert!(draft.date.is_none());
    assert!(draft.priority.is_none());
    assert!(draft.tags.is_empty());
}

#[test]
fn an_unmarked_reading_returns_to_the_title() {
    let text = "comprar pan el proximo lunes #casa";
    let read = tisty_nl::parse(text, &now(), "es");
    assert_eq!(read.title, "comprar pan");

    let edits = answers::tasks::Edits {
        no_tags: vec!["casa".to_string()],
        ..Default::default()
    };
    assert_eq!(
        edits.retitled(text, &read, "es").as_deref(),
        Some("comprar pan #casa")
    );
}

#[test]
fn a_refusal_the_window_showed_says_what_it_was_about() {
    use crate::answers::tasks::note_trouble;

    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());

    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::paths::Paths::new(kept.path().join("data"), kept.path().join("config"));
    tisty_core::witness::keeps(tisty_core::witness::file(&paths), false);

    note_trouble("noSuchDoc".into(), Some("ycqcwz50-0007".into()));
    note_trouble("noSuchDoc".into(), None);
    note_trouble("comprar pan".into(), Some("ycqcwz50-0008".into()));

    let seen = tisty_core::witness::recent(&paths, 50);
    let shown: Vec<&String> = seen
        .iter()
        .filter(|line| line.contains("the window showed a refusal"))
        .collect();
    assert_eq!(shown.len(), 2, "{shown:?}");
    assert!(shown[0].contains("ycqcwz50-0007"), "{shown:?}");
    assert!(!shown[1].contains("at="), "{shown:?}");
    assert!(
        !seen.iter().any(|line| line.contains("ycqcwz50-0008")),
        "{seen:?}"
    );
}

#[test]
fn only_a_code_we_ship_is_written_down() {
    assert_eq!(refusal_code("pastDeadline"), Some("pastDeadline"));
    assert_eq!(refusal_code("internalNamed"), Some("internalNamed"));
    assert_eq!(refusal_code("comprar pan"), None);
}

#[test]
fn choosing_a_different_date_leaves_the_title_alone() {
    let text = "comprar pan mañana";
    let read = tisty_nl::parse(text, &now(), "es");
    let edits = answers::tasks::Edits {
        date: Some("2026-08-20".to_string()),
        ..Default::default()
    };
    assert_eq!(edits.retitled(text, &read, "es"), None);
}

fn every_day(until: Option<jiff::civil::Date>) -> Change {
    Change {
        repeat: Some(tisty_core::model::Repeat {
            from: tisty_core::model::From::Due,
            each: tisty_core::model::Cadence {
                every: 1,
                unit: tisty_core::model::Unit::Day,
            },
            until,
        }),
        ..Default::default()
    }
}

#[test]
fn a_series_cannot_be_told_to_have_ended_already() {
    let past = repeated(&every_day(Some(jiff::civil::date(2026, 8, 4))), &now());

    assert!(
        matches!(past, Err(ref why) if why.code == "pastEnd"),
        "{past:?}"
    );
}

#[test]
fn today_is_late_enough_to_end_on() {
    assert!(repeated(&every_day(Some(jiff::civil::date(2026, 8, 5))), &now()).is_ok());
    assert!(repeated(&every_day(Some(jiff::civil::date(2027, 1, 1))), &now()).is_ok());
    assert!(repeated(&every_day(None), &now()).is_ok());
}

#[test]
fn the_guide_travels_inside_the_binary_rather_than_beside_it() {
    assert!(
        answers::settings::GUIDE_ES.starts_with("# "),
        "la guia en espanol no viaja"
    );
    assert!(
        answers::settings::GUIDE_EN.starts_with("# "),
        "la guia en ingles no viaja"
    );
}

#[test]
fn the_guide_carries_pages_of_its_own_to_show_what_a_page_is() {
    for (told, leaves, tongue) in [
        (
            answers::settings::GUIDE_ES,
            answers::settings::GUIDE_PAGES_ES,
            "es",
        ),
        (
            answers::settings::GUIDE_EN,
            answers::settings::GUIDE_PAGES_EN,
            "en",
        ),
    ] {
        assert_eq!(leaves.len(), 2, "the {tongue} guide lost a page");
        for (marker, leaf) in leaves {
            assert!(
                told.contains(&format!("]({marker})")),
                "the {tongue} guide never names {marker}"
            );
            assert!(
                leaf.starts_with("# "),
                "a {tongue} page has no title: {marker}"
            );
            assert!(
                tisty_core::docs::survives(leaf).is_ok(),
                "the {tongue} page {marker} would open read-only: {:?}",
                tisty_core::docs::survives(leaf)
            );
        }
        assert!(
            leaves.iter().any(|(_, one)| one.contains("](rina.jpg)")),
            "no {tongue} page shows the picture"
        );
        assert!(
            leaves.iter().any(|(_, one)| one.contains("```rust")),
            "no {tongue} page shows any code"
        );
    }
}

#[test]
fn every_picture_the_guide_names_is_where_the_bundler_looks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/guide");
    for (told, tongue) in [
        (answers::settings::GUIDE_ES, "es"),
        (answers::settings::GUIDE_EN, "en"),
    ] {
        for shot in answers::settings::PICTURES {
            if !told.contains(&format!("]({shot})")) {
                continue;
            }
            assert!(root.join(tongue).join(shot).is_file(), "falta {shot}");
        }
    }
}

fn counted_one_by_one(state: &State) -> std::collections::BTreeMap<String, usize> {
    let mut counts: std::collections::BTreeMap<String, usize> = crate::summing::asked_about()
        .into_iter()
        .map(|(key, filter)| (key.to_string(), state.matching(&filter, today()).len()))
        .collect();
    counts.insert("routines".to_string(), tisty_core::series::how_many(state));
    counts.insert("tags".to_string(), state.tags().len());
    counts.insert(
        "quadrants".to_string(),
        state
            .matching(&Filter::default(), today())
            .iter()
            .filter(|task| !task.priority.set() && task.repeat.is_none())
            .count(),
    );
    counts.insert(
        "tray".to_string(),
        state
            .matching(&Filter::default(), today())
            .iter()
            .filter(|task| crate::summing::waits_in_the_tray(task))
            .count(),
    );
    for list in state.ordered_lists() {
        counts.insert(list.id.to_string(), state.tasks_in(list.id).count());
    }
    counts
}

#[test]
fn counting_in_one_pass_says_what_counting_one_by_one_said() {
    let from = today();
    let mut state = State::default();
    let list = tisty_core::model::List::new(tisty_core::model::ListId::generate(), "casa", "a0");
    state.lists.insert(list.id, list.clone());

    for n in 0..24 {
        let mut task = held(&format!("una de tantas {n}"));
        if n % 2 == 0 {
            task.date = Some(tisty_core::model::DateSpec::all_day(
                away(from, n % 5 - 2),
                "America/Santiago",
            ));
        }
        if n % 3 == 0 {
            task.list = Some(list.id);
        }
        if n % 4 == 0 {
            task.priority = tisty_core::model::Priority::Do;
        }
        if n % 5 == 0 {
            task.status = tisty_core::model::Status::Done;
        }
        if n % 7 == 0 {
            task.hidden = true;
        }
        if n % 6 == 0 {
            task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
                every: 1,
                unit: tisty_core::model::Unit::Week,
            }));
        }
        kept(&mut state, task);
    }

    assert_eq!(crate::summing::tally(&state), counted_one_by_one(&state));
}

fn a_column_is_as_long_as_the_window_says() -> usize {
    let said = std::fs::read_to_string("../src/views.ts").expect("the window says how far it asks");
    let at = said
        .split("export const A_COLUMN = ")
        .nth(1)
        .expect("and it says it there");
    at.split(';')
        .next()
        .and_then(|one| one.trim().parse().ok())
        .expect("as a number")
}

#[test]
fn what_a_column_carries_stays_a_column_however_many_there_are() {
    let mut state = State::default();
    for n in 0..5_000 {
        let mut task = held(&format!("una de las muchas que hay {n}"));
        task.description = Some("con una nota de las que la gente escribe de verdad".into());
        kept(&mut state, task);
    }

    let asked = View {
        most: Some(a_column_is_as_long_as_the_window_says()),
        ..bare()
    };
    let (column, total) = crate::summing::a_column(&state, Some(asked), today()).unwrap();
    let (whole_of_it, _) = crate::summing::a_column(&state, Some(bare()), today()).unwrap();

    assert_eq!(total, 5_000, "the window is told how many there are");
    assert_eq!(
        column.len(),
        a_column_is_as_long_as_the_window_says(),
        "and is handed a column of them"
    );
    assert_eq!(
        whole_of_it.len(),
        5_000,
        "asking for no window still brings them all"
    );

    let carried = serde_json::to_string(&column).unwrap().len();
    let whole = serde_json::to_string(&whole_of_it).unwrap().len();

    assert!(
        carried <= 80 * 1024,
        "a column of {} tasks weighs {carried} bytes, where 80 KiB is what it takes",
        column.len()
    );
    assert!(
        whole > carried * 10,
        "the whole lot weighs {whole} and the column {carried}: the cap is not doing anything"
    );
}

#[test]
fn a_list_card_is_handed_what_comes_soonest_and_no_more_than_three() {
    let mut state = State::default();
    let list = tisty_core::model::List::new(tisty_core::model::ListId::generate(), "casa", "a0");
    state.lists.insert(list.id, list.clone());
    let elsewhere =
        tisty_core::model::List::new(tisty_core::model::ListId::generate(), "trabajo", "a1");
    state.lists.insert(elsewhere.id, elsewhere.clone());

    let mut undated = held("sin fecha");
    undated.list = Some(list.id);
    kept(&mut state, undated);
    for (n, day) in [(1, 20), (2, 10), (3, 5), (4, 1)] {
        let mut one = held(&format!("la de {n}"));
        one.list = Some(list.id);
        one.date = Some(tisty_core::model::DateSpec::all_day(
            away(today(), day),
            "America/Santiago",
        ));
        kept(&mut state, one);
    }
    let mut lonely = held("la de la otra lista");
    lonely.list = Some(elsewhere.id);
    kept(&mut state, lonely);

    let soonest = crate::summing::soonest_in(&state, today());
    let here: Vec<&str> = soonest[&list.id.to_string()]
        .iter()
        .map(|one| one.title.as_str())
        .collect();

    assert_eq!(
        here,
        ["la de 4", "la de 3", "la de 2"],
        "the nearest day comes first, three of them, and what has no day waits behind"
    );
    assert_eq!(soonest[&elsewhere.id.to_string()].len(), 1);
}

#[test]
fn a_list_of_undated_work_still_names_three_of_it() {
    let mut state = State::default();
    let list = tisty_core::model::List::new(tisty_core::model::ListId::generate(), "casa", "a0");
    state.lists.insert(list.id, list.clone());
    for n in 0..5 {
        let mut one = held(&format!("alguna vez {n}"));
        one.list = Some(list.id);
        one.order = format!("a{n}");
        kept(&mut state, one);
    }

    let soonest = crate::summing::soonest_in(&state, today());

    assert_eq!(soonest[&list.id.to_string()].len(), crate::summing::SOONEST);
}

#[test]
fn a_board_is_handed_a_hand_of_each_quarter_and_not_the_pile() {
    use tisty_core::model::Priority;

    let mut state = State::default();
    for n in 0..900 {
        let mut task = held(&format!("una de tantas {n}"));
        task.priority = match n % 5 {
            0 => Priority::Do,
            1 => Priority::Decide,
            2 => Priority::Delegate,
            3 => Priority::Minor,
            _ => Priority::Unset,
        };
        kept(&mut state, task);
    }

    let (hand, all) = crate::summing::a_board(&state, 20, today());
    let mut by: std::collections::BTreeMap<Priority, usize> = Default::default();
    for task in &hand {
        *by.entry(task.priority).or_default() += 1;
    }

    assert_eq!(by[&Priority::Do], 20);
    assert_eq!(by[&Priority::Decide], 20);
    assert_eq!(by[&Priority::Delegate], 20);
    assert_eq!(by[&Priority::Minor], 20);
    assert_eq!(by[&Priority::Unset], 20, "the unplaced pile is a hand too");
    assert_eq!(hand.len(), 100);

    assert_eq!(all, 900, "what is said is how many there are");
    let (whole, _) = crate::summing::a_board(&state, usize::MAX, today());
    assert_eq!(whole.len(), 900, "asking for no hand still deals them all");
}

#[test]
fn what_repeats_never_waits_in_the_unplaced_pile() {
    let mut state = State::default();
    let mut every_week = held("regar las plantas");
    every_week.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 1,
        unit: tisty_core::model::Unit::Week,
    }));
    kept(&mut state, every_week);
    kept(&mut state, held("una suelta"));

    let (hand, _) = crate::summing::a_board(&state, 10, today());

    assert_eq!(hand.len(), 1);
    assert_eq!(hand[0].title, "una suelta");
}

#[test]
fn the_spread_is_handed_every_day_and_only_a_hand_of_the_tray() {
    let mut state = State::default();
    for n in 0..300 {
        let mut dated = held(&format!("con dia {n}"));
        dated.date = Some(tisty_core::model::DateSpec::all_day(
            today(),
            "America/Santiago",
        ));
        kept(&mut state, dated);
        kept(&mut state, held(&format!("en la bandeja {n}")));
    }

    let (drawn, all) = crate::summing::a_spread(&state, 20, today());

    let waiting = drawn
        .iter()
        .filter(|task| crate::summing::waits_in_the_tray(task))
        .count();
    assert_eq!(waiting, 20, "the tray is a hand");
    assert_eq!(
        drawn.len() - waiting,
        300,
        "every day is drawn, so every day travels"
    );
    assert_eq!(
        all, 600,
        "what is said is how many there are, not how many were dealt"
    );
}

#[test]
fn what_repeats_is_never_dealt_to_the_spread() {
    let mut state = State::default();
    let mut weekly = held("regar las plantas");
    weekly.date = Some(tisty_core::model::DateSpec::all_day(
        today(),
        "America/Santiago",
    ));
    weekly.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 1,
        unit: tisty_core::model::Unit::Week,
    }));
    kept(&mut state, weekly);
    let mut once = held("ir al dentista");
    once.date = Some(tisty_core::model::DateSpec::all_day(
        today(),
        "America/Santiago",
    ));
    kept(&mut state, once);

    let (drawn, _) = crate::summing::a_spread(&state, 10, today());

    assert_eq!(drawn.len(), 1);
    assert_eq!(drawn[0].title, "ir al dentista");
}

#[test]
fn a_routine_nobody_prioritised_is_not_counted_as_waiting_to_be_placed() {
    let mut state = State::default();
    let mut weekly = held("regar las plantas");
    weekly.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
        every: 1,
        unit: tisty_core::model::Unit::Week,
    }));
    kept(&mut state, weekly);

    let counts = crate::summing::tally(&state);

    assert_eq!(counts["quadrants"], 0, "the board would not draw it either");
}

#[test]
fn a_list_card_is_never_handed_what_the_person_folded_away() {
    let mut state = State::default();
    let list = tisty_core::model::List::new(tisty_core::model::ListId::generate(), "casa", "a0");
    state.lists.insert(list.id, list.clone());
    let list = list.id;
    let mut folded = held("lo que escondio");
    folded.list = Some(list);
    folded.hidden = true;
    folded.date = Some(tisty_core::model::DateSpec::all_day(
        today(),
        "America/Santiago",
    ));
    kept(&mut state, folded);
    let mut plain = held("lo que se ve");
    plain.list = Some(list);
    kept(&mut state, plain);

    let soonest = crate::summing::soonest_in(&state, today());

    let named: Vec<&str> = soonest[&list.to_string()]
        .iter()
        .map(|task| task.title.as_str())
        .collect();
    assert_eq!(named, vec!["lo que se ve"]);
}

#[test]
fn a_card_breaks_a_tie_the_way_the_person_ordered_it() {
    let mut state = State::default();
    let list = tisty_core::model::List::new(tisty_core::model::ListId::generate(), "casa", "a0");
    state.lists.insert(list.id, list.clone());
    let list = list.id;
    for title in ["tercera", "primera", "segunda"] {
        let mut task = held(title);
        task.list = Some(list);
        task.date = Some(tisty_core::model::DateSpec::all_day(
            today(),
            "America/Santiago",
        ));
        kept(&mut state, task);
    }
    let by_hand = ["primera", "segunda", "tercera"];
    for (at, title) in by_hand.iter().enumerate() {
        let id = state
            .tasks
            .values()
            .find(|task| task.title == *title)
            .map(|task| task.id)
            .expect("the task is there");
        if let Some(task) = state.tasks.get_mut(&id) {
            task.order = format!("a{at}");
        }
    }

    let soonest = crate::summing::soonest_in(&state, today());

    let named: Vec<&str> = soonest[&list.to_string()]
        .iter()
        .map(|task| task.title.as_str())
        .collect();
    assert_eq!(named, by_hand);
}
