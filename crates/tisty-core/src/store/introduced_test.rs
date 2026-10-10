use super::*;

fn a_key(seed: u8) -> String {
    crate::signing::shown(&ed25519_dalek::SigningKey::from_bytes(&[seed; 32]))
}

#[test]
fn a_machine_in_the_folder_says_its_key_its_name_when_it_began_and_its_host() {
    let dir = tempfile::tempdir().unwrap();
    let agent = DeviceId("dev_agent".into());
    let mut store = crate::Store::open(dir.path(), agent.clone()).unwrap();
    store
        .append(Op::DeviceJoin {
            d: agent.clone(),
            k: Some(crate::event::DeviceKind::Agent),
            p: Some(a_key(1)),
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

    assert_eq!(said.key.as_deref(), Some(a_key(1).as_str()));
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

#[test]
fn the_key_a_machine_is_answered_for_is_the_first_one_that_reads() {
    let dir = tempfile::tempdir().unwrap();
    let who = DeviceId("dev_b".into());
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    let good = crate::signing::shown(&crate::signing::mine(&paths, &who).unwrap());
    let other = DeviceId("dev_c".into());
    let theirs = crate::signing::shown(&crate::signing::mine(&paths, &other).unwrap());
    let mut store = crate::Store::open(dir.path().join("store"), who.clone()).unwrap();
    for (d, p) in [
        (&who, "not a key"),
        (&other, theirs.as_str()),
        (&who, good.as_str()),
    ] {
        store
            .append(Op::DeviceKey {
                d: d.clone(),
                p: p.into(),
            })
            .unwrap();
    }
    drop(store);
    let at = dir.path().join("store").join("dev_b");

    assert_eq!(
        crate::store::key_said_in(&at, &who).as_deref(),
        Some(good.as_str())
    );
    assert_eq!(
        introduced_in(&at, &who).key.as_deref(),
        Some(good.as_str()),
        "a person was shown a key they cannot answer for"
    );
    assert!(crate::store::says_a_key_in(&at, &who));
    assert!(!crate::store::says_a_key_in(&at, &DeviceId("dev_z".into())));
}

#[test]
fn the_key_shown_for_a_machine_is_the_one_it_rotated_to() {
    let dir = tempfile::tempdir().unwrap();
    let who = DeviceId("dev_b".into());
    let old = a_key(1);
    let new = a_key(2);
    let mut store = crate::Store::open(dir.path().join("store"), who.clone()).unwrap();
    for op in [
        Op::DeviceKey {
            d: who.clone(),
            p: old.clone(),
        },
        Op::DeviceRotate {
            d: who.clone(),
            p: new.clone(),
        },
    ] {
        store.append(op).unwrap();
    }
    drop(store);
    let at = dir.path().join("store").join("dev_b");

    assert_eq!(
        crate::store::key_said_in(&at, &who).as_deref(),
        Some(new.as_str()),
        "a person was shown the key the machine moved away from"
    );
}

#[test]
fn a_rotation_from_no_key_plants_none_and_still_owes_a_signature() {
    let dir = tempfile::tempdir().unwrap();
    let who = DeviceId("dev_b".into());
    let mut store = crate::Store::open(dir.path().join("store"), who.clone()).unwrap();
    store
        .append(Op::DeviceRotate {
            d: who.clone(),
            p: a_key(2),
        })
        .unwrap();
    drop(store);
    let at = dir.path().join("store").join("dev_b");

    assert!(crate::store::says_a_key_in(&at, &who));
    assert_eq!(crate::store::key_said_in(&at, &who), None);
    let mut keys = std::collections::BTreeMap::new();
    crate::signing::rotated(&mut keys, &who, &who, &a_key(2));
    assert!(keys.is_empty(), "a rotation from no key planted one");
}
