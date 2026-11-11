//! What Tisty seals opens with the contract's own opener, the one written for the sample envelopes.

#[rustfmt::skip]
#[path = "../../../contracts/dist/rust/tests/vectors.rs"]
mod opening;

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use jiff::Timestamp;
use opening::{document_of, opened};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tisty_contracts::{Document, Sealing, check_html};
use tisty_envelope::family::{Family, Tongue, sealed_family};
use tisty_envelope::pictures::made;
use tisty_envelope::sealing::{Refused, Writer};
use tisty_share_proto::identity::{Certificate, Root, ShareKey};
use tisty_share_proto::reviewing;

const SINCE: &str = "2026-11-03T12:00:00Z";

// A real picture, one pixel wide.
fn png() -> Vec<u8> {
    STANDARD
        .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEBgIApD5fRAAAAABJRU5ErkJggg==")
        .unwrap()
}

fn root() -> Root {
    Root::from_bytes(&[0x33; 32])
}

fn signer() -> (ShareKey, Certificate) {
    let key = ShareKey::from_bytes(&[0x44; 32]);
    let certificate = root().certify(&key.public(), SINCE.parse::<Timestamp>().unwrap());
    (key, certificate)
}

fn key_text(byte: u8) -> String {
    URL_SAFE_NO_PAD.encode([byte; 32])
}

fn sealing(id: &str, link_key: Option<String>) -> Sealing {
    Sealing {
        id: id.to_owned(),
        version: 1,
        from: None,
        link_key,
        files_hash: None,
        files_key: None,
        owner: None,
        token: None,
        terms: None,
    }
}

mod editing {
    use serde_json::{Value, json};
    use tisty_contracts::{Document, EnvelopeKind, Opening, Owner, Revocation, Served, Verdict};
    use tisty_envelope::reading::{Unopened, files_for, reading};
    use tisty_envelope::sealing::{Refused, Writer};
    use tisty_share_proto::identity::{Certificate, Root, ShareKey};

    use super::{key_text, sealing};

    const ID: &str = "editado";
    const LINK_KEY: u8 = 0x11;
    const TOKEN: u8 = 0x70;
    const OWNER: u8 = 0x30;
    const EDITOR: u8 = 0x40;

    fn person(byte: u8) -> (ShareKey, Certificate) {
        let key = ShareKey::from_bytes(&[byte + 1; 32]);
        let certificate = Root::from_bytes(&[byte; 32])
            .certify(&key.public(), "2026-11-03T12:00:00Z".parse().unwrap());
        (key, certificate)
    }

    fn words(owner_name: &str) -> Document {
        serde_json::from_value(
            json!({ "id": ID, "owner_name": owner_name, "html": "<p>a</p>", "body": "a" }),
        )
        .unwrap()
    }

    fn asked() -> Opening {
        Opening {
            id: ID.into(),
            kind: EnvelopeKind::Main,
            link_key: Some(key_text(LINK_KEY)),
        }
    }

    fn nothing() -> Served {
        Served {
            received: None,
            revocations: None,
            reviewed: None,
            reviewed_received: None,
        }
    }

    fn owner_in(main: &[u8]) -> Owner {
        let size = u32::from_be_bytes(main[4..8].try_into().unwrap()) as usize;
        let manifest: Value = serde_json::from_slice(&main[8..8 + size]).unwrap();
        serde_json::from_value(manifest["owner"].clone()).unwrap()
    }

    fn owned(token: Option<u8>) -> Vec<u8> {
        let (key, certificate) = person(OWNER);
        let mut going = sealing(ID, Some(key_text(LINK_KEY)));
        going.token = token.map(key_text);
        Writer::new(&key, &certificate)
            .sealed(words("Mario"), vec![], vec![], &going)
            .unwrap()
            .main
    }

    fn edited(owner: Owner, token: Option<u8>, name: &str) -> Result<Vec<u8>, Refused> {
        let (key, certificate) = person(EDITOR);
        let mut going = sealing(ID, Some(key_text(LINK_KEY)));
        going.owner = Some(owner);
        going.token = token.map(key_text);
        Writer::new(&key, &certificate)
            .sealed(words(name), vec![], vec![], &going)
            .map(|sealed| sealed.main)
    }

    fn unhexed(said: &Value) -> Vec<u8> {
        let said = said.as_str().unwrap();
        (0..said.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&said[at..at + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn an_owner_seals_and_its_reader_says_who_owns_it() {
        let main = owned(Some(TOKEN));
        let read = reading(&main, &asked(), &nothing()).unwrap();
        assert_eq!(read.owner.as_deref(), Some(read.signer.as_str()));
        assert_eq!(read.document["owner_name"], "Mario");
        assert!(owner_in(&main).permit.is_some());
        assert!(owner_in(&owned(None)).permit.is_none());
    }

    fn tongue_of(id: &str, lang: &str, link_key: u8) -> tisty_envelope::family::Tongue {
        tisty_envelope::family::Tongue {
            document: serde_json::from_value(json!({
                "id": id, "lang": lang, "owner_name": "Mario", "html": "<p>a</p>", "body": "a"
            }))
            .unwrap(),
            pictures: vec![],
            files: vec![],
            sealing: sealing(id, Some(key_text(link_key))),
            cited: vec![],
        }
    }

    #[test]
    fn a_language_an_editor_adds_never_takes_the_place_of_the_owner_s_of_its_lang() {
        let mut anchor = tongue_of(ID, "es", LINK_KEY);
        anchor.sealing.owner = Some(owner_in(&owned(Some(TOKEN))));
        anchor.sealing.token = Some(key_text(TOKEN));
        anchor.document.languages = Some(
            serde_json::from_value(
                json!([{ "lang": "en", "document": "marioEn1", "key": key_text(0x13) }]),
            )
            .unwrap(),
        );
        let family = tisty_envelope::family::Family {
            anchor,
            others: vec![tongue_of("anaEnglish", "en", 0x12)],
        };
        let (key, certificate) = person(EDITOR);
        let going =
            tisty_envelope::family::sealed_family(&mut Writer::new(&key, &certificate), family)
                .unwrap();
        let read = reading(&going[0].sealed.main, &asked(), &nothing()).unwrap();
        let named: Vec<&str> = read.document["languages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one["document"].as_str().unwrap())
            .collect();
        assert_eq!(named, ["anaEnglish", "marioEn1"]);
    }

    #[test]
    fn a_language_an_editor_adds_waits_in_the_map_for_the_owners_review() {
        let mut anchor = tongue_of(ID, "es", LINK_KEY);
        anchor.sealing.owner = Some(owner_in(&owned(Some(TOKEN))));
        anchor.sealing.token = Some(key_text(TOKEN));
        let family = tisty_envelope::family::Family {
            anchor,
            others: vec![tongue_of("anaEnglish", "en", 0x12)],
        };
        let (key, certificate) = person(EDITOR);
        let going =
            tisty_envelope::family::sealed_family(&mut Writer::new(&key, &certificate), family)
                .unwrap();
        let read = reading(&going[0].sealed.main, &asked(), &nothing()).unwrap();
        assert_eq!(read.document["languages"][0]["document"], "anaEnglish");
        assert_eq!(read.document["languages"][0]["pending"], true);
    }

    #[test]
    fn an_editor_with_the_token_signs_under_its_own_root_and_the_owner_stays() {
        let owner = owner_in(&owned(Some(TOKEN)));
        let main = edited(owner.clone(), Some(TOKEN), "Mario").unwrap();
        let read = reading(&main, &asked(), &nothing()).unwrap();
        assert_eq!(read.owner.as_deref(), Some(owner.root.as_str()));
        assert_ne!(read.signer, owner.root);
        assert_eq!(read.verdict, Verdict::Holds);
    }

    #[test]
    fn a_writer_does_not_seal_what_no_reader_would_open() {
        let owner = owner_in(&owned(Some(TOKEN)));
        for (token, name) in [
            (None, "Mario"),
            (Some(0x71), "Mario"),
            (Some(TOKEN), "Zorro"),
        ] {
            let refused = edited(owner.clone(), token, name);
            assert!(
                matches!(refused, Err(Refused::DoesNotAdd(_))),
                "{token:?} {name}"
            );
        }
    }

    #[test]
    fn what_is_public_has_no_owner_s_block_yet() {
        let (key, certificate) = person(OWNER);
        let mut going = sealing(ID, None);
        going.token = Some(key_text(TOKEN));
        let refused =
            Writer::new(&key, &certificate).sealed(words("Mario"), vec![], vec![], &going);
        assert!(matches!(refused, Err(Refused::DoesNotAdd(_))));
        let refused = Writer::new(&key, &certificate).sealed(
            words("Mario"),
            vec![],
            vec![],
            &sealing(ID, None),
        );
        assert!(matches!(refused, Err(Refused::DoesNotAdd(_))));
    }

    #[test]
    fn a_writer_does_not_carry_an_owner_s_block_that_does_not_hold() {
        let owner = owner_in(&owned(Some(TOKEN)));
        let (other, _) = person(EDITOR);
        let mut no_signature = owner.clone();
        no_signature.signed = owner.certificate.clone();
        let mut other_machine = owner.clone();
        other_machine.machine = other.shown();
        let mut other_permit = owner;
        other_permit.permit = Some(key_text(0x72));
        for broken in [no_signature, other_machine, other_permit] {
            let refused = edited(broken, Some(TOKEN), "Mario");
            assert!(
                matches!(refused, Err(Refused::DoesNotAdd(_))),
                "{refused:?}"
            );
        }
    }

    #[test]
    fn a_revocation_of_its_owner_s_machine_weighs_on_what_an_editor_signed() {
        let (owner_key, _) = person(OWNER);
        let main = edited(owner_in(&owned(Some(TOKEN))), Some(TOKEN), "Mario").unwrap();
        let revoked = Root::from_bytes(&[OWNER; 32])
            .revoke(&owner_key.public(), "2026-11-04T00:00:00Z".parse().unwrap());
        let served = Served {
            received: Some("2026-11-05T00:00:00Z".into()),
            revocations: Some(vec![Revocation {
                root: revoked.root,
                machine: revoked.share,
                at: revoked.at.to_string(),
                signed: revoked.sig,
            }]),
            reviewed: None,
            reviewed_received: None,
        };
        let read = reading(&main, &asked(), &served).unwrap();
        assert_eq!(read.verdict, Verdict::Doubtful);
    }

    #[test]
    fn this_reader_opens_and_refuses_what_the_samples_say() {
        let all: Value =
            serde_json::from_str(include_str!("../../../contracts/vectors/vectors.json")).unwrap();
        for one in all["editing"]["opens"].as_array().unwrap() {
            let read = reading(&unhexed(&one["envelope"]), &asked(), &nothing()).unwrap();
            assert_eq!(
                read.signer,
                one["signer"].as_str().unwrap(),
                "{}",
                one["says"]
            );
        }
        for one in all["editing"]["refused"].as_array().unwrap() {
            let refused = reading(&unhexed(&one["envelope"]), &asked(), &nothing());
            assert!(refused.is_err(), "{}", one["says"]);
        }
    }

    #[test]
    fn this_reader_tells_a_later_version_from_a_damaged_one_and_leaves_a_rest_aside() {
        let all: Value =
            serde_json::from_str(include_str!("../../../contracts/vectors/vectors.json")).unwrap();
        let hostile = Opening {
            id: "hostile".into(),
            kind: EnvelopeKind::Main,
            link_key: None,
        };
        for one in all["newer_than_this"].as_array().unwrap() {
            let read = reading(&unhexed(&one["envelope"]), &hostile, &nothing());
            assert_eq!(read.err(), Some(Unopened::Newer), "{}", one["says"]);
        }
        let old = &all["before_the_first_shared"]["envelope"];
        let read = reading(&unhexed(old), &asked(), &nothing());
        assert_eq!(read.err(), Some(Unopened::TooOld));
        let rest = &all["editing"]["rest_left_aside"];
        let read = reading(&unhexed(rest), &asked(), &nothing()).unwrap();
        assert!(read.document.get("owner_name").is_none());
        assert_eq!(read.document["body"], "a");
    }

    #[test]
    fn this_reader_still_opens_what_each_published_version_wrote() {
        let all: Value =
            serde_json::from_str(include_str!("../../../contracts/vectors/vectors.json")).unwrap();
        for (version, said) in all["frozen"].as_object().unwrap() {
            let opening = |id: &str, kind, link_key: Option<&str>| Opening {
                id: id.into(),
                kind,
                link_key: link_key.map(str::to_owned),
            };
            let key = said["link_key"].as_str();
            let main = opening("k7Qm2xPa", EnvelopeKind::Main, key);
            let read =
                reading(&unhexed(&said["main"]["envelope"]), &main, &nothing()).expect(version);
            assert_eq!(read.document, said["main"]["document"], "{version}");

            let files = opening("k7Qm2xPa", EnvelopeKind::Files, key);
            files_for(
                &read.document,
                &unhexed(&said["files"]["envelope"]),
                &files,
                &nothing(),
            )
            .expect(version);

            let public = opening("wordsOnly", EnvelopeKind::Main, None);
            let open =
                reading(&unhexed(&said["public"]["envelope"]), &public, &nothing()).expect(version);
            assert_eq!(open.document, said["public"]["document"], "{version}");
        }
    }
}

fn document(value: Value) -> Document {
    serde_json::from_value(value).unwrap()
}

fn the_key(byte: u8) -> Vec<u8> {
    vec![byte; 32]
}

fn a_picture_said() -> Value {
    json!([{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": png().len(), "width": 1, "height": 1 }])
}

#[test]
fn a_private_document_opens_as_the_samples_do() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = document(json!({
        "id": "k7Qm2xPa",
        "title": "Paseos",
        "lang": "es",
        "signed_as": "Mario",
        "pictures": a_picture_said(),
        "files": [{ "at": 0, "name": "cartilla.pdf", "kind": "application/pdf", "size": 3 }],
        "html": "<h1 id=\"paseos\">Paseos</h1><img src=\"tisty:picture/0\" alt=\"Rina\" width=\"1\" height=\"1\"><a href=\"tisty:page/1\">Enero</a><a class=\"card\" href=\"tisty:file/0\">Cartilla</a>",
        "body": "# Paseos\n",
        "pages": [{ "id": "p1", "page_of": "k7Qm2xPa", "at": 1, "html": "<h1 id=\"enero\">Enero</h1>", "body": "# Enero\n" }],
    }));
    let sealed = writer
        .sealed(
            said,
            vec![png()],
            vec![b"pdf".to_vec()],
            &sealing("k7Qm2xPa", Some(key_text(0x11))),
        )
        .unwrap();

    let main = opened(&sealed.main, ("k7Qm2xPa", "main"), Some(&the_key(0x11))).unwrap();
    let held = document_of(&main);
    assert_eq!(held["signed_as"], "Mario");
    assert_eq!(held["pages"][0]["body"], "# Enero\n");
    assert_eq!(main.parts[2], png());

    let files = opened(
        sealed.files.as_ref().unwrap(),
        ("k7Qm2xPa", "files"),
        Some(&the_key(0x11)),
    )
    .unwrap();
    assert_eq!(files.parts[0], b"pdf");
    let hash = Sha256::digest([files.manifest, files.index].concat());
    assert_eq!(held["files_hash"], URL_SAFE_NO_PAD.encode(hash));

    assert!(opened(&sealed.main, ("k7Qm2xPa", "main"), Some(&the_key(0x22))).is_err());
}

#[test]
fn a_public_document_goes_unsealed_and_says_so() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = document(json!({ "id": "wordsOnly", "html": "<p>Hola</p>", "body": "Hola\n" }));
    let sealed = writer
        .sealed(said, vec![], vec![], &sealing("wordsOnly", None))
        .unwrap();
    assert!(sealed.files.is_none());
    let held = document_of(&opened(&sealed.main, ("wordsOnly", "main"), None).unwrap());
    assert_eq!(held["body"], "Hola\n");
}

#[test]
fn a_comma_does_not_send_its_files_again() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = |body: &str| {
        document(json!({
            "id": "x",
            "files": [{ "at": 0, "name": "cartilla.pdf", "size": 3 }],
            "html": "<a class=\"card\" href=\"tisty:file/0\">Cartilla</a>",
            "body": body,
        }))
    };
    let first = writer
        .sealed(
            said("a"),
            vec![],
            vec![b"pdf".to_vec()],
            &sealing("x", Some(key_text(1))),
        )
        .unwrap();
    let opened_first = document_of(&opened(&first.main, ("x", "main"), Some(&the_key(1))).unwrap());
    let hash = opened_first["files_hash"].as_str().unwrap().to_owned();

    let mut again = sealing("x", Some(key_text(1)));
    again.version = 2;
    again.from = Some(1);
    again.files_hash = Some(hash.clone());
    again.files_key = Some(key_text(1));
    let second = writer.sealed(said("a,"), vec![], vec![], &again).unwrap();
    assert!(second.files.is_none());
    let held = document_of(&opened(&second.main, ("x", "main"), Some(&the_key(1))).unwrap());
    assert_eq!(held["files_hash"], hash);

    let refused = writer.sealed(said("b"), vec![], vec![b"pdf".to_vec()], &again);
    assert!(
        refused.is_err(),
        "either the old envelope or a new one, never both"
    );

    let mut other_key = sealing("x", Some(key_text(2)));
    other_key.version = 2;
    other_key.files_hash = Some(hash.clone());
    other_key.files_key = Some(key_text(1));
    assert!(
        writer
            .sealed(said("c"), vec![], vec![], &other_key)
            .is_err()
    );

    let mut turned_public = other_key.clone();
    turned_public.link_key = None;
    assert!(
        writer
            .sealed(said("c"), vec![], vec![], &turned_public)
            .is_err()
    );

    let mut turned_private = other_key;
    turned_private.link_key = Some(key_text(1));
    turned_private.files_key = None;
    assert!(
        writer
            .sealed(said("c"), vec![], vec![], &turned_private)
            .is_err()
    );

    let nothing_named = document(json!({ "id": "x", "html": "<p>a</p>", "body": "a" }));
    assert!(
        writer
            .sealed(nothing_named, vec![], vec![], &again)
            .is_err()
    );
}

#[test]
fn it_writes_the_same_certificate_and_revocation_as_the_samples() {
    let samples: Value =
        serde_json::from_str(include_str!("../../../contracts/vectors/vectors.json")).unwrap();
    let (key, certificate) = signer();
    assert_eq!(certificate.root, samples["revocation"]["root"]);
    assert_eq!(certificate.share, samples["revocation"]["machine"]);
    let revocation = root().revoke(&key.public(), SINCE.parse::<Timestamp>().unwrap());
    assert_eq!(revocation.sig, samples["revocation"]["signed"]);
    assert_eq!(revocation.at.to_string(), samples["revocation"]["at"]);
}

fn refused(value: Value, sealing: &Sealing) -> Refused {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    writer
        .sealed(document(value), vec![], vec![], sealing)
        .unwrap_err()
}

#[test]
fn nothing_outside_the_contract_leaves() {
    let at = sealing("x", Some(key_text(1)));
    assert!(matches!(
        refused(
            json!({ "id": "x", "html": "<script>a()</script>", "body": "a" }),
            &at
        ),
        Refused::OutsideTheContract("html", _)
    ));
    assert!(matches!(
        refused(
            json!({ "id": "x", "html": "<figure class=\"diagram\"><pre><code class=\"language-mermaid\">a</code></pre></figure>", "body": "a" }),
            &at
        ),
        Refused::OutsideTheContract("html", _)
    ));
    assert!(matches!(
        refused(
            json!({ "id": "x", "html": "</p a=\"<p title='\"><script>a()</script><p b='>x", "body": "a" }),
            &at
        ),
        Refused::OutsideTheContract("html", _)
    ));
    assert!(matches!(
        refused(json!({ "id": "y", "html": "<p>a</p>", "body": "a" }), &at),
        Refused::DoesNotAdd(_)
    ));
    assert!(matches!(
        refused(
            json!({ "id": "x", "signed_as": "a 123 456", "html": "<p>a</p>", "body": "a" }),
            &at
        ),
        Refused::OutsideTheContract("document", _)
    ));
}

#[test]
fn html_written_loosely_cannot_hang_the_seal() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = document(json!({ "id": "x", "html": "<p title=x>a</p><p =>b</p>", "body": "a" }));
    let _ = writer.sealed(said, vec![], vec![], &sealing("x", Some(key_text(1))));
}

#[test]
fn a_reference_that_points_at_nothing_never_leaves_however_it_is_written() {
    let at = sealing("x", Some(key_text(1)));
    for html in [
        "<a href=\"tisty:page/1\">a</a>",
        "<a href=\"tisty:file/0\">a</a>",
        "<a href=\"tisty:cited/0\">a</a>",
        "<a href=\"TISTY:page/9\">a</a>",
        "<a href=\" tisty:page/9\">a</a>",
        "<a HREF=\"tisty:page/9\">a</a>",
        "<a href='tisty:page/9'>a</a>",
        "<a href=tisty:page/9>a</a>",
        "<img src=\"tisty:picture/0\" alt=\"a\" width=\"1\" height=\"1\">",
    ] {
        assert!(
            matches!(
                refused(json!({ "id": "x", "html": html, "body": "a" }), &at),
                Refused::DoesNotAdd(_)
            ),
            "{html}"
        );
    }
}

#[test]
fn a_writer_does_not_get_to_say_what_does_not_add_up() {
    let at = sealing("x", Some(key_text(1)));
    let base = json!({ "id": "x", "html": "<p>a</p>", "body": "a" });
    let with = |field: &str, value: Value| {
        let mut out = base.clone();
        out[field] = value;
        out
    };
    let cases = [
        with(
            "files",
            json!([{ "at": 0, "name": "C:\\Users\\Mario\\cartilla.pdf" }]),
        ),
        with(
            "pages",
            json!([{ "id": "p1", "page_of": "otro", "at": 1, "html": "<p>a</p>", "body": "a" }]),
        ),
        with(
            "pages",
            json!([
                { "id": "p1", "page_of": "x", "at": 1, "html": "<p>a</p>", "body": "a" },
                { "id": "p2", "page_of": "x", "at": 1, "html": "<p>a</p>", "body": "a" },
            ]),
        ),
        with("languages", json!([{ "lang": "en", "document": "y" }])),
    ];
    for one in cases {
        let (key, certificate) = signer();
        let mut writer = Writer::new(&key, &certificate);
        let files = if one.get("files").is_some() {
            vec![b"pdf".to_vec()]
        } else {
            vec![]
        };
        let wrong = writer.sealed(document(one.clone()), vec![], files, &at);
        assert!(wrong.is_err(), "{one}");
    }
    let mut late = sealing("x", Some(key_text(1)));
    late.from = Some(1);
    assert!(matches!(
        refused(base.clone(), &late),
        Refused::DoesNotAdd(_)
    ));
}

#[test]
fn a_picture_that_is_not_what_it_says_never_leaves() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = |value: Value| {
        document(json!({ "id": "x", "pictures": value, "html": "<p>a</p>", "body": "a" }))
    };
    let at = sealing("x", Some(key_text(1)));
    assert!(
        writer
            .sealed(said(a_picture_said()), vec![b"pdf".to_vec()], vec![], &at)
            .is_err()
    );
    assert!(
        writer
            .sealed(said(a_picture_said()), vec![], vec![], &at)
            .is_err()
    );
    let lying_size = json!([{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": 1, "width": 1, "height": 1 }]);
    assert!(
        writer
            .sealed(said(lying_size), vec![png()], vec![], &at)
            .is_err()
    );
    let lying_width = json!([{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": png().len(), "width": 1600, "height": 1 }]);
    assert!(
        writer
            .sealed(said(lying_width), vec![png()], vec![], &at)
            .is_err()
    );
    let a_path = json!([{ "at": 0, "name": "C:/fotos/rina.png", "kind": "image/png", "size": png().len(), "width": 1, "height": 1 }]);
    assert!(
        writer
            .sealed(said(a_path), vec![png()], vec![], &at)
            .is_err()
    );
    assert!(
        writer
            .sealed(said(a_picture_said()), vec![png()], vec![], &at)
            .is_ok()
    );
}

#[test]
fn a_picture_made_here_passes_what_the_seal_asks() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let source = image::RgbImage::from_pixel(4, 2, image::Rgb([1, 2, 3]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgb8(source)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    let picture = made(&bytes).unwrap();
    let said = document(json!({
        "id": "x",
        "pictures": [{ "at": 0, "name": "picture-0.png", "kind": picture.kind.said(), "size": picture.bytes.len(), "width": picture.width, "height": picture.height }],
        "html": "<img src=\"tisty:picture/0\" alt=\"a\" width=\"4\" height=\"2\">",
        "body": "a",
    }));
    let sealed = writer
        .sealed(
            said,
            vec![picture.bytes.clone()],
            vec![],
            &sealing("x", Some(key_text(1))),
        )
        .unwrap();
    let main = opened(&sealed.main, ("x", "main"), Some(&the_key(1))).unwrap();
    assert_eq!(main.parts[2], picture.bytes);
}

fn tongue(id: &str, lang: &str, key: Option<u8>) -> Tongue {
    Tongue {
        document: document(
            json!({ "id": id, "lang": lang, "title": id, "html": "<p>a</p>", "body": "a" }),
        ),
        pictures: vec![],
        files: vec![],
        sealing: sealing(id, key.map(key_text)),
        cited: vec![],
    }
}

fn alone(one: Tongue) -> Family {
    Family {
        anchor: one,
        others: vec![],
    }
}

fn sealed(family: Family) -> Result<Vec<tisty_envelope::family::Going>, Refused> {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    sealed_family(&mut writer, family)
}

fn left_out_by(
    anchor: &str,
    language: &str,
    by: &(ShareKey, Certificate),
) -> tisty_contracts::Review {
    let of = reviewing::of(&[1; 32], language).unwrap();
    let said = reviewing::Said {
        anchor,
        of: &of,
        seq: 1,
        kept: None,
    };
    let signed = reviewing::signed(&by.0, &by.1, &said).unwrap();
    serde_json::from_value(json!({
        "of": URL_SAFE_NO_PAD.encode(&of),
        "kept": false,
        "seq": 1,
        "root": signed.certificate.root,
        "machine": signed.certificate.share,
        "since": SINCE,
        "certificate": signed.certificate.sig,
        "signed": signed.sig,
    }))
    .unwrap()
}

fn map_after(left_out: tisty_contracts::Review) -> Vec<String> {
    let mut anchor = tongue("hola", "es", Some(1));
    anchor.document.reviews = Some(vec![left_out]);
    let going = sealed(Family {
        anchor,
        others: vec![
            tongue("hello", "en", Some(2)),
            tongue("ola", "pt-BR", Some(3)),
        ],
    })
    .unwrap();
    let anchor =
        document_of(&opened(&going[0].sealed.main, ("hola", "main"), Some(&the_key(1))).unwrap());
    anchor["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["document"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn a_language_its_owner_left_out_never_goes_back_in_the_map() {
    assert_eq!(map_after(left_out_by("hola", "hello", &signer())), ["ola"]);
}

#[test]
fn a_left_out_someone_else_signed_leaves_the_map_as_it_is() {
    let key = ShareKey::from_bytes(&[0x56; 32]);
    let certificate =
        Root::from_bytes(&[0x55; 32]).certify(&key.public(), SINCE.parse::<Timestamp>().unwrap());
    let stranger = (key, certificate);
    assert_eq!(
        map_after(left_out_by("hola", "hello", &stranger)),
        ["hello", "ola"]
    );
    assert_eq!(
        map_after(left_out_by("otroDoc", "hello", &signer())),
        ["hello", "ola"]
    );
}

#[test]
fn each_language_goes_apart_and_only_the_anchor_holds_the_map() {
    let going = sealed(Family {
        anchor: tongue("hola", "es", Some(1)),
        others: vec![
            tongue("hello", "en", Some(2)),
            tongue("ola", "pt-BR", Some(3)),
        ],
    })
    .unwrap();
    assert_eq!(going.len(), 3);
    let anchor =
        document_of(&opened(&going[0].sealed.main, ("hola", "main"), Some(&the_key(1))).unwrap());
    assert_eq!(anchor["languages"][0]["lang"], "en");
    assert_eq!(anchor["languages"][0]["document"], "hello");
    assert_eq!(anchor["languages"][0]["key"], key_text(2));
    assert!(anchor["languages"][0].get("pending").is_none());
    assert_eq!(anchor["languages"][1]["document"], "ola");
    let english =
        document_of(&opened(&going[1].sealed.main, ("hello", "main"), Some(&the_key(2))).unwrap());
    assert_eq!(english["lang"], "en");
    assert!(english.get("languages").is_none());
    assert_eq!(
        english["anchor"], "hola",
        "a language says which anchor it belongs to"
    );
    assert!(anchor.get("anchor").is_none());
}

#[test]
fn a_language_this_writer_does_not_reach_stays_in_the_map_as_it_arrived() {
    let mut anchor = tongue("hola", "es", Some(1));
    anchor.document.languages = Some(
        serde_json::from_value(json!([
            { "lang": "en", "document": "hello", "key": key_text(2) },
            { "lang": "fr", "document": "salut", "key": key_text(4) },
        ]))
        .unwrap(),
    );
    let going = sealed(Family {
        anchor,
        others: vec![tongue("hello", "en", Some(2))],
    })
    .unwrap();
    let map = document_of(
        &opened(&going[0].sealed.main, ("hola", "main"), Some(&the_key(1))).unwrap(),
    )["languages"]
        .clone();
    let ids: Vec<&str> = map
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["document"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["hello", "salut"]);
}

#[test]
fn the_paper_of_a_family_is_the_anchors() {
    let mut anchor = tongue("hola", "es", Some(1));
    anchor.document.sheet =
        serde_json::from_value(json!({ "paper": "letter", "on": true })).unwrap();
    let mut other = tongue("hello", "en", Some(2));
    other.document.sheet = serde_json::from_value(json!({ "paper": "a4" })).unwrap();
    let going = sealed(Family {
        anchor,
        others: vec![other],
    })
    .unwrap();
    let english =
        document_of(&opened(&going[1].sealed.main, ("hello", "main"), Some(&the_key(2))).unwrap());
    assert_eq!(english["sheet"]["paper"], "letter");
}

#[test]
fn a_family_must_say_its_languages_once_each() {
    let mut anchor = tongue("hola", "es", Some(1));
    anchor.document.lang = None;
    assert!(
        sealed(Family {
            anchor,
            others: vec![tongue("hello", "en", Some(2))]
        })
        .is_err()
    );
    let mut other = tongue("hello", "en", Some(2));
    other.document.lang = None;
    assert!(
        sealed(Family {
            anchor: tongue("hola", "es", Some(1)),
            others: vec![other]
        })
        .is_err()
    );
    assert!(
        sealed(Family {
            anchor: tongue("hola", "es", Some(1)),
            others: vec![tongue("otra", "es", Some(2))]
        })
        .is_err()
    );
}

#[test]
fn what_is_public_never_holds_a_key() {
    let private_language = Family {
        anchor: tongue("hola", "es", None),
        others: vec![tongue("hello", "en", Some(2))],
    };
    assert!(matches!(
        sealed(private_language),
        Err(Refused::DoesNotAdd(_))
    ));
    let mut holder = tongue("hola", "es", None);
    holder.cited = vec![Arc::new(alone(tongue("otro", "es", Some(9))))];
    assert!(matches!(sealed(alone(holder)), Err(Refused::DoesNotAdd(_))));
}

#[test]
fn what_is_public_may_cite_what_is_public() {
    let mut holder = tongue("hola", "es", None);
    holder.document.html = "<a class=\"card\" href=\"tisty:cited/0\">Otro</a>".to_owned();
    holder.cited = vec![Arc::new(alone(tongue("otro", "es", None)))];
    let going = sealed(alone(holder)).unwrap();
    let held = document_of(&opened(&going[0].sealed.main, ("hola", "main"), None).unwrap());
    assert_eq!(held["cited"][0]["document"], "otro");
    assert!(held["cited"][0].get("key").is_none());
}

#[test]
fn what_is_cited_keeps_its_own_envelopes_and_its_key_goes_along() {
    let mut holder = tongue("hola", "es", Some(1));
    holder.document.html = "<a class=\"card\" href=\"tisty:cited/0\">Otro</a>".to_owned();
    holder.cited = vec![Arc::new(alone(tongue("otro", "es", Some(9))))];
    let going = sealed(alone(holder)).unwrap();
    assert_eq!(going[0].id, "hola");
    assert_eq!(going[1].id, "otro");
    let held =
        document_of(&opened(&going[0].sealed.main, ("hola", "main"), Some(&the_key(1))).unwrap());
    assert_eq!(held["cited"][0]["document"], "otro");
    assert_eq!(held["cited"][0]["key"], key_text(9));
    assert!(opened(&going[1].sealed.main, ("otro", "main"), Some(&the_key(9))).is_ok());
}

#[test]
fn two_languages_may_cite_one_document_and_it_goes_once() {
    let mut anchor = tongue("hola", "es", Some(1));
    anchor.cited = vec![Arc::new(alone(tongue("otro", "es", Some(9))))];
    let mut other = tongue("hello", "en", Some(2));
    other.cited = vec![Arc::new(alone(tongue("otro", "es", Some(9))))];
    let going = sealed(Family {
        anchor,
        others: vec![other],
    })
    .unwrap();
    let ids: Vec<&str> = going.iter().map(|one| one.id.as_str()).collect();
    assert_eq!(ids, ["hola", "hello", "otro"]);
}

#[test]
fn a_picture_is_drawn_at_its_own_size_and_pages_go_in_their_order() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let wide = document(json!({
        "id": "x",
        "pictures": a_picture_said(),
        "html": "<img src=\"tisty:picture/0\" alt=\"a\" width=\"900\" height=\"1\">",
        "body": "a",
    }));
    assert!(
        writer
            .sealed(wide, vec![png()], vec![], &sealing("x", None))
            .is_err()
    );
    let page = |id: &str, at: u64| json!({ "id": id, "page_of": "x", "at": at, "html": "<p>a</p>", "body": "a" });
    for pages in [
        json!([page("b", 2), page("a", 1)]),
        json!([page("a", 1), page("a", 2)]),
    ] {
        let said = document(json!({ "id": "x", "pages": pages, "html": "<p>a</p>", "body": "a" }));
        assert!(
            writer
                .sealed(said, vec![], vec![], &sealing("x", None))
                .is_err()
        );
    }
}

#[test]
fn a_picture_travels_once_and_says_nothing_of_where_it_came_from() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let picture = |name: &str, at: u64| json!({ "at": at, "name": name, "kind": "image/png", "size": png().len(), "width": 1, "height": 1 });
    let said = |pictures: Value| {
        document(json!({ "id": "x", "pictures": pictures, "html": "<p>a</p>", "body": "a" }))
    };
    let named_home = said(json!([picture("en-casa.png", 0)]));
    assert!(
        writer
            .sealed(named_home, vec![png()], vec![], &sealing("x", None))
            .is_err()
    );
    let twice = said(json!([
        picture("picture-0.png", 0),
        picture("picture-1.png", 1)
    ]));
    assert!(
        writer
            .sealed(twice, vec![png(), png()], vec![], &sealing("x", None))
            .is_err()
    );
}

#[test]
fn the_fingerprint_is_the_one_the_contract_shows() {
    let vectors: Value =
        serde_json::from_str(include_str!("../../../contracts/vectors/vectors.json")).unwrap();
    assert_eq!(
        vectors["identity"]["fingerprint"],
        root().person().fingerprint()
    );
}

#[test]
fn two_documents_never_go_under_one_id() {
    assert!(
        sealed(Family {
            anchor: tongue("aaaa", "es", Some(1)),
            others: vec![tongue("aaaa", "en", Some(1))],
        })
        .is_err()
    );

    let mut itself = tongue("aaaa", "es", Some(1));
    itself.cited = vec![Arc::new(alone(tongue("aaaa", "es", Some(1))))];
    assert!(sealed(alone(itself)).is_err());

    let mut told = tongue("otro", "es", Some(2));
    told.document.html = "<p>otra cosa</p>".to_owned();
    let mut holder = tongue("aaaa", "es", Some(1));
    holder.cited = vec![Arc::new(alone(tongue("otro", "es", Some(2))))];
    let mut other = tongue("bbbb", "en", Some(1));
    other.cited = vec![Arc::new(alone(told))];
    assert!(
        sealed(Family {
            anchor: holder,
            others: vec![other],
        })
        .is_err()
    );
}

#[test]
fn an_attribute_a_browser_reads_after_a_slash_counts_too() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = document(json!({
        "id": "x",
        "html": "<a title/href=\"tisty:page/9\">a</a>",
        "body": "a",
    }));
    assert_eq!(check_html(&said.html), Vec::<String>::new());
    assert!(
        writer
            .sealed(said, vec![], vec![], &sealing("x", None))
            .is_err()
    );
}

#[test]
fn what_cites_cites_directly_and_only_once() {
    let mut twice = tongue("hola", "es", Some(1));
    twice.cited = vec![
        Arc::new(alone(tongue("otro", "es", Some(9)))),
        Arc::new(alone(tongue("otro", "es", Some(9)))),
    ];
    assert!(sealed(alone(twice)).is_err());
    let mut deep = tongue("otro", "es", Some(9));
    deep.cited = vec![Arc::new(alone(tongue("tercero", "es", Some(8))))];
    let mut holder = tongue("hola", "es", Some(1));
    holder.cited = vec![Arc::new(alone(deep))];
    assert!(sealed(alone(holder)).is_err());
}

// What Tisty seals, kept where the JavaScript opener reads it: a browser has to open exactly this.
#[test]
fn what_rust_seals_is_what_javascript_is_handed() {
    let (key, certificate) = signer();
    // Heads of their own, since these samples share a key with the contract's.
    let mut next = 0x60;
    let mut writer = Writer::with_head(&key, &certificate, move || {
        next += 1;
        Some([next; 32])
    });
    let said = document(json!({
        "id": "writtenByTisty",
        "title": "Paseos",
        "lang": "es",
        "signed_as": "Mario",
        "pictures": a_picture_said(),
        "files": [{ "at": 0, "name": "cartilla.pdf", "kind": "application/pdf", "size": 3 }],
        "html": "<h1 id=\"paseos\">Paseos</h1><img src=\"tisty:picture/0\" alt=\"Rina\" width=\"1\" height=\"1\"><a class=\"card\" href=\"tisty:file/0\">Cartilla</a>",
        "body": "# Paseos\n",
    }));
    let sealed = writer
        .sealed(
            said,
            vec![png()],
            vec![b"pdf".to_vec()],
            &sealing("writtenByTisty", Some(key_text(0x11))),
        )
        .unwrap();
    let hexed = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|one| format!("{one:02x}"))
            .collect::<String>()
    };
    let written = json!({
        "link_key": key_text(0x11),
        "main": hexed(&sealed.main),
        "files": hexed(sealed.files.as_ref().unwrap()),
    });
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/vectors/written-by-tisty.json"
    );
    let text = format!("{}\n", serde_json::to_string_pretty(&written).unwrap());
    if std::env::var_os("TISTY_WRITE_SAMPLES").is_some() {
        std::fs::write(path, &text).unwrap();
    }
    let kept: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let unhexed = |said: &Value| -> Vec<u8> {
        let said = said.as_str().unwrap();
        (0..said.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&said[at..at + 2], 16).unwrap())
            .collect()
    };
    // Compressed bytes and their sizes depend on the deflate this build links, so what has to hold is what they open to.
    let held = |bytes: &[u8], kind: &str| {
        let one = opened(bytes, ("writtenByTisty", kind), Some(&the_key(0x11))).unwrap();
        let mut manifest: Value = serde_json::from_slice(&one.manifest).unwrap();
        manifest["parts"].take();
        (manifest, one.parts)
    };
    for (kind, fresh) in [
        ("main", &sealed.main),
        ("files", sealed.files.as_ref().unwrap()),
    ] {
        assert_eq!(
            held(&unhexed(&kept[kind]), kind),
            held(fresh, kind),
            "run with TISTY_WRITE_SAMPLES=1, then node contracts/generate.mjs"
        );
    }
}

fn mapped(entries: serde_json::Value) -> Vec<tisty_envelope::Translation> {
    serde_json::from_value(entries).unwrap()
}

fn named_in_the_map(going: &[tisty_envelope::family::Going]) -> Vec<String> {
    let anchor =
        document_of(&opened(&going[0].sealed.main, ("hola", "main"), Some(&the_key(1))).unwrap());
    anchor["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["document"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn the_owner_s_own_language_takes_the_place_of_its_old_entry_and_never_of_one_that_waits() {
    let mut anchor = tongue("hola", "es", Some(1));
    anchor.document.languages = Some(mapped(json!([
        { "lang": "en", "document": "oldEngl1", "key": key_text(4) },
        { "lang": "fr", "document": "anaFren1", "key": key_text(5), "pending": true },
    ])));
    let going = sealed(Family {
        anchor,
        others: vec![
            tongue("hello", "en", Some(2)),
            tongue("salut", "fr", Some(3)),
        ],
    })
    .unwrap();

    assert_eq!(named_in_the_map(&going), ["hello", "salut", "anaFren1"]);
}

#[test]
fn a_map_past_nineteen_languages_is_refused_and_none_is_taken_out() {
    let mut anchor = tongue("hola", "es", Some(1));
    let held: Vec<serde_json::Value> = (0..19u8)
        .map(|one| json!({ "lang": format!("z{}", char::from(b'a' + one)), "document": format!("other{one:03}") }))
        .collect();
    anchor.document.languages = Some(mapped(serde_json::Value::Array(held)));

    let refused = sealed(Family {
        anchor,
        others: vec![tongue("hello", "en", Some(2))],
    });

    assert!(matches!(refused, Err(Refused::DoesNotAdd(_))));
}
