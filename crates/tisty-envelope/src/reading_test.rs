use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use tisty_contracts::{
    Document, EnvelopeKind, EnvelopeReader, Opening, Orientation, Paper, Sealing, Served, VERSION,
    Verdict, check,
};
use tisty_share_proto::identity::{Root, ShareKey};

use super::{
    Reader, Unopened, editors_kept, files_for, laid, later_than_this, opened, opens_under_this,
    picture_holds, reading, under_the_floor,
};
use crate::sealing::Writer;

const VECTORS: &str = include_str!("../../../contracts/vectors/vectors.json");

fn all() -> Value {
    serde_json::from_str(VECTORS).unwrap()
}

fn hex(said: &Value) -> Vec<u8> {
    let said = said.as_str().unwrap();
    (0..said.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&said[at..at + 2], 16).unwrap())
        .collect()
}

fn asked(id: &str, kind: EnvelopeKind, link_key: Option<&str>) -> Opening {
    Opening {
        id: id.to_owned(),
        kind,
        link_key: link_key.map(str::to_owned),
    }
}

fn nothing_served() -> Served {
    Served {
        received: None,
        revocations: None,
        reviewed: None,
    }
}

fn main_of(all: &Value) -> Opening {
    asked("k7Qm2xPa", EnvelopeKind::Main, all["link_key"].as_str())
}

fn files_of(all: &Value) -> Opening {
    asked("k7Qm2xPa", EnvelopeKind::Files, all["link_key"].as_str())
}

fn hostile(kind: EnvelopeKind) -> Opening {
    asked("hostile", kind, None)
}

#[test]
fn it_reads_the_samples_as_the_contract_writes_them() {
    let all = all();
    let read = reading(
        &hex(&all["main"]["envelope"]),
        &main_of(&all),
        &nothing_served(),
    )
    .unwrap();
    assert_eq!(read.document, all["main"]["document"]);
    assert_eq!(read.verdict, Verdict::Holds);
    let pictures: Vec<Vec<u8>> = all["main"]["pictures"]
        .as_array()
        .unwrap()
        .iter()
        .map(hex)
        .collect();
    assert_eq!(read.pictures, pictures);
    assert!(picture_holds(
        &read.pictures[0],
        &read.document["pictures"][0]
    ));

    let open = files_for(
        &read.document,
        &hex(&all["files"]["envelope"]),
        &files_of(&all),
        &nothing_served(),
    )
    .unwrap();
    let said: Vec<Vec<u8>> = all["files"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(hex)
        .collect();
    assert_eq!(open.parts, said);
}

#[test]
fn it_opens_what_is_public_without_a_key() {
    let all = all();
    let public = asked("wordsOnly", EnvelopeKind::Main, None);
    let read = reading(&hex(&all["public"]["envelope"]), &public, &nothing_served()).unwrap();
    assert_eq!(read.document, all["public"]["document"]);
}

#[test]
fn it_refuses_all_a_reader_refuses() {
    let all = all();
    let main = hostile(EnvelopeKind::Main);
    for one in all["refused"].as_array().unwrap() {
        let read = reading(&hex(&one["envelope"]), &main, &nothing_served());
        assert!(read.is_err(), "{}", one["says"]);
    }
    assert!(reading(&hex(&all["past_its_end"]), &main, &nothing_served()).is_err());
}

#[test]
fn it_refuses_files_a_document_does_not_answer_for() {
    let all = all();
    let held = reading(
        &hex(&all["main"]["envelope"]),
        &main_of(&all),
        &nothing_served(),
    )
    .unwrap()
    .document;
    for one in all["files_refused"].as_array().unwrap() {
        let read = files_for(
            &held,
            &hex(&one["files"]),
            &files_of(&all),
            &nothing_served(),
        );
        assert!(read.is_err(), "{}", one["says"]);
    }
    let files = hostile(EnvelopeKind::Files);
    for one in all["files_lying"].as_array().unwrap() {
        let read = files_for(
            &one["document"],
            &hex(&one["files"]),
            &files,
            &nothing_served(),
        );
        assert!(read.is_err(), "{}", one["says"]);
    }
}

#[test]
fn only_a_later_minor_of_the_same_major_says_more_than_this_writer_knows() {
    let mut said = VERSION.split('.');
    let major: u64 = said.next().unwrap().parse().unwrap();
    let minor: u64 = said.next().unwrap().parse().unwrap();
    assert!(later_than_this(&format!("{major}.{}.0", minor + 1)));
    assert!(!later_than_this(VERSION));
    assert!(!later_than_this(&format!("{major}.{minor}.9")));
    assert!(!later_than_this(&format!("{}.0.0", major + 1)));
    assert!(!later_than_this(&format!(
        "{major}.{}.0-draft.1",
        minor + 1
    )));
    assert!(!later_than_this("no version"));
}

#[test]
fn a_floor_weighs_major_and_minor_and_takes_an_unknown_seal_as_under_it() {
    assert!(under_the_floor(Some("1.2.0"), "1.3.0"));
    assert!(!under_the_floor(Some("1.3.0"), "1.3.0"));
    assert!(!under_the_floor(Some("1.3.9"), "1.3.0"));
    assert!(!under_the_floor(Some("1.3.0"), "1.3.5"));
    assert!(!under_the_floor(Some("2.0.0"), "1.9.0"));
    assert!(under_the_floor(None, "1.3.0"));
    assert!(under_the_floor(Some("garbled"), "1.3.0"));
    assert!(!under_the_floor(None, "no floor"));
}

#[test]
fn it_opens_what_a_later_minor_only_adds() {
    let all = all();
    let main = hostile(EnvelopeKind::Main);
    for one in all["opening"].as_array().unwrap() {
        let read = reading(&hex(&one["envelope"]), &main, &nothing_served());
        assert!(read.is_ok(), "{}: {:?}", one["says"], read.err());
    }
}

#[test]
fn a_newer_version_is_told_from_a_damaged_one() {
    let all = all();
    let read = reading(
        &hex(&all["a_newer_major"]),
        &hostile(EnvelopeKind::Main),
        &nothing_served(),
    );
    assert_eq!(read.err(), Some(Unopened::Newer));
}

fn under(version: &str) -> Vec<u8> {
    let manifest = serde_json::to_vec(&json!({ "contract": version })).unwrap();
    let mut out = b"TENV".to_vec();
    out.extend_from_slice(&u32::try_from(manifest.len()).unwrap().to_be_bytes());
    out.extend_from_slice(&manifest);
    out
}

#[test]
fn the_first_version_shared_counts_only_under_its_own_major() {
    assert_eq!(opens_under_this("2.0.0", "2.0.0"), Ok(()));
    assert_eq!(opens_under_this("2.1.0", "2.0.0"), Ok(()));
    assert_eq!(opens_under_this("1.2.0", "1.5.0"), Ok(()));
    assert_eq!(opens_under_this("1.1.0", "1.5.0"), Err(Unopened::TooOld));
    assert_eq!(opens_under_this("1.9.0", "2.0.0"), Err(Unopened::TooOld));
}

#[test]
fn a_version_has_one_spelling() {
    for said in ["1", "1.x.0", "1.+2.0", "01.2.0", "+1.2.0", "1.02.0"] {
        let read = opens_under_this(said, "1.2.0");
        assert!(matches!(read, Err(Unopened::Damaged(_))), "{said}");
    }
}

#[test]
fn files_that_do_not_fit_under_a_later_minor_read_as_newer() {
    let all = all();
    let said = &all["newer_files"];
    let read = files_for(
        &said["document"],
        &hex(&said["files"]),
        &hostile(EnvelopeKind::Files),
        &nothing_served(),
    );
    assert_eq!(read.err(), Some(Unopened::Newer));
}

#[test]
fn a_version_is_read_before_anything_else_in_the_manifest() {
    let any = asked("x", EnvelopeKind::Main, None);
    let read = |version: &str| opened(&under(version), &any, &nothing_served()).err();
    assert_eq!(read("2.0.0"), Some(Unopened::Newer));
    assert_eq!(read("99"), Some(Unopened::Newer));
    assert_eq!(read("0.9.0"), Some(Unopened::TooOld));
    assert_eq!(read("1.0.0-draft.8"), Some(Unopened::Draft));
    assert_eq!(read("2.0.0-draft.1"), Some(Unopened::Draft));
    assert!(matches!(read("uno"), Some(Unopened::Damaged(_))));
    // A manifest that says nothing is what stops these: damaged here, and newer under a later minor.
    assert!(matches!(
        read(tisty_contracts::VERSION),
        Some(Unopened::Damaged(_))
    ));
    assert_eq!(read("1.99.0"), Some(Unopened::Newer));
    assert_eq!(read("1.1.0"), Some(Unopened::TooOld));
}

#[test]
fn it_weighs_a_signature_by_what_the_server_serves() {
    let all = all();
    let bytes = hex(&all["main"]["envelope"]);
    let signed_by = &all["signed_by"];
    for one in all["verdicts"].as_array().unwrap() {
        let served: Served = serde_json::from_value(one["served"].clone()).unwrap();
        let verdict = opened(&bytes, &main_of(&all), &served).unwrap().verdict;
        assert_eq!(
            serde_json::to_value(verdict).unwrap(),
            one["verdict"],
            "{} signed by {signed_by}",
            one["says"]
        );
    }
}

#[test]
fn a_signature_that_does_not_hold_shows_no_words() {
    let all = all();
    let mut bytes = hex(&all["main"]["envelope"]);
    let at = bytes
        .windows(8)
        .position(|one| one == b"\"signed\"")
        .unwrap()
        + 11;
    bytes[at] = if bytes[at] == b'A' { b'B' } else { b'A' };
    let read = opened(&bytes, &main_of(&all), &nothing_served());
    assert!(
        matches!(read, Err(Unopened::Signature(_))),
        "{:?}",
        read.err()
    );
}

#[test]
fn another_key_does_not_open_it_and_a_key_answers_only_for_what_is_sealed() {
    let all = all();
    let another = URL_SAFE_NO_PAD.encode([0u8; 32]);
    let main = asked("k7Qm2xPa", EnvelopeKind::Main, Some(&another));
    // Told by its commitment, before anything is opened.
    assert_eq!(
        opened(&hex(&all["main"]["envelope"]), &main, &nothing_served()).err(),
        Some(Unopened::Damaged("it was sealed with another key"))
    );
    let public = asked("wordsOnly", EnvelopeKind::Main, all["link_key"].as_str());
    assert!(opened(&hex(&all["public"]["envelope"]), &public, &nothing_served()).is_err());
    let no_key = asked("k7Qm2xPa", EnvelopeKind::Main, None);
    assert!(opened(&hex(&all["main"]["envelope"]), &no_key, &nothing_served()).is_err());
}

#[test]
fn neither_another_document_nor_another_kind_passes_for_the_one_asked_for() {
    let all = all();
    let other = asked("other", EnvelopeKind::Main, None);
    assert!(opened(&hex(&all["public"]["envelope"]), &other, &nothing_served()).is_err());
    assert!(
        opened(
            &hex(&all["files"]["envelope"]),
            &main_of(&all),
            &nothing_served()
        )
        .is_err()
    );
}

#[test]
fn changing_or_cutting_it_does_not_open_it() {
    let all = all();
    let main = main_of(&all);
    let whole = hex(&all["main"]["envelope"]);
    let mut last = whole.clone();
    *last.last_mut().unwrap() ^= 1;
    assert!(opened(&last, &main, &nothing_served()).is_err());
    let mut manifest = whole.clone();
    manifest[40] ^= 1;
    assert!(opened(&manifest, &main, &nothing_served()).is_err());
    assert!(opened(&whole[..whole.len() - 1], &main, &nothing_served()).is_err());
    let mut longer = whole.clone();
    longer.push(0);
    assert!(opened(&longer, &main, &nothing_served()).is_err());
}

#[test]
fn it_looks_at_a_picture_before_it_draws_it() {
    let all = all();
    for one in all["pictures"]["hold"].as_array().unwrap() {
        assert!(picture_holds(&hex(&one["bytes"]), one), "{}", one["kind"]);
    }
    for one in all["pictures"]["fail"].as_array().unwrap() {
        assert!(!picture_holds(&hex(&one["bytes"]), one), "{}", one["says"]);
    }
}

#[test]
fn a_paper_it_does_not_know_is_read_as_a4_and_never_fails() {
    let unknown = json!({
        "id": "x",
        "html": "<p>x</p>",
        "body": "x",
        "sheet": { "paper": "a3", "orientation": "diagonal", "on": true },
    });
    assert!(check("document", &unknown).is_empty());
    assert!(serde_json::from_value::<Document>(unknown.clone()).is_err());
    let read = laid(&unknown);
    assert_eq!(read.paper, Paper::A4);
    assert_eq!(read.orientation, Orientation::Portrait);
    assert!(read.on);

    let said = laid(&json!({ "sheet": { "paper": "letter", "orientation": "landscape" } }));
    assert_eq!(said.paper, Paper::Letter);
    assert_eq!(said.orientation, Orientation::Landscape);
    assert!(!said.on);
    assert_eq!(laid(&json!({})).paper, Paper::A4);
}

#[test]
fn what_tisty_seals_for_itself_it_also_reads() {
    let key = ShareKey::from_bytes(&[0x44; 32]);
    let certificate = Root::from_bytes(&[0x33; 32])
        .certify(&key.public(), "2026-11-03T12:00:00Z".parse().unwrap());
    let mut writer = Writer::new(&key, &certificate);
    let document = serde_json::from_value(json!({ "id": "x", "html": "<p>x</p>", "body": "x" }));
    let sealing = Sealing {
        id: "x".into(),
        version: 1,
        from: None,
        link_key: None,
        files_hash: None,
        files_key: None,
        owner: None,
        token: None,
        terms: None,
    };
    let sealed = writer
        .sealed(document.unwrap(), vec![], vec![], &sealing)
        .unwrap();
    let main = asked("x", EnvelopeKind::Main, None);
    let read = reading(&sealed.main, &main, &nothing_served()).unwrap();
    assert_eq!(read.verdict, Verdict::Holds);
    assert_eq!(laid(&read.document).paper, Paper::A4);
}

#[test]
fn the_contract_reader_opens_a_document_and_its_files_apart() {
    let all = all();
    let mut reader = Reader {
        served: nothing_served(),
    };
    let parts = reader
        .open(hex(&all["main"]["envelope"]), main_of(&all))
        .unwrap();
    assert_eq!(parts.len(), 3);
    let files = reader
        .open(hex(&all["files"]["envelope"]), files_of(&all))
        .unwrap();
    assert_eq!(files.len(), 1);
    let bad = reader.open(
        hex(&all["refused"][0]["envelope"]),
        hostile(EnvelopeKind::Main),
    );
    assert!(bad.is_err());
}

#[test]
fn an_editor_that_breaks_the_contract_or_belongs_elsewhere_is_left_aside_and_the_document_kept() {
    let all = all();
    let holds = all["editors"][0]["entry"].clone();
    let mut unseen = holds.clone();
    unseen["name"] = json!("Ma\u{200B}rio");
    let mut nameless = holds.clone();
    nameless.as_object_mut().unwrap().remove("name");
    let elsewhere = all["editors"][2]["entry"].clone();
    let said = json!([holds, unseen, nameless, "not an editor", elsewhere]);
    let id = all["editors"][0]["entry"]["in"].clone();
    let held = json!({ "id": id, "html": "<p>x</p>", "body": "x" });

    let kept = editors_kept(held.clone(), Some(said.clone()), true);
    assert_eq!(kept["editors"], json!([all["editors"][0]["entry"]]));
    assert_eq!(kept["body"], "x");

    let public = editors_kept(held, Some(said), false);
    assert!(public.get("editors").is_none());
}

#[test]
fn a_document_keeps_only_the_editors_the_contract_says_it_keeps() {
    let all = all();
    let said = &all["editors_in_a_document"];
    let held = json!({ "id": said["id"], "html": "<p>x</p>", "body": "x" });
    let kept = editors_kept(held, Some(said["editors"].clone()), true);
    assert_eq!(kept["editors"], said["kept"]);
}
