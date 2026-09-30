use super::*;

fn a_place() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    let dest = room.path().join("shared");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::create_dir_all(dest.join(STORE)).unwrap();
    (room, data, dest)
}

#[test]
fn a_folder_from_before_this_file_existed_is_adopted_rather_than_refused() {
    let (_room, data, dest) = a_place();
    std::fs::create_dir_all(dest.join(STORE).join("dev_b")).unwrap();

    before_reading(&data, &dest).expect("a folder written by an older build was turned away");

    stamp(&data, &dest);
    assert!(
        dest.join(NAMED).is_file(),
        "it was adopted without saying so"
    );
    before_reading(&data, &dest).expect("what it stamped itself it then refused");
}

#[test]
fn a_shape_this_build_does_not_know_stops_it_before_a_byte_is_written() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), format!("shape = {}\n", OURS + 1)).unwrap();

    assert!(
        matches!(before_reading(&data, &dest), Err(Trouble::Shape(_))),
        "a folder arranged by a build that knows more was read anyway"
    );
}

#[test]
fn a_folder_that_said_its_shape_and_went_quiet_is_not_read_as_empty() {
    let (_room, data, dest) = a_place();
    std::fs::create_dir_all(dest.join(STORE).join("dev_b")).unwrap();
    stamp(&data, &dest);
    before_reading(&data, &dest).unwrap();

    std::fs::remove_file(dest.join(NAMED)).unwrap();

    assert!(
        matches!(before_reading(&data, &dest), Err(Trouble::Unshaped(_))),
        "a folder whose shape went missing read as one with nothing in it"
    );
}

#[test]
fn another_folder_from_before_this_file_is_still_one_to_take_up() {
    let (room, data, dest) = a_place();
    stamp(&data, &dest);
    before_reading(&data, &dest).unwrap();

    let older = room.path().join("older");
    std::fs::create_dir_all(older.join(STORE).join("dev_b")).unwrap();

    before_reading(&data, &older).expect(
        "a machine that had met one folder saying its shape could never take up an older one",
    );
}

#[test]
fn what_says_a_shape_and_cannot_be_read_is_not_taken_for_nothing() {
    let (_room, data, dest) = a_place();
    std::fs::write(dest.join(NAMED), "folders = []\n").unwrap();

    assert!(matches!(
        before_reading(&data, &dest),
        Err(Trouble::Unshaped(_))
    ));
}
