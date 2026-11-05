//! The whole way a document goes, from a real store to what a reader opens. A window colours and draws in the middle, so `app/scripts/journey.mjs` runs these two around Chrome.

#[allow(dead_code)]
mod common;
#[rustfmt::skip]
#[path = "../../../contracts/dist/rust/tests/vectors.rs"]
mod opening;
#[path = "../../tisty-envelope/tests/real/mod.rs"]
mod real;

use std::collections::BTreeMap;
use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::{Home, a_picture, asked};
use opening::{files_for, picture_holds, reading};
use real::every_real_document;
use serde_json::{Value, json};
use tisty_contracts::{EnvelopeKind, Opening, Served};
use tisty_core::model::DocId;
use tisty_sharing::{Asked, Finished, Sealed, Told};

const PDF: &[u8] = b"%PDF-1.7 cartilla";
const OF_ITS_PAGE: &str = "# Enero\n\n```mermaid\npie\n  \"a\" : 1\n```\n";
const CITED: &str = "# Otro\n\nAlgo aparte.\n";

fn written(picture: &str, file: &str) -> String {
    format!(
        r#"---
title: Paseos
lang: es
paper: letter
tags: [rina]
Task: 01M6
TranslationKey: paseos
list: Trabajo privado
---

# Paseos

Con ::dog orange:: y $a\,b + c$, Rina.

![Rina]({picture})

[La cartilla]({file} "cartilla.pdf"), [[Enero]], [[Otro]], [la nota][n] y [[Sin llegar]].

[n]: C:/Users/mario/notas/secreto.md

```rust
fn main() {{}}
```

```mermaid
flowchart TD
  A[Inicio] --> B[Fin]
```
"#
    )
}

fn in_english(picture: &str) -> String {
    format!("---\nlang: en\n---\n\n# Walks\n\nWith Rina.\n\n![Rina]({picture})\n")
}

fn folder() -> PathBuf {
    std::env::var_os("TISTY_JOURNEY")
        .expect("app/scripts/journey.mjs says where the journey is kept")
        .into()
}

fn home() -> Home {
    Home::at(&folder().join("home"))
}

fn asked_for(all: &Value) -> Vec<(String, Asked)> {
    let doc = |said: &Value| said.as_str().unwrap().parse::<DocId>().unwrap();
    let mut anchor = asked(doc(&all["anchor"]));
    anchor.cited = vec![doc(&all["cited"])];
    anchor.signed_as = Some("Rina".into());
    let mut out = vec![("written".to_owned(), anchor)];
    for (at, one) in all["real"].as_array().unwrap().iter().enumerate() {
        let mut each = asked(doc(&one["doc"]));
        // Half go as what is public, so both ways of opening meet every document there is.
        each.public = at % 2 == 1;
        out.push((one["name"].as_str().unwrap().to_owned(), each));
    }
    out
}

#[test]
#[ignore = "app/scripts/journey.mjs runs it before Chrome"]
fn ready_for_the_window() {
    let mut home = home();
    home.with_a_name();
    let picture = home.attached("rina.png", &a_picture(32, 24, 1));
    let cartilla = home.attached("cartilla.pdf", PDF);

    let anchor = home.wrote(&written(&picture, &cartilla), None);
    home.wrote(OF_ITS_PAGE, Some(anchor));
    let english = home.wrote(&in_english(&picture), None);
    home.joined(english, anchor);
    let cited = home.wrote(CITED, None);

    let real: Vec<Value> = every_real_document()
        .into_iter()
        .map(|(name, text)| json!({ "name": name, "doc": home.wrote(&text, None).to_string() }))
        .collect();
    let all = json!({ "anchor": anchor.to_string(), "cited": cited.to_string(), "real": real });

    let mut ready = Vec::new();
    for (_, one) in asked_for(&all) {
        ready.extend(home.gathered(&one).unfinished());
    }
    std::fs::write(folder().join("asked.json"), all.to_string()).unwrap();
    std::fs::write(
        folder().join("ready.json"),
        serde_json::to_vec(&ready).unwrap(),
    )
    .unwrap();
}

fn answers_for(gathered: &tisty_sharing::Gathered, all: &BTreeMap<String, Value>) -> Vec<Finished> {
    gathered
        .unfinished()
        .into_iter()
        .map(|one| {
            let said = &all[&one.name];
            Finished {
                name: one.name,
                html: said["html"].as_str().unwrap().to_owned(),
                not_drawn: said["notDrawn"].as_u64().unwrap() as usize,
            }
        })
        .collect()
}

fn key_of(sealed: &Sealed, id: &str) -> Option<String> {
    sealed
        .remembering
        .iter()
        .find(|(_, one)| one.id == id)
        .and_then(|(_, one)| one.link_key.clone())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|one| format!("{one:02x}")).collect()
}

fn it_arrived_whole(held: &Value, main: &opening::Opened) {
    let html = held["html"].as_str().unwrap();
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
            "trabajo privado",
            "translationkey",
            "(cartilla",
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
    assert_eq!(held["title"], "Paseos");
    assert_eq!(held["lang"], "es");
    assert_eq!(held["sheet"]["paper"], "letter");
    assert_eq!(held["signed_as"], "Rina");
    assert_eq!(held["tags"], json!(["rina"]));
    assert_eq!(held["files"][0]["name"], "cartilla.pdf");
    assert_eq!(held["languages"][0]["lang"], "en");
    assert_eq!(held["languages"][0]["title"], "Walks");
    assert!(held["languages"][0]["key"].is_string());
    assert_eq!(held["cited"][0]["title"], "Otro");
    assert!(held["cited"][0]["key"].is_string());
    assert!(picture_holds(&main.parts[2], &held["pictures"][0]));
}

fn nothing_served() -> Served {
    Served {
        received: None,
        revocations: None,
    }
}

// The contract's reader and Tisty's have to say the same of every envelope.
fn it_opens(sealed: &Sealed, out: &mut Vec<Value>, opened: &mut Vec<Value>, name: &str) {
    for one in &sealed.going {
        let key = key_of(sealed, &one.id);
        let bytes = key
            .as_ref()
            .map(|said| URL_SAFE_NO_PAD.decode(said).unwrap());
        let asked = (one.id.as_str(), "main");
        let (main, held) = reading(&one.sealed.main, asked, bytes.as_deref()).unwrap();
        let opening = Opening {
            id: one.id.clone(),
            kind: EnvelopeKind::Main,
            link_key: key.clone(),
        };
        let ours = tisty_envelope::reading::reading(&one.sealed.main, &opening, &nothing_served())
            .unwrap();
        assert_eq!(ours.document, held);

        if name == "written" && held["lang"] == "es" {
            it_arrived_whole(&held, &main);
        }
        if let Some(files) = &one.sealed.files {
            let asked = (one.id.as_str(), "files");
            let files = files_for(&held, files, asked, bytes.as_deref()).unwrap();
            assert_eq!(files.parts[0], PDF);
        }
        out.push(json!({
            "id": one.id,
            "key": key,
            "main": hex(&one.sealed.main),
            "files": one.sealed.files.as_deref().map(hex),
        }));
        opened.push(json!({ "name": name, "document": held }));
    }
}

#[test]
#[ignore = "app/scripts/journey.mjs runs it after Chrome"]
fn sealed_from_the_window() {
    let raw = std::fs::read(folder().join("finished.json")).unwrap();
    let answers: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let answers: BTreeMap<String, Value> = answers
        .into_iter()
        .map(|one| (one["name"].as_str().unwrap().to_owned(), one))
        .collect();
    let all: Value =
        serde_json::from_slice(&std::fs::read(folder().join("asked.json")).unwrap()).unwrap();
    let home = home();

    let mut out = Vec::new();
    let mut opened = Vec::new();
    let mut wrong = Vec::new();
    for (name, one) in asked_for(&all) {
        let mut gathered = home.gathered(&one);
        let finished = answers_for(&gathered, &answers);
        gathered.finished(finished).unwrap();
        let (key, certificate) = home.signer();
        let mut writer = tisty_envelope::sealing::Writer::new(&key, &certificate);
        match gathered.sealed(&mut writer) {
            Ok(sealed) => {
                if name == "written" {
                    assert_eq!(sealed.going.len(), 3, "its two languages and what it cites");
                    assert_eq!(sealed.told, Vec::<Told>::new());
                }
                it_opens(&sealed, &mut out, &mut opened, &name);
            }
            Err(refused) => wrong.push(format!("{name}: {refused}")),
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    std::fs::write(
        folder().join("sealed.json"),
        serde_json::to_vec(&out).unwrap(),
    )
    .unwrap();
    std::fs::write(
        folder().join("opened.json"),
        serde_json::to_vec(&opened).unwrap(),
    )
    .unwrap();
}
