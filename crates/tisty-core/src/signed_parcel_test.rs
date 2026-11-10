use tisty_share_proto::seed::Seed;

use tisty_share_proto::editing;

use super::*;

fn signer_parts() -> (ShareKey, Certificate) {
    let root = Seed::from_bytes(&[7; 32]).root();
    let key = ShareKey::from_bytes(&[3; 32]);
    let certificate = root.certify(&key.public(), "2026-11-10T12:00:00Z".parse().unwrap());
    (key, certificate)
}

fn carried() -> Prints {
    let mut prints = Prints::default();
    assert!(prints.add("docs/acta.md", digest(b"# Acta")));
    prints
}

#[test]
fn a_name_that_comes_twice_is_refused() {
    let mut prints = carried();
    assert!(!prints.add("docs/acta.md", digest(b"# Otra")));
}

#[test]
fn a_parcel_whose_names_came_twice_is_broken_however_well_it_was_signed() {
    let (key, certificate) = signer_parts();
    let signer = Signer {
        key: &key,
        certificate: &certificate,
        name: Some("ana"),
    };
    let prints = carried();
    let entry = signed(&signer, b"{}", &prints).unwrap();

    assert!(matches!(
        closed(Some(&entry), b"{}", Some(&prints), &[]),
        Closed::By { .. }
    ));
    assert_eq!(closed(Some(&entry), b"{}", None, &[]), Closed::Broken);
    assert_eq!(closed(Some(&[]), b"{}", Some(&prints), &[]), Closed::Broken);
}

#[test]
fn a_name_that_hides_a_letter_is_never_shown_even_when_signed() {
    let (key, certificate) = signer_parts();
    let prints = carried();
    let hiding = "ana\u{202E}tnemvedgr";
    let message = said(b"{}", &prints, Some(hiding));
    let signature = Signature {
        name: Some(hiding.into()),
        signed: key.sign(&certificate, ABOUT, &message).unwrap(),
    };
    let entry = serde_json::to_vec(&signature).unwrap();

    match closed(Some(&entry), b"{}", Some(&prints), &[]) {
        Closed::By { name, .. } => assert_eq!(name, None),
        other => panic!("a good signature read as {other:?}"),
    }
}

#[test]
fn a_name_that_cannot_be_shown_goes_out_unsigned_rather_than_altered() {
    let (key, certificate) = signer_parts();
    let signer = Signer {
        key: &key,
        certificate: &certificate,
        name: Some(" ana "),
    };
    let prints = carried();
    let entry = signed(&signer, b"{}", &prints).unwrap();
    let signature: Signature = serde_json::from_slice(&entry).unwrap();

    assert_eq!(signature.name, None);
}

fn vouched() -> Vouched {
    Vouched {
        root: "R".into(),
        name: Some("ana".into()),
    }
}

fn editor(folio: &str, name: &str) -> serde_json::Value {
    let (key, certificate) = signer_parts();
    let signed = editing::signed(&key, &certificate, folio, false, name).unwrap();
    serde_json::json!({
        "in": folio,
        "root": signed.certificate.root,
        "machine": signed.certificate.share,
        "since": signed.certificate.since.to_string(),
        "certificate": signed.certificate.sig,
        "name": name,
        "signed": signed.sig,
    })
}

#[test]
fn editors_with_no_folio_document_to_answer_for_are_not_kept() {
    let listed = serde_json::json!([editor("folio-1", "beto")]);

    assert_eq!(edited(Some(&vouched()), None, Some(&listed)), None);
    assert_eq!(edited(None, Some("folio-1"), Some(&listed)), None);
    assert_eq!(
        edited(Some(&vouched()), Some("folio-2"), Some(&listed)),
        None
    );
    assert!(edited(Some(&vouched()), Some("folio-1"), Some(&listed)).is_some());
}

#[test]
fn no_more_editors_are_kept_than_a_version_carries() {
    let many: Vec<serde_json::Value> = (0..100)
        .map(|n| editor("folio-1", &format!("editor{n}")))
        .collect();
    let kept = edited(
        Some(&vouched()),
        Some("folio-1"),
        Some(&serde_json::Value::from(many)),
    )
    .unwrap();

    assert_eq!(kept.editors.as_array().map(Vec::len), Some(64));
}

#[test]
fn an_entry_whose_name_was_changed_after_signing_is_left_out() {
    let mut forged = editor("folio-1", "beto");
    forged["name"] = "otro".into();
    let listed = serde_json::json!([forged, editor("folio-1", "carla")]);

    let kept = edited(Some(&vouched()), Some("folio-1"), Some(&listed)).unwrap();

    assert_eq!(
        kept.editors,
        serde_json::json!([editor("folio-1", "carla")])
    );
}

#[test]
fn only_an_entrys_own_fields_reach_the_log() {
    let mut padded = editor("folio-1", "beto");
    padded["junk"] = "x".repeat(100_000).into();
    let listed = serde_json::json!([padded]);

    let kept = edited(Some(&vouched()), Some("folio-1"), Some(&listed)).unwrap();

    assert_eq!(kept.editors, serde_json::json!([editor("folio-1", "beto")]));
}

#[test]
fn a_signed_name_that_could_not_be_shown_is_left_out() {
    let listed = serde_json::json!([editor("folio-1", "12345 67890")]);

    assert_eq!(
        edited(Some(&vouched()), Some("folio-1"), Some(&listed)),
        None
    );
}

#[test]
fn a_stopped_share_names_the_document_its_owner_shared() {
    let mut owners = editor("folio-1", "ana");
    owners["owned"] = true.into();
    let listed = serde_json::json!([editor("folio-0", "beto"), owners]);
    assert_eq!(stopped_in(&listed).as_deref(), Some("folio-1"));

    let unowned = serde_json::json!([
        editor("folio-0", "beto"),
        editor("folio-1", "carla"),
        editor("folio-1", "dora"),
    ]);
    assert_eq!(stopped_in(&unowned).as_deref(), Some("folio-1"));
}
