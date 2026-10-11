use super::*;

fn joined(parts: &[&str]) -> String {
    parts.concat()
}

#[test]
fn a_provider_token_never_reaches_the_log() {
    let google = joined(&["ya29", ".a0AfH6SMBx7kq2Lw9ZpQ4rT8vYc1"]);
    let refresh = joined(&["1//0", "4gXyZ7kLmN2pQ5rS8tU1vW3"]);
    let bearer = joined(&[
        "eyJhbGciOiJSUzI1NiJ9",
        ".eyJzdWIiOiIxMjM0In0",
        ".c2lnbmF0dXJl",
    ]);
    for secret in [&google, &refresh, &bearer] {
        let said = masked(&format!("Authorization: Bearer {secret} was refused"));
        assert!(!said.contains(secret.as_str()), "{said}");
        assert!(said.contains("was refused"), "{said}");
    }
}

#[test]
fn a_signed_address_keeps_where_it_points_and_loses_its_signature() {
    let said =
        masked("fetch https://www.googleapis.com/drive/v3/files/abc?alt=media&sig=Zx9 failed");

    assert_eq!(
        said,
        "fetch https://www.googleapis.com/drive/v3/files/abc?… failed"
    );
}

#[test]
fn a_value_named_as_a_secret_is_hidden_whatever_it_looks_like() {
    let said = masked(r#"{"refresh_token":"short1","expires_in":3599} and client_secret=abc"#);

    assert!(!said.contains("short1"), "{said}");
    assert!(!said.contains("abc"), "{said}");
    assert!(said.contains("3599"), "{said}");
}

#[test]
fn what_a_person_needs_to_read_a_failure_stays() {
    for plain in [
        "the provider answered 429 Too Many Requests; status code: 429",
        "signing key: permission denied (os error 5)",
        r"C:\Users\someone\AppData\Local\tisty\data\store\dev_a\0001.tisty",
        "attachments/ab/foto-0123456789abcdef0123456789abcdef.png came short",
        "01J9ZK3M7Q8R2T4V6X8Z0B1C3D is not a revision the provider knows",
    ] {
        assert_eq!(masked(plain), plain);
    }
}

#[test]
fn an_opaque_token_after_bearer_is_hidden_whatever_its_shape() {
    let microsoft = joined(&["EwBwA8l6BAAU", "kj/Q2+x9", "Zp/7w=="]);
    let refresh = joined(&["M.C5", "07_BAY.0.U.-Ck", "!8x*Zp$Q2"]);
    let opaque = joined(&["a1/b2", "+c3"]);
    for secret in [&microsoft, &opaque] {
        let said = masked(&format!("Authorization: Bearer {secret} expired"));
        assert!(!said.contains(secret.as_str()), "{said}");
        assert!(said.ends_with("expired"), "{said}");
    }
    let said = masked(&format!("the provider refused {refresh} as stale"));
    assert!(!said.contains("8x*Zp"), "{said}");
    assert!(said.ends_with("as stale"), "{said}");
}
