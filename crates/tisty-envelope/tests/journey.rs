//! The whole way a document goes, from what a person wrote to what a reader opens. A window colours and draws in the middle, so `app/scripts/journey.mjs` runs these two around Chrome.

#[rustfmt::skip]
#[path = "../../../contracts/dist/rust/tests/vectors.rs"]
mod opening;
mod real;

use std::collections::BTreeMap;
use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use jiff::Timestamp;
use opening::{files_for, reading};
use real::every_real_document;
use serde_json::{Value, json};
use tisty_contracts::{Document, Sealing, check_html};
use tisty_envelope::family::{Family, Tongue, sealed_family};
use tisty_envelope::pictures::made;
use tisty_envelope::sealing::Writer;
use tisty_envelope::travelling::{Placed, Places, Reached, body_ready, named, ready};
use tisty_share_proto::identity::{Certificate, Root, ShareKey};

const WRITTEN: &str = r#"---
title: Paseos
Task: 01M6
TranslationKey: paseos
---

# Paseos

Con ::dog orange:: y $a\,b + c$, Rina.

![Rina](rina.png)

[La cartilla](cartilla.pdf "cartilla.pdf"), [[Enero]], [[Otro]], [la nota][n] y [[Sin llegar]].

[n]: C:/Users/mario/notas/secreto.md

```rust
fn main() {}
```

```mermaid
flowchart TD
  A[Inicio] --> B[Fin]
```
"#;
const OF_ITS_PAGE: &str = "# Enero\n\n```mermaid\npie\n  \"a\" : 1\n```\n";
const IN_ENGLISH: &str = "# Walks\n\nWith Rina.\n";
const CITED: &str = "# Otro\n\nAlgo aparte.\n";
const PDF: &[u8] = b"%PDF-1.7 cartilla";
// A real picture, one pixel wide.
const PICTURE: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEBgIApD5fRAAAAABJRU5ErkJggg==";

fn folder() -> PathBuf {
    std::env::var_os("TISTY_JOURNEY")
        .expect("app/scripts/journey.mjs says where the journey is kept")
        .into()
}

fn places() -> Places {
    let mut out = Places::default();
    out.pictures.insert(
        "rina.png".into(),
        Placed {
            at: 0,
            width: 1,
            height: 1,
        },
    );
    out.files.insert("cartilla.pdf".into(), 0);
    out.documents.insert("Enero".into(), Reached::Page(1));
    out.documents.insert("Otro".into(), Reached::Cited(0));
    out
}

fn drawn(text: &str, places: &Places) -> String {
    ready(&tisty_markdown::html(&tisty_markdown::read(text)), places)
}

fn real_ones() -> Vec<(String, String)> {
    every_real_document()
}

#[test]
#[ignore = "app/scripts/journey.mjs runs it before Chrome"]
fn ready_for_the_window() {
    let found = named(&tisty_markdown::html(&tisty_markdown::read(WRITTEN)));
    assert_eq!(found.pictures, ["rina.png"]);
    assert_eq!(
        found.links,
        ["cartilla.pdf", "C:/Users/mario/notas/secreto.md"]
    );
    assert_eq!(found.documents, ["Enero", "Otro", "Sin llegar"]);

    let places = places();
    let none = Places::default();
    let mut out = vec![
        json!({ "name": "written", "html": drawn(WRITTEN, &places) }),
        json!({ "name": "page", "html": drawn(OF_ITS_PAGE, &none) }),
        json!({ "name": "english", "html": drawn(IN_ENGLISH, &none) }),
        json!({ "name": "cited", "html": drawn(CITED, &none) }),
    ];
    for (name, text) in real_ones() {
        out.push(json!({ "name": name, "html": drawn(&text, &none) }));
    }
    std::fs::write(
        folder().join("ready.json"),
        serde_json::to_vec(&out).unwrap(),
    )
    .unwrap();
}

fn signer() -> (ShareKey, Certificate) {
    let key = ShareKey::from_bytes(&[0x44; 32]);
    let since = "2026-11-03T12:00:00Z".parse::<Timestamp>().unwrap();
    let certificate = Root::from_bytes(&[0x33; 32]).certify(&key.public(), since);
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

fn plain_tongue(id: &str, lang: &str, title: &str, html: &str, text: &str, key: u8) -> Tongue {
    Tongue {
        document: document(json!({
            "id": id,
            "lang": lang,
            "title": title,
            "html": html,
            "body": body_ready(text, &Places::default()),
        })),
        pictures: vec![],
        files: vec![],
        sealing: sealing(id, Some(key_text(key))),
        cited: vec![],
    }
}

fn the_family(finished: &BTreeMap<String, String>, picture: &[u8]) -> Family {
    let places = places();
    let anchor = Tongue {
        document: document(json!({
            "id": "paseos",
            "lang": "es",
            "title": "Paseos",
            "signed_as": "Rina",
            "pictures": [{ "at": 0, "name": "picture-0.png", "kind": "image/png", "size": picture.len(), "width": 1, "height": 1 }],
            "files": [{ "at": 0, "name": "cartilla.pdf", "kind": "application/pdf", "size": PDF.len() }],
            "html": finished["written"],
            "body": body_ready(WRITTEN, &places),
            "pages": [{ "id": "enero", "page_of": "paseos", "at": 1, "html": finished["page"], "body": body_ready(OF_ITS_PAGE, &places) }],
        })),
        pictures: vec![picture.to_vec()],
        files: vec![PDF.to_vec()],
        sealing: sealing("paseos", Some(key_text(1))),
        cited: vec![Family {
            anchor: plain_tongue("otro", "es", "Otro", &finished["cited"], CITED, 3),
            others: vec![],
        }],
    };
    Family {
        anchor,
        others: vec![plain_tongue(
            "paseosEn",
            "en",
            "Walks",
            &finished["english"],
            IN_ENGLISH,
            2,
        )],
    }
}

fn it_arrived_whole(held: &Value, main: &opening::Opened, picture: &[u8]) {
    let html = held["html"].as_str().unwrap();
    assert_eq!(check_html(html), Vec::<String>::new());
    for said in [
        "tisty:picture/0",
        "tisty:file/0",
        "tisty:page/1",
        "tisty:cited/0",
        "<math",
        "></mspace><mi>b</mi>",
        "hljs-",
        "data-theme=\"light\"",
        "data-theme=\"dark\"",
        "Sin llegar",
    ] {
        assert!(html.contains(said), "{said} is not in {html}");
    }
    assert!(
        html.contains("<svg") && html.contains("span class=\"ico\""),
        "its icon is not drawn: {html}"
    );
    let body = held["body"].as_str().unwrap();
    for text in [html, body] {
        for gone in [
            "01m6",
            "rina.png",
            "secreto",
            "translationkey",
            "(cartilla.pdf",
        ] {
            assert!(
                !text.to_lowercase().contains(gone),
                "{gone} went along in {text}"
            );
        }
        assert!(!text.contains("language-mermaid"), "{text}");
    }
    assert!(
        body.contains("tisty:file/0") && body.contains("tisty:picture/0"),
        "{body}"
    );
    let page = held["pages"][0]["html"].as_str().unwrap();
    assert!(page.contains("data-theme=\"dark\""), "{page}");
    assert_eq!(held["signed_as"], "Rina");
    assert_eq!(held["languages"][0]["document"], "paseosEn");
    assert_eq!(held["languages"][0]["key"], key_text(2));
    assert_eq!(held["cited"][0]["document"], "otro");
    assert_eq!(held["cited"][0]["key"], key_text(3));
    assert_eq!(main.parts[2], picture);
}

#[test]
#[ignore = "app/scripts/journey.mjs runs it after Chrome"]
fn sealed_from_the_window() {
    let raw = std::fs::read(folder().join("finished.json")).unwrap();
    let answers: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let finished: BTreeMap<String, String> = answers
        .iter()
        .map(|one| {
            (
                one["name"].as_str().unwrap().to_owned(),
                one["html"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    for one in &answers[..4] {
        assert_eq!(one["notDrawn"], 0, "{}", one["name"]);
    }

    let (key, certificate) = signer();
    let mut writer = Writer::new(&key, &certificate);
    let picture = made(&STANDARD.decode(PICTURE).unwrap()).unwrap().bytes;
    let going = sealed_family(&mut writer, the_family(&finished, &picture)).unwrap();
    let mut out = Vec::new();
    for one in &going {
        let byte = match one.id.as_str() {
            "paseos" => 1,
            "paseosEn" => 2,
            "otro" => 3,
            other => panic!("{other} was never written"),
        };
        let key = vec![byte; 32];
        let (main, held) = reading(&one.sealed.main, (&one.id, "main"), Some(&key)).unwrap();
        if byte == 1 {
            it_arrived_whole(&held, &main, &picture);
            let files = one.sealed.files.as_ref().unwrap();
            let files = files_for(&held, files, (&one.id, "files"), Some(&key)).unwrap();
            assert_eq!(files.parts[0], PDF);
        }
        out.push(json!({ "name": one.id, "document": held }));
    }
    assert_eq!(going.len(), 3);

    let mut wrong = Vec::new();
    for (at, (name, text)) in real_ones().iter().enumerate() {
        let id = format!("real{at}");
        let said = document(json!({
            "id": id,
            "html": finished[name],
            "body": body_ready(text, &Places::default()),
        }));
        match writer.sealed(said, vec![], vec![], &sealing(&id, None)) {
            Ok(sealed) => {
                let (_, held) = reading(&sealed.main, (&id, "main"), None).unwrap();
                out.push(json!({ "name": name, "document": held }));
            }
            Err(refused) => wrong.push(format!("{name}: {refused}")),
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    std::fs::write(
        folder().join("opened.json"),
        serde_json::to_vec(&out).unwrap(),
    )
    .unwrap();
}
