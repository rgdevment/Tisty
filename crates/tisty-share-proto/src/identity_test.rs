use super::*;

fn root(n: u8) -> Root {
    Root::from_bytes(&[n; 32])
}

fn share(n: u8) -> ShareKey {
    ShareKey::from_bytes(&[n; 32])
}

fn at(second: i64) -> Timestamp {
    Timestamp::from_second(second).unwrap()
}

fn signed_by(root: &Root, key: &ShareKey, said: &[u8]) -> Signed {
    let certificate = root.certify(&key.public(), at(1_000));
    key.sign(&certificate, "doc", said).unwrap()
}

#[test]
fn two_machines_of_one_person_sign_with_different_keys_and_the_same_name() {
    let person = root(1);
    let laptop = signed_by(&person, &share(2), b"hello");
    let desktop = signed_by(&person, &share(3), b"hello");

    assert_ne!(laptop.sig, desktop.sig);
    let Verdict::Holds { person: one, .. } = check(&laptop, "doc", b"hello", &[]) else {
        panic!("the laptop's signature did not hold");
    };
    let Verdict::Holds { person: two, .. } = check(&desktop, "doc", b"hello", &[]) else {
        panic!("the desktop's signature did not hold");
    };
    assert_eq!(one, two);
    assert_eq!(one.fingerprint(), person.person().fingerprint());
}

#[test]
fn a_changed_text_or_another_kind_of_thing_refuses_the_signature() {
    let signed = signed_by(&root(1), &share(2), b"hello");

    assert_eq!(check(&signed, "doc", b"hellO", &[]), Verdict::Refused);
    assert_eq!(check(&signed, "envelope", b"hello", &[]), Verdict::Refused);
}

#[test]
fn where_about_ends_and_the_text_begins_is_part_of_what_is_signed() {
    let certificate = root(1).certify(&share(2).public(), at(1_000));
    let signed = share(2).sign(&certificate, "ab", b"c").unwrap();

    assert_eq!(check(&signed, "a", b"bc", &[]), Verdict::Refused);
}

#[test]
fn a_certificate_from_another_root_does_not_answer_for_the_person() {
    let mut signed = signed_by(&root(1), &share(2), b"hello");
    signed.certificate.root = root(9).person().shown();

    assert_eq!(check(&signed, "doc", b"hello", &[]), Verdict::Refused);
}

#[test]
fn a_certificate_moved_in_time_no_longer_holds() {
    let mut certificate = root(1).certify(&share(2).public(), at(1_000));
    certificate.since = at(5);

    assert_eq!(certificate.vouches(), Err(Fault::Refused));
}

#[test]
fn a_damaged_record_is_unreadable_not_forged() {
    let mut signed = signed_by(&root(1), &share(2), b"hello");
    signed.sig = "not base64 at all!".into();
    assert_eq!(check(&signed, "doc", b"hello", &[]), Verdict::Unreadable);

    let mut signed = signed_by(&root(1), &share(2), b"hello");
    signed.certificate.share.truncate(10);
    assert_eq!(check(&signed, "doc", b"hello", &[]), Verdict::Unreadable);
}

#[test]
fn a_key_will_not_sign_under_a_certificate_for_another_key() {
    let certificate = root(1).certify(&share(2).public(), at(1_000));

    assert!(share(3).sign(&certificate, "doc", b"hello").is_none());
}

#[test]
fn a_revoked_key_still_answers_but_is_told_apart() {
    let person = root(1);
    let lost = share(2);
    let signed = signed_by(&person, &lost, b"hello");
    let revoked = person.revoke(&lost.public(), at(2_000));

    assert_eq!(
        check(&signed, "doc", b"hello", std::slice::from_ref(&revoked)),
        Verdict::Revoked {
            person: person.person(),
            at: at(2_000)
        }
    );
}

#[test]
fn revoking_one_machine_leaves_the_others_holding() {
    let person = root(1);
    let kept = signed_by(&person, &share(3), b"hello");
    let revoked = person.revoke(&share(2).public(), at(2_000));

    assert!(matches!(
        check(&kept, "doc", b"hello", &[revoked]),
        Verdict::Holds { .. }
    ));
}

#[test]
fn only_the_root_that_vouched_can_take_a_key_back() {
    let signed = signed_by(&root(1), &share(2), b"hello");
    let stranger = root(9).revoke(&share(2).public(), at(2_000));

    assert!(matches!(
        check(&signed, "doc", b"hello", &[stranger]),
        Verdict::Holds { .. }
    ));
}

#[test]
fn a_forged_revocation_takes_nothing_back() {
    let person = root(1);
    let signed = signed_by(&person, &share(2), b"hello");
    let mut forged = person.revoke(&share(2).public(), at(2_000));
    forged.at = at(3_000);

    assert!(matches!(
        check(&signed, "doc", b"hello", &[forged]),
        Verdict::Holds { .. }
    ));
}

#[test]
fn only_whole_seconds_are_kept() {
    let half_past = Timestamp::from_millisecond(1_000_500).unwrap();
    let certificate = root(1).certify(&share(2).public(), half_past);

    assert_eq!(certificate.since, at(1_000));
    assert!(certificate.vouches().is_ok());
}

#[test]
fn a_certificate_travels_as_json_and_comes_back_whole() {
    let signed = signed_by(&root(1), &share(2), b"hello");

    let back: Signed = serde_json::from_str(&serde_json::to_string(&signed).unwrap()).unwrap();

    assert_eq!(back, signed);
    assert!(matches!(
        check(&back, "doc", b"hello", &[]),
        Verdict::Holds { .. }
    ));
}

#[test]
fn the_fingerprint_is_twenty_digits_in_four_groups() {
    let fingerprint = root(1).person().fingerprint();

    let groups: Vec<&str> = fingerprint.split(' ').collect();
    assert_eq!(groups.len(), 4);
    assert!(
        groups
            .iter()
            .all(|one| one.len() == 5 && one.chars().all(|c| c.is_ascii_digit()))
    );
    assert_ne!(fingerprint, root(2).person().fingerprint());
}

#[test]
fn the_suffix_is_four_characters_that_cannot_be_misread() {
    for n in 0..=255 {
        let suffix = root(n).person().suffix();
        assert_eq!(suffix.chars().count(), 4, "{suffix}");
        assert!(
            suffix.chars().all(|c| !"ilou".contains(c)),
            "{suffix} has a letter people misread"
        );
    }
    assert_ne!(root(1).person().suffix(), root(2).person().suffix());
}

#[test]
fn a_person_reads_back_from_what_is_shown() {
    let person = root(1).person();

    assert_eq!(Person::read(&person.shown()), Some(person));
    assert_eq!(Person::read("short"), None);
}

#[test]
fn a_key_comes_back_from_its_bytes() {
    let first = root(1);
    let again = Root::from_bytes(&first.to_bytes());

    assert_eq!(first.person(), again.person());
    assert_eq!(
        share(2).public(),
        ShareKey::from_bytes(&share(2).to_bytes()).public()
    );
}

#[test]
fn minted_keys_differ() {
    assert_ne!(
        ShareKey::minted().unwrap().public(),
        ShareKey::minted().unwrap().public()
    );
}
