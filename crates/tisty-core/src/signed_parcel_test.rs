use tisty_share_proto::seed::Seed;

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

#[test]
fn editors_with_no_folio_document_to_answer_for_are_not_kept() {
    let listed = serde_json::json!([{ "in": "folio-1", "name": "beto" }]);

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
        .map(|n| serde_json::json!({ "in": "folio-1", "name": format!("n{n}") }))
        .collect();
    let kept = edited(
        Some(&vouched()),
        Some("folio-1"),
        Some(&serde_json::Value::from(many)),
    )
    .unwrap();
    let carried: Vec<serde_json::Value> = serde_json::from_str(&kept.editors).unwrap();

    assert_eq!(carried.len(), 64);
}
