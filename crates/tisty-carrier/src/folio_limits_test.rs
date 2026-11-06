use std::cell::Cell;
use std::time::{Duration, SystemTime};

use super::{
    ASKED, ASKED_EVERY, FolioLimits, KEPT, LIMITS_URL, SIGNATURE_URL, accepted, heed, kept,
    read_under, uploads_halted,
};
use crate::policy::{PAUSED_AT_MOST, READ_AT_MOST, Refused};

// What Folio really published and signed with its own script, not with the minisign tool.
const REAL: &[u8] = include_bytes!("folio_signed/limits.json");
const REAL_SIG: &str = include_str!("folio_signed/limits.json.minisig");
const KEY: &str = super::KEY;

const OTHER_KEY: &str = include_str!("cloud/signed/key.pub");
const OTHER: &[u8] = include_bytes!("cloud/signed/lowered.json");
const OTHER_SIG: &str = include_str!("cloud/signed/lowered.json.minisig");

fn seeded(serial: u64) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        serial,
        ..FolioLimits::default()
    };
    std::fs::write(home.path().join(KEPT), serde_json::to_vec(&limits).unwrap()).unwrap();
    home
}

#[test]
fn the_signature_folio_really_made_is_accepted_under_its_pinned_key() {
    let limits = read_under(REAL, REAL_SIG, KEY).unwrap();

    assert!(limits.serial >= 1);
    assert!(limits.files_at_most.is_some());
    assert!(limits.files_bytes_at_most.is_some());
}

#[test]
fn one_changed_byte_is_refused_before_any_field_is_believed() {
    let mut tampered = REAL.to_vec();
    let near_end = tampered.len() - 4;
    tampered[near_end] = b'9';

    assert_eq!(read_under(&tampered, REAL_SIG, KEY), Err(Refused::Unsigned));
    assert_eq!(
        read_under(REAL, "not a signature", KEY),
        Err(Refused::Unsigned)
    );
    assert_eq!(
        read_under(REAL, REAL_SIG, OTHER_KEY),
        Err(Refused::Unsigned)
    );
}

#[test]
fn a_file_signed_under_another_name_cannot_stand_in_for_the_limits() {
    assert_eq!(
        read_under(OTHER, OTHER_SIG, OTHER_KEY),
        Err(Refused::Unsigned)
    );
}

#[test]
fn past_the_size_it_is_refused_without_looking_further() {
    assert_eq!(
        read_under(&vec![b' '; READ_AT_MOST + 1], REAL_SIG, KEY),
        Err(Refused::TooBig)
    );
}

#[test]
fn a_field_it_does_not_know_is_left_aside() {
    let limits: FolioLimits =
        serde_json::from_str(r#"{"serial": 4, "files_at_most": 3, "said_later": {"a": 1}}"#)
            .unwrap();

    assert_eq!((limits.serial, limits.files_at_most), (4, Some(3)));
}

#[test]
fn an_older_serial_never_replaces_a_newer_one() {
    let real = read_under(REAL, REAL_SIG, KEY).unwrap().serial;
    let home = seeded(real + 1);

    assert_eq!(
        accepted(home.path(), REAL, REAL_SIG),
        Err(Refused::Behind {
            kept: real + 1,
            said: real
        })
    );
    assert_eq!(kept(home.path()).map(|one| one.serial), Some(real + 1));
}

#[test]
fn the_same_or_a_newer_serial_is_kept() {
    let real = read_under(REAL, REAL_SIG, KEY).unwrap().serial;

    for before in [real - 1, real] {
        let home = seeded(before);
        accepted(home.path(), REAL, REAL_SIG).unwrap();
        assert_eq!(kept(home.path()).map(|one| one.serial), Some(real));
    }
}

#[test]
fn a_pause_holds_only_while_ahead_and_never_past_a_week() {
    let now = 1_000;
    let until = |at| FolioLimits {
        uploads_halted_until: Some(at),
        ..FolioLimits::default()
    };

    assert!(uploads_halted(Some(&until(now + 1)), now));
    assert!(uploads_halted(Some(&until(now + PAUSED_AT_MOST)), now));
    assert!(!uploads_halted(Some(&until(now)), now));
    assert!(!uploads_halted(Some(&until(now + PAUSED_AT_MOST + 1)), now));
    assert!(!uploads_halted(Some(&FolioLimits::default()), now));
    assert!(!uploads_halted(None, now));
}

#[test]
fn it_asks_at_most_once_an_hour_and_takes_what_is_signed() {
    let home = tempfile::tempdir().unwrap();
    let asked = Cell::new(0);
    let bring = |url: &str| {
        asked.set(asked.get() + 1);
        match url {
            LIMITS_URL => Some(REAL.to_vec()),
            SIGNATURE_URL => Some(REAL_SIG.as_bytes().to_vec()),
            _ => None,
        }
    };

    assert!(matches!(heed(home.path(), bring), Some(Ok(_))));
    assert!(kept(home.path()).is_some());
    assert_eq!(asked.get(), 2);

    assert_eq!(heed(home.path(), bring), None);
    assert_eq!(asked.get(), 2);
}

#[test]
fn what_is_kept_stands_when_nothing_arrives_or_the_signature_fails() {
    let real = read_under(REAL, REAL_SIG, KEY).unwrap().serial;

    let unreached = seeded(real);
    assert_eq!(heed(unreached.path(), |_| None), None);
    assert_eq!(kept(unreached.path()).map(|one| one.serial), Some(real));

    let forged = seeded(real);
    let bring = |url: &str| match url {
        LIMITS_URL => Some(OTHER.to_vec()),
        _ => Some(REAL_SIG.as_bytes().to_vec()),
    };
    assert_eq!(heed(forged.path(), bring), Some(Err(Refused::Unsigned)));
    assert_eq!(kept(forged.path()).map(|one| one.serial), Some(real));
}

#[test]
fn a_file_that_met_the_signature_of_another_publish_is_read_again_once() {
    let home = tempfile::tempdir().unwrap();
    let asked = Cell::new(0);
    let bring = |url: &str| {
        asked.set(asked.get() + 1);
        match (url, asked.get()) {
            (LIMITS_URL, 1) => Some(OTHER.to_vec()),
            (LIMITS_URL, _) => Some(REAL.to_vec()),
            _ => Some(REAL_SIG.as_bytes().to_vec()),
        }
    };

    assert!(matches!(heed(home.path(), bring), Some(Ok(_))));
    assert_eq!(asked.get(), 4);
}

#[test]
fn a_signature_that_keeps_failing_is_read_again_only_once() {
    let home = tempfile::tempdir().unwrap();
    let asked = Cell::new(0);
    let bring = |url: &str| {
        asked.set(asked.get() + 1);
        match url {
            LIMITS_URL => Some(OTHER.to_vec()),
            _ => Some(REAL_SIG.as_bytes().to_vec()),
        }
    };

    assert_eq!(heed(home.path(), bring), Some(Err(Refused::Unsigned)));
    assert_eq!(asked.get(), 4);
}

#[test]
fn past_the_hour_it_asks_again() {
    let home = tempfile::tempdir().unwrap();
    let bring = |url: &str| match url {
        LIMITS_URL => Some(REAL.to_vec()),
        _ => Some(REAL_SIG.as_bytes().to_vec()),
    };
    assert!(heed(home.path(), bring).is_some());
    assert!(heed(home.path(), bring).is_none());

    let marked = std::fs::File::options()
        .write(true)
        .open(home.path().join(ASKED))
        .unwrap();
    let long_ago = SystemTime::now() - ASKED_EVERY - Duration::from_secs(60);
    marked.set_modified(long_ago).unwrap();
    drop(marked);

    assert!(heed(home.path(), bring).is_some());
    assert!(heed(home.path(), bring).is_none());
}

#[test]
fn a_mark_dated_ahead_costs_one_read_and_then_holds_again() {
    let home = tempfile::tempdir().unwrap();
    let bring = |url: &str| match url {
        LIMITS_URL => Some(REAL.to_vec()),
        _ => Some(REAL_SIG.as_bytes().to_vec()),
    };
    assert!(heed(home.path(), bring).is_some());
    let marked = std::fs::File::options()
        .write(true)
        .open(home.path().join(ASKED))
        .unwrap();
    marked
        .set_modified(SystemTime::now() + Duration::from_secs(24 * 60 * 60))
        .unwrap();
    drop(marked);

    assert!(heed(home.path(), bring).is_some());
    assert!(heed(home.path(), bring).is_none());
}

#[test]
fn limits_that_passed_their_signature_are_given_even_if_they_cannot_be_kept() {
    let home = tempfile::tempdir().unwrap();
    let not_a_folder = home.path().join("a-file");
    std::fs::write(&not_a_folder, b"").unwrap();

    assert!(accepted(&not_a_folder, REAL, REAL_SIG).is_ok());
    assert_eq!(kept(&not_a_folder), None);
}
