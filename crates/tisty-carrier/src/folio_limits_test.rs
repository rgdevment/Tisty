use std::cell::Cell;
use std::time::{Duration, SystemTime};

use super::{
    ASKED, ASKED_EVERY, FolioLimits, FolioMaker, FolioNote, KEPT, LIMITS_URL, NOTE_AT_MOST,
    NOTE_LINES_AT_MOST, SIGNATURE_KEPT, SIGNATURE_URL, SIGNED_KEPT, SharingNow, accepted, heed,
    heed_now, kept, maker_root, read_under, sharing_now, uploads_halted,
};
use crate::policy::{PAUSED_AT_MOST, READ_AT_MOST, Refused};

// What Folio really published and signed with its own script, not with the minisign tool.
const REAL: &[u8] = include_bytes!("folio_signed/limits.json");
const REAL_SIG: &str = include_str!("folio_signed/limits.json.minisig");
const KEY: &str = super::KEY;
const CLOSED: SharingNow = SharingNow::Closed { note: None };

const OTHER_KEY: &str = include_str!("cloud/signed/key.pub");
const OTHER: &[u8] = include_bytes!("cloud/signed/lowered.json");
const OTHER_SIG: &str = include_str!("cloud/signed/lowered.json.minisig");

// As a Tisty that accepted signed limits keeps them: the copy it warns from, and the signed one beside it.
fn kept_as(home: &std::path::Path, limits: &FolioLimits) {
    std::fs::write(home.join(KEPT), serde_json::to_vec(limits).unwrap()).unwrap();
    std::fs::write(home.join(SIGNED_KEPT), REAL).unwrap();
    std::fs::write(home.join(SIGNATURE_KEPT), REAL_SIG).unwrap();
}

fn seeded(serial: u64) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        serial,
        ..FolioLimits::default()
    };
    kept_as(home.path(), &limits);
    home
}

fn opened(home: &std::path::Path, serial: u64) {
    let limits = FolioLimits {
        serial,
        sharing_open: Some(true),
        ..FolioLimits::default()
    };
    kept_as(home, &limits);
}

fn answering_real(url: &str) -> Option<Vec<u8>> {
    match url {
        LIMITS_URL => Some(REAL.to_vec()),
        SIGNATURE_URL => Some(REAL_SIG.as_bytes().to_vec()),
        _ => None,
    }
}

#[test]
fn signed_limits_that_say_nothing_of_sharing_leave_it_closed() {
    let home = tempfile::tempdir().unwrap();

    assert_eq!(sharing_now(home.path(), answering_real), CLOSED);
}

fn closed_saying(es: Option<String>, en: Option<String>) -> SharingNow {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        serial: 1,
        sharing_closed_note: Some(FolioNote { es, en }),
        ..FolioLimits::default()
    };
    kept_as(home.path(), &limits);
    std::fs::write(home.path().join(ASKED), b"").unwrap();
    sharing_now(home.path(), |_| None)
}

#[test]
fn what_folio_signs_to_say_while_closed_comes_along_without_its_spaces() {
    let told = closed_saying(Some("  Abrimos el lunes.\n".to_owned()), None);

    assert_eq!(
        told,
        SharingNow::Closed {
            note: Some(FolioNote {
                es: Some("Abrimos el lunes.".to_owned()),
                en: None,
            }),
        }
    );
}

fn said_back(words: &str) -> Option<String> {
    let SharingNow::Closed { note: Some(note) } = closed_saying(Some(words.to_owned()), None)
    else {
        panic!("it was not closed with a note");
    };
    note.es
}

#[test]
fn a_note_keeps_its_line_breaks_up_to_the_most_lines_it_may_have() {
    let most = ["a"; NOTE_LINES_AT_MOST].join("\n");

    assert_eq!(said_back("uno\r\n\r\ndos").as_deref(), Some("uno\n\ndos"));
    assert_eq!(said_back(&most), Some(most.clone()));
    assert_eq!(said_back(&format!("{most}\na")), None);
}

#[test]
fn a_note_with_characters_that_hide_or_turn_text_is_left_out_whole() {
    for hidden in [
        "a\u{202e}b",
        "a\u{200b}b",
        "a\u{2066}b",
        "a\tb",
        "a\u{0007}b",
        "a\u{feff}b",
    ] {
        assert_eq!(said_back(hidden), None, "{hidden:?}");
    }
}

#[test]
fn a_blank_or_oversized_word_from_folio_leaves_the_standard_notice() {
    let longest = "a".repeat(NOTE_AT_MOST);
    let told = closed_saying(Some("   ".to_owned()), Some(format!("{longest}a")));
    assert_eq!(
        told,
        SharingNow::Closed {
            note: Some(FolioNote::default()),
        }
    );

    let told = closed_saying(None, Some(longest.clone()));
    assert_eq!(
        told,
        SharingNow::Closed {
            note: Some(FolioNote {
                es: None,
                en: Some(longest),
            }),
        }
    );
}

#[test]
fn signed_limits_that_open_sharing_are_believed_within_the_hour_without_asking() {
    let home = tempfile::tempdir().unwrap();
    opened(home.path(), 1);
    std::fs::write(home.path().join(ASKED), b"").unwrap();

    let told = sharing_now(home.path(), |_| panic!("it asked within the hour"));

    assert_eq!(told, SharingNow::Open);
}

#[test]
fn closed_is_kept_within_the_hour_without_asking() {
    let home = seeded(1);
    std::fs::write(home.path().join(ASKED), b"").unwrap();

    let told = sharing_now(home.path(), |_| panic!("it asked within the hour"));

    assert_eq!(told, CLOSED);
}

#[test]
fn past_the_hour_an_unreached_look_is_not_taken_from_what_is_kept() {
    let home = tempfile::tempdir().unwrap();
    opened(home.path(), 1);

    assert_eq!(sharing_now(home.path(), |_| None), SharingNow::Unreached);
}

#[test]
fn a_look_that_reached_nothing_does_not_hold_back_the_next_try() {
    let home = tempfile::tempdir().unwrap();

    assert_eq!(sharing_now(home.path(), |_| None), SharingNow::Unreached);
    assert_eq!(sharing_now(home.path(), answering_real), CLOSED);
}

#[test]
fn a_bad_signature_never_opens_sharing() {
    let home = tempfile::tempdir().unwrap();
    opened(home.path(), 1);
    let bring = |url: &str| match url {
        LIMITS_URL => Some(OTHER.to_vec()),
        _ => Some(REAL_SIG.as_bytes().to_vec()),
    };

    assert_eq!(sharing_now(home.path(), bring), SharingNow::Unreached);
}

#[test]
fn an_older_file_leaves_what_is_kept_standing() {
    let home = tempfile::tempdir().unwrap();
    opened(home.path(), u64::MAX);

    assert_eq!(sharing_now(home.path(), answering_real), SharingNow::Open);
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
fn asked_now_it_reads_again_past_the_hourly_mark() {
    let home = tempfile::tempdir().unwrap();
    let bring = |url: &str| match url {
        LIMITS_URL => Some(REAL.to_vec()),
        SIGNATURE_URL => Some(REAL_SIG.as_bytes().to_vec()),
        _ => None,
    };

    assert!(matches!(heed(home.path(), bring), Some(Ok(_))));
    assert_eq!(heed(home.path(), bring), None);
    assert!(matches!(heed_now(home.path(), bring), Some(Ok(_))));
}

#[test]
fn it_reads_the_terms_folio_signs_and_keeps_them() {
    let limits: FolioLimits = serde_json::from_str(
        r#"{"serial": 4, "terms": {"version": "2026-11-06", "es": "a", "en": "b"}}"#,
    )
    .unwrap();

    let terms = limits.terms.unwrap();
    assert_eq!(
        (terms.version.as_str(), terms.es.as_str(), terms.en.as_str()),
        ("2026-11-06", "a", "b")
    );
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

#[test]
fn the_maker_root_folio_signed_is_kept_with_the_rest() {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        serial: 1,
        maker: Some(FolioMaker {
            root: "AfnGoRnNecrr5RMnprcTbdZdk7_LABXtpqchY-L0b0s".into(),
            ..FolioMaker::default()
        }),
        ..FolioLimits::default()
    };
    std::fs::write(home.path().join(KEPT), serde_json::to_vec(&limits).unwrap()).unwrap();

    assert_eq!(kept(home.path()), Some(limits));
    assert_eq!(kept(seeded(1).path()).and_then(|one| one.maker), None);
}

#[test]
fn only_the_maker_finds_sharing_open_while_folio_is_closed() {
    let note = Some(FolioNote {
        es: Some("Pronto".into()),
        en: None,
    });
    let closed = SharingNow::Closed { note };

    assert_eq!(closed.clone().opened_to(false), closed);
    assert_eq!(closed.opened_to(true), SharingNow::Open);
    assert_eq!(SharingNow::Open.opened_to(false), SharingNow::Open);
    assert_eq!(SharingNow::Unreached.opened_to(true), SharingNow::Unreached);
}

#[test]
fn the_maker_is_read_only_from_what_folio_signed() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(maker_root(home.path()), None);

    accepted(home.path(), REAL, REAL_SIG).unwrap();
    assert!(home.path().join(SIGNED_KEPT).is_file());
    assert_eq!(maker_root(home.path()), None);

    let claimed = FolioLimits {
        serial: 99,
        maker: Some(FolioMaker {
            root: "AfnGoRnNecrr5RMnprcTbdZdk7_LABXtpqchY-L0b0s".into(),
            ..FolioMaker::default()
        }),
        ..FolioLimits::default()
    };
    let forged = serde_json::to_vec(&claimed).unwrap();
    std::fs::write(home.path().join(KEPT), &forged).unwrap();
    assert_eq!(
        maker_root(home.path()),
        None,
        "the copy kept for warnings vouches for nobody"
    );

    std::fs::write(home.path().join(SIGNED_KEPT), &forged).unwrap();
    assert_eq!(
        maker_root(home.path()),
        None,
        "words the signature does not cover"
    );

    std::fs::write(home.path().join(SIGNED_KEPT), REAL).unwrap();
    std::fs::remove_file(home.path().join(SIGNATURE_KEPT)).unwrap();
    assert_eq!(maker_root(home.path()), None);
}

#[test]
fn the_maker_comes_as_one_object_with_its_root_and_optionally_its_name_and_domain() {
    let said = br#"{"serial":30,"maker":{"root":"AfnGoRnNecrr5RMnprcTbdZdk7_LABXtpqchY-L0b0s","name":"rgdevment","domain":"rgdevment.dev"}}"#;
    let limits: FolioLimits = serde_json::from_slice(said).unwrap();
    let maker = limits.maker.unwrap();
    assert_eq!(maker.root, "AfnGoRnNecrr5RMnprcTbdZdk7_LABXtpqchY-L0b0s");
    assert_eq!(maker.name.as_deref(), Some("rgdevment"));
    assert_eq!(maker.domain.as_deref(), Some("rgdevment.dev"));

    let bare: FolioLimits = serde_json::from_slice(br#"{"serial":30,"maker_root":"x"}"#).unwrap();
    assert_eq!(bare.maker, None, "the field it once was names nobody");
}

#[test]
fn a_tisty_that_never_kept_the_signed_copy_asks_for_it_before_the_hour_is_up() {
    let home = tempfile::tempdir().unwrap();
    let before = FolioLimits {
        serial: 1,
        sharing_open: Some(true),
        ..FolioLimits::default()
    };
    std::fs::write(home.path().join(KEPT), serde_json::to_vec(&before).unwrap()).unwrap();
    std::fs::write(home.path().join(ASKED), b"").unwrap();
    let asked = Cell::new(0);
    let counting = |url: &str| {
        asked.set(asked.get() + 1);
        answering_real(url)
    };

    sharing_now(home.path(), counting);
    assert!(
        asked.get() > 0,
        "no signed copy is kept yet, so the limits are asked for now"
    );
    assert!(home.path().join(SIGNED_KEPT).is_file());

    asked.set(0);
    std::fs::write(home.path().join(ASKED), b"").unwrap();
    sharing_now(home.path(), counting);
    assert_eq!(
        asked.get(),
        0,
        "with the signed copy kept, the hour holds again"
    );
}
