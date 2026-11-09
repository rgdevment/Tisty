use tisty_core::event::Signature;
use tisty_sharing::Remembered;

use super::{SignsAs, signs_as, who_of};
use crate::answers::folio::card_of;

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
    let card = card_of(&was, &Signature::default(), Some("juan-root")).unwrap();
    let said = serde_json::to_value(&card).unwrap();
    assert_eq!(
        said["owner"],
        serde_json::json!({ "name": "Mario Hidalgo", "checked": false })
    );
    assert_eq!(
        said["signer"],
        serde_json::json!({ "name": "Ana", "checked": false })
    );
    assert_eq!(
        said["editors"],
        serde_json::json!([
            { "name": "Ana", "checked": false },
            { "name": "Juan", "checked": true },
        ])
    );

    let mine = card_of(
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
    let who = who_of(&was, Some("mario-root"));
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
    let who = who_of(&was, Some("mario-root"));
    assert!(who.signer.is_some_and(|one| !one.checked));
}
