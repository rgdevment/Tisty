//! What Tisty seals opens with the contract's own opener, the one written for the sample envelopes.

#[rustfmt::skip]
#[path = "../../../contracts/dist/rust/tests/vectors.rs"]
mod opening;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jiff::Timestamp;
use opening::{document_of, opened};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tisty_contracts::{Document, Sealing};
use tisty_envelope::family::{Family, Tongue, sealed_family, sealed_with_cited};
use tisty_envelope::pictures::made;
use tisty_envelope::sealing::{Refused, Writer};
use tisty_share_proto::identity::{Certificate, Root, ShareKey};

const SINCE: &str = "2026-11-03T12:00:00Z";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot decoded by the seal";

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
    }
}

fn document(value: Value) -> Document {
    serde_json::from_value(value).unwrap()
}

fn the_key(byte: u8) -> Vec<u8> {
    vec![byte; 32]
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
        "pictures": [{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": PNG.len(), "width": 1, "height": 1 }],
        "files": [{ "at": 0, "name": "cartilla.pdf", "kind": "application/pdf", "size": 3 }],
        "html": "<h1 id=\"paseos\">Paseos</h1><img src=\"tisty:picture/0\" alt=\"Rina\" width=\"1\" height=\"1\"><a href=\"tisty:page/1\">Enero</a><a class=\"card\" href=\"tisty:file/0\">Cartilla</a>",
        "body": "# Paseos\n",
        "pages": [{ "id": "p1", "page_of": "k7Qm2xPa", "at": 1, "html": "<h1 id=\"enero\">Enero</h1>", "body": "# Enero\n" }],
    }));
    let sealed = writer
        .sealed(
            said,
            vec![PNG.to_vec()],
            vec![b"pdf".to_vec()],
            &sealing("k7Qm2xPa", Some(key_text(0x11))),
        )
        .unwrap();

    let main = opened(&sealed.main, ("k7Qm2xPa", "main"), Some(&the_key(0x11))).unwrap();
    let held = document_of(&main);
    assert_eq!(held["signed_as"], "Mario");
    assert_eq!(held["pages"][0]["body"], "# Enero\n");
    assert_eq!(main.parts[2], PNG);

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

#[test]
fn nothing_outside_the_contract_leaves() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let refused = |writer: &mut Writer, value: Value, pictures: Vec<Vec<u8>>| {
        writer
            .sealed(
                document(value),
                pictures,
                vec![],
                &sealing("x", Some(key_text(1))),
            )
            .unwrap_err()
    };
    assert!(matches!(
        refused(
            &mut writer,
            json!({ "id": "x", "html": "<script>a()</script>", "body": "a" }),
            vec![]
        ),
        Refused::OutsideTheContract("html", _)
    ));
    assert!(matches!(
        refused(
            &mut writer,
            json!({ "id": "x", "html": "<figure class=\"diagram\"><pre><code class=\"language-mermaid\">a</code></pre></figure>", "body": "a" }),
            vec![]
        ),
        Refused::OutsideTheContract("html", _)
    ));
    assert!(matches!(
        refused(
            &mut writer,
            json!({ "id": "y", "html": "<p>a</p>", "body": "a" }),
            vec![]
        ),
        Refused::DoesNotAdd(_)
    ));
    assert!(matches!(
        refused(
            &mut writer,
            json!({ "id": "x", "signed_as": "a 123 456", "html": "<p>a</p>", "body": "a" }),
            vec![]
        ),
        Refused::OutsideTheContract("document", _)
    ));
}

#[test]
fn a_reference_that_points_at_nothing_never_leaves() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    for html in [
        "<a href=\"tisty:page/1\">a</a>",
        "<a href=\"tisty:file/0\">a</a>",
        "<a href=\"tisty:cited/0\">a</a>",
        "<img src=\"tisty:picture/0\" alt=\"a\" width=\"1\" height=\"1\">",
    ] {
        let refused = writer
            .sealed(
                document(json!({ "id": "x", "html": html, "body": "a" })),
                vec![],
                vec![],
                &sealing("x", Some(key_text(1))),
            )
            .unwrap_err();
        assert!(matches!(refused, Refused::DoesNotAdd(_)), "{html}");
    }
}

#[test]
fn a_picture_that_is_not_what_it_says_never_leaves() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let said = |size: usize| {
        document(json!({
            "id": "x",
            "pictures": [{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": size, "width": 1, "height": 1 }],
            "html": "<p>a</p>",
            "body": "a",
        }))
    };
    let at = sealing("x", Some(key_text(1)));
    assert!(
        writer
            .sealed(said(3), vec![b"pdf".to_vec()], vec![], &at)
            .is_err()
    );
    assert!(writer.sealed(said(PNG.len()), vec![], vec![], &at).is_err());
    assert!(
        writer
            .sealed(said(1), vec![PNG.to_vec()], vec![], &at)
            .is_err()
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

fn tongue(id: &str, lang: &str, byte: u8) -> Tongue {
    Tongue {
        document: document(
            json!({ "id": id, "lang": lang, "title": id, "html": "<p>a</p>", "body": "a" }),
        ),
        pictures: vec![],
        files: vec![],
        sealing: sealing(id, Some(key_text(byte))),
    }
}

#[test]
fn each_language_goes_apart_and_only_the_anchor_holds_the_map() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let going = sealed_family(
        &mut writer,
        Family {
            anchor: tongue("hola", "es", 1),
            others: vec![tongue("hello", "en", 2), tongue("ola", "pt-BR", 3)],
        },
    )
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
fn a_family_must_say_its_languages() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let mut anchor = tongue("hola", "es", 1);
    anchor.document.lang = None;
    assert!(
        sealed_family(
            &mut writer,
            Family {
                anchor,
                others: vec![tongue("hello", "en", 2)]
            }
        )
        .is_err()
    );
    let mut other = tongue("hello", "en", 2);
    other.document.lang = None;
    assert!(
        sealed_family(
            &mut writer,
            Family {
                anchor: tongue("hola", "es", 1),
                others: vec![other]
            }
        )
        .is_err()
    );
}

#[test]
fn what_is_cited_keeps_its_own_envelopes_and_its_key_goes_along() {
    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let mut holder = tongue("hola", "es", 1);
    holder.document.html = "<a class=\"card\" href=\"tisty:cited/0\">Otro</a>".to_owned();
    let going = sealed_with_cited(
        &mut writer,
        Family {
            anchor: holder,
            others: vec![],
        },
        vec![Family {
            anchor: tongue("otro", "es", 9),
            others: vec![],
        }],
    )
    .unwrap();
    assert_eq!(going[0].id, "hola");
    assert_eq!(going[1].id, "otro");
    let held =
        document_of(&opened(&going[0].sealed.main, ("hola", "main"), Some(&the_key(1))).unwrap());
    assert_eq!(held["cited"][0]["document"], "otro");
    assert_eq!(held["cited"][0]["key"], key_text(9));
    assert!(opened(&going[1].sealed.main, ("otro", "main"), Some(&the_key(9))).is_ok());
}

// What Tisty seals, kept where the JavaScript opener reads it: a browser has to open exactly this.
#[test]
fn what_rust_seals_is_what_javascript_is_handed() {
    let (key, certificate) = signer();
    let mut writer = Writer::with_head(&key, &certificate, || Some([0x22; 32]));
    let said = document(json!({
        "id": "writtenByTisty",
        "title": "Paseos",
        "lang": "es",
        "signed_as": "Mario",
        "pictures": [{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": PNG.len(), "width": 1, "height": 1 }],
        "files": [{ "at": 0, "name": "cartilla.pdf", "kind": "application/pdf", "size": 3 }],
        "html": "<h1 id=\"paseos\">Paseos</h1><img src=\"tisty:picture/0\" alt=\"Rina\" width=\"1\" height=\"1\"><a class=\"card\" href=\"tisty:file/0\">Cartilla</a>",
        "body": "# Paseos\n",
    }));
    let sealed = writer
        .sealed(
            said,
            vec![PNG.to_vec()],
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
    let kept = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    assert_eq!(
        kept, text,
        "run with TISTY_WRITE_SAMPLES=1, then node contracts/generate.mjs"
    );
}
