use super::*;
use tisty_core::Op;
use tisty_core::config::Sync;

const ID: &str = "dev_a-0001";
const BASE: &str = "# Roger\n\nsale a las ocho\n";
const MINE: &str = "# Roger\n\nsale a las nueve\n";
const THEIRS: &str = "# Roger\n\nsale a las diez\n";

fn at_odds(room: &std::path::Path) -> Mutex<Session> {
    let shared = room.join("shared");
    let paths = tisty_core::Paths::new(room.join("data"), room.join("config"));
    let mut session = Session::at(paths).unwrap();
    session
        .keep(|config| config.sync = Some(Sync::folder(&shared)))
        .unwrap();
    for (folder, body) in [(session.paths.docs(), MINE), (shared.join("docs"), THEIRS)] {
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(format!("{ID}.md")), body).unwrap();
    }
    tisty_core::docs::keep_carried(session.paths.data(), ID, BASE).unwrap();
    Mutex::new(session)
}

fn here(session: &Mutex<Session>) -> String {
    tisty_core::docs::read(&held(session).paths.docs(), ID).unwrap()
}

fn locked(session: &Mutex<Session>) {
    let id = ulid::Ulid::generate();
    let mut session = held(session);
    session
        .commit(Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                file: ID.into(),
                order: "a".into(),
                ..Default::default()
            },
        })
        .unwrap();
    session.commit(Op::DocLock { id }).unwrap();
}

#[test]
fn a_document_two_machines_wrote_shows_what_each_one_said() {
    let room = tempfile::tempdir().unwrap();
    let session = at_odds(room.path());

    let torn = torn(&session, ID).unwrap();

    assert_eq!(torn.rifts.len(), 1);
    assert_eq!(torn.print, print_of_three(BASE, MINE, THEIRS));
}

#[test]
fn what_the_person_picked_is_written_here() {
    let room = tempfile::tempdir().unwrap();
    let session = at_odds(room.path());
    let print = torn(&session, ID).unwrap().print;

    woven(&session, ID, &["theirs".to_string()], &print).unwrap();

    assert_eq!(here(&session), THEIRS);
}

#[test]
fn a_choice_made_over_words_that_moved_since_writes_nothing() {
    let room = tempfile::tempdir().unwrap();
    let session = at_odds(room.path());

    let refused = woven(&session, ID, &["theirs".to_string()], "de antes").unwrap_err();

    assert_eq!(refused.code, "movedUnderfoot");
    assert_eq!(here(&session), MINE);
}

#[test]
fn a_document_never_carried_has_nothing_to_choose_between() {
    let room = tempfile::tempdir().unwrap();
    let session = at_odds(room.path());
    tisty_core::docs::forget_carried(held(&session).paths.data(), ID);

    let torn = torn(&session, ID).unwrap();
    let refused = woven(&session, ID, &[], "").unwrap_err();

    assert!(torn.rifts.is_empty());
    assert_eq!(refused.code, "noBase");
}

#[test]
fn a_locked_document_is_neither_shown_torn_nor_woven() {
    let room = tempfile::tempdir().unwrap();
    let session = at_odds(room.path());
    locked(&session);

    let shown = torn(&session, ID)
        .err()
        .expect("a locked document was shown");
    let refused = woven(&session, ID, &["theirs".to_string()], "").unwrap_err();

    assert_eq!(shown.code, "documentLocked");
    assert_eq!(refused.code, "documentLocked");
    assert_eq!(here(&session), MINE);
}
