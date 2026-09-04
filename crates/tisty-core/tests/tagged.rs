use tisty_core::{DeviceId, Op, Store};

/// An old build clearing what it cannot see: the tags of a document it saved.
#[test]
fn a_note_from_an_older_build_leaves_the_tags_where_they_are() {
    let doc = ulid::Ulid::generate();
    let add = format!(
        r#"{{"v":10,"ts":"2026-09-04T22:00:00Z","by":"dev_a3f9","op":"doc.add","id":"{doc}","d":{{"file":"a3f1-0001","order":"a0","said":{{"title":"Alquiler","bytes":31,"tags":["legal","dinero"]}}}}}}"#
    );
    let old = format!(
        r#"{{"v":10,"ts":"2026-09-04T22:01:00Z","by":"dev_b111","opt":true,"op":"doc.said","id":"{doc}","d":{{"title":"Alquiler","bytes":40}}}}"#
    );

    let room = tempfile::tempdir().unwrap();
    let dir = room.path().join("store").join("dev_a3f9");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("active.tisty"), format!("{add}\n{old}\n")).unwrap();

    let events = tisty_core::store::read_all(room.path().join("store")).unwrap();
    let state = tisty_core::State::replay(&events);
    let kept = state.docs.values().next().unwrap();

    assert_eq!(
        kept.tags.len(),
        2,
        "una maquina en la version anterior borro las etiquetas del documento al guardarlo"
    );
}

/// The round trip `body -> tags -> log -> state -> body` has to settle, or a save that writes a
/// note would write another one every time the document is read.
#[test]
fn the_tags_of_a_body_settle_after_one_note() {
    let body = "# Alquiler\n\nesto es #legal y #dinero y otra vez #Legal\n";
    let doc = ulid::Ulid::generate();
    let said = tisty_core::event::Said::of(body);

    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());
    let mut store = Store::open(&root, device.clone()).unwrap();
    store
        .append(Op::DocAdd {
            id: doc,
            d: tisty_core::event::DocAdd {
                file: "a3f1-0001".into(),
                order: "a0".into(),
                said: Some(said.clone()),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();
    drop(store);

    let events = tisty_core::store::read_all(&root).unwrap();
    let state = tisty_core::State::replay(&events);
    let kept = state.docs.get(&doc).unwrap();

    assert!(
        !tisty_core::event::Said::of(body).news_for(kept),
        "leer el mismo cuerpo otra vez volveria a escribir una nota: bucle"
    );
}
