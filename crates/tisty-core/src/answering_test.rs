use super::*;
use crate::Store;
use crate::event::Op;

struct Wrote {
    _room: tempfile::TempDir,
    paths: crate::Paths,
    who: DeviceId,
    dir: std::path::PathBuf,
    ours: std::cell::RefCell<std::collections::BTreeSet<String>>,
}

fn a_machine_that_wrote(lines: usize) -> Wrote {
    let room = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(room.path().join("data"), room.path().join("config"));
    let who = DeviceId("dev_a".into());
    let key = signing::mine(&paths, &who).unwrap();
    let mut store = Store::open(paths.store(), who.clone())
        .unwrap()
        .signing_with(Some(key));
    for n in 0..lines {
        store.append(a_line(n)).unwrap();
    }
    let dir = paths.store().join(&who.0);
    Wrote {
        _room: room,
        paths,
        who,
        dir,
        ours: Default::default(),
    }
}

fn a_line(n: usize) -> Op {
    Op::TaskAdd {
        id: ulid::Ulid::generate(),
        d: crate::event::TaskAdd::new(format!("the {n} thing"), "a0"),
    }
}

impl Wrote {
    fn signs(&self) -> Store {
        let key = signing::mine(&self.paths, &self.who).unwrap();
        Store::open(self.paths.store(), self.who.clone())
            .unwrap()
            .signing_with(Some(key))
    }

    fn by(&self) -> VerifyingKey {
        signing::mine(&self.paths, &self.who)
            .unwrap()
            .verifying_key()
    }

    fn asked(&self, from: Reached) -> Result<Reached, Adrift> {
        answers(&self.dir, &self.who, &self.by(), from, &|named| {
            self.ours.borrow().contains(named)
        })
    }

    fn ours_too(&self, segment: &str) {
        self.ours.borrow_mut().insert(segment.to_string());
    }
}

#[test]
fn what_a_machine_signed_answers_for_itself() {
    let held = a_machine_that_wrote(3);

    held.asked(Reached::default())
        .expect("a history it signed itself did not answer");
}

#[test]
fn a_line_changed_under_a_signature_does_not_answer() {
    let held = a_machine_that_wrote(3);
    let whole = std::fs::read_to_string(held.dir.join("active.tisty")).unwrap();
    std::fs::write(
        held.dir.join("active.tisty"),
        whole.replace("the 1 thing", "the X thing"),
    )
    .unwrap();

    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Disowned("active.tisty".into()))
    );
}

/// The signature answers for a prefix, so folding the whole file would take in whatever was put
/// past it — and the next thing that machine writes would sign that too.
#[test]
fn a_line_added_past_a_signature_does_not_answer() {
    let held = a_machine_that_wrote(2);
    let mut whole = std::fs::read_to_string(held.dir.join("active.tisty")).unwrap();
    let forged = whole.lines().next().unwrap().to_string();
    whole.push_str(&forged);
    whole.push('\n');
    std::fs::write(held.dir.join("active.tisty"), whole).unwrap();

    assert!(
        held.asked(Reached::default()).is_err(),
        "a line appended past the signature came in under it"
    );
}

#[test]
fn a_signature_made_by_somebody_else_does_not_answer() {
    let held = a_machine_that_wrote(2);
    let other = signing::mine(&held.paths, &DeviceId("dev_b".into()))
        .unwrap()
        .verifying_key();

    assert!(
        answers(&held.dir, &held.who, &other, Reached::default(), &|_| false).is_err(),
        "one machine's key answered for what another signed"
    );
}

#[test]
fn a_history_with_nothing_to_check_is_not_read_at_all() {
    let held = a_machine_that_wrote(2);
    std::fs::remove_file(held.dir.join("active.sig")).unwrap();

    assert_eq!(
        held.asked(Reached::default()),
        Ok(Reached::default()),
        "a history written before signing was folded for nothing"
    );
}

/// Signing only ever starts. A machine that has shown one signature and then shows a segment
/// without one has had it taken away, which is the cheapest attack there is on any of this.
#[test]
fn a_signature_taken_away_is_not_a_history_from_before_signing() {
    let held = a_machine_that_wrote(2);
    let answered = held.asked(Reached::default()).unwrap();
    assert!(answered.signing, "it did not notice the machine signs");

    std::fs::remove_file(held.dir.join("active.sig")).unwrap();

    assert!(
        matches!(held.asked(answered), Err(Adrift::Disowned(_))),
        "a machine stopped signing and its history came in anyway"
    );
}

#[test]
fn a_segment_still_being_written_may_answer_where_a_closed_one_did_not() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    store.rotate().unwrap();
    store.append(a_line(9)).unwrap();
    drop(store);
    std::fs::remove_file(held.dir.join("000001.sig")).unwrap();

    let answered = held
        .asked(Reached::default())
        .expect("an unsigned closed segment before a signed one was refused");

    assert!(answered.signing);
    assert_eq!(
        answered.segment, 0,
        "it marked a segment that never answered"
    );
}

#[test]
fn what_already_answered_is_not_read_again() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    store.rotate().unwrap();
    store.append(a_line(9)).unwrap();
    drop(store);

    let answered = held.asked(Reached::default()).unwrap();
    assert_eq!(
        answered.segment, 1,
        "a closed segment that answered was not marked"
    );

    held.ours_too("000001.tisty");
    std::fs::remove_file(held.dir.join("000001.tisty")).unwrap();
    std::fs::create_dir(held.dir.join("000001.tisty")).unwrap();
    std::fs::remove_file(held.dir.join("000001.sig")).unwrap();

    held.asked(answered)
        .expect("it read again what it had already checked");
}

/// The memo says what we answered for, not what the far side holds now, so its number alone is
/// never enough: a segment that is no longer the copy we hold is read again however old it is.
#[test]
fn a_segment_no_longer_the_copy_we_hold_is_read_again() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    store.rotate().unwrap();
    store.append(a_line(9)).unwrap();
    drop(store);
    let answered = held.asked(Reached::default()).unwrap();
    assert_eq!(answered.segment, 1);

    let whole = std::fs::read_to_string(held.dir.join("000001.tisty")).unwrap();
    std::fs::write(
        held.dir.join("000001.tisty"),
        whole.replace("the 1 thing", "the X thing"),
    )
    .unwrap();

    assert!(
        held.asked(answered).is_err(),
        "a closed segment already numbered was passed over without asking whether it is still ours"
    );
}

/// The memo's tip already holds every segment up to its number. Reading one of those again and
/// folding it onto that tip would count it twice, and the verdict for a history nobody touched
/// would be the permanent one.
#[test]
fn a_segment_nobody_touched_is_not_disowned_for_being_read_again() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    store.rotate().unwrap();
    store.append(a_line(9)).unwrap();
    drop(store);
    held.ours_too("000001.tisty");
    held.ours_too("active.tisty");
    let answered = held.asked(Reached::default()).unwrap();
    assert_eq!(answered.segment, 1);

    held.ours.borrow_mut().clear();

    let again = held
        .asked(answered)
        .expect("a history nobody touched was called somebody's hand");
    assert_eq!(again.segment, 1);
    assert_eq!(again.tip, answered.tip);
    assert!(again.signing);
}

#[test]
fn a_signature_that_does_not_verify_is_a_hand_and_never_heals() {
    let held = a_machine_that_wrote(2);
    let other = DeviceId("dev_b".into());
    let theirs = signing::mine(&held.paths, &other).unwrap();
    let said = signing::signed(
        &theirs,
        &signing::About {
            device: &held.who.0,
            segment: "active.tisty",
        },
        &signing::Covers {
            tip: signing::tip_of(
                signing::NOTHING_BEFORE,
                &std::fs::read(held.dir.join("active.tisty")).unwrap(),
            ),
            at: std::fs::metadata(held.dir.join("active.tisty"))
                .unwrap()
                .len(),
        },
    );
    std::fs::write(held.dir.join("active.sig"), said.as_bytes()).unwrap();

    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Disowned("active.tisty".into()))
    );
}

#[test]
fn a_signature_that_will_not_parse_can_heal() {
    let held = a_machine_that_wrote(2);
    let answered = held.asked(Reached::default()).unwrap();
    assert!(answered.signing);

    std::fs::write(held.dir.join("active.sig"), b"{\"tip\":\"ab").unwrap();

    assert_eq!(
        held.asked(answered),
        Err(Adrift::Unreadable("active.tisty".into()))
    );
}

/// A sidecar cut off mid-character, or zero-padded by the folder it travelled through, is a
/// signature that will not read — not one somebody took away.
#[test]
fn a_sidecar_whose_bytes_are_not_text_can_heal() {
    let held = a_machine_that_wrote(2);
    let answered = held.asked(Reached::default()).unwrap();

    std::fs::write(held.dir.join("active.sig"), [0xff, 0xfe, 0x41]).unwrap();

    assert_eq!(
        held.asked(answered),
        Err(Adrift::Unreadable("active.tisty".into()))
    );
}

#[test]
fn every_signature_taken_away_is_not_a_history_from_before_signing() {
    let held = a_machine_that_wrote(2);
    let answered = held.asked(Reached::default()).unwrap();
    assert!(answered.signing);

    for one in std::fs::read_dir(&held.dir)
        .unwrap()
        .filter_map(|one| one.ok())
    {
        if one
            .path()
            .extension()
            .is_some_and(|one| one == signing::SIG)
        {
            std::fs::remove_file(one.path()).unwrap();
        }
    }

    assert_eq!(
        held.asked(answered),
        Err(Adrift::Disowned(held.who.0.clone())),
        "every signature was stripped and the history came in anyway"
    );
    assert!(
        held.asked(Reached::default()).is_ok(),
        "a history that never signed is not a signature taken away"
    );
}

struct Cut {
    far: tempfile::TempDir,
    old_active: Vec<u8>,
    old_sig: Vec<u8>,
}

impl Cut {
    fn dir(&self) -> std::path::PathBuf {
        self.far.path().join("dev_a")
    }
}

fn a_rotation_cut_after_the_closed_segment_was_copied() -> (Wrote, Cut) {
    let held = a_machine_that_wrote(2);
    let old_active = std::fs::read(held.dir.join("active.tisty")).unwrap();
    let old_sig = std::fs::read(held.dir.join("active.sig")).unwrap();
    let mut store = held.signs();
    store.append(a_line(2)).unwrap();
    store.rotate().unwrap();
    store.append(a_line(3)).unwrap();
    drop(store);

    let far = tempfile::tempdir().unwrap();
    let dir = far.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    for named in ["000001.tisty", "000001.sig", "000001.count"] {
        std::fs::copy(held.dir.join(named), dir.join(named)).unwrap();
    }
    std::fs::write(dir.join("active.tisty"), &old_active).unwrap();
    std::fs::write(dir.join("active.sig"), &old_sig).unwrap();
    (
        held,
        Cut {
            far,
            old_active,
            old_sig,
        },
    )
}

#[test]
fn the_live_segment_a_cut_rotation_left_behind_is_skipped_not_disowned() {
    let (held, cut) = a_rotation_cut_after_the_closed_segment_was_copied();

    let answered = answers(
        &cut.dir(),
        &held.who,
        &held.by(),
        Reached::default(),
        &|_| false,
    );

    let reached = answered.expect("a machine was disowned for a rotation that was cut short");
    assert_eq!(reached.segment, 1, "the closed segment did not answer");
    assert!(reached.signing);
}

#[test]
fn a_live_segment_that_is_not_what_the_closed_one_held_is_still_disowned() {
    let (held, cut) = a_rotation_cut_after_the_closed_segment_was_copied();
    let forged = String::from_utf8(cut.old_active.clone())
        .unwrap()
        .replace("the 1 thing", "the X thing");
    std::fs::write(cut.dir().join("active.tisty"), forged).unwrap();
    std::fs::write(cut.dir().join("active.sig"), &cut.old_sig).unwrap();

    let answered = answers(
        &cut.dir(),
        &held.who,
        &held.by(),
        Reached::default(),
        &|_| false,
    );

    assert_eq!(
        answered,
        Err(Adrift::Disowned("active.tisty".into())),
        "a live segment that changed what was closed came in as a leftover"
    );
}

#[test]
fn a_rotation_that_cannot_be_signed_leaves_the_segment_open() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    std::fs::create_dir(held.dir.join("000001.sig")).unwrap();

    let closed = store.rotate().unwrap();

    assert!(!closed, "it closed a segment nothing answers for");
    assert!(held.dir.join("active.tisty").is_file());
    assert!(!held.dir.join("000001.tisty").exists());
}

#[test]
fn a_live_segment_left_by_a_rotation_is_skipped_however_many_segments_closed_since() {
    let held = a_machine_that_wrote(2);
    let old_active = std::fs::read(held.dir.join("active.tisty")).unwrap();
    let old_sig = std::fs::read(held.dir.join("active.sig")).unwrap();
    let mut store = held.signs();
    store.append(a_line(2)).unwrap();
    store.rotate().unwrap();
    store.append(a_line(3)).unwrap();
    store.rotate().unwrap();
    store.append(a_line(4)).unwrap();
    drop(store);
    let far = tempfile::tempdir().unwrap();
    let dir = far.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    for named in [
        "000001.tisty",
        "000001.sig",
        "000001.count",
        "000002.tisty",
        "000002.sig",
        "000002.count",
    ] {
        std::fs::copy(held.dir.join(named), dir.join(named)).unwrap();
    }
    std::fs::write(dir.join("active.tisty"), &old_active).unwrap();
    std::fs::write(dir.join("active.sig"), &old_sig).unwrap();

    let answered = answers(&dir, &held.who, &held.by(), Reached::default(), &|_| false);

    let reached = answered.expect("a leftover was disowned because two segments closed after it");
    assert_eq!(reached.segment, 2);
}

#[test]
fn a_segment_that_cannot_be_closed_keeps_taking_what_is_written_and_signs_it() {
    let max = crate::store::SEGMENT_MAX_EVENTS;
    let held = a_machine_that_wrote(0);
    let mut store = held.signs();
    store
        .append_batch((0..max - 5).map(a_line).collect())
        .unwrap();
    std::fs::create_dir(held.dir.join("000001.sig")).unwrap();

    store.append_batch((0..20).map(a_line).collect()).expect(
        "a batch past the end of a segment was refused for a signature it could not close with",
    );

    let lines = std::fs::read_to_string(held.dir.join("active.tisty"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(lines, max + 15, "the batch was not written whole");
    held.asked(Reached::default())
        .expect("what was written was left without a signature that covers it");
}

#[test]
fn a_segment_that_could_not_be_closed_is_closed_once_it_can() {
    let max = crate::store::SEGMENT_MAX_EVENTS;
    let held = a_machine_that_wrote(0);
    let mut store = held.signs();
    store.append_batch((0..max).map(a_line).collect()).unwrap();
    std::fs::create_dir(held.dir.join("000001.sig")).unwrap();
    store.append(a_line(max)).unwrap();
    assert!(!held.dir.join("000001.tisty").exists());
    std::fs::remove_dir(held.dir.join("000001.sig")).unwrap();

    store.append(a_line(max + 1)).unwrap();

    assert!(held.dir.join("000001.tisty").is_file());
    let lines = std::fs::read_to_string(held.dir.join("active.tisty"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(lines, 1);
    let reached = held
        .asked(Reached::default())
        .expect("the segment closed late did not answer");
    assert_eq!(reached.segment, 1);
}

#[test]
fn a_rotation_that_fails_after_the_rename_leaves_no_signature_without_a_segment() {
    let max = crate::store::SEGMENT_MAX_EVENTS;
    let held = a_machine_that_wrote(0);
    let mut store = held.signs();
    store
        .append_batch((0..max - 5).map(a_line).collect())
        .unwrap();
    std::fs::create_dir(held.dir.join("000001.count")).unwrap();

    let refused = store.append_batch((0..20).map(a_line).collect());

    assert!(refused.is_err());
    assert!(!held.dir.join("active.tisty").exists());
    assert!(
        !held.dir.join("active.sig").exists(),
        "a signature was left for a segment that is not there"
    );
}

#[test]
fn an_empty_live_segment_is_not_taken_for_a_leftover() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    store.rotate().unwrap();
    store.append(a_line(2)).unwrap();
    drop(store);
    std::fs::write(held.dir.join("active.tisty"), b"").unwrap();

    let answered = held.asked(Reached::default());

    assert!(
        matches!(answered, Err(Adrift::Unreadable(_))),
        "an emptied live segment was passed over as a leftover: {answered:?}"
    );
}
