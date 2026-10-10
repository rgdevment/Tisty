use super::*;
use crate::Store;
use crate::event::{Event, Op};

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

    fn sixteen(&self) -> crate::sixteen::Sixteen {
        crate::sixteen::Sixteen::at(
            &self.dir,
            &self.who,
            signing::mine(&self.paths, &self.who).unwrap(),
        )
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
fn a_history_from_before_signing_answers_as_one() {
    let held = a_machine_that_wrote(0);
    held.sixteen().unsigned((0..2).map(a_line));

    assert_eq!(
        held.asked(Reached::default()),
        Ok(Reached::default()),
        "a history written before signing was taken for a signed one"
    );
}

/// Signing only ever starts. A machine that has shown one signature and then shows a segment
/// without one has had it taken away, which is the cheapest attack there is on any of this.
#[test]
fn a_signature_taken_away_is_not_a_history_from_before_signing() {
    let held = a_machine_that_wrote(0);
    held.sixteen().signed((0..2).map(a_line));
    let answered = held.asked(Reached::default()).unwrap();
    assert!(answered.signing, "it did not notice the machine signs");

    std::fs::remove_file(held.dir.join("active.sig")).unwrap();

    assert!(
        matches!(held.asked(answered), Err(Adrift::Disowned(_))),
        "a machine stopped signing and its history came in anyway"
    );
}

#[test]
fn a_seal_taken_away_leaves_the_live_segment_on_its_way() {
    let held = a_machine_that_wrote(2);
    let answered = held.asked(Reached::default()).unwrap();
    assert!(answered.signing);

    unsealed(&held.dir.join("active.tisty"));

    assert_eq!(
        held.asked(answered),
        Err(Adrift::Unreadable("active.tisty".into()))
    );
}

#[test]
fn a_closed_segment_that_lost_its_seals_is_disowned() {
    let held = a_machine_that_wrote(2);
    let mut store = held.signs();
    store.rotate().unwrap();
    store.append(a_line(9)).unwrap();
    drop(store);

    unsealed(&held.dir.join("000001.tisty"));

    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Disowned("000001.tisty".into()))
    );
}

#[test]
fn a_segment_still_being_written_may_answer_where_a_closed_one_did_not() {
    let held = a_machine_that_wrote(0);
    let mut old = held.sixteen();
    old.signed((0..2).map(a_line));
    old.closed();
    old.signed([a_line(9)]);
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
fn a_closed_segment_from_before_the_seal_is_answered_for_by_the_first_seal() {
    let held = a_machine_that_wrote(0);
    let mut old = held.sixteen();
    old.signed((0..2).map(a_line));
    old.closed();
    std::fs::remove_file(held.dir.join("000001.sig")).unwrap();
    held.signs().append(a_line(9)).unwrap();

    let answered = held
        .asked(Reached {
            signing: true,
            ..Default::default()
        })
        .expect("a seal did not answer for the segment written before it");

    assert_eq!(answered.segment, 1);
}

#[test]
fn a_history_the_16_signed_is_sealed_where_it_stands() {
    let held = a_machine_that_wrote(0);
    held.sixteen().signed((0..2).map(a_line));
    let before = std::fs::read(held.dir.join("active.tisty")).unwrap();

    held.signs().append(a_line(9)).unwrap();

    let now = std::fs::read(held.dir.join("active.tisty")).unwrap();
    assert!(
        now.starts_with(&before),
        "the history was rewritten to seal it"
    );
    assert!(
        !held.dir.join("000001.tisty").exists(),
        "it rotated to seal what was there"
    );
    let answered = held
        .asked(Reached::default())
        .expect("a history the 16 signed did not answer once sealed");
    assert!(answered.signing);
}

#[test]
fn a_history_from_before_signing_is_sealed_whole_by_the_first_write_with_a_key() {
    let held = a_machine_that_wrote(0);
    let mut old = held.sixteen();
    old.unsigned((0..2).map(a_line));
    old.closed();
    old.unsigned([a_line(3)]);

    held.signs().append(a_line(9)).unwrap();

    let answered = held
        .asked(Reached::default())
        .expect("the first seal did not answer for the history before it");
    assert!(answered.signing);
    assert_eq!(answered.segment, 1);
}

#[test]
fn what_was_written_past_the_last_seal_is_on_its_way_until_the_next_write_seals_it() {
    let held = a_machine_that_wrote(2);
    let event = Event::new(held.who.clone(), jiff::Timestamp::now(), a_line(7));
    let mut active = std::fs::OpenOptions::new()
        .append(true)
        .open(held.dir.join("active.tisty"))
        .unwrap();
    std::io::Write::write_all(
        &mut active,
        format!("{}\n", serde_json::to_string(&event).unwrap()).as_bytes(),
    )
    .unwrap();
    drop(active);

    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Unreadable("active.tisty".into()))
    );

    held.signs().append(a_line(9)).unwrap();

    held.asked(Reached::default())
        .expect("the next write did not seal what was on its way");
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
    let held = a_machine_that_wrote(0);
    held.sixteen().signed((0..2).map(a_line));
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
fn a_seal_made_with_another_key_is_a_hand_and_never_heals() {
    let held = a_machine_that_wrote(2);
    let other = signing::mine(&held.paths, &DeviceId("dev_b".into())).unwrap();

    resealed(&held, &other);

    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Disowned("active.tisty".into()))
    );
}

#[test]
fn a_signature_that_will_not_parse_can_heal() {
    let held = a_machine_that_wrote(0);
    held.sixteen().signed((0..2).map(a_line));
    let answered = held.asked(Reached::default()).unwrap();
    assert!(answered.signing);

    std::fs::write(held.dir.join("active.sig"), b"{\"tip\":\"ab").unwrap();

    assert_eq!(
        held.asked(answered),
        Err(Adrift::Unreadable("active.tisty".into()))
    );
}

#[test]
fn a_seal_that_will_not_read_can_heal() {
    let held = a_machine_that_wrote(2);
    let active = held.dir.join("active.tisty");
    let mut kept = without_the_last_line(&active);
    kept.extend_from_slice(b"{\"v\":17,\"op\":\"seal\",\"seg\":1}\n");
    std::fs::write(&active, kept).unwrap();

    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Unreadable("active.tisty".into()))
    );
}

/// A sidecar cut off mid-character, or zero-padded by the folder it travelled through, is a
/// signature that will not read — not one somebody took away.
#[test]
fn a_sidecar_whose_bytes_are_not_text_can_heal() {
    let held = a_machine_that_wrote(0);
    held.sixteen().signed((0..2).map(a_line));
    let answered = held.asked(Reached::default()).unwrap();

    std::fs::write(held.dir.join("active.sig"), [0xff, 0xfe, 0x41]).unwrap();

    assert_eq!(
        held.asked(answered),
        Err(Adrift::Unreadable("active.tisty".into()))
    );
}

#[test]
fn every_signature_taken_away_is_not_a_history_from_before_signing() {
    let held = a_machine_that_wrote(0);
    let mut old = held.sixteen();
    old.signed((0..2).map(a_line));
    old.closed();
    old.signed([a_line(3)]);
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

    assert!(
        matches!(held.asked(answered), Err(Adrift::Disowned(_))),
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
}

impl Cut {
    fn dir(&self) -> std::path::PathBuf {
        self.far.path().join("dev_a")
    }
}

fn a_rotation_cut_after_the_closed_segment_was_copied() -> (Wrote, Cut) {
    let held = a_machine_that_wrote(2);
    let old_active = std::fs::read(held.dir.join("active.tisty")).unwrap();
    let mut store = held.signs();
    store.append(a_line(2)).unwrap();
    store.rotate().unwrap();
    store.append(a_line(3)).unwrap();
    drop(store);

    let far = tempfile::tempdir().unwrap();
    let dir = far.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(held.dir.join("000001.tisty"), dir.join("000001.tisty")).unwrap();
    std::fs::write(dir.join("active.tisty"), &old_active).unwrap();
    (held, Cut { far, old_active })
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
fn a_live_segment_left_by_a_rotation_is_skipped_however_many_segments_closed_since() {
    let held = a_machine_that_wrote(2);
    let old_active = std::fs::read(held.dir.join("active.tisty")).unwrap();
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
    for named in ["000001.tisty", "000002.tisty"] {
        std::fs::copy(held.dir.join(named), dir.join(named)).unwrap();
    }
    std::fs::write(dir.join("active.tisty"), &old_active).unwrap();

    let answered = answers(&dir, &held.who, &held.by(), Reached::default(), &|_| false);

    let reached = answered.expect("a leftover was disowned because two segments closed after it");
    assert_eq!(reached.segment, 2);
}

#[test]
fn a_machine_without_its_key_keeps_writing_and_never_closes_a_segment() {
    let max = crate::store::SEGMENT_MAX_EVENTS;
    let held = a_machine_that_wrote(0);
    let mut keyless = Store::open(held.paths.store(), held.who.clone()).unwrap();

    keyless
        .append_batch((0..max + 15).map(a_line).collect())
        .expect("a machine without its key was kept from writing");

    assert!(!held.dir.join("000001.tisty").exists());
    let lines = std::fs::read_to_string(held.dir.join("active.tisty"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(lines, max + 15, "the batch was not written whole");
    assert_eq!(
        held.asked(Reached::default()),
        Err(Adrift::Unreadable("active.tisty".into())),
        "what nobody sealed came in, or was called a hand"
    );
}

#[test]
fn a_segment_that_could_not_be_closed_is_closed_once_the_key_is_back() {
    let max = crate::store::SEGMENT_MAX_EVENTS;
    let held = a_machine_that_wrote(0);
    let mut keyless = Store::open(held.paths.store(), held.who.clone()).unwrap();
    keyless
        .append_batch((0..max + 1).map(a_line).collect())
        .unwrap();
    drop(keyless);

    held.signs().append(a_line(max + 1)).unwrap();

    assert!(held.dir.join("000001.tisty").is_file());
    let lines = std::fs::read_to_string(held.dir.join("active.tisty"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(lines, 2, "one event and its seal");
    let reached = held
        .asked(Reached::default())
        .expect("the segment closed late did not answer");
    assert_eq!(reached.segment, 1);
}

#[test]
fn a_rotation_cut_between_its_seal_and_its_rename_is_finished_by_the_next_write() {
    let held = a_machine_that_wrote(2);
    let active = held.dir.join("active.tisty");
    let bytes = std::fs::read(&active).unwrap();
    let closing = crate::seal::Seal {
        seg: 1,
        at: bytes.len() as u64,
        tip: signing::tip_of(signing::NOTHING_BEFORE, &bytes),
        n: 2,
        closed: true,
    };
    let key = signing::mine(&held.paths, &held.who).unwrap();
    let mut cut = bytes.clone();
    cut.extend_from_slice(crate::seal::line(&key, &held.who.0, &closing).as_bytes());
    std::fs::write(&active, cut).unwrap();
    held.asked(Reached::default())
        .expect("a segment sealed closed but not yet renamed was refused");

    held.signs().append(a_line(9)).unwrap();

    assert!(held.dir.join("000001.tisty").is_file());
    let lines = std::fs::read_to_string(&active).unwrap().lines().count();
    assert_eq!(lines, 2, "one event and its seal");
    let reached = held.asked(Reached::default()).unwrap();
    assert_eq!(reached.segment, 1);
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

fn without_the_last_line(at: &std::path::Path) -> Vec<u8> {
    let bytes = std::fs::read(at).unwrap();
    let body = &bytes[..bytes.len() - 1];
    let cut = body
        .iter()
        .rposition(|one| *one == b'\n')
        .map_or(0, |at| at + 1);
    bytes[..cut].to_vec()
}

fn unsealed(at: &std::path::Path) {
    let kept: String = std::fs::read_to_string(at)
        .unwrap()
        .lines()
        .filter(|line| !line.contains("\"op\":\"seal\""))
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(at, kept).unwrap();
}

fn resealed(held: &Wrote, key: &signing::SigningKey) {
    let active = held.dir.join("active.tisty");
    let bytes = std::fs::read(&active).unwrap();
    let last = bytes[..bytes.len() - 1]
        .iter()
        .rposition(|one| *one == b'\n')
        .map_or(0, |at| at + 1);
    let crate::seal::Line::Seal(read) = crate::seal::read(&bytes[last..]) else {
        panic!("the last line was not a seal");
    };
    let mut kept = bytes[..last].to_vec();
    kept.extend_from_slice(crate::seal::line(key, &held.who.0, &read.seal).as_bytes());
    std::fs::write(&active, kept).unwrap();
}

fn said_its_key(held: &Wrote) -> signing::SigningKey {
    let key = signing::mine(&held.paths, &held.who).unwrap();
    held.signs()
        .append(Op::DeviceKey {
            d: held.who.clone(),
            p: signing::shown(&key),
        })
        .unwrap();
    key
}

fn written_with(held: &Wrote, key: &signing::SigningKey, op: Op) {
    Store::open(held.paths.store(), held.who.clone())
        .unwrap()
        .signing_with(Some(key.clone()))
        .append(op)
        .unwrap();
}

fn rotated(held: &Wrote, from: &signing::SigningKey, to: &signing::SigningKey) {
    written_with(
        held,
        from,
        Op::DeviceRotate {
            d: held.who.clone(),
            p: signing::shown(to),
        },
    );
}

fn trusting(held: &Wrote, keys: &[&signing::SigningKey]) -> Result<Reached, Adrift> {
    let trusted: Vec<VerifyingKey> = keys.iter().map(|one| one.verifying_key()).collect();
    answers_trusting(&held.dir, &held.who, &trusted, Reached::default(), &|_| {
        false
    })
}

#[test]
fn a_machine_confirmed_under_its_old_key_is_answered_for_under_the_new_one() {
    let held = a_machine_that_wrote(0);
    let old = said_its_key(&held);
    let new = signing::SigningKey::from_bytes(&[7; 32]);
    rotated(&held, &old, &new);
    written_with(&held, &new, a_line(1));

    let reached = trusting(&held, &[&old]).expect("a rotation its old key sealed was refused");

    assert_eq!(
        reached.moved,
        Some(new.verifying_key().to_bytes()),
        "the key it moved to was not handed back to be kept"
    );
}

#[test]
fn a_rotation_kept_here_reads_the_whole_history_again_without_asking() {
    let held = a_machine_that_wrote(0);
    let old = said_its_key(&held);
    let new = signing::SigningKey::from_bytes(&[7; 32]);
    rotated(&held, &old, &new);
    written_with(&held, &new, a_line(1));

    let reached = trusting(&held, &[&new, &old]).expect("the rotation already kept was refused");

    assert_eq!(reached.moved, None);
    assert!(reached.signing);
}

#[test]
fn a_rotation_with_nothing_after_it_answers_under_the_old_key() {
    let held = a_machine_that_wrote(0);
    let old = said_its_key(&held);
    let new = signing::SigningKey::from_bytes(&[7; 32]);
    rotated(&held, &old, &new);

    let reached = trusting(&held, &[&old]).expect("a rotation nothing followed yet was refused");

    assert_eq!(reached.moved, Some(new.verifying_key().to_bytes()));
}

#[test]
fn a_seal_by_the_old_key_after_its_rotation_is_disowned() {
    let held = a_machine_that_wrote(0);
    let old = said_its_key(&held);
    let new = signing::SigningKey::from_bytes(&[7; 32]);
    rotated(&held, &old, &new);
    written_with(&held, &old, a_line(1));

    assert_eq!(
        trusting(&held, &[&old]),
        Err(Adrift::Disowned("active.tisty".into()))
    );
}

#[test]
fn a_seal_by_a_new_key_before_the_rotation_that_names_it_is_disowned() {
    let held = a_machine_that_wrote(0);
    let old = said_its_key(&held);
    let new = signing::SigningKey::from_bytes(&[7; 32]);
    written_with(&held, &new, a_line(1));

    assert_eq!(
        trusting(&held, &[&old]),
        Err(Adrift::Disowned("active.tisty".into()))
    );
}

#[test]
fn a_rotation_its_old_key_never_sealed_is_disowned() {
    let held = a_machine_that_wrote(0);
    let old = said_its_key(&held);
    let new = signing::SigningKey::from_bytes(&[7; 32]);
    rotated(&held, &new, &new);
    written_with(&held, &new, a_line(1));

    assert_eq!(
        trusting(&held, &[&old]),
        Err(Adrift::Disowned("active.tisty".into()))
    );
}

#[test]
fn a_key_confirmed_here_named_by_a_stranger_s_rotation_lets_nothing_in() {
    let held = a_machine_that_wrote(0);
    let stranger = said_its_key(&held);
    let confirmed = signing::SigningKey::from_bytes(&[7; 32]);
    rotated(&held, &stranger, &confirmed);

    let answered = trusting(&held, &[&confirmed]);

    assert!(
        matches!(answered, Err(Adrift::Unreadable(_))),
        "a history nobody here answered for came in by naming a key somebody did: {answered:?}"
    );
}
