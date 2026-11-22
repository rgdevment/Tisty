use super::*;
use crate::answers::papers::{merged, written_by_hand};
use tisty_core::Op;

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn filed(session: &mut Session, body: &str) -> String {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                sown: false,
                made: None,
                by: None,
                said: None,
                file: made.id.clone(),
                order: made.id.clone(),
                folder: None,
                page_of: None,
                task_of: None,
                edited: None,
            },
        })
        .unwrap();
    made.id
}

fn turn(hand: &str, n: u64) -> Option<Turn> {
    Some(Turn {
        hand: hand.to_string(),
        n,
    })
}

fn print(body: &str) -> String {
    tisty_core::docs::print_of_text(body)
}

fn on_disk(session: &Session, id: &str) -> String {
    tisty_core::docs::read(&session.paths.docs(), id).unwrap()
}

fn saved(
    session: &mut Session,
    id: &str,
    body: &str,
    since: &str,
    anyway: bool,
    by: Option<Turn>,
) -> Answer<()> {
    in_turn(
        session,
        id,
        by,
        |session| {
            written_by_hand(
                session,
                id.to_string(),
                body.to_string(),
                Some(anyway),
                Some(since.to_string()),
            )
        },
        |_| true,
    )
    .map(|_| ())
}

fn woven(session: &mut Session, id: &str, base: &str, mine: &str, by: Option<Turn>) -> bool {
    in_turn(
        session,
        id,
        by,
        |session| merged(session, id.to_string(), base, mine),
        Option::is_some,
    )
    .unwrap()
    .is_some()
}

fn someone_else_writes(session: &Session, id: &str, body: &str) {
    tisty_core::docs::write(&session.paths.docs(), id, body).unwrap();
}

#[test]
fn a_save_that_arrives_after_a_newer_one_lands_nowhere() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("a", 2),
    )
    .unwrap();

    let late = saved(
        &mut session,
        &id,
        "# Compras\n\nleche y\n",
        &print(opened),
        false,
        turn("a", 1),
    );

    assert_eq!(late.unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), "# Compras\n\nleche y pan\n");
}

#[test]
fn a_late_mine_stands_never_writes_over_what_came_after_it() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("a", 2),
    )
    .unwrap();

    let late = saved(
        &mut session,
        &id,
        "# Compras\n\nhuevos\n",
        &print(opened),
        true,
        turn("a", 1),
    );

    assert_eq!(late.unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), "# Compras\n\nleche y pan\n");
}

#[test]
fn a_save_refused_as_moved_still_shuts_out_every_older_one() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    someone_else_writes(&session, &id, "# Compras\n\nleche\n\ncafé\n");
    let moved = saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("a", 2),
    );
    assert_eq!(moved.unwrap_err().code, "documentMoved");

    let late = saved(
        &mut session,
        &id,
        "# Compras\n\nhuevos\n",
        &print(opened),
        true,
        turn("a", 1),
    );

    assert_eq!(late.unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), "# Compras\n\nleche\n\ncafé\n");
}

#[test]
fn refused_as_moved_the_window_learns_which_of_its_saves_the_disk_last_took() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("a", 1),
    )
    .unwrap();

    let moved = saved(
        &mut session,
        &id,
        "# Compras\n\nleche\n",
        &print(opened),
        false,
        turn("a", 2),
    )
    .unwrap_err();

    assert_eq!(moved.code, "documentMoved");
    assert_eq!(moved.landed, Some(1));
}

#[test]
fn a_document_that_took_none_of_the_saves_says_so_when_it_refuses() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    someone_else_writes(&session, &id, "# Compras\n\ncafé\n");

    let moved = saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("a", 1),
    )
    .unwrap_err();

    assert_eq!(moved.code, "documentMoved");
    assert_eq!(moved.landed, None);
}

#[test]
fn the_weave_after_a_refused_save_belongs_to_the_same_turn() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n\npan\n";
    let id = filed(&mut session, opened);
    someone_else_writes(&session, &id, "# Compras\n\nleche\n\npan\n\ncafé\n");
    let mine = "# Lista\n\nleche\n\npan\n";
    assert!(saved(&mut session, &id, mine, &print(opened), false, turn("a", 1)).is_err());

    assert!(woven(&mut session, &id, opened, mine, turn("a", 1)));

    assert_eq!(on_disk(&session, &id), "# Lista\n\nleche\n\npan\n\ncafé\n");
    let again = saved(&mut session, &id, "x", &print(opened), false, turn("a", 2));
    assert_eq!(again.unwrap_err().landed, Some(1));
}

#[test]
fn a_weave_sent_by_a_save_the_window_gave_up_on_lands_nowhere_after_a_newer_one() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n\npan\n";
    let id = filed(&mut session, opened);
    someone_else_writes(&session, &id, "# Compras\n\nleche\n\npan\n\ncafé\n");
    let newer = "# Compras\n\nleche\n\npan\n\ncafé\n\nhuevos\n";
    let theirs = print("# Compras\n\nleche\n\npan\n\ncafé\n");
    saved(&mut session, &id, newer, &theirs, false, turn("a", 2)).unwrap();

    let late = in_turn(
        &mut session,
        &id,
        turn("a", 1),
        |session| merged(session, id.clone(), opened, "# Lista\n\nleche\n\npan\n"),
        Option::is_some,
    );

    assert_eq!(late.map(|_| ()).unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), newer);
}

#[test]
fn opening_again_shuts_out_what_the_page_sent_before_it() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);

    session.turns.opened(&id, &turn("a", 3).unwrap());
    let late = saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        true,
        turn("a", 2),
    );

    assert_eq!(late.unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), opened);
}

#[test]
fn a_page_opened_again_after_a_reload_takes_the_document_over() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    session.turns.opened(&id, &turn("antes", 40).unwrap());

    session.turns.opened(&id, &turn("ahora", 1).unwrap());
    saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("ahora", 2),
    )
    .unwrap();
    let gone = saved(
        &mut session,
        &id,
        "# Compras\n\nhuevos\n",
        &print(opened),
        true,
        turn("antes", 41),
    );

    assert_eq!(gone.unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), "# Compras\n\nleche y pan\n");
}

#[test]
fn a_write_that_is_no_save_of_the_window_goes_through_as_before() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    session.turns.opened(&id, &turn("a", 9).unwrap());

    saved(
        &mut session,
        &id,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        None,
    )
    .unwrap();

    assert_eq!(on_disk(&session, &id), "# Compras\n\nleche y pan\n");
}

#[test]
fn a_step_back_shuts_out_a_save_older_than_it_even_with_nothing_to_go_back_to() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let id = filed(&mut session, opened);
    let then = "# Compras\n\nleche y pan\n";
    saved(&mut session, &id, then, &print(opened), false, turn("a", 1)).unwrap();
    let _ = in_turn(
        &mut session,
        &id,
        turn("a", 3),
        |session| crate::answers::earlier::went_back(session, &id),
        |_| true,
    );

    let late = saved(
        &mut session,
        &id,
        "# Compras\n\nhuevos\n",
        &print(then),
        false,
        turn("a", 2),
    );

    assert_eq!(late.unwrap_err().code, "saveLate");
    assert_eq!(on_disk(&session, &id), then);
}

#[test]
fn a_save_that_reaches_a_deleted_document_never_writes_it_back() {
    let (_tmp, mut session) = session();
    let opened = "# Compras\n\nleche\n";
    let file = filed(&mut session, opened);
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    session.drop_doc(&id.to_string()).unwrap();

    let late = saved(
        &mut session,
        &file,
        "# Compras\n\nleche y pan\n",
        &print(opened),
        false,
        turn("a", 1),
    );

    assert_eq!(late.unwrap_err().code, "noSuchDoc");
    assert!(
        tisty_core::docs::read(&session.paths.docs(), &file).is_err(),
        "the deleted document came back on the disk"
    );
}
