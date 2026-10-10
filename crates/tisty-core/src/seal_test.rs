use super::*;

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn a_seal(closed: bool) -> Seal {
    Seal {
        seg: 3,
        at: 40_960,
        tip: [7; 32],
        n: 212,
        closed,
    }
}

fn sealed(line: &str) -> Read {
    match read(line.as_bytes()) {
        Line::Seal(read) => *read,
        other => panic!("not read as a seal: {other:?}"),
    }
}

#[test]
fn a_seal_reads_back_as_written_and_answers_to_its_key() {
    for closed in [false, true] {
        let said = line(&key(1), "dev_a", &a_seal(closed));

        let read = sealed(&said);

        assert!(said.ends_with("\"}\n"));
        assert_eq!(read.seal, a_seal(closed));
        assert!(holds(&key(1).verifying_key(), "dev_a", &read));
    }
}

#[test]
fn a_seal_answers_for_one_machine_and_one_key() {
    let read = sealed(&line(&key(1), "dev_a", &a_seal(false)));

    assert!(!holds(&key(2).verifying_key(), "dev_a", &read));
    assert!(!holds(&key(1).verifying_key(), "dev_b", &read));
}

#[test]
fn a_byte_changed_in_front_of_the_signature_does_not_answer() {
    let said = line(&key(1), "dev_a", &a_seal(false));
    let changed = said.replace("\"n\":212", "\"n\":213");

    let read = sealed(&changed);

    assert_eq!(read.seal.n, 213);
    assert!(!holds(&key(1).verifying_key(), "dev_a", &read));
}

#[test]
fn a_field_a_later_build_adds_is_signed_and_read_past() {
    use ed25519_dalek::Signer;

    let signed = format!(
        "{{\"v\":{SCHEMA_VERSION},\"op\":\"seal\",\"seg\":3,\"at\":40960,\"tip\":\"{}\",\"n\":212,\"inst\":\"abc\"",
        hexed(&[7; 32])
    );
    let sig = key(1).sign(&over("dev_a", signed.as_bytes()));
    let said = format!("{signed},\"sig\":\"{}\"}}\n", hexed(&sig.to_bytes()));

    let read = sealed(&said);

    assert_eq!(read.seal, a_seal(false));
    assert!(holds(&key(1).verifying_key(), "dev_a", &read));
    let stripped = said.replace(",\"inst\":\"abc\"", "");
    assert!(
        !holds(&key(1).verifying_key(), "dev_a", &sealed(&stripped)),
        "a field taken out was not noticed"
    );
}

#[test]
fn only_a_seal_at_seventeen_or_later_is_one() {
    let said = line(&key(1), "dev_a", &a_seal(false));
    let older = said.replacen(&format!("\"v\":{SCHEMA_VERSION}"), "\"v\":16", 1);
    let event = r#"{"v":17,"ts":"2026-10-01T12:00:00Z","by":"dev_a","op":"task.add","id":"01M4H6D9PH0TPEW1N7Q8D4WAN6","d":{"title":"\"op\":\"seal\"","order":"V"}}"#;

    assert_eq!(read(older.as_bytes()), Line::Other);
    assert_eq!(read(event.as_bytes()), Line::Other);
    assert_eq!(read(b"not even json"), Line::Other);
}

#[test]
fn a_seal_that_will_not_read_whole_is_broken_and_not_another_line() {
    let said = line(&key(1), "dev_a", &a_seal(false));
    let torn = &said[..said.len() - 20];
    let doubled = said.replacen(",\"sig\":\"", ",\"sig\":\"00\",\"sig\":\"", 1);
    let bad_hex = said.replacen("\"tip\":\"07", "\"tip\":\"zz", 1);

    assert_eq!(read(torn.as_bytes()), Line::Broken);
    assert_eq!(read(doubled.as_bytes()), Line::Broken);
    assert_eq!(read(bad_hex.as_bytes()), Line::Broken);
}

#[test]
fn a_seal_written_with_windows_line_endings_still_reads() {
    let said = line(&key(1), "dev_a", &a_seal(false)).replace('\n', "\r\n");

    assert!(holds(&key(1).verifying_key(), "dev_a", &sealed(&said)));
}
