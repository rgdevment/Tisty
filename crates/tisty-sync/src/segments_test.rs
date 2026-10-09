use super::*;

fn put(dir: &Path, named: &str, body: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(named), body).unwrap();
}

fn read(dir: &Path, named: &str) -> String {
    std::fs::read_to_string(dir.join(named)).unwrap_or_default()
}

#[test]
fn what_is_installed_is_what_was_copied_aside_however_the_folder_changes_after() {
    let room = tempfile::tempdir().unwrap();
    let (theirs, mine, aside) = (
        room.path().join("theirs"),
        room.path().join("mine"),
        room.path().join("aside"),
    );
    put(&theirs, "000001.tisty", "lo cerrado\n");
    put(&mine, "000001.tisty", "lo cerrado\n");
    put(&theirs, "active.tisty", "lo comprobado\n");
    put(&theirs, "active.sig", "la firma\n");
    let known: Named = [std::ffi::OsString::from("000001.tisty")]
        .into_iter()
        .collect();

    staged(&theirs, &mine, &known, &aside).unwrap();
    put(&theirs, "active.tisty", "lo que nadie comprobó\n");
    copy_segments(&aside, &mine, false, &known).unwrap();

    assert_eq!(read(&mine, "active.tisty"), "lo comprobado\n");
    assert_eq!(read(&mine, "active.sig"), "la firma\n");
    assert_eq!(read(&aside, "000001.tisty"), "lo cerrado\n");
}

#[test]
fn what_both_sides_hold_the_same_is_copied_aside_from_this_side() {
    let room = tempfile::tempdir().unwrap();
    let (theirs, mine, aside) = (
        room.path().join("theirs"),
        room.path().join("mine"),
        room.path().join("aside"),
    );
    put(&theirs, "000001.tisty", "cambiado en la carpeta\n");
    put(&mine, "000001.tisty", "lo de aquí\n");
    let known: Named = [std::ffi::OsString::from("000001.tisty")]
        .into_iter()
        .collect();

    staged(&theirs, &mine, &known, &aside).unwrap();

    assert_eq!(read(&aside, "000001.tisty"), "lo de aquí\n");
}

#[test]
fn only_plain_files_are_copied_aside_and_an_old_copy_is_cleared_first() {
    let room = tempfile::tempdir().unwrap();
    let (theirs, mine, aside) = (
        room.path().join("theirs"),
        room.path().join("mine"),
        room.path().join("aside"),
    );
    put(&theirs, "active.tisty", "lo nuevo\n");
    std::fs::create_dir_all(theirs.join("un-directorio")).unwrap();
    put(&aside, "de-antes.tisty", "lo de la vuelta pasada\n");

    staged(&theirs, &mine, &Named::default(), &aside).unwrap();

    assert_eq!(read(&aside, "active.tisty"), "lo nuevo\n");
    assert!(!aside.join("un-directorio").exists());
    assert!(!aside.join("de-antes.tisty").exists());
}

#[test]
fn a_signature_changed_beside_segments_held_the_same_is_something_new() {
    let room = tempfile::tempdir().unwrap();
    let (theirs, mine) = (room.path().join("theirs"), room.path().join("mine"));
    put(&theirs, "active.tisty", "lo mismo\n");
    put(&mine, "active.tisty", "lo mismo\n");
    put(&theirs, "active.sig", "la firma de allá\n");
    put(&mine, "active.sig", "la firma de acá\n");

    assert!(beside_differs(&theirs, &mine));

    put(&mine, "active.sig", "la firma de allá\n");
    assert!(!beside_differs(&theirs, &mine));
}

#[test]
fn a_live_segment_the_closed_one_already_holds_is_not_carried() {
    let room = tempfile::tempdir().unwrap();
    let (theirs, mine) = (room.path().join("theirs"), room.path().join("mine"));
    put(&theirs, "000001.tisty", "uno\ndos\ntres\n");
    put(&theirs, "active.tisty", "uno\ndos\n");
    put(&theirs, "active.sig", "la firma vieja\n");

    let (done, _) = copy_segments(&theirs, &mine, false, &Named::default()).unwrap();

    assert_eq!(done, 1);
    assert_eq!(read(&mine, "000001.tisty"), "uno\ndos\ntres\n");
    assert!(
        !mine.join("active.tisty").exists(),
        "a leftover of a rotation became the live segment"
    );
}

#[test]
fn an_empty_live_segment_is_carried_all_the_same() {
    let room = tempfile::tempdir().unwrap();
    let (theirs, mine) = (room.path().join("theirs"), room.path().join("mine"));
    put(&theirs, "000001.tisty", "uno\ndos\n");
    put(&theirs, "active.tisty", "");

    copy_segments(&theirs, &mine, false, &Named::default()).unwrap();

    assert!(mine.join("active.tisty").is_file());
}

#[test]
fn a_leftover_live_segment_does_not_stop_one_history_from_growing_into_the_other() {
    let room = tempfile::tempdir().unwrap();
    let (ours, theirs) = (room.path().join("ours"), room.path().join("theirs"));
    put(&ours, "000001.tisty", "uno\ndos\ntres\n");
    put(&ours, "active.tisty", "uno\ndos\n");
    put(&theirs, "000001.tisty", "uno\ndos\ntres\n");
    put(&theirs, "000002.tisty", "cuatro\n");

    let grew = one_grew_from_the_other(&ours, &theirs);

    assert!(
        matches!(grew, Grew::Yes),
        "a leftover was counted as part of the history"
    );
}
