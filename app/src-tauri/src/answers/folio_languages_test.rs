use tisty_sharing::Remembered;

use super::{
    Proposed, adopted_here, answered, known_ids, proposals_merged, proposed_of, shown_of,
    withdrawn_by,
};
use crate::answers::folio_receiving::Opened;

fn proposing(lang: &str, by: &str) -> Proposed {
    Proposed {
        lang: lang.into(),
        document: format!("{lang}Doc12"),
        key: "key".into(),
        by: Some(by.into()),
    }
}

fn entry(id: &str, admin: Option<&str>, held_by: Option<&str>) -> Remembered {
    Remembered {
        id: id.into(),
        version: 2,
        link_key: Some("key".into()),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: admin.map(str::to_owned),
        edit_token: Some("token".into()),
        words_print: None,
        signed_as: None,
        owner: None,
        owner_name: None,
        cited: Vec::new(),
        held_by: held_by.map(str::to_owned),
        ttl_days: None,
        carried: None,
        sealed_under: None,
        version_by: None,
        version_root: None,
        left_out: Vec::new(),
    }
}

fn session() -> (tempfile::TempDir, crate::Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = crate::Session::at(paths).unwrap();
    (tmp, session)
}

fn kept(session: &mut crate::Session, file: &str, was: Remembered) {
    session
        .commit(tisty_core::Op::SharingKept {
            file: file.into(),
            d: was,
        })
        .unwrap();
}

fn naming(ids: &[&str]) -> Opened {
    let languages: Vec<_> = ids
        .iter()
        .map(|id| serde_json::json!({ "lang": "fr", "document": id }))
        .collect();
    Opened {
        document: serde_json::json!({ "languages": languages }),
        pictures: Vec::new(),
        files: Vec::new(),
        owner: None,
        owner_root: None,
        signer: "r".into(),
        version: 3,
        verdict: tisty_contracts::Verdict::Holds,
        contract: None,
    }
}

#[test]
fn a_proposal_is_kept_beside_the_editors_and_the_latest_for_a_language_wins() {
    let was = Remembered {
        carried: Some(r#"{"editors":[]}"#.into()),
        ..entry("Anchor12", Some("admin"), None)
    };
    let once = Remembered {
        carried: proposals_merged(&was, vec![proposing("en", "ana")]),
        ..was
    };
    let twice = Remembered {
        carried: proposals_merged(&once, vec![proposing("en", "bea"), proposing("fr", "ana")]),
        ..once
    };

    assert_eq!(shown_of(&twice).len(), 2);
    assert!(twice.carried.as_deref().unwrap().contains("\"editors\""));
    assert_eq!(proposed_of(&twice)[0].by.as_deref(), Some("bea"));
}

#[test]
fn an_answered_language_is_never_proposed_again() {
    let (_tmp, mut session) = session();
    let was = entry("Anchor12", Some("admin"), None);
    let waiting = Remembered {
        carried: proposals_merged(&was, vec![proposing("en", "ana"), proposing("fr", "ana")]),
        ..was
    };
    kept(&mut session, "anchor", waiting.clone());
    kept(
        &mut session,
        "spanish",
        entry("Spanish1", Some("admin"), Some("Anchor12")),
    );
    let done = answered(&waiting, &proposing("en", "ana"));

    let known = known_ids(&session.state, &done);
    assert_eq!(shown_of(&done).len(), 1);
    for id in ["Anchor12", "Spanish1", "enDoc12", "frDoc12"] {
        assert!(known.contains(id), "{id} could be proposed again");
    }
}

#[test]
fn an_editor_stops_sending_the_language_the_owner_s_version_no_longer_names() {
    let (_tmp, mut session) = session();
    let anchor = entry("Anchor12", None, None);
    kept(&mut session, "anchor", anchor.clone());
    kept(
        &mut session,
        "mine",
        entry("MyFrench", Some("admin"), Some("Anchor12")),
    );
    kept(
        &mut session,
        "theirs",
        entry("English1", None, Some("Anchor12")),
    );

    let named = withdrawn_by(&session.state, &anchor, &naming(&["MyFrench"]));
    assert!(named.is_empty());

    let gone = withdrawn_by(&session.state, &anchor, &naming(&["Adopted1"]));
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0].1.id, "MyFrench");
    assert!(
        gone[0]
            .1
            .carried
            .as_deref()
            .unwrap()
            .contains("\"withdrawn\":true")
    );
}

#[test]
fn an_adopted_language_is_the_owner_s_own_and_joins_the_others() {
    let (_tmp, mut session) = session();
    session
        .commit(tisty_core::Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some("mario".into()),
                ..Default::default()
            },
        })
        .unwrap();
    let body = "---\nlang: es\n---\n# Viaje\n";
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    let anchor = ulid::Ulid::generate();
    session
        .commit(tisty_core::Op::DocAdd {
            id: anchor,
            d: tisty_core::event::DocAdd {
                file: made.id,
                order: "a0".into(),
                said: Some(tisty_core::event::Said::of(body)),
                by: Some("mario".into()),
                ..Default::default()
            },
        })
        .unwrap();
    let received = tisty_sharing::Received {
        title: None,
        body: "---\nlang: fr\n---\n# Voyage\n".into(),
        pages: Vec::new(),
        owner_name: None,
        signed_as: Some("ana".into()),
        carried: None,
    };

    let adopted = adopted_here(&mut session, &received, anchor).unwrap();
    let kept = &session.state.docs[&adopted];

    assert!(!kept.guest);
    assert!(session.state.mine(kept));
    assert_eq!(session.state.anchor_of(kept), anchor);
}
