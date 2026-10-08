use super::Session;
use super::answers::storing::went_back_on;
use tisty_core::Paths;

struct Desk {
    _tmp: tempfile::TempDir,
    paths: Paths,
    shared: std::path::PathBuf,
}

fn desk() -> Desk {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let shared = tmp.path().join("shared");
    std::fs::create_dir_all(shared.join("store")).unwrap();
    Desk {
        _tmp: tmp,
        paths,
        shared,
    }
}

fn sharing_the_same_store(desk: &Desk, session: &Session) {
    let ours = tisty_core::store::identity(session.paths.store()).unwrap();
    std::fs::write(desk.shared.join("store").join(".store-id"), ours.as_bytes()).unwrap();
}

#[test]
fn a_store_that_went_back_on_a_copy_does_not_carry_to_the_folder_it_went_back_from() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    sharing_the_same_store(&desk, &session);

    assert!(
        !went_back_on(
            &session,
            tisty_carrier::considering(desk.shared.clone()).theirs()
        ),
        "nothing was restored, so the folder is just a folder"
    );

    session
        .keep(|c| c.restored_at = Some(jiff::Timestamp::now()))
        .unwrap();

    assert!(
        went_back_on(
            &session,
            tisty_carrier::considering(desk.shared.clone()).theirs()
        ),
        "carrying would bring back the very history the copy went back on"
    );
}

#[test]
fn a_folder_holding_somebody_elses_store_is_not_what_this_one_went_back_from() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    std::fs::write(
        desk.shared.join("store").join(".store-id"),
        b"01M0ZX62YMRXMABJ6Q4FEF69WT",
    )
    .unwrap();
    session
        .keep(|c| c.restored_at = Some(jiff::Timestamp::now()))
        .unwrap();

    assert!(
        !went_back_on(
            &session,
            tisty_carrier::considering(desk.shared.clone()).theirs()
        ),
        "another store has its own door, and this one is not it"
    );
}

#[test]
fn an_empty_folder_is_open_to_a_store_that_went_back() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    session
        .keep(|c| c.restored_at = Some(jiff::Timestamp::now()))
        .unwrap();

    assert!(
        !went_back_on(
            &session,
            tisty_carrier::considering(desk.shared.clone()).theirs()
        ),
        "a folder that holds nothing has nothing to bring back"
    );
}
