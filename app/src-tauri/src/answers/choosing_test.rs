use super::*;
use crate::answers::carried_cloud::{a_session, holding_our_store, restored};

fn on_disk(session: &Mutex<Session>) -> Option<Sync> {
    let at = held(session).paths.config_file();
    tisty_core::Config::load(&at).unwrap().unwrap().sync
}

fn carried_into(session: &Mutex<Session>, folder: &Path) {
    let mut session = held(session);
    session
        .keep(|config| config.sync = Some(Sync::folder(folder)))
        .unwrap();
    std::fs::create_dir_all(session.paths.cache()).unwrap();
    std::fs::write(
        session.paths.cache().join("carried-to"),
        tisty_core::paths::told_of(folder),
    )
    .unwrap();
}

fn refused(answer: Answer<After>) -> (&'static str, Option<String>) {
    let refusal = answer.err().expect("the change was not refused");
    (refusal.code, refusal.name)
}

#[test]
fn a_folder_is_looked_at_with_the_session_let_go_of() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let folder = room.path().join("shared");
    std::fs::create_dir_all(&folder).unwrap();
    let free = std::cell::Cell::new(None);

    let after = choose_weighing(
        &session,
        Some(folder.display().to_string()),
        |standing, dest| {
            free.set(Some(session.try_lock().is_ok()));
            standing.weighed(dest)
        },
    )
    .unwrap();
    after.done();

    assert_eq!(
        free.get(),
        Some(true),
        "the window waited on a folder behind the session every command needs"
    );
    assert_eq!(on_disk(&session), Some(Sync::folder(folder)));
}

#[test]
fn a_place_chosen_while_a_folder_was_looked_at_is_weighed_before_it_is_left() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let (meanwhile, wanted) = (room.path().join("meanwhile"), room.path().join("wanted"));
    std::fs::create_dir_all(&meanwhile).unwrap();
    let looked = std::cell::Cell::new(0);

    let answer = choose_weighing(
        &session,
        Some(wanted.display().to_string()),
        |standing, dest| {
            let weighed = standing.weighed(dest);
            if looked.replace(looked.get() + 1) == 0 {
                carried_into(&session, &meanwhile);
            }
            weighed
        },
    );

    assert_eq!(
        refused(answer),
        ("sharedAwayToLeave", Some(meanwhile.display().to_string()))
    );
    assert_eq!(looked.get(), 2);
    assert_eq!(on_disk(&session), Some(Sync::folder(meanwhile)));
}

#[test]
fn a_folder_this_store_went_back_from_is_not_chosen() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let folder = room.path().join("shared");
    holding_our_store(&session, &folder);
    restored(&session);

    let answer = choose(&session, Some(format!(" {} ", folder.display())));

    assert_eq!(
        refused(answer),
        ("restoredApart", Some(folder.display().to_string()))
    );
    assert_eq!(on_disk(&session), None);
}

#[test]
fn a_folder_inside_the_store_or_around_it_is_not_chosen() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let data = held(&session).paths.data().to_path_buf();

    for tangled in [data.join("dentro"), data.join(".").join("..")] {
        let named = tangled.display().to_string();
        assert_eq!(
            refused(choose(&session, Some(named.clone()))),
            ("remoteInsideStore", Some(named))
        );
    }
    assert_eq!(on_disk(&session), None);
}

#[test]
fn choosing_no_folder_leaves_this_machine_alone() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));

    choose(&session, Some("  ".into())).unwrap().done();

    assert_eq!(on_disk(&session), Some(Sync::alone()));
}

#[test]
fn nothing_the_window_asks_about_a_folder_runs_on_its_own_thread() {
    let said = std::fs::read_to_string("src/answers/storing.rs").unwrap();
    let orders: Vec<&str> = said.split("#[tauri::command").skip(1).collect();

    for name in ["keepers", "keeper_of", "make_room", "choose_sync"] {
        let order = orders
            .iter()
            .find(|order| order.contains(&format!("pub fn {name}(")))
            .unwrap_or_else(|| panic!("{name} moved elsewhere"));
        assert!(
            order.starts_with("(async)]"),
            "{name} would freeze the window while a folder out of reach answers"
        );
    }
}
