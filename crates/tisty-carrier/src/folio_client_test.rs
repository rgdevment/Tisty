use std::cell::RefCell;
use std::collections::VecDeque;

use super::{
    Answered, Asked, Claim, Door, First, Refused, Stopped, Stored, Taken, Verb, Version, Why,
    appeal, edit_link, fetch, fetch_files, latest, pause, read_link, resume, send_files,
    send_version, solved, start, status, stop, taken_back, zero_bits_at_start,
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
                headers: Vec::new(),
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
        (
            409,
            r#"{"refused":"languages_full"}"#,
            Refused::LanguagesFull,
        ),
        (409, r#"{"refused":"machines_full"}"#, Refused::MachinesFull),
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
        (403, r#"{"said":"no refusal at all"}"#),
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
fn a_refusal_from_a_later_folio_is_told_as_a_refusal_and_never_as_stored() {
    let folio = FakeFolio::answering(&[(403, r#"{"refused":"said_later"}"#)]);

    assert_eq!(
        send_version(&folio, &next_version()),
        Err(Stopped::Refused(Refused::Unknown))
    );
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

#[test]
fn a_second_stop_answers_missing_and_still_counts_as_stopped() {
    let folio = FakeFolio::answering(&[(404, r#"{"refused":"missing"}"#)]);

    assert_eq!(stop(&folio, "Ab3xK9pQ", "token"), Ok(()));
    assert_eq!(folio.asked.borrow()[0].0, Verb::Delete);
    assert_eq!(folio.asked.borrow()[0].1, "/docs/Ab3xK9pQ");
    assert_eq!(folio.header(0, "Folio-Admin").as_deref(), Some("token"));
}

#[test]
fn managing_carries_the_admin_token_to_each_route() {
    let folio = FakeFolio::answering(&[
        (200, r#"{"done":true}"#),
        (200, r#"{"done":true}"#),
        (
            200,
            r#"{"status":{"latest":2,"diesOn":20000,"pausedAt":null,"blockedAt":null,"appealedAt":null,"mayCorrect":false,"fixBy":null,"readTotal":3,"days":{}}}"#,
        ),
    ]);

    assert_eq!(pause(&folio, "Ab3xK9pQ", "token"), Ok(()));
    assert_eq!(resume(&folio, "Ab3xK9pQ", "token"), Ok(()));
    let told = status(&folio, "Ab3xK9pQ", "token").unwrap();

    assert_eq!(folio.asked.borrow()[0].1, "/docs/Ab3xK9pQ/pause");
    assert_eq!(folio.asked.borrow()[1].1, "/docs/Ab3xK9pQ/resume");
    assert_eq!(folio.asked.borrow()[2].1, "/docs/Ab3xK9pQ/admin");
    assert_eq!((told.latest, told.read_total, told.paused_at), (2, 3, None));
    assert_eq!((told.may_correct, told.fix_by), (false, None));
}

#[test]
fn the_status_tells_a_deadline_and_a_block_that_a_new_version_lifts() {
    let folio = FakeFolio::answering(&[(
        200,
        r#"{"status":{"latest":2,"diesOn":20000,"pausedAt":null,"blockedAt":19990,"appealedAt":19991,"mayCorrect":true,"fixBy":20005,"readTotal":3,"days":{}}}"#,
    )]);

    let told = status(&folio, "Ab3xK9pQ", "token").unwrap();

    assert_eq!((told.may_correct, told.fix_by), (true, Some(20005)));
    assert_eq!(told.appealed_at, Some(19991));
}

#[test]
fn a_refusal_this_version_does_not_know_is_still_a_refusal() {
    let refused: Refused = serde_json::from_str(r#"{"refused":"from_the_future"}"#).unwrap();

    assert_eq!(refused, Refused::Unknown);
}

#[test]
fn the_links_keep_their_secrets_after_the_hash() {
    assert_eq!(
        read_link("Ab3xK9pQ", "key"),
        "https://tisty.cl/Ab3xK9pQ#key"
    );
    assert_eq!(
        edit_link("Ab3xK9pQ", "key", "token"),
        "https://tisty.cl/Ab3xK9pQ#key.token"
    );
}

#[test]
fn it_says_which_version_is_there() {
    let folio = FakeFolio::answering(&[(200, r#"{"version":7,"hash":"h"}"#)]);

    assert_eq!(latest(&folio, "Ab3xK9pQ").map(|one| one.version), Ok(7));
    assert_eq!(folio.asked.borrow()[0].0, Verb::Get);
}

#[test]
fn what_a_root_took_back_goes_as_folio_names_it() {
    let folio = FakeFolio::answering(&[(200, r#"{"done":true}"#)]);

    taken_back(
        &folio,
        &[Taken {
            root: "r".to_owned(),
            machine: "m".to_owned(),
            at: "2026-11-06T12:00:00Z".to_owned(),
            signed: "s".to_owned(),
        }],
    )
    .unwrap();

    assert_eq!(folio.asked.borrow()[0].1, "/revocations");
    let sent: serde_json::Value = serde_json::from_slice(&folio.asked.borrow()[0].3).unwrap();
    assert_eq!(sent[0]["machine"], "m");
    assert_eq!(sent[0]["signed"], "s");
}

#[test]
fn an_appeal_says_how_it_was_taken() {
    let folio = FakeFolio::answering(&[
        (200, r#"{"done":true,"claim":"accepted"}"#),
        (409, r#"{"refused":"appealed"}"#),
    ]);

    assert_eq!(appeal(&folio, "Ab3xK9pQ", "t"), Ok(Claim::Accepted));
    assert_eq!(
        appeal(&folio, "Ab3xK9pQ", "t"),
        Err(Stopped::Refused(Refused::Appealed))
    );
}

#[test]
fn a_document_comes_with_what_its_headers_say() {
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().push_back(Some(Answered {
        status: 200,
        headers: vec![
            ("folio-version".to_owned(), "4".to_owned()),
            ("folio-arrived".to_owned(), "1000".to_owned()),
            ("folio-revocations".to_owned(), "[]".to_owned()),
        ],
        body: b"sealed".to_vec(),
    }));

    let fetched = fetch(&folio, "Ab3xK9pQ").unwrap();

    assert_eq!(fetched.bytes, b"sealed");
    assert_eq!((fetched.version, fetched.arrived_ms), (4, 1000));
    assert_eq!(fetched.revocations.as_deref(), Some("[]"));
}

#[test]
fn a_document_without_its_version_is_not_believed() {
    let folio = FakeFolio::answering(&[(200, "sealed")]);

    assert_eq!(fetch(&folio, "Ab3xK9pQ"), Err(Stopped::Garbled));
}

#[test]
fn a_hash_that_could_change_the_path_is_never_asked_for() {
    let folio = FakeFolio::default();

    assert_eq!(
        fetch_files(&folio, "Ab3xK9pQ", "../admin"),
        Err(Stopped::Garbled)
    );
    assert!(folio.asked.borrow().is_empty());
}

#[test]
fn a_refusal_over_the_terms_reads_the_same_whatever_else_it_says() {
    for said in [
        r#"{"refused":"terms","version":"2"}"#,
        r#"{"refused":"terms","version":2}"#,
        r#"{"refused":"terms"}"#,
    ] {
        let refused: Refused = serde_json::from_str(said).unwrap();
        assert!(matches!(refused, Refused::Terms { .. }), "{said}");
    }
}

#[test]
fn a_refusal_over_the_terms_keeps_the_version_folio_asks_for_however_it_is_written() {
    let version_of = |said: &str| match serde_json::from_str(said).unwrap() {
        Refused::Terms { version } => version,
        other => panic!("{other:?}"),
    };

    assert_eq!(
        version_of(r#"{"refused":"terms","version":"12"}"#).as_deref(),
        Some("12")
    );
    assert_eq!(
        version_of(r#"{"refused":"terms","version":12}"#).as_deref(),
        Some("12")
    );
    assert_eq!(version_of(r#"{"refused":"terms","version":null}"#), None);
    assert_eq!(
        version_of(r#"{"refused":"terms","version":12.0}"#).as_deref(),
        Some("12")
    );
    assert_eq!(version_of(r#"{"refused":"terms","version":12.5}"#), None);
    assert_eq!(version_of(r#"{"refused":"terms"}"#), None);
}

#[test]
fn a_new_key_out_of_room_is_a_refusal_of_its_own() {
    let refused: Refused = serde_json::from_str(r#"{"refused":"room_full"}"#).unwrap();

    assert_eq!(refused, Refused::RoomFull);
}

#[test]
fn a_blocked_key_is_a_refusal_of_its_own() {
    let refused: Refused = serde_json::from_str(r#"{"refused":"key_blocked"}"#).unwrap();

    assert_eq!(refused, Refused::KeyBlocked);
}

#[test]
fn an_envelope_refused_for_a_reason_this_version_does_not_know_is_still_a_refusal() {
    let refused: Refused =
        serde_json::from_str(r#"{"refused":"envelope","why":"from_the_future"}"#).unwrap();

    assert_eq!(refused, Refused::Envelope { why: Why::Unknown });
}
