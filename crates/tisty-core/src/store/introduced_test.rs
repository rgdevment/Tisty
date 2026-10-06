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
