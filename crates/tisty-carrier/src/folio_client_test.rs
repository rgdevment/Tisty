use std::cell::RefCell;
use std::collections::VecDeque;

use super::{
    Answered, Asked, Door, First, Refused, Stopped, Stored, Verb, Version, Why, send_files,
    send_version, solved, start, zero_bits_at_start,
};

type Heard = (Verb, String, Vec<(&'static str, String)>, Vec<u8>);

#[derive(Default)]
struct FakeFolio {
    answers: RefCell<VecDeque<Option<Answered>>>,
    asked: RefCell<Vec<Heard>>,
}

impl FakeFolio {
    fn answering(answers: &[(u16, &str)]) -> Self {
        let fake = Self::default();
        for (status, body) in answers {
            fake.answers.borrow_mut().push_back(Some(Answered {
                status: *status,
                body: body.as_bytes().to_vec(),
            }));
        }
        fake
    }

    fn header(&self, at: usize, name: &str) -> Option<String> {
        self.asked.borrow()[at]
            .2
            .iter()
            .find(|(one, _)| *one == name)
            .map(|(_, value)| value.clone())
    }
}

impl Door for FakeFolio {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered> {
        self.asked.borrow_mut().push((
            asked.verb,
            asked.path.clone(),
            asked.headers.clone(),
            asked.body.to_vec(),
        ));
        self.answers.borrow_mut().pop_front().flatten()
    }
}

fn next_version<'a>() -> Version<'a> {
    Version {
        id: "Ab3xK9pQ",
        envelope: b"sealed",
        files_hash: None,
        first: None,
    }
}

#[test]
fn a_proof_solves_what_folios_own_page_solves() {
    assert_eq!(solved("folio-salt", 12), Some(503));
    assert_eq!(solved("otra", 4), Some(13));
}

#[test]
fn a_proof_past_what_folio_may_ask_is_not_attempted() {
    assert_eq!(solved("salt", 33), None);
}

#[test]
fn zero_bits_are_counted_within_a_byte_and_across_bytes() {
    assert!(zero_bits_at_start(&[0, 0b0000_1111], 12));
    assert!(!zero_bits_at_start(&[0, 0b0001_1111], 12));
    assert!(zero_bits_at_start(&[0, 0], 16));
    assert!(!zero_bits_at_start(&[0, 1], 16));
    assert!(zero_bits_at_start(&[255], 0));
}

#[test]
fn a_start_gives_the_id_folio_chose() {
    let folio = FakeFolio::answering(&[(200, r#"{"id":"Ab3xK9pQ","mac":"m","salt":"s"}"#)]);

    let given = start(&folio).unwrap();

    assert_eq!(
        (given.id.as_str(), given.mac.as_str(), given.salt.as_str()),
        ("Ab3xK9pQ", "m", "s")
    );
    assert_eq!(folio.asked.borrow()[0].0, Verb::Post);
    assert_eq!(folio.asked.borrow()[0].1, "/start");
}

#[test]
fn an_id_that_could_change_the_path_is_refused() {
    for id in ["../admin", "short", "Ab3xK9p/", "Ab3xK9pQ1"] {
        let body = format!(r#"{{"id":"{id}","mac":"m","salt":"s"}}"#);
        let folio = FakeFolio::answering(&[(200, &body)]);

        assert_eq!(start(&folio), Err(Stopped::Garbled), "{id}");
    }
}

#[test]
fn the_first_version_brings_all_a_start_asks_for() {
    let folio = FakeFolio::answering(&[(200, r#"{"stored":true,"version":1,"filesWanted":true}"#)]);

    let stored = send_version(
        &folio,
        &Version {
            files_hash: Some("hash"),
            first: Some(First {
                mac: "m",
                admin: "token",
                ttl_days: 3,
                anchor: Some("anchor"),
            }),
            ..next_version()
        },
    );

    assert_eq!(
        stored,
        Ok(Stored {
            version: 1,
            files_wanted: true
        })
    );
    assert_eq!(folio.asked.borrow()[0].0, Verb::Put);
    assert_eq!(folio.asked.borrow()[0].1, "/docs/Ab3xK9pQ");
    assert_eq!(folio.asked.borrow()[0].3, b"sealed");
    assert_eq!(folio.header(0, "Folio-Start").as_deref(), Some("m"));
    assert_eq!(folio.header(0, "Folio-Admin").as_deref(), Some("token"));
    assert_eq!(folio.header(0, "Folio-Ttl-Days").as_deref(), Some("3"));
    assert_eq!(folio.header(0, "Folio-Anchor").as_deref(), Some("anchor"));
    assert_eq!(folio.header(0, "Folio-Files").as_deref(), Some("hash"));
}

#[test]
fn a_later_version_says_nothing_of_what_only_the_first_brings() {
    let folio = FakeFolio::answering(&[(200, r#"{"stored":true,"version":2}"#)]);

    let stored = send_version(&folio, &next_version()).unwrap();

    assert!(!stored.files_wanted);
    assert!(folio.asked.borrow()[0].2.is_empty());
}

#[test]
fn asked_for_a_proof_it_solves_it_and_sends_the_same_request_again() {
    let folio = FakeFolio::answering(&[
        (429, r#"{"refused":"proof","salt":"folio-salt","bits":12}"#),
        (200, r#"{"stored":true,"version":1}"#),
    ]);

    assert_eq!(
        send_version(&folio, &next_version()).map(|one| one.version),
        Ok(1)
    );
    assert_eq!(folio.header(0, "Folio-Proof"), None);
    assert_eq!(folio.header(1, "Folio-Proof").as_deref(), Some("503"));
    assert_eq!(folio.asked.borrow()[1].3, b"sealed");
}

#[test]
fn a_second_salt_replaces_the_first_proof() {
    let folio = FakeFolio::answering(&[
        (429, r#"{"refused":"proof","salt":"folio-salt","bits":12}"#),
        (429, r#"{"refused":"proof","salt":"otra","bits":4}"#),
        (200, r#"{"stored":true,"version":1}"#),
    ]);

    send_version(&folio, &next_version()).unwrap();

    assert_eq!(folio.header(2, "Folio-Proof").as_deref(), Some("13"));
    assert_eq!(
        folio.asked.borrow()[2]
            .2
            .iter()
            .filter(|(name, _)| *name == "Folio-Proof")
            .count(),
        1
    );
}

#[test]
fn a_server_that_never_stops_asking_for_proofs_stops_being_asked() {
    let proof = r#"{"refused":"proof","salt":"otra","bits":4}"#;
    let folio = FakeFolio::answering(&[(429, proof), (429, proof), (429, proof), (429, proof)]);

    assert!(matches!(
        send_version(&folio, &next_version()),
        Err(Stopped::Refused(Refused::Proof { .. }))
    ));
    assert_eq!(folio.asked.borrow().len(), 4);
}

#[test]
fn each_refusal_folio_documents_is_told_apart() {
    let cases = [
        (400, r#"{"refused":"bad_upload"}"#, Refused::BadUpload),
        (403, r#"{"refused":"paused"}"#, Refused::Paused),
        (403, r#"{"refused":"blocked"}"#, Refused::Blocked),
        (403, r#"{"refused":"not_yours"}"#, Refused::NotYours),
        (403, r#"{"refused":"revoked"}"#, Refused::Revoked),
        (404, r#"{"refused":"missing"}"#, Refused::Missing),
        (409, r#"{"refused":"behind"}"#, Refused::Behind),
        (409, r#"{"refused":"shares_full"}"#, Refused::SharesFull),
        (413, r#"{"refused":"too_big"}"#, Refused::TooBig),
        (
            422,
            r#"{"refused":"envelope","why":"key"}"#,
            Refused::Envelope { why: Why::Key },
        ),
        (426, r#"{"refused":"outdated"}"#, Refused::Outdated),
        (
            429,
            r#"{"refused":"slow","until":5}"#,
            Refused::Slow { until: 5 },
        ),
        (
            429,
            r#"{"refused":"quota","until":6}"#,
            Refused::Quota { until: 6 },
        ),
        (500, r#"{"refused":"failed"}"#, Refused::Failed),
        (503, r#"{"refused":"uploads_off"}"#, Refused::UploadsOff),
    ];

    for (status, body, refused) in cases {
        let folio = FakeFolio::answering(&[(status, body)]);

        assert_eq!(
            send_version(&folio, &next_version()),
            Err(Stopped::Refused(refused)),
            "{body}"
        );
    }
}

#[test]
fn what_cannot_be_read_never_passes_for_stored() {
    for (status, body) in [
        (200, "<html>"),
        (200, r#"{"stored":true}"#),
        (502, "Bad gateway"),
        (403, r#"{"refused":"said_later"}"#),
    ] {
        let folio = FakeFolio::answering(&[(status, body)]);

        assert_eq!(
            send_version(&folio, &next_version()),
            Err(Stopped::Garbled),
            "{body}"
        );
    }
}

#[test]
fn a_server_out_of_reach_is_said_apart() {
    let folio = FakeFolio::default();

    assert_eq!(start(&folio), Err(Stopped::Unreached));
    assert_eq!(
        send_version(&folio, &next_version()),
        Err(Stopped::Unreached)
    );
}

#[test]
fn the_files_go_to_their_own_path() {
    let folio = FakeFolio::answering(&[(200, r#"{"stored":true,"hash":"h"}"#)]);

    assert_eq!(send_files(&folio, "Ab3xK9pQ", b"files"), Ok(()));
    assert_eq!(folio.asked.borrow()[0].1, "/docs/Ab3xK9pQ/files");
    assert_eq!(folio.asked.borrow()[0].3, b"files");
}

#[test]
fn files_that_the_latest_version_does_not_name_are_refused() {
    let folio = FakeFolio::answering(&[(409, r#"{"refused":"behind"}"#)]);

    assert_eq!(
        send_files(&folio, "Ab3xK9pQ", b"files"),
        Err(Stopped::Refused(Refused::Behind))
    );
}
