use tisty_envelope::family::Going;
use tisty_envelope::sealing::{Refused, Sealed as Envelopes};
use tisty_sharing::{Told, Unshared};

use super::{doc_id, refused, summed_up};

#[test]
fn each_reason_is_said_in_words_the_window_knows() {
    for (wrong, code) in [
        (Unshared::NoSuchDoc, "noSuchDoc"),
        (Unshared::NotOwn, "notOneOwnDoc"),
        (Unshared::Shut, "documentLocked"),
        (Unshared::NotADocument, "onlyADocumentShared"),
        (Unshared::Unfinished("a".into()), "cannotFinish"),
        (Unshared::Unread("a".into()), "cannotSeal"),
        (Unshared::Unwritten("a".into()), "cannotSeal"),
        (Unshared::Refused(Refused::Unsealed), "cannotSeal"),
    ] {
        assert_eq!(refused(&wrong).code, code);
    }
}

#[test]
fn what_is_no_id_is_no_document() {
    assert_eq!(doc_id("nope").err().unwrap().code, "noSuchDoc");
    let id = ulid::Ulid::generate();
    assert_eq!(doc_id(&id.to_string()).ok(), Some(id));
}

#[test]
fn it_sums_up_all_its_envelopes_and_what_the_person_is_told() {
    let going = |main: usize, files: Option<usize>| Going {
        id: "x".into(),
        sealed: Envelopes {
            main: vec![0; main],
            files: files.map(|size| vec![0; size]),
            files_hash: None,
        },
    };
    let said = summed_up(tisty_sharing::Sealed {
        going: vec![going(10, Some(5)), going(1, None)],
        told: vec![Told::NameLeftOut],
        remembering: vec![],
    });
    assert_eq!(said.envelopes, 3);
    assert_eq!(said.bytes, 16);
    assert_eq!(said.told, [Told::NameLeftOut]);
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn what_only_the_machine_can_read_it_hands_over_and_leaves_nothing_behind() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("picture.png");
    let mut pixels = std::io::Cursor::new(Vec::new());
    image_of_one_colour(&mut pixels);
    std::fs::write(&at, pixels.into_inner()).unwrap();

    let bytes = super::turned(&at).expect("this machine reads a PNG");
    assert!(bytes.starts_with(&[0xFF, 0xD8, 0xFF]));
    let made = tisty_envelope::pictures::made_or_turned(&bytes, || None).unwrap();
    assert_eq!(made.kind, tisty_envelope::pictures::Kind::Jpeg);
}

#[cfg(any(windows, target_os = "macos"))]
fn image_of_one_colour(into: &mut std::io::Cursor<Vec<u8>>) {
    // A real picture, one pixel wide, written so no decoder here is needed to make it.
    use base64::Engine;
    let said = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEBgIApD5fRAAAAABJRU5ErkJggg==";
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(said)
        .unwrap();
    std::io::Write::write_all(into, &bytes).unwrap();
}
