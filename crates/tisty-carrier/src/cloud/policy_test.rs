use super::{
    PAUSED_AT_MOST, Policy, READ_AT_MOST, Refused, accepted, applied, halted, kept, pause_here,
    paused_here, read,
};
use crate::Remote;
use crate::fakes::Fake;

const KEY: &str = include_str!("signed/key.pub");
const LOWERED: &[u8] = include_bytes!("signed/lowered.json");
const LOWERED_SIG: &str = include_str!("signed/lowered.json.minisig");
const RAISED: &[u8] = include_bytes!("signed/raised.json");
const RAISED_SIG: &str = include_str!("signed/raised.json.minisig");
const OLDER: &[u8] = include_bytes!("signed/older.json");
const OLDER_SIG: &str = include_str!("signed/older.json.minisig");
const HALTED: &[u8] = include_bytes!("signed/halted.json");
const HALTED_SIG: &str = include_str!("signed/halted.json.minisig");
const UNREAD: &[u8] = include_bytes!("signed/unread.json");
const UNREAD_SIG: &str = include_str!("signed/unread.json.minisig");

#[test]
fn a_signed_word_is_read_and_anything_else_is_refused_before_it_is_believed() {
    assert_eq!(
        read(LOWERED, LOWERED_SIG, KEY).map(|one| one.units_a_day),
        Ok(Some(1000))
    );

    let mut tampered = LOWERED.to_vec();
    let near_end = tampered.len() - 2;
    tampered[near_end] = b'9';
    assert_eq!(read(&tampered, LOWERED_SIG, KEY), Err(Refused::Unsigned));
    assert_eq!(read(LOWERED, RAISED_SIG, KEY), Err(Refused::Unsigned));
    assert_eq!(
        read(LOWERED, "not a signature", KEY),
        Err(Refused::Unsigned)
    );
    assert_eq!(
        read(LOWERED, LOWERED_SIG, "not a key"),
        Err(Refused::Unsigned)
    );
    assert_eq!(read(UNREAD, UNREAD_SIG, KEY), Err(Refused::Unreadable));
    assert_eq!(
        read(&vec![b' '; READ_AT_MOST + 1], LOWERED_SIG, KEY),
        Err(Refused::TooBig)
    );
}

#[test]
fn a_word_lowers_every_limit_freely_but_lifts_one_no_further_than_twice() {
    let shipped = Fake::drive().limits();

    let lowered = applied(shipped, read(LOWERED, LOWERED_SIG, KEY).ok().as_ref());
    let raised = applied(shipped, read(RAISED, RAISED_SIG, KEY).ok().as_ref());
    let silent = applied(shipped, None);

    assert_eq!((lowered.units_a_day, lowered.bytes_a_day), (1000, 2048));
    assert_eq!(raised.units_a_day, shipped.units_a_day * 2);
    assert_eq!(raised.bytes_a_day, shipped.bytes_a_day * 2);
    assert_eq!(silent, shipped);
}

#[test]
fn the_newest_word_is_kept_for_good_and_an_older_one_is_a_replay() {
    let home = tempfile::tempdir().unwrap();

    accepted(home.path(), LOWERED, LOWERED_SIG, KEY).unwrap();
    let replayed = accepted(home.path(), OLDER, OLDER_SIG, KEY);

    assert_eq!(replayed, Err(Refused::Behind { kept: 5, said: 3 }));
    assert_eq!(kept(home.path()).map(|one| one.serial), Some(5));
    accepted(home.path(), RAISED, RAISED_SIG, KEY).unwrap();
    assert_eq!(kept(home.path()).map(|one| one.serial), Some(6));
}

#[test]
fn a_pause_from_afar_holds_only_until_its_date_and_never_more_than_a_week_ahead() {
    let paused = read(HALTED, HALTED_SIG, KEY).unwrap();
    let until = paused.halted_until.unwrap();

    assert!(halted(Some(&paused), until - 60));
    assert!(!halted(Some(&paused), until));
    assert!(
        !halted(Some(&paused), until - PAUSED_AT_MOST - 1),
        "a pause set past a week would hold for good if its word were lost"
    );
    assert!(!halted(Some(&Policy::default()), 0));
    assert!(!halted(None, 0));
}

#[test]
fn the_person_can_pause_the_cloud_on_this_machine_and_take_it_back() {
    let home = tempfile::tempdir().unwrap();

    pause_here(home.path(), true).unwrap();
    assert!(paused_here(home.path()));
    pause_here(home.path(), false).unwrap();
    pause_here(home.path(), false).unwrap();
    assert!(!paused_here(home.path()));
}
