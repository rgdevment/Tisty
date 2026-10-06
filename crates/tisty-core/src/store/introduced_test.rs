use super::*;

#[test]
fn a_machine_in_the_folder_says_its_key_its_name_when_it_began_and_its_host() {
    let dir = tempfile::tempdir().unwrap();
    let agent = DeviceId("dev_agent".into());
    let mut store = crate::Store::open(dir.path(), agent.clone()).unwrap();
    store
        .append(Op::DeviceJoin {
            d: agent.clone(),
            k: Some(crate::event::DeviceKind::Agent),
            p: Some("aa".into()),
        })
        .unwrap();
    store
        .append(Op::DeviceHost {
            d: agent.clone(),
            of: DeviceId("dev_host".into()),
            p: None,
        })
        .unwrap();
    store
        .append(Op::DeviceNamed {
            d: agent.clone(),
            name: "  Claude  ".into(),
            os: None,
        })
        .unwrap();
    drop(store);

    let said = introduced_in(&dir.path().join("dev_agent"), &agent);

    assert_eq!(said.key.as_deref(), Some("aa"));
    assert_eq!(said.named.map(|one| one.name).as_deref(), Some("Claude"));
    assert_eq!(said.host, Some(DeviceId("dev_host".into())));
    assert!(said.since.is_some());
}

#[test]
fn a_folder_with_nothing_of_it_says_nothing() {
    let dir = tempfile::tempdir().unwrap();

    assert_eq!(
        introduced_in(&dir.path().join("dev_none"), &DeviceId("dev_none".into())),
        Introduced::default()
    );
}

fn written(dir: &Path, signed: bool) -> (DeviceId, crate::model::DocId) {
    let who = DeviceId("dev_w".into());
    let paths = crate::Paths::new(dir.join("data"), dir.join("config"));
    let key = crate::signing::mine(&paths, &who).unwrap();
    let doc = ulid::Ulid::generate();
    let mut store = crate::Store::open(dir.join("store"), who.clone())
        .unwrap()
        .signing_with(signed.then(|| key.clone()));
    store
        .append(Op::DeviceJoin {
            d: who.clone(),
            k: Some(crate::event::DeviceKind::Machine),
            p: Some(crate::signing::shown(&key)),
        })
        .unwrap();
    store
        .append(Op::DocSaid {
            id: doc,
            d: crate::event::Said {
                title: "El expediente".into(),
                print: Some("la-huella".into()),
                ..Default::default()
            },
        })
        .unwrap();
    (who, doc)
}

#[test]
fn a_waiting_history_that_holds_up_under_its_own_key_says_its_prints() {
    let dir = tempfile::tempdir().unwrap();
    let (who, doc) = written(dir.path(), true);

    assert_eq!(
        prints_in(&dir.path().join("store").join("dev_w"), &who, None),
        Some(vec![(doc, "la-huella".to_string())])
    );
}

#[test]
fn a_waiting_history_that_does_not_hold_up_says_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (who, _) = written(dir.path(), false);

    assert_eq!(
        prints_in(&dir.path().join("store").join("dev_w"), &who, None),
        None,
        "an unsigned history claiming a key vouched for a body"
    );
}

#[test]
fn a_history_signed_under_another_key_than_the_one_known_here_says_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (who, _) = written(dir.path(), true);
    let other = crate::signing::shown(
        &crate::signing::mine(
            &crate::Paths::new(dir.path().join("otra"), dir.path().join("otra-config")),
            &who,
        )
        .unwrap(),
    );

    assert_eq!(
        prints_in(&dir.path().join("store").join("dev_w"), &who, Some(&other)),
        None,
        "a history replaced under a fresh key held up because it vouched for itself"
    );
}

#[test]
fn every_print_a_waiting_machine_gave_a_document_counts() {
    let dir = tempfile::tempdir().unwrap();
    let (who, doc) = written(dir.path(), true);
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    let key = crate::signing::mine(&paths, &who).unwrap();
    let mut store = crate::Store::open(dir.path().join("store"), who.clone())
        .unwrap()
        .signing_with(Some(key));
    store
        .append(Op::DocSaid {
            id: doc,
            d: crate::event::Said {
                title: "El expediente".into(),
                print: Some("la-huella-despues".into()),
                ..Default::default()
            },
        })
        .unwrap();
    drop(store);

    let mut said = prints_in(&dir.path().join("store").join("dev_w"), &who, None).unwrap();
    said.sort();

    assert_eq!(
        said,
        vec![
            (doc, "la-huella".to_string()),
            (doc, "la-huella-despues".to_string())
        ],
        "a body this machine wrote earlier and the folder still holds was put to the person"
    );
}
