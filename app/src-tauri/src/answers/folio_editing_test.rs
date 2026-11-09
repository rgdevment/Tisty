use tisty_core::event::Signature;
use tisty_sharing::Remembered;

use super::{SignsAs, signs_as};
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
        owner: Some("{}".to_owned()),
        owner_name: Some("Mario Hidalgo".to_owned()),
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: None,
        sealed_under: None,
        version_by: Some("Ana".to_owned()),
    }
}

#[test]
fn whoever_edits_through_a_link_signs_as_their_alias_until_they_choose_their_name() {
    let mine = signature(Some("ana.dev"), Some("Ana Pérez"));
    assert_eq!(
        signs_as(&arrived(None, Some("token"), None), &mine),
        Some(SignsAs {
            now: "ana.dev".to_owned(),
            other: Some("Ana Pérez".to_owned()),
        })
    );
    assert_eq!(
        signs_as(&arrived(None, Some("token"), Some("Ana Pérez")), &mine),
        Some(SignsAs {
            now: "Ana Pérez".to_owned(),
            other: Some("ana.dev".to_owned()),
        })
    );
}

#[test]
fn a_name_that_is_no_longer_one_of_mine_gives_way_to_one_that_is() {
    let mine = signature(Some("ana.dev"), None);
    assert_eq!(
        signs_as(&arrived(None, Some("token"), Some("Ana Pérez")), &mine),
        Some(SignsAs {
            now: "ana.dev".to_owned(),
            other: None,
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
        None
    );
}

#[test]
fn a_copy_names_its_owner_and_whoever_signed_the_version_it_holds() {
    let card = card_of(&arrived(None, None, None), &Signature::default()).unwrap();
    let said = serde_json::to_value(&card).unwrap();
    assert_eq!(said["owner"], "Mario Hidalgo");
    assert_eq!(said["signer"], "Ana");

    let mine = card_of(&arrived(Some("admin"), None, None), &Signature::default()).unwrap();
    let said = serde_json::to_value(&mine).unwrap();
    assert_eq!(said["owner"], serde_json::Value::Null);
}
