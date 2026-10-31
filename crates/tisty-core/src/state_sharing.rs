use super::*;
use crate::event::{DeviceId, Event, Shared};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn by(ms: i64, who: &str, op: Op) -> Event {
    let mut event = Event::new(DeviceId(who.into()), at(ms), op);
    event.optional = event.op.is_optional();
    event
}

fn shared(version: u64) -> Shared {
    Shared {
        server: "https://tisty.cl".into(),
        link: "k7Qm2xPa".into(),
        key: "tWj3cJFtRiwp6ms2QS6DKXwHL2aGnEB6HcDUhX9r99E".into(),
        edit: "edit-token".into(),
        admin: "admin-token".into(),
        editable: false,
        gated: false,
        version,
    }
}

#[test]
fn a_shared_document_keeps_its_links_and_the_highest_version_either_machine_put_up() {
    let doc = Ulid::generate();
    let state = State::replay(&[
        by(
            1,
            "dev_a",
            Op::ShareDoc {
                id: doc,
                d: shared(1),
            },
        ),
        by(
            2,
            "dev_b",
            Op::ShareVersion {
                id: doc,
                version: 3,
            },
        ),
        by(
            3,
            "dev_a",
            Op::ShareVersion {
                id: doc,
                version: 2,
            },
        ),
    ]);
    let kept = state.shares.get(&doc).expect("the share is kept");
    assert_eq!(kept.version, 3);
    assert_eq!(kept.link, "k7Qm2xPa");
    assert_eq!(kept.admin, "admin-token");
}

#[test]
fn stopping_forgets_the_links_and_a_version_after_it_brings_nothing_back() {
    let doc = Ulid::generate();
    let state = State::replay(&[
        by(
            1,
            "dev_a",
            Op::ShareDoc {
                id: doc,
                d: shared(1),
            },
        ),
        by(2, "dev_a", Op::ShareStop { id: doc }),
        by(
            3,
            "dev_b",
            Op::ShareVersion {
                id: doc,
                version: 2,
            },
        ),
    ]);
    assert!(state.shares.is_empty());
}

#[test]
fn sharing_is_optional_for_an_older_tisty_and_undo_steps_over_it() {
    let doc = Ulid::generate();
    for op in [
        Op::ShareDoc {
            id: doc,
            d: shared(1),
        },
        Op::ShareVersion {
            id: doc,
            version: 2,
        },
        Op::ShareStop { id: doc },
    ] {
        assert!(op.is_optional(), "{op:?}");
        assert!(op.settles(), "{op:?}");
        assert!(!op.destroys(), "{op:?}");
        assert!(crate::event::KNOWN_OPS.contains(&name_of(&op).as_str()));
    }
}

#[test]
fn a_share_travels_in_the_log_with_only_what_it_says() {
    let line = serde_json::to_value(Op::ShareDoc {
        id: Ulid::nil(),
        d: shared(1),
    })
    .unwrap();
    assert_eq!(line["op"], "share.doc");
    assert_eq!(line["d"]["version"], 1);
    assert!(line["d"].get("editable").is_none());
    assert!(line["d"].get("gated").is_none());
}

fn name_of(op: &Op) -> String {
    serde_json::to_value(op).unwrap()["op"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn every_share_op_is_written_to_the_log_and_read_back_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = crate::store::Store::open(dir.path(), DeviceId("dev_a".into())).unwrap();
    let doc = Ulid::generate();
    for op in [
        Op::ShareDoc {
            id: doc,
            d: shared(1),
        },
        Op::ShareVersion {
            id: doc,
            version: 2,
        },
        Op::ShareStop { id: doc },
    ] {
        let said = store.append(op.clone()).unwrap();
        let line = serde_json::to_string(&said).unwrap();
        let back: Event = serde_json::from_str(&line).unwrap();
        assert_eq!(back.op, op);
    }
}
