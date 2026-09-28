use super::Session;
use crate::answers::papers::beside_docs;
use crate::answers::shelves::beside_folders;
use tisty_core::{Op, Paths};

struct Desk {
    _tmp: tempfile::TempDir,
    paths: Paths,
}

fn desk() -> Desk {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    Desk { _tmp: tmp, paths }
}

fn folder(session: &mut Session, name: &str, order: &str) -> tisty_core::model::FolderId {
    let id = ulid::Ulid::generate();
    session
        .commit(Op::FolderAdd {
            id,
            d: tisty_core::event::FolderAdd {
                name: name.into(),
                order: order.into(),
                parent: None,
                icon: None,
                color: None,
            },
        })
        .unwrap();
    id
}

fn shelf(session: &Session) -> Vec<String> {
    session
        .state
        .under(None)
        .into_iter()
        .map(|one| one.name.clone())
        .collect()
}

#[test]
fn a_first_run_holds_its_lists_until_the_welcome_says_where_the_copies_go() {
    let desk = desk();
    let session = Session::at(desk.paths.clone()).unwrap();

    assert!(
        session.state.lists.is_empty(),
        "sown, this store stops looking new and could not adopt a folder that already holds one"
    );
}

#[test]
fn the_lists_arrive_once_the_welcome_is_through_and_never_a_second_time() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();

    session.sow_if_due();
    let first = session.state.lists.len();
    session.sow_if_due();

    assert!(
        first > 0,
        "a machine that syncs nothing still starts with lists"
    );
    assert_eq!(session.state.lists.len(), first);
    assert_eq!(session.config.sown, Some(true));
}

#[test]
fn a_folder_that_already_holds_a_store_is_the_meeting_place_itself() {
    let desk = desk();
    let shared = desk.paths.data().join("shared");
    std::fs::create_dir_all(shared.join(tisty_sync::STORE)).unwrap();

    assert_eq!(super::room(&shared), shared);
}

#[test]
fn anywhere_else_holds_a_folder_of_ours_inside_it() {
    let desk = desk();
    let shared = desk.paths.data().join("Documents");
    std::fs::create_dir_all(&shared).unwrap();

    assert_eq!(super::room(&shared), shared.join(tisty_core::keepers::OURS));
}

#[test]
fn a_guide_that_came_from_another_machine_is_taken_rather_than_planted_again() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let file = "guia-a1b2".to_string();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                file: file.clone(),
                order: "a1".into(),
                said: Some(tisty_core::event::Said {
                    title: tisty_core::docs::titled(crate::answers::settings::GUIDE_ES),
                    bytes: None,
                    tags: Some(Vec::new()),
                    by: None,
                }),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();

    assert_eq!(
        crate::answers::settings::guide_already_here(&session).map(|one| one.0),
        Some(file)
    );
}

#[test]
fn a_document_of_your_own_is_never_mistaken_for_the_guide() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                file: "notas-c3d4".into(),
                order: "a1".into(),
                said: Some(tisty_core::event::Said {
                    title: "Mis notas".into(),
                    bytes: None,
                    tags: Some(Vec::new()),
                    by: None,
                }),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();

    assert_eq!(crate::answers::settings::guide_already_here(&session), None);
}

#[test]
fn a_folder_dropped_before_one_that_is_gone_lands_last_rather_than_nowhere() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let moving = folder(&mut session, "mover", "a0");
    folder(&mut session, "uno", "a1");
    folder(&mut session, "dos", "a2");

    let ops = beside_folders(&session.state, moving, None, Some(ulid::Ulid::generate()));
    session.commit_all(ops).unwrap();

    assert_eq!(
        shelf(&session),
        ["uno", "dos", "mover"],
        "a neighbour that vanished must not leave the row where it was"
    );
}

#[test]
fn a_folder_dropped_before_another_lands_right_there() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    folder(&mut session, "uno", "a1");
    let two = folder(&mut session, "dos", "a2");
    let moving = folder(&mut session, "mover", "a3");

    let ops = beside_folders(&session.state, moving, None, Some(two));
    session.commit_all(ops).unwrap();

    assert_eq!(shelf(&session), ["uno", "mover", "dos"]);
}

#[test]
fn dropping_at_the_front_over_and_over_never_grows_a_key_out_of_hand() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let one = folder(&mut session, "uno", "a1");
    let two = folder(&mut session, "dos", "a2");

    for n in 0..600 {
        let (moving, before) = match n % 2 {
            0 => (two, one),
            _ => (one, two),
        };
        let ops = beside_folders(&session.state, moving, None, Some(before));
        session.commit_all(ops).unwrap();
    }

    let longest = session
        .state
        .under(None)
        .into_iter()
        .map(|one| one.order.len())
        .max()
        .unwrap_or(0);
    assert!(longest <= 24, "a key grew to {longest}");
    assert_eq!(shelf(&session).len(), 2);
}

#[test]
fn a_document_dropped_before_one_that_is_gone_lands_last_too() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let mut made = |name: &str, order: &str| {
        let id = ulid::Ulid::generate();
        session
            .commit(Op::DocAdd {
                id,
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: name.into(),
                    order: order.into(),
                    folder: None,
                    page_of: None,
                },
            })
            .unwrap();
        id
    };
    let moving = made("dev_a-0001", "a0");
    made("dev_a-0002", "a1");
    made("dev_a-0003", "a2");

    let ops = beside_docs(&session.state, moving, None, Some(ulid::Ulid::generate()));
    session.commit_all(ops).unwrap();

    let mut sitting: Vec<&tisty_core::model::Kept> = session.state.docs.values().collect();
    sitting.sort_by(|a, b| a.order.cmp(&b.order));
    assert_eq!(
        sitting.last().map(|one| one.file.as_str()),
        Some("dev_a-0001")
    );
}
