use super::*;
use crate::event::DeviceId;
use crate::moved::{Leaving, said};

const BY: &str = "01J0000000000000000000000A";

fn word(to: &str) -> String {
    said(
        &crate::signing::SigningKey::from_bytes(&[7; 32]),
        Leaving {
            by: &DeviceId(BY.to_string()),
            to,
            at: 1_760_000_000,
            store: "store",
            place: "folder",
        },
    )
}

#[test]
fn a_folder_left_for_the_cloud_says_where_its_copies_went_in_both_languages_and_how_to_come_back() {
    let place = tempfile::tempdir().unwrap();

    left_notice(place.path(), &word("cloud:ana@example.com"));

    let note = std::fs::read_to_string(place.path().join(NOTICE)).unwrap();
    let name = crate::config::nicknamed(BY);
    assert!(note.contains(&format!("«{name}»")), "{note}");
    assert!(note.contains("Google Drive (ana@example.com)"), "{note}");
    assert!(note.contains("elige esta carpeta"), "{note}");
    assert!(note.contains("choose this folder"), "{note}");
    assert!(
        note.contains("Configuración › Tus datos"),
        "the note names a place the app does not: {note}"
    );
    assert!(
        note.contains("Settings › Your data"),
        "the note names a place the app does not: {note}"
    );
    assert!(note.contains("2025-10-0"), "the date is missing: {note}");
}

#[test]
fn a_folder_left_for_another_folder_or_this_computer_alone_says_so() {
    let to_folder = noticed(&crate::moved::read(&word("folder:Nueva")).unwrap());
    let alone = noticed(&crate::moved::read(&word("alone")).unwrap());

    assert!(to_folder.contains("another folder (Nueva)"), "{to_folder}");
    assert!(alone.contains("only on that computer"), "{alone}");
}

#[test]
fn a_file_of_the_same_name_tisty_did_not_write_is_never_written_over_nor_taken_away() {
    let place = tempfile::tempdir().unwrap();
    let theirs = place.path().join(NOTICE);
    std::fs::write(&theirs, "mis notas").unwrap();

    left_notice(place.path(), &word("alone"));
    notice_gone(place.path());

    assert_eq!(std::fs::read_to_string(&theirs).unwrap(), "mis notas");
}

#[test]
fn coming_back_to_the_folder_takes_tisty_s_note_away_and_leaves_the_rest() {
    let place = tempfile::tempdir().unwrap();
    std::fs::write(place.path().join("otra.txt"), "mía").unwrap();
    left_notice(place.path(), &word("alone"));
    left_notice(place.path(), &word("cloud:ana@example.com"));
    assert!(
        std::fs::read_to_string(place.path().join(NOTICE))
            .unwrap()
            .contains("Google Drive"),
        "a later move did not bring the note up to date"
    );

    notice_gone(place.path());

    assert!(!place.path().join(NOTICE).exists());
    assert!(place.path().join("otra.txt").exists());
}

#[test]
fn a_word_that_does_not_read_leaves_no_note() {
    let place = tempfile::tempdir().unwrap();

    left_notice(place.path(), "no es json");

    assert!(!place.path().join(NOTICE).exists());
}

#[test]
fn the_note_speaks_this_computer_s_language_first_and_the_other_after() {
    let left = crate::moved::read(&word("cloud:ana@example.com")).unwrap();

    let spanish = noticed_in(&left, "es-CL");
    let english = noticed_in(&left, "en-US");

    assert!(
        spanish.starts_with("Tisty · README\n\nEl equipo"),
        "{spanish}"
    );
    assert!(spanish.contains("The computer"), "{spanish}");
    assert!(
        english.starts_with("Tisty · README\n\nThe computer"),
        "{english}"
    );
    assert!(english.contains("El equipo"), "{english}");
}
