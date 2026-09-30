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
