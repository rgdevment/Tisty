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
            under: Some("Deploy".into()),
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
            under: None,
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

#[test]
fn a_key_is_placed_under_its_heading_past_the_properties_and_code_comments() {
    let text = "---
title: x
# not a heading
---
# Servers

## Staging

```sh
# a comment
```
ghp_16C7e42F292c6912E7710c838347Ae178B4a
";
    assert_eq!(heading_over(text, 12).as_deref(), Some("Staging"));
    assert_eq!(heading_over(text, 3), None);
    assert_eq!(
        heading_over(
            "#hashtag
token",
            2
        ),
        None
    );
}
