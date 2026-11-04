//! What Tisty seals opens with the contract's own opener, the one written for the sample envelopes.

#[rustfmt::skip]
#[path = "../../../contracts/dist/rust/tests/vectors.rs"]
mod opening;

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
        assert!(matches!(wrong, Err(Refused::DoesNotAdd(_))), "{one}");
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
    assert_eq!(anchor["languages"][1]["document"], "ola");
    let english =
        document_of(&opened(&going[1].sealed.main, ("hello", "main"), Some(&the_key(2))).unwrap());
    assert_eq!(english["lang"], "en");
    assert!(english.get("languages").is_none());
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
    holder.cited = vec![alone(tongue("otro", "es", Some(9)))];
    assert!(matches!(sealed(alone(holder)), Err(Refused::DoesNotAdd(_))));
}

#[test]
fn what_is_public_may_cite_what_is_public() {
    let mut holder = tongue("hola", "es", None);
    holder.document.html = "<a class=\"card\" href=\"tisty:cited/0\">Otro</a>".to_owned();
    holder.cited = vec![alone(tongue("otro", "es", None))];
    let going = sealed(alone(holder)).unwrap();
    let held = document_of(&opened(&going[0].sealed.main, ("hola", "main"), None).unwrap());
    assert_eq!(held["cited"][0]["document"], "otro");
    assert!(held["cited"][0].get("key").is_none());
}

#[test]
fn what_is_cited_keeps_its_own_envelopes_and_its_key_goes_along() {
    let mut holder = tongue("hola", "es", Some(1));
    holder.document.html = "<a class=\"card\" href=\"tisty:cited/0\">Otro</a>".to_owned();
    holder.cited = vec![alone(tongue("otro", "es", Some(9)))];
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
    anchor.cited = vec![alone(tongue("otro", "es", Some(9)))];
    let mut other = tongue("hello", "en", Some(2));
    other.cited = vec![alone(tongue("otro", "es", Some(9)))];
    let going = sealed(Family {
        anchor,
        others: vec![other],
    })
    .unwrap();
    let ids: Vec<&str> = going.iter().map(|one| one.id.as_str()).collect();
    assert_eq!(ids, ["hola", "hello", "otro"]);
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
fn its_properties_do_not_say_what_stays_on_this_machine() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = document(json!({
        "id": "x",
        "properties": [{ "key": "Task", "values": ["01M6"] }],
        "html": "<p>a</p>",
        "body": "a",
    }));
    assert!(
        writer
            .sealed(said, vec![], vec![], &sealing("x", None))
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
    itself.cited = vec![alone(tongue("aaaa", "es", Some(1)))];
    assert!(sealed(alone(itself)).is_err());

    let mut told = tongue("otro", "es", Some(2));
    told.document.html = "<p>otra cosa</p>".to_owned();
    let mut holder = tongue("aaaa", "es", Some(1));
    holder.cited = vec![alone(tongue("otro", "es", Some(2)))];
    let mut other = tongue("bbbb", "en", Some(1));
    other.cited = vec![alone(told)];
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
        alone(tongue("otro", "es", Some(9))),
        alone(tongue("otro", "es", Some(9))),
    ];
    assert!(sealed(alone(twice)).is_err());
    let mut deep = tongue("otro", "es", Some(9));
    deep.cited = vec![alone(tongue("tercero", "es", Some(8)))];
    let mut holder = tongue("hola", "es", Some(1));
    holder.cited = vec![alone(deep)];
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
