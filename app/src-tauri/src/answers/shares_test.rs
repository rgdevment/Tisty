use super::*;

#[test]
fn a_key_in_a_document_is_named_by_its_line_and_what_it_is_called() {
    let text = "# Deploy

AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMIK7MDENGbPxRfiCYEXAMPLEKEY
";
    let found = secrets(text);
    assert!(
        found.contains(&Secret {
            line: 4,
            named: "AWS_SECRET_ACCESS_KEY".into(),
        }),
        "{found:?}"
    );
}

#[test]
fn a_private_key_is_named_by_where_it_begins() {
    let text = "Notes
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAA
-----END OPENSSH PRIVATE KEY-----
";
    assert_eq!(
        secrets(text),
        [Secret {
            line: 2,
            named: String::new(),
        }]
    );
}

#[test]
fn words_about_passwords_are_not_a_password() {
    assert!(
        secrets(
            "# Passwords

Use a password manager, and never reuse a token.
"
        )
        .is_empty()
    );
}
