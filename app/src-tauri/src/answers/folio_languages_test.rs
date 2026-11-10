use tisty_sharing::Remembered;

use super::{Proposed, proposals_merged, proposed_of, shown_of, without};

fn proposing(lang: &str, by: &str) -> Proposed {
    Proposed {
        lang: lang.into(),
        document: format!("{lang}Doc12"),
        key: "key".into(),
        by: Some(by.into()),
    }
}

fn owner(carried: Option<&str>) -> Remembered {
    Remembered {
        id: "Anchor12".into(),
        version: 2,
        link_key: Some("key".into()),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: Some("admin".into()),
        edit_token: Some("token".into()),
        words_print: None,
        signed_as: None,
        owner: None,
        owner_name: None,
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: carried.map(str::to_owned),
        sealed_under: None,
        version_by: None,
        version_root: None,
        left_out: Vec::new(),
    }
}

#[test]
fn a_proposal_is_kept_beside_the_editors_and_the_latest_for_a_language_wins() {
    let was = owner(Some(r#"{"editors":[]}"#));
    let once = Remembered {
        carried: proposals_merged(&was, vec![proposing("en", "ana")]),
        ..was
    };
    let twice = Remembered {
        carried: proposals_merged(&once, vec![proposing("en", "bea"), proposing("fr", "ana")]),
        ..once
    };

    let shown = shown_of(&twice);
    assert_eq!(shown.len(), 2);
    assert!(twice.carried.as_deref().unwrap().contains("\"editors\""));
    assert_eq!(proposed_of(&twice)[0].by.as_deref(), Some("bea"));
}

#[test]
fn a_reviewed_language_leaves_the_proposals_and_nothing_else() {
    let was = owner(None);
    let proposing = Remembered {
        carried: proposals_merged(&was, vec![proposing("en", "ana")]),
        ..was
    };

    let done = without(&proposing, "en");

    assert!(proposed_of(&done).is_empty());
    assert_eq!(done.carried, None);
}
