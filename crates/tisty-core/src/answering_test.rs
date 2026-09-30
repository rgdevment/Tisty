use super::*;
use crate::Store;
use crate::event::Op;

fn a_machine_that_wrote(at: &Path, lines: usize) -> (crate::Paths, DeviceId, std::path::PathBuf) {
    let paths = crate::Paths::new(at.join("data"), at.join("config"));
    let who = DeviceId("dev_a".into());
    let key = signing::mine(&paths, &who).unwrap();
    let mut store = Store::open(paths.store(), who.clone())
        .unwrap()
        .signing_with(Some(key));
    for n in 0..lines {
        store
            .append(Op::TaskAdd {
                id: ulid::Ulid::generate(),
                d: crate::event::TaskAdd::new(format!("the {n} thing"), "a0"),
            })
            .unwrap();
    }
    let dir = paths.store().join(&who.0);
    (paths, who, dir)
}

fn by(paths: &crate::Paths, who: &DeviceId) -> VerifyingKey {
    signing::mine(paths, who).unwrap().verifying_key()
}

#[test]
fn what_a_machine_signed_answers_for_itself() {
    let tmp = tempfile::tempdir().unwrap();
    let (paths, who, dir) = a_machine_that_wrote(tmp.path(), 3);

    answers(&dir, &who, &by(&paths, &who), Reached::default())
        .expect("a history it signed itself did not answer");
}

#[test]
fn a_line_changed_under_a_signature_does_not_answer() {
    let tmp = tempfile::tempdir().unwrap();
    let (paths, who, dir) = a_machine_that_wrote(tmp.path(), 3);
    let whole = std::fs::read_to_string(dir.join("active.tisty")).unwrap();
    std::fs::write(
        dir.join("active.tisty"),
        whole.replace("the 1 thing", "the X thing"),
    )
    .unwrap();

    assert_eq!(
        answers(&dir, &who, &by(&paths, &who), Reached::default()),
        Err("active.tisty".to_string())
    );
}

#[test]
fn a_signature_made_by_somebody_else_does_not_answer() {
    let tmp = tempfile::tempdir().unwrap();
    let (paths, who, dir) = a_machine_that_wrote(tmp.path(), 2);
    let other = by(&paths, &DeviceId("dev_b".into()));

    assert!(answers(&dir, &who, &other, Reached::default()).is_err());
}

#[test]
fn a_history_with_nothing_to_check_is_not_read_at_all() {
    let tmp = tempfile::tempdir().unwrap();
    let (paths, who, dir) = a_machine_that_wrote(tmp.path(), 2);
    std::fs::remove_file(dir.join("active.sig")).unwrap();

    assert_eq!(
        answers(&dir, &who, &by(&paths, &who), Reached::default()),
        Ok(Reached::default()),
        "a history written before signing was folded for nothing"
    );
}

#[test]
fn a_segment_carrying_no_signature_is_not_refused_for_the_lack_of_one() {
    let tmp = tempfile::tempdir().unwrap();
    let (paths, who, dir) = a_machine_that_wrote(tmp.path(), 2);
    let key = signing::mine(&paths, &who).unwrap();
    let mut store = Store::open(paths.store(), who.clone())
        .unwrap()
        .signing_with(Some(key));
    store.rotate().unwrap();
    store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: crate::event::TaskAdd::new("written after the turn", "a0"),
        })
        .unwrap();
    drop(store);
    std::fs::remove_file(dir.join("active.sig")).unwrap();

    let held = answers(&dir, &who, &by(&paths, &who), Reached::default())
        .expect("a segment with no signature was refused for having none");

    assert_eq!(
        held.segment, 1,
        "the closed segment that did answer was not remembered"
    );
}

#[test]
fn what_already_answered_is_not_read_again() {
    let tmp = tempfile::tempdir().unwrap();
    let (paths, who, dir) = a_machine_that_wrote(tmp.path(), 2);
    let key = signing::mine(&paths, &who).unwrap();
    let mut store = Store::open(paths.store(), who.clone())
        .unwrap()
        .signing_with(Some(key));
    store.rotate().unwrap();
    store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: crate::event::TaskAdd::new("after the turn", "a0"),
        })
        .unwrap();
    drop(store);

    let held = answers(&dir, &who, &by(&paths, &who), Reached::default()).unwrap();
    assert_eq!(
        held.segment, 1,
        "a closed segment that answered was not marked"
    );

    std::fs::remove_file(dir.join("000001.tisty")).unwrap();
    std::fs::create_dir(dir.join("000001.tisty")).unwrap();

    answers(&dir, &who, &by(&paths, &who), held)
        .expect("it read again what it had already checked");
}
