use super::*;
use crate::answers::carried_cloud::{
    a_session, carried_cloud, cloud_that, forgotten, holding_our_store, restored,
};
use crate::answers::choosing::{Standing, choosing};
use tisty_core::config::Sync;

fn folder(at: &str) -> Option<Sync> {
    Some(Sync::folder(at))
}

#[test]
fn a_round_that_stopped_keeps_saying_why_while_the_folder_is_the_same() {
    let why = Refusal::about("syncNewer", "dev_b");

    let stuck = stuck_after(Some(&why), &folder("G:/shared"), &folder("G:/shared"))
        .expect("the stop went unsaid");

    assert_eq!(
        (stuck.code, stuck.name.as_deref()),
        ("syncNewer", Some("dev_b"))
    );
}

#[test]
fn a_round_that_got_through_clears_the_stop() {
    assert!(stuck_after(None, &folder("G:/shared"), &folder("G:/shared")).is_none());
}

#[test]
fn a_stop_from_a_folder_left_while_the_round_ran_is_not_raised() {
    let why = Refusal::of("syncNewer");

    assert!(stuck_after(Some(&why), &folder("G:/shared"), &folder("D:/other")).is_none());
    assert!(stuck_after(Some(&why), &folder("G:/shared"), &Some(Sync::Local)).is_none());
}

#[test]
fn having_no_folder_is_no_stop_to_report() {
    assert!(stuck_after(Some(&Refusal::of("noRemote")), &None, &None).is_none());
}

#[test]
fn a_store_restored_apart_from_its_folder_says_so() {
    let why = Refusal::about("restoredApart", "G:/shared");

    assert!(stuck_after(Some(&why), &folder("G:/shared"), &folder("G:/shared")).is_some());
}

#[test]
fn a_folder_no_round_ever_finished_with_can_be_left_freely() {
    let kept = tempfile::tempdir().unwrap();
    let away = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    session.config.sync = Some(Sync::folder(away.path().to_path_buf()));

    assert!(
        Standing::of(&session)
            .stranded_by_leaving(&Sync::alone())
            .is_none()
    );

    std::fs::create_dir_all(session.paths.cache()).unwrap();
    std::fs::write(
        session.paths.cache().join("carried-to"),
        tisty_core::paths::told_of(away.path()),
    )
    .unwrap();

    let stopped = Standing::of(&session)
        .stranded_by_leaving(&Sync::alone())
        .expect("a folder holding what was let go was left");
    assert_eq!(
        (stopped.code, stopped.name),
        ("sharedAwayToLeave", Some(away.path().display().to_string()))
    );
}

fn later() -> Sync {
    Sync::Unknown(toml::from_str("how = \"nube\"").unwrap())
}

#[test]
fn a_way_a_later_build_chose_is_never_left_from_here() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    session.config.sync = Some(later());
    session.config.holds = Some(tisty_core::config::Holds::Everywhere);

    for toward in [Sync::alone(), Sync::folder("G:/otra")] {
        let stopped = Standing::of(&session)
            .stranded_by_leaving(&toward)
            .expect("a choice a later build made was replaced, whatever the setting says");
        assert_eq!(stopped.code, "syncLaterToLeave");
    }
}

#[test]
fn staying_on_the_way_a_later_build_chose_is_not_leaving_it() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    session.config.sync = Some(later());

    assert!(
        Standing::of(&session)
            .stranded_by_leaving(&later())
            .is_none()
    );
}

#[test]
fn leaving_nothing_is_always_free() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();

    assert!(
        Standing::of(&session)
            .stranded_by_leaving(&Sync::folder("G:/otra"))
            .is_none()
    );
    session.config.sync = Some(Sync::alone());
    assert!(
        Standing::of(&session)
            .stranded_by_leaving(&Sync::folder("G:/otra"))
            .is_none()
    );
}

#[test]
fn a_way_a_newer_build_wrote_after_this_window_opened_is_not_replaced() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    session.config.sync = Some(Sync::folder("G:/compartida"));
    session
        .keep(|c| c.sync = Some(Sync::folder("G:/compartida")))
        .unwrap();
    let mut written = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    written.sync = Some(later());
    written.save(&paths).unwrap();

    let stopped = session
        .keep_unless(choosing(Sync::folder("G:/otra"), true))
        .expect_err("what a newer build wrote was written over");

    assert_eq!(stopped.code, "syncLaterToLeave");
    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(later()));
    assert_eq!(
        session.config.sync,
        Some(later()),
        "the window went on believing in a way that is no longer written"
    );
}

fn written_since_the_window_opened(paths: &tisty_core::Paths, way: Option<Sync>) {
    let mut written = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    written.sync = way;
    written.save(paths).unwrap();
}

#[test]
fn choosing_a_folder_over_a_way_a_newer_build_wrote_meanwhile_is_refused_and_writes_nothing() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    session
        .keep(|c| c.sync = Some(Sync::folder("G:/compartida")))
        .unwrap();
    written_since_the_window_opened(&paths, Some(later()));
    let session = Mutex::new(session);

    let stopped = choose(&session, Some("G:/otra".to_string()))
        .err()
        .expect("replaced");

    assert_eq!(stopped.code, "syncLaterToLeave");
    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(later()));
    assert_eq!(held(&session).config.sync, Some(later()));
}

#[test]
fn a_window_that_still_believes_in_a_later_way_is_not_stuck_on_it() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    session.keep(|c| c.sync = Some(later())).unwrap();
    written_since_the_window_opened(&paths, None);
    let session = Mutex::new(session);

    choose(&session, Some("G:/otra".to_string()))
        .expect("a way that is no longer written refused the change")
        .done();

    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(Sync::folder("G:/otra")));
    assert_eq!(held(&session).config.sync, Some(Sync::folder("G:/otra")));
}

#[test]
fn settings_this_version_cannot_read_are_never_written_over() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    let said = std::fs::read_to_string(paths.config_file()).unwrap();
    let unreadable = format!("holds = \"cloudonly\"\n{said}");
    std::fs::write(paths.config_file(), &unreadable).unwrap();

    let stopped = session
        .keep(|c| c.locale = Some("es".into()))
        .expect_err("settings written by a newer build were overwritten");
    let chosen = choose(&Mutex::new(session), Some("G:/otra".to_string()))
        .err()
        .expect("overwritten");

    assert_eq!(stopped.code, "settingsUnreadable");
    assert_eq!(chosen.code, "settingsUnreadable");
    assert_eq!(
        std::fs::read_to_string(paths.config_file()).unwrap(),
        unreadable
    );
}

#[test]
fn a_change_that_is_not_refused_is_written_onto_what_is_on_disk() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();

    session
        .keep_unless(choosing(Sync::folder("G:/compartida"), true))
        .unwrap();

    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(Sync::folder("G:/compartida")));
}

#[test]
fn a_way_a_later_build_chose_pins_no_stop_on_the_window() {
    let why = Refusal::of("syncLater");

    assert!(stuck_after(Some(&why), &Some(later()), &Some(later())).is_none());
}

#[test]
fn a_command_that_needs_a_carrier_says_why_there_is_none() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();

    assert_eq!(
        session.carrying().err().map(|one| one.code),
        Some("noRemote")
    );
    session.config.sync = Some(later());
    assert_eq!(
        session.carrying().err().map(|one| one.code),
        Some("syncLater")
    );
    session.config.sync = Some(Sync::folder("G:/compartida"));
    assert!(session.carrying().is_ok());
}

#[test]
fn a_cloud_is_left_only_through_a_move_once_this_machine_carried_into_it() {
    let cloud = Sync::cloud(tisty_core::config::CloudAt {
        provider: "google".into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    });
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    let mut joined = tisty_core::Config::load_or_init(&paths).unwrap();
    joined.sync = Some(cloud.clone());
    let refused = choosing(Sync::alone(), true)(&mut joined);
    assert_eq!(refused.map(|one| one.code), Some("cloudToLeave"));
    assert_eq!(joined.sync, Some(cloud.clone()));

    let mut never = joined.clone();
    never.sync = Some(cloud);
    assert!(choosing(Sync::alone(), false)(&mut never).is_none());
    assert_eq!(never.sync, Some(Sync::alone()));
}

#[test]
fn the_histories_that_wait_are_read_from_the_cloud_s_mirror_as_from_a_folder() {
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    let ana = tisty_core::config::CloudAt {
        provider: "google".into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    };

    assert_eq!(session.histories(), None);
    session.config.sync = Some(Sync::folder("G:/compartida"));
    assert_eq!(
        session.histories(),
        Some(std::path::PathBuf::from("G:/compartida"))
    );
    session.config.sync = Some(Sync::cloud(ana.clone()));
    assert_eq!(
        session.histories(),
        Some(tisty_carrier::mirrored_at(
            &paths.cloud_home(&ana.provider, &ana.account)
        ))
    );
}

fn step(stage: &'static str, done: usize, whole: usize) -> Bringing {
    Bringing {
        stage,
        done,
        whole,
        joining: false,
        up: None,
    }
}

fn shown(step: Option<Bringing>) -> Option<(&'static str, usize)> {
    step.map(|one| (one.stage, one.done))
}

fn after(first: std::time::Instant, millis: u64) -> std::time::Instant {
    first + std::time::Duration::from_millis(millis)
}

#[test]
fn the_first_step_of_a_stage_is_said_at_once() {
    let mut pacing = Pacing::default();

    assert_eq!(
        shown(pacing.told(step("log", 1, 9), std::time::Instant::now())),
        Some(("log", 1))
    );
    assert!(pacing.flushed().is_none());
}

#[test]
fn a_step_of_the_same_stage_too_soon_after_the_last_is_held_not_said() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("log", 1, 9), first);

    assert!(pacing.told(step("log", 2, 9), after(first, 249)).is_none());
    assert_eq!(shown(pacing.flushed()), Some(("log", 2)));
    assert!(
        pacing.flushed().is_none(),
        "a held step was said twice on flush"
    );
}

#[test]
fn only_the_latest_held_step_is_said_on_flush() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("log", 1, 9), first);
    pacing.told(step("log", 2, 9), after(first, 10));
    pacing.told(step("log", 3, 9), after(first, 20));

    assert_eq!(shown(pacing.flushed()), Some(("log", 3)));
}

#[test]
fn a_step_of_the_same_stage_a_full_beat_later_is_said() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("log", 1, 9), first);

    assert_eq!(
        shown(pacing.told(step("log", 2, 9), after(first, 250))),
        Some(("log", 2))
    );
}

#[test]
fn a_step_of_another_stage_is_said_however_soon_it_comes() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("log", 1, 9), first);

    assert_eq!(
        shown(pacing.told(step("uploading", 1, 9), after(first, 1))),
        Some(("uploading", 1))
    );
}

#[test]
fn the_last_step_of_a_stage_is_said_however_soon_it_comes() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("papers", 1, 3), first);

    assert_eq!(
        shown(pacing.told(step("papers", 3, 3), after(first, 1))),
        Some(("papers", 3))
    );
}

#[test]
fn a_step_said_clears_the_one_held_before_it() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("log", 1, 9), first);
    pacing.told(step("log", 2, 9), after(first, 10));
    pacing.told(step("log", 3, 9), after(first, 300));

    assert!(
        pacing.flushed().is_none(),
        "a step already passed was said again on flush"
    );
}

#[test]
fn the_beat_counts_from_the_last_step_said_not_the_first() {
    let mut pacing = Pacing::default();
    let first = std::time::Instant::now();
    pacing.told(step("log", 1, 9), first);
    pacing.told(step("log", 2, 9), after(first, 300));

    assert!(pacing.told(step("log", 3, 9), after(first, 400)).is_none());
}

#[test]
fn an_upload_step_tells_the_window_its_part_and_bytes() {
    let sending = Bringing {
        stage: "uploading",
        done: 2,
        whole: 5,
        joining: true,
        up: Some(Up {
            part: 7,
            sent: 1024,
            weighs: 4096,
        }),
    };

    assert_eq!(
        serde_json::to_value(sending).unwrap(),
        serde_json::json!({
            "stage": "uploading",
            "done": 2,
            "whole": 5,
            "joining": true,
            "up": {"part": 7, "sent": 1024, "weighs": 4096},
        })
    );
}

#[test]
fn a_step_that_sends_nothing_carries_no_upload_at_all() {
    let value = serde_json::to_value(step("log", 1, 9)).unwrap();

    assert!(value.get("up").is_none(), "{value}");
}

#[test]
fn a_cloud_closed_by_this_computer_is_told_apart_from_one_closed_by_another() {
    let closed = || Refusal::of("cloudSealing");

    assert_eq!(sealing_here(closed(), None).code, "cloudSealing");
    assert_eq!(sealing_here(closed(), Some(false)).code, "cloudSealingHere");
    assert_eq!(
        sealing_here(closed(), Some(true)).code,
        "cloudSealingStopped"
    );
    assert_eq!(
        sealing_here(Refusal::of("cloudFull"), Some(false)).code,
        "cloudFull"
    );
    let here = Refusal::of("cloudSealingHere");
    assert!(stuck_after(Some(&here), &folder("G:/shared"), &folder("G:/shared")).is_none());
}

const ANSWERED: std::time::Duration = std::time::Duration::from_secs(10);
const NEVER_ASKED: std::time::Duration = std::time::Duration::from_millis(300);

#[test]
fn a_round_of_a_restored_store_asks_whose_store_the_cloud_holds_with_the_session_let_go_of() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (at, asked) = carried_cloud(&session, &std::sync::Arc::new(OneAtATime::idle()));
    restored(&session);

    assert!(setting_out(&session).is_ok());

    let (free, _) = asked
        .recv_timeout(ANSWERED)
        .expect("a restored store never asked whose store the cloud holds");
    assert!(
        free,
        "the window waited on the network behind the session every command needs"
    );
    forgotten(&at);
}

#[test]
fn a_round_of_a_store_never_restored_asks_the_cloud_nothing_before_it_sets_out() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (at, asked) = carried_cloud(&session, &std::sync::Arc::new(OneAtATime::idle()));

    assert!(setting_out(&session).is_ok());

    assert!(asked.recv_timeout(NEVER_ASKED).is_err());
    forgotten(&at);
}

#[test]
fn a_round_never_sets_out_to_the_folder_the_store_went_back_from() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let shared = room.path().join("shared");
    holding_our_store(&session, &shared);
    restored(&session);
    held(&session)
        .keep(|config| config.sync = Some(Sync::folder(&shared)))
        .unwrap();

    let refused = setting_out(&session).err().expect("the round set out");

    assert_eq!(
        (refused.code, refused.name),
        ("restoredApart", Some(shared.display().to_string()))
    );
}

#[test]
fn a_folder_chosen_while_the_cloud_answered_is_asked_before_a_round_sets_out_to_it() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let shared = room.path().join("shared");
    holding_our_store(&session, &shared);
    let (theirs, folder) = (session.clone(), shared.clone());
    let at = cloud_that(&session, move || {
        held(&theirs)
            .keep(|config| config.sync = Some(Sync::folder(&folder)))
            .unwrap();
    });
    restored(&session);

    let refused = setting_out(&session).err().expect("the round set out");

    assert_eq!(
        (refused.code, refused.name),
        ("restoredApart", Some(shared.display().to_string()))
    );
    forgotten(&at);
}

#[test]
fn a_round_sets_out_with_the_carrier_that_answered_though_another_was_built_meanwhile() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let theirs = session.clone();
    let at = cloud_that(&session, move || {
        held(&theirs).reach.taken();
    });
    restored(&session);
    let asked = held(&session).carrying().ok().unwrap();

    let setting = setting_out(&session).ok().unwrap();

    assert!(std::sync::Arc::ptr_eq(&asked, &setting.carrier));
    let rebuilt = held(&session).carrying().ok().unwrap();
    assert!(
        !std::sync::Arc::ptr_eq(&asked, &rebuilt),
        "nothing was rebuilt while the cloud answered"
    );
    forgotten(&at);
}

fn id_of(session: &Session, file: &str) -> tisty_core::model::DocId {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .map(|one| one.id)
        .unwrap()
}

fn long_body(n: usize, edition: usize) -> String {
    let line = "una linea de texto que ocupa lugar en el documento y se repite\n";
    format!(
        "# Documento {n}\n\nedicion {edition}\n\n{}",
        line.repeat(50)
    )
}

fn kept_docs(session: &Mutex<Session>, many: usize) -> Vec<String> {
    let mut session = held(session);
    (0..many)
        .map(|_| {
            crate::answers::papers::doc_made(&mut session, None, None, false)
                .unwrap()
                .id
        })
        .collect()
}

fn rewritten_by_an_agent(session: &Mutex<Session>, files: &[String], edition: usize) {
    let docs = held(session).paths.docs();
    for (n, file) in files.iter().enumerate() {
        tisty_core::docs::write(&docs, file, &long_body(n, edition)).unwrap();
    }
}

fn round_before(session: &Mutex<Session>, files: &[String]) -> (usize, std::time::Duration) {
    let mut session = held(session);
    let started = std::time::Instant::now();
    let docs = session.paths.docs();
    let told = files
        .iter()
        .filter(|file| {
            tisty_core::docs::read(&docs, file).is_ok_and(|body| session.retell(file, &body, None))
        })
        .count();
    (told, started.elapsed())
}

fn round_after(session: &Mutex<Session>, files: &[String]) -> (usize, std::time::Duration) {
    let docs = held(session).paths.docs();
    let read = read_stamped(&docs, files);
    let mut session = held(session);
    let started = std::time::Instant::now();
    let told = retold(&mut session, read);
    (told, started.elapsed())
}

fn worst_wait_while<T>(
    session: &Mutex<Session>,
    probe: &str,
    round: impl FnOnce() -> T,
) -> (T, std::time::Duration) {
    let over = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let waiter = scope.spawn(|| {
            let mut worst = std::time::Duration::ZERO;
            let mut n = 0;
            while !over.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(1));
                let asked = std::time::Instant::now();
                let mut session = held(session);
                worst = worst.max(asked.elapsed());
                n += 1;
                let body = format!("# Sonda\n\nescritura {n}\n");
                crate::answers::papers::written_by_hand(
                    &mut session,
                    probe.to_string(),
                    body,
                    Some(true),
                    None,
                )
                .unwrap();
            }
            worst
        });
        let done = round();
        over.store(true, std::sync::atomic::Ordering::Relaxed);
        (done, waiter.join().unwrap())
    })
}

#[test]
fn a_round_over_many_documents_lets_the_commands_of_the_window_through() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let files = kept_docs(&session, 400);
    let spare = kept_docs(&session, 60);
    let (writable, lockable) = spare.split_at(30);
    let over = std::sync::atomic::AtomicBool::new(false);
    let bound = std::time::Duration::from_secs(4);
    let breath = || std::thread::sleep(std::time::Duration::from_millis(3));

    let slowest = std::thread::scope(|scope| {
        let writer = scope.spawn(|| {
            let mut worst = std::time::Duration::ZERO;
            let mut n = 0;
            while !over.load(std::sync::atomic::Ordering::Relaxed) {
                n += 1;
                for file in writable {
                    let asked = std::time::Instant::now();
                    let mut session = held(&session);
                    worst = worst.max(asked.elapsed());
                    let body = format!(
                        "# Escrito {n}

{}",
                        "palabra ".repeat(300)
                    );
                    crate::answers::papers::written_by_hand(
                        &mut session,
                        file.clone(),
                        body,
                        Some(true),
                        None,
                    )
                    .unwrap();
                    drop(session);
                    breath();
                }
            }
            worst
        });
        let locker = scope.spawn(|| {
            let mut worst = std::time::Duration::ZERO;
            let mut shut = true;
            while !over.load(std::sync::atomic::Ordering::Relaxed) {
                for file in lockable {
                    let asked = std::time::Instant::now();
                    let mut session = held(&session);
                    worst = worst.max(asked.elapsed());
                    let id = id_of(&session, file);
                    let ops = crate::answers::tongues::with_kin(&session.state, id)
                        .map(|id| match shut {
                            true => tisty_core::Op::DocLock { id },
                            false => tisty_core::Op::DocUnlock { id },
                        })
                        .collect();
                    session.commit_all(ops).unwrap();
                    drop(session);
                    breath();
                }
                shut = !shut;
            }
            worst
        });
        let maker = scope.spawn(|| {
            let mut worst = std::time::Duration::ZERO;
            while !over.load(std::sync::atomic::Ordering::Relaxed) {
                let asked = std::time::Instant::now();
                let mut session = held(&session);
                worst = worst.max(asked.elapsed());
                crate::answers::papers::doc_made(&mut session, None, None, false).unwrap();
                drop(session);
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            worst
        });
        for edition in 1..=2 {
            rewritten_by_an_agent(&session, &files, edition);
            let docs = held(&session).paths.docs();
            let read = read_stamped(&docs, &files);
            let mut session = held(&session);
            retold(&mut session, read);
        }
        over.store(true, std::sync::atomic::Ordering::Relaxed);
        [writer.join(), locker.join(), maker.join()].map(|one| one.expect("a command panicked"))
    });

    println!("worst wait for the session during the rounds: {slowest:?}");
    assert!(
        slowest.iter().all(|one| *one < bound),
        "a command waited {slowest:?}"
    );
}

#[test]
fn the_round_holds_the_session_for_less_time_than_reading_every_file_under_it() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let many = 500;
    let files = kept_docs(&session, many);
    let probe = kept_docs(&session, 1).remove(0);

    rewritten_by_an_agent(&session, &files, 1);
    let ((told_before, before), waited_before) =
        worst_wait_while(&session, &probe, || round_before(&session, &files));

    rewritten_by_an_agent(&session, &files, 2);
    let ((told_after, after), waited_after) =
        worst_wait_while(&session, &probe, || round_after(&session, &files));

    println!(
        "news for every file, lock held: before {} ms, after {} ms, N={many}, doc {} bytes; worst wait of a save: before {} ms, after {} ms",
        before.as_millis(),
        after.as_millis(),
        long_body(0, 1).len(),
        waited_before.as_millis(),
        waited_after.as_millis()
    );
    assert_eq!((told_before, told_after), (many, many));
    assert!(after * 5 < before, "{after:?} is not a fifth of {before:?}");

    let ((quiet_before, still_before), _) =
        worst_wait_while(&session, &probe, || round_before(&session, &files));
    let ((quiet_after, still_after), _) =
        worst_wait_while(&session, &probe, || round_after(&session, &files));
    println!(
        "nothing new in any file, lock held: before {} ms, after {} ms, N={many}",
        still_before.as_millis(),
        still_after.as_millis()
    );
    assert_eq!((quiet_before, quiet_after), (0, 0));
}

fn a_book_of_two_pages(session: &Mutex<Session>) -> (String, [String; 2]) {
    let mut session = held(session);
    let book = crate::answers::papers::doc_made(&mut session, None, None, false)
        .unwrap()
        .id;
    let up = id_of(&session, &book);
    let pages = [(); 2].map(|()| {
        crate::answers::papers::doc_made(&mut session, None, Some(up), false)
            .unwrap()
            .id
    });
    let body = book_naming(&pages, false, "base");
    crate::answers::papers::written_by_hand(&mut session, book.clone(), body, Some(true), None)
        .unwrap();
    (book, pages)
}

fn book_naming(pages: &[String; 2], reversed: bool, note: &str) -> String {
    let [first, second] = match reversed {
        true => [&pages[1], &pages[0]],
        false => [&pages[0], &pages[1]],
    };
    format!(
        "# Libro\n\n{note}\n\n{}\n\n{}\n",
        tisty_markdown::refs::card(first, "x"),
        tisty_markdown::refs::card(second, "y")
    )
}

fn order_of_pages(session: &Mutex<Session>, book: &str) -> Vec<String> {
    let session = held(session);
    let up = id_of(&session, book);
    session
        .state
        .pages_of(up)
        .into_iter()
        .map(|one| one.file.clone())
        .collect()
}

fn pause_for_the_clock() {
    std::thread::sleep(std::time::Duration::from_millis(30));
}

#[test]
fn a_book_saved_from_the_window_after_it_was_read_keeps_its_newest_page_order() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (book, pages) = a_book_of_two_pages(&session);
    let docs = held(&session).paths.docs();
    tisty_core::docs::write(&docs, &book, &book_naming(&pages, true, "vieja")).unwrap();
    let read = read_stamped(&docs, std::slice::from_ref(&book));
    pause_for_the_clock();

    let newest = book_naming(&pages, false, "nueva y mas larga que la vieja");
    crate::answers::papers::written_by_hand(
        &mut held(&session),
        book.clone(),
        newest.clone(),
        Some(true),
        None,
    )
    .unwrap();
    let told = retold(&mut held(&session), read);

    assert_eq!(told, 0, "the stale body was told as news");
    assert_eq!(
        order_of_pages(&session, &book),
        pages.to_vec(),
        "the stale page order won"
    );
    assert_eq!(tisty_core::docs::read(&docs, &book).unwrap(), newest);
}

#[test]
fn a_book_written_by_an_agent_after_it_was_read_is_told_by_its_newest_body() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (book, pages) = a_book_of_two_pages(&session);
    let docs = held(&session).paths.docs();
    tisty_core::docs::write(&docs, &book, &book_naming(&pages, true, "vieja")).unwrap();
    let read = read_stamped(&docs, std::slice::from_ref(&book));
    pause_for_the_clock();

    let newest = book_naming(&pages, false, "la del agente, mas larga que la vieja");
    tisty_core::docs::write(&docs, &book, &newest).unwrap();
    let told = retold(&mut held(&session), read);

    assert_eq!(told, 1, "the agent's body was never told");
    assert_eq!(
        order_of_pages(&session, &book),
        pages.to_vec(),
        "the stale page order won"
    );
    let session = held(&session);
    let kept = &session.state.docs[&id_of(&session, &book)];
    assert_eq!(kept.bytes, Some(newest.len() as u64));
    assert_eq!(tisty_core::docs::read(&docs, &book).unwrap(), newest);
}

#[test]
fn a_book_an_agent_wrote_before_the_round_is_told_in_the_order_it_names() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (book, pages) = a_book_of_two_pages(&session);
    let docs = held(&session).paths.docs();
    tisty_core::docs::write(&docs, &book, &book_naming(&pages, true, "del agente")).unwrap();

    let read = read_stamped(&docs, std::slice::from_ref(&book));
    let told = retold(&mut held(&session), read);

    assert_eq!(told, 1);
    let reversed = [pages[1].clone(), pages[0].clone()];
    assert_eq!(order_of_pages(&session, &book), reversed.to_vec());
}

#[test]
fn a_file_that_vanished_after_it_was_read_is_skipped_without_telling_anything() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let files = kept_docs(&session, 2);
    rewritten_by_an_agent(&session, &files, 1);
    let docs = held(&session).paths.docs();
    let read = read_stamped(&docs, &files);
    std::fs::remove_file(tisty_core::docs::resolve(&docs, &files[0]).unwrap()).unwrap();

    assert_eq!(retold(&mut held(&session), read), 1);
}

fn carried_both_ways(setting: &Setting) -> tisty_carrier::Moved {
    let mut quiet = |_: tisty_carrier::Reached| {};
    setting
        .carrier
        .carry(
            &setting.home,
            tisty_carrier::Round {
                way: tisty_carrier::Way::Both,
                alive: &setting.alive,
                holds: setting.holds,
                saying: &mut quiet,
            },
        )
        .unwrap()
}

#[test]
fn a_document_changed_on_both_sides_stays_a_conflict_after_the_round_tells_what_arrived() {
    let home = tempfile::tempdir().unwrap();
    let shared = home.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let session = a_session(home.path());
    held(&session)
        .keep(|config| config.sync = Some(Sync::folder(&shared)))
        .unwrap();
    let file = kept_docs(&session, 1).remove(0);
    let docs = held(&session).paths.docs();
    tisty_core::docs::write(&docs, &file, "# Kit\n\nla introduccion\n\nel cierre\n").unwrap();
    let setting = setting_out(&session).ok().unwrap();
    carried_both_ways(&setting);
    held(&session).retell(&file, "# Kit\n\nla introduccion\n\nel cierre\n", None);

    let ours = "# Kit\n\nla introduccion del mac\n\nel cierre\n";
    let theirs = "# Kit\n\nla introduccion de windows\n\nel cierre\n";
    tisty_core::docs::write(&docs, &file, ours).unwrap();
    tisty_core::docs::write(&shared.join("docs"), &file, theirs).unwrap();

    let done = carried_both_ways(&setting);
    assert_eq!(done.undecided.len(), 1, "{done:?}");
    assert!(done.joined.is_empty(), "{done:?}");

    let asked: Vec<String> = done
        .to_answer()
        .into_iter()
        .chain(done.undecided_ids())
        .collect();
    let read = read_stamped(&docs, &asked);
    retold(&mut held(&session), read);

    assert_eq!(tisty_core::docs::read(&docs, &file).unwrap(), ours);
    assert_eq!(
        tisty_core::docs::read(&shared.join("docs"), &file).unwrap(),
        theirs
    );
    let again = carried_both_ways(&setting);
    assert_eq!(
        again.undecided.len(),
        1,
        "the conflict was cleared: {again:?}"
    );
}

#[test]
fn one_round_with_news_in_many_files_makes_exactly_one_commit() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let files = kept_docs(&session, 50);
    rewritten_by_an_agent(&session, &files, 1);
    let docs = held(&session).paths.docs();
    let read = read_stamped(&docs, &files);
    let mut session = held(&session);
    let before = session.writes();

    let told = retold(&mut session, read);

    assert_eq!(told, 50);
    assert_eq!(session.writes(), before + 1);
}

fn written_at(docs: &std::path::Path, file: &str, body: &str, mtime: std::time::SystemTime) {
    tisty_core::docs::write(docs, file, body).unwrap();
    let at = tisty_core::docs::resolve(docs, file).unwrap();
    std::fs::File::options()
        .write(true)
        .open(at)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
}

#[test]
fn a_file_just_written_has_no_stamp_so_a_same_size_save_in_the_same_tick_is_still_seen() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (book, pages) = a_book_of_two_pages(&session);
    let docs = held(&session).paths.docs();
    let now = std::time::SystemTime::now() + std::time::Duration::from_secs(1);
    written_at(&docs, &book, &book_naming(&pages, true, "misma"), now);
    let read = read_stamped(&docs, std::slice::from_ref(&book));
    assert!(read[0].stamp.is_none(), "a fresh file was stamped");

    let newest = book_naming(&pages, false, "misma");
    assert_eq!(newest.len(), book_naming(&pages, true, "misma").len());
    written_at(&docs, &book, &newest, now);

    assert_eq!(retold(&mut held(&session), read), 1);
    assert_eq!(
        order_of_pages(&session, &book),
        pages.to_vec(),
        "the stale page order won"
    );
}

#[test]
fn a_settled_file_keeps_its_stamp_so_the_body_read_outside_the_lock_is_the_one_told() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (book, pages) = a_book_of_two_pages(&session);
    let docs = held(&session).paths.docs();
    let settled = std::time::SystemTime::now() - std::time::Duration::from_secs(5);
    written_at(&docs, &book, &book_naming(&pages, true, "misma"), settled);
    let read = read_stamped(&docs, std::slice::from_ref(&book));
    assert!(read[0].stamp.is_some(), "a settled file was not stamped");

    written_at(&docs, &book, &book_naming(&pages, false, "misma"), settled);
    retold(&mut held(&session), read);

    let reversed = [pages[1].clone(), pages[0].clone()];
    assert_eq!(order_of_pages(&session, &book), reversed.to_vec());
}

fn window_saving_while<T>(
    session: &Mutex<Session>,
    file: &str,
    round: impl FnOnce() -> T,
) -> (T, std::time::Duration, String) {
    let over = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let window = scope.spawn(|| {
            let mut worst = std::time::Duration::ZERO;
            let mut last = String::new();
            let mut n = 0;
            while !over.load(std::sync::atomic::Ordering::Relaxed) {
                n += 1;
                let asked = std::time::Instant::now();
                let mut session = held(session);
                worst = worst.max(asked.elapsed());
                last = format!("# Ventana\n\nguardado {n}\n");
                crate::answers::papers::written_by_hand(
                    &mut session,
                    file.to_string(),
                    last.clone(),
                    Some(true),
                    None,
                )
                .unwrap();
                drop(session);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            (worst, last)
        });
        let done = round();
        over.store(true, std::sync::atomic::Ordering::Relaxed);
        let (worst, last) = window.join().expect("the window panicked");
        (done, worst, last)
    })
}

fn the_state_tells(session: &Mutex<Session>, file: &str, body: &str) -> bool {
    let session = held(session);
    let kept = &session.state.docs[&id_of(&session, file)];
    kept.bytes == Some(body.len() as u64)
        && kept.print.as_deref() == Some(tisty_core::docs::print_of_text(body).as_str())
}

#[test]
fn a_thousand_documents_changed_from_outside_never_cost_the_window_its_own_save() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let many = 1000;
    let files = kept_docs(&session, many);
    let docs = held(&session).paths.docs();
    let saved = &files[0];

    rewritten_by_an_agent(&session, &files, 1);
    let ((told_before, before), waited_before, last) =
        window_saving_while(&session, saved, || round_before(&session, &files));
    assert_eq!(tisty_core::docs::read(&docs, saved).unwrap(), last);
    assert!(the_state_tells(&session, saved, &last));

    rewritten_by_an_agent(&session, &files, 2);
    let ((told_after, after), waited_after, last) =
        window_saving_while(&session, saved, || round_after(&session, &files));

    println!(
        "N={many} agent writes + a window save loop, lock held: before {} ms, after {} ms; worst wait of the window: before {} ms, after {} ms",
        before.as_millis(),
        after.as_millis(),
        waited_before.as_millis(),
        waited_after.as_millis()
    );
    let all_but_maybe_the_saved = |told: usize| (many - 1..=many).contains(&told);
    assert!(all_but_maybe_the_saved(told_before) && all_but_maybe_the_saved(told_after));
    assert_eq!(
        tisty_core::docs::read(&docs, saved).unwrap(),
        last,
        "the last save of the window was lost"
    );
    assert!(
        the_state_tells(&session, saved, &last),
        "the state tells another body than the one of the window"
    );
    assert!(after * 5 < before, "{after:?} is not a fifth of {before:?}");
}

struct Conflict {
    session: std::sync::Arc<Mutex<Session>>,
    setting: Setting,
    file: String,
    docs: std::path::PathBuf,
    theirs_at: std::path::PathBuf,
    asked: Vec<String>,
}

const THEIRS: &str = "# Kit\n\nla introduccion de windows\n\nel cierre\n";
const FROM_THE_WINDOW: &str = "# Kit\n\nla introduccion escrita en la ventana\n\nel cierre\n";

fn a_conflict(home: &std::path::Path) -> Conflict {
    let shared = home.join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let session = a_session(home);
    held(&session)
        .keep(|config| config.sync = Some(Sync::folder(&shared)))
        .unwrap();
    let file = kept_docs(&session, 1).remove(0);
    let docs = held(&session).paths.docs();
    let base = "# Kit\n\nla introduccion\n\nel cierre\n";
    tisty_core::docs::write(&docs, &file, base).unwrap();
    let setting = setting_out(&session).ok().unwrap();
    carried_both_ways(&setting);
    held(&session).retell(&file, base, None);
    let ours = "# Kit\n\nla introduccion del mac\n\nel cierre\n";
    tisty_core::docs::write(&docs, &file, ours).unwrap();
    let theirs_at = shared.join("docs");
    tisty_core::docs::write(&theirs_at, &file, THEIRS).unwrap();
    let done = carried_both_ways(&setting);
    assert_eq!(done.undecided.len(), 1, "{done:?}");
    let asked = done
        .to_answer()
        .into_iter()
        .chain(done.undecided_ids())
        .collect();
    Conflict {
        session,
        setting,
        file,
        docs,
        theirs_at,
        asked,
    }
}

fn the_body_of_the_window_survives_and_the_conflict_stays(one: &Conflict) {
    assert_eq!(
        tisty_core::docs::read(&one.docs, &one.file).unwrap(),
        FROM_THE_WINDOW,
        "the body of the window was overwritten here"
    );
    assert_eq!(
        tisty_core::docs::read(&one.theirs_at, &one.file).unwrap(),
        THEIRS,
        "the shared folder was overwritten"
    );
    assert!(the_state_tells(&one.session, &one.file, FROM_THE_WINDOW));
    let again = carried_both_ways(&one.setting);
    assert_eq!(
        again.undecided.len(),
        1,
        "the conflict was cleared: {again:?}"
    );
}

fn saved_from_the_window(one: &Conflict) {
    crate::answers::papers::written_by_hand(
        &mut held(&one.session),
        one.file.clone(),
        FROM_THE_WINDOW.to_string(),
        Some(true),
        None,
    )
    .unwrap();
}

#[test]
fn a_conflict_saved_again_in_the_window_between_reading_and_telling_keeps_the_body_of_the_window() {
    let home = tempfile::tempdir().unwrap();
    let one = a_conflict(home.path());
    let read = read_stamped(&one.docs, &one.asked);

    saved_from_the_window(&one);
    let started = std::time::Instant::now();
    retold(&mut held(&one.session), read);
    let after = started.elapsed();

    println!(
        "conflict saved in the window between reading and telling, lock held: after {} ms",
        after.as_millis()
    );
    the_body_of_the_window_survives_and_the_conflict_stays(&one);
}

#[test]
fn a_conflict_saved_in_the_window_during_the_old_round_keeps_the_body_of_the_window() {
    let home = tempfile::tempdir().unwrap();
    let one = a_conflict(home.path());

    let ((_, before), _, _) = window_saving_while(&one.session, &one.file, || {
        round_before(&one.session, &one.asked)
    });
    saved_from_the_window(&one);

    println!(
        "conflict, old round with the window saving, lock held: before {} ms",
        before.as_millis()
    );
    the_body_of_the_window_survives_and_the_conflict_stays(&one);
}

#[test]
fn a_conflict_saved_in_the_window_during_the_new_round_keeps_the_body_of_the_window() {
    let home = tempfile::tempdir().unwrap();
    let one = a_conflict(home.path());

    let ((_, after), waited, _) = window_saving_while(&one.session, &one.file, || {
        round_after(&one.session, &one.asked)
    });
    saved_from_the_window(&one);

    println!(
        "conflict, new round with the window saving, lock held: after {} ms; worst wait of the window {} ms",
        after.as_millis(),
        waited.as_millis()
    );
    the_body_of_the_window_survives_and_the_conflict_stays(&one);
}
