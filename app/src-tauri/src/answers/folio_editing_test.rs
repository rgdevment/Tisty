use tisty_core::event::Signature;
use tisty_sharing::Remembered;

use super::{SignsAs, others_asked, said_after_sending, signs_as, who_of};
use crate::answers::folio::card_of;

fn maker(root: &str) -> tisty_carrier::folio_limits::FolioMaker {
    tisty_carrier::folio_limits::FolioMaker {
        root: root.to_owned(),
        name: Some("rgdevment".to_owned()),
        domain: Some("rgdevment.dev".to_owned()),
    }
}

fn signature(alias: Option<&str>, name: Option<&str>) -> Signature {
    Signature {
        alias: alias.map(str::to_owned),
        name: name.map(str::to_owned),
        ..Signature::default()
    }
}

fn arrived(admin: Option<&str>, token: Option<&str>, signed_as: Option<&str>) -> Remembered {
    Remembered {
        id: "Ab3xK9pQ".to_owned(),
        version: 3,
        link_key: Some("key".to_owned()),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: admin.map(str::to_owned),
        edit_token: token.map(str::to_owned),
        words_print: None,
        signed_as: signed_as.map(str::to_owned),
        owner: Some(r#"{"root":"mario-root"}"#.to_owned()),
        owner_name: Some("Mario Hidalgo".to_owned()),
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: None,
        sealed_under: None,
        version_by: Some("Ana".to_owned()),
        version_root: None,
        left_out: Vec::new(),
    }
}

#[test]
fn whoever_edits_through_a_link_signs_as_their_alias_until_they_choose_their_name() {
    let mine = signature(Some("ana.dev"), Some("Ana Pérez"));
    assert_eq!(
        signs_as(&arrived(None, Some("token"), None), &mine),
        Some(SignsAs {
            now: Some("ana.dev".to_owned()),
            other: Some("Ana Pérez".to_owned()),
        })
    );
    assert_eq!(
        signs_as(&arrived(None, Some("token"), Some("Ana Pérez")), &mine),
        Some(SignsAs {
            now: Some("Ana Pérez".to_owned()),
            other: Some("ana.dev".to_owned()),
        })
    );
}

#[test]
fn the_row_names_what_sealing_falls_back_to_and_promises_no_name_without_an_alias() {
    let named_only = signature(None, Some("Ana Pérez"));
    assert_eq!(
        signs_as(&arrived(None, Some("token"), None), &named_only),
        Some(SignsAs {
            now: None,
            other: Some("Ana Pérez".to_owned()),
        })
    );
    let mine = signature(Some("ana.dev"), None);
    assert_eq!(
        signs_as(&arrived(None, Some("token"), Some("Ana Pérez")), &mine),
        Some(SignsAs {
            now: Some("Ana Pérez".to_owned()),
            other: Some("ana.dev".to_owned()),
        })
    );
}

#[test]
fn only_whoever_edits_through_a_link_is_asked_how_they_sign() {
    let mine = signature(Some("ana.dev"), Some("Ana Pérez"));
    assert_eq!(
        signs_as(&arrived(Some("admin"), Some("token"), None), &mine),
        None
    );
    assert_eq!(signs_as(&arrived(None, None, None), &mine), None);
    assert_eq!(
        signs_as(&arrived(None, Some("token"), None), &Signature::default()),
        Some(SignsAs {
            now: None,
            other: None,
        })
    );
}

fn edited(by: &[(&str, &str, &str, bool)]) -> Option<String> {
    let editors: Vec<serde_json::Value> = by
        .iter()
        .map(|(id, root, name, owned)| {
            serde_json::json!({
                "in": id, "root": root, "machine": "m", "since": "2026-11-03T12:00:00Z",
                "certificate": "c", "name": name, "owned": owned, "signed": "s",
            })
        })
        .collect();
    Some(serde_json::json!({ "editors": editors }).to_string())
}

#[test]
fn a_copy_names_who_created_it_who_edited_it_last_and_everyone_who_edited_it() {
    let was = Remembered {
        carried: edited(&[
            ("Ab3xK9pQ", "mario-root", "Mario Hidalgo", true),
            ("Ab3xK9pQ", "ana-root", "Ana", false),
            ("Ab3xK9pQ", "juan-root", "Juan", false),
            ("otroDoc", "pedro-root", "Pedro", false),
        ]),
        ..arrived(None, None, None)
    };
    let card = card_of(
        "tisty.cl",
        &was,
        &Signature::default(),
        Some(&maker("juan-root")),
    )
    .unwrap();
    let said = serde_json::to_value(&card).unwrap();
    assert_eq!(
        said["owner"],
        serde_json::json!({ "name": "Mario Hidalgo", "checked": false, "print": null })
    );
    assert_eq!(
        said["signer"],
        serde_json::json!({ "name": "Ana", "checked": false, "print": null })
    );
    assert_eq!(
        said["editors"],
        serde_json::json!([
            { "name": "Ana", "checked": false, "print": null },
            { "name": "Juan", "checked": true, "print": null },
        ])
    );

    let mine = card_of(
        "tisty.cl",
        &arrived(Some("admin"), None, None),
        &Signature::default(),
        None,
    )
    .unwrap();
    let said = serde_json::to_value(&mine).unwrap();
    assert_eq!(said["owner"], serde_json::Value::Null);
}

#[test]
fn the_makers_root_is_checked_wherever_it_signs() {
    let was = Remembered {
        carried: edited(&[("Ab3xK9pQ", "mario-root", "Mario Hidalgo", true)]),
        version_by: Some("Mario Hidalgo".to_owned()),
        version_root: Some("mario-root".to_owned()),
        ..arrived(None, None, None)
    };
    let who = who_of(&was, Some(&maker("mario-root")));
    assert!(who.owner.is_some_and(|one| one.checked));
    assert!(who.signer.is_some_and(|one| one.checked));
    assert!(who.editors.is_empty());
}

#[test]
fn whoever_signs_with_the_makers_name_under_another_root_is_not_checked() {
    let was = Remembered {
        carried: edited(&[("Ab3xK9pQ", "mario-root", "rgdevment", true)]),
        version_by: Some("rgdevment".to_owned()),
        version_root: Some("impostor-root".to_owned()),
        ..arrived(None, None, None)
    };
    let who = who_of(&was, Some(&maker("mario-root")));
    let signer = serde_json::to_value(who.signer.unwrap()).unwrap();
    assert_eq!(signer["checked"], false);
    assert_eq!(signer["imitates"], true);
    let owner = serde_json::to_value(who.owner.unwrap()).unwrap();
    assert_eq!(owner["checked"], true);
    assert!(owner.get("imitates").is_none());
}

#[test]
fn everyone_named_on_a_copy_carries_the_fingerprint_their_security_shows() {
    let person = |byte| {
        tisty_share_proto::seed::Seed::from_bytes(&[byte; 32])
            .root()
            .person()
    };
    let (mario, ana, juan) = (person(1), person(2), person(3));
    let was = Remembered {
        owner: Some(serde_json::json!({ "root": mario.shown() }).to_string()),
        carried: edited(&[
            ("Ab3xK9pQ", &ana.shown(), "Ana", false),
            ("Ab3xK9pQ", &juan.shown(), "Juan", false),
        ]),
        version_root: Some(ana.shown()),
        ..arrived(None, None, None)
    };

    let who = who_of(&was, None);

    assert_eq!(
        who.owner.and_then(|one| one.print),
        Some(mario.fingerprint())
    );
    assert_eq!(
        who.signer.and_then(|one| one.print),
        Some(ana.fingerprint())
    );
    let editors: Vec<_> = who.editors.into_iter().map(|one| one.print).collect();
    assert_eq!(
        editors,
        vec![Some(ana.fingerprint()), Some(juan.fingerprint())]
    );
}

#[test]
fn others_are_asked_about_before_sending_and_never_told_after() {
    let told = vec![
        tisty_sharing::Told::NameLeftOut,
        tisty_sharing::Told::OthersEdited {
            editors: vec!["Ana".to_owned()],
        },
    ];
    assert_eq!(others_asked(&told), vec!["Ana".to_owned()]);
    assert_eq!(
        said_after_sending(told),
        vec![tisty_sharing::Told::NameLeftOut]
    );
    assert!(others_asked(&[]).is_empty());
}

#[test]
fn changing_the_edit_link_reaches_every_language_this_tisty_owns_to_edit() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let mut session = crate::Session::at(paths).unwrap();
    let add = |session: &mut crate::Session, body: &str| {
        let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body)
            .unwrap();
        let id = ulid::Ulid::generate();
        session
            .commit(tisty_core::Op::DocAdd {
                id,
                d: tisty_core::event::DocAdd {
                    file: made.id.clone(),
                    order: format!("a{id}"),
                    said: Some(tisty_core::event::Said::of(body)),
                    ..Default::default()
                },
            })
            .unwrap();
        (id, made.id)
    };
    let (es, es_file) = add(&mut session, "---\nlang: es\n---\n# Viaje\n");
    let (en, en_file) = add(&mut session, "---\nlang: en\n---\n# Trip\n");
    let (fr, fr_file) = add(&mut session, "---\nlang: fr\n---\n# Voyage\n");
    for one in [en, fr] {
        session
            .commit(tisty_core::Op::DocTongue {
                id: one,
                d: tisty_core::event::Tongue { of: Some(es) },
            })
            .unwrap();
    }
    let entries = [
        (&es_file, "Spanish1", Some("t")),
        (&en_file, "English1", Some("t")),
        (&fr_file, "French12", None),
    ];
    for (file, id, token) in entries {
        session
            .commit(tisty_core::Op::SharingKept {
                file: file.clone(),
                d: Remembered {
                    id: id.to_owned(),
                    ..arrived(Some("admin"), token, None)
                },
            })
            .unwrap();
    }

    let languages = super::its_languages(&session, &en.to_string()).unwrap();
    let ids: Vec<&str> = languages.iter().map(|(_, one)| one.id.as_str()).collect();

    assert_eq!(ids, ["English1", "Spanish1"]);
}
