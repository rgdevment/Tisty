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
    // Put together here, so a secret scanner never takes the made-up key for a real one.
    let text = [
        "Notes",
        "-----BEGIN OPENSSH PRIVATE KEY-----",
        "not a key",
        "",
    ]
    .join("\n");
    assert_eq!(
        secrets(&text),
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
    assert_eq!(headings_over(text)[11].as_deref(), Some("Staging"));
    assert_eq!(headings_over(text)[2], None);
    assert_eq!(
        headings_over(
            "#hashtag
token"
        )[1],
        None
    );
}

#[test]
fn a_rule_at_the_top_is_no_properties_and_hides_no_heading() {
    let text = "---\n# Servers\n\nwhere they live\n---\nghp_16C7e42F292c6912E7710c838347Ae178B4a\n";
    assert_eq!(headings_over(text)[5].as_deref(), Some("Servers"));
    assert_eq!(
        properties_end("---\ntitle: x\ntags: [a]\n---\n# Body\n"),
        Some(4)
    );
    assert_eq!(properties_end("---\n---\n"), None);
    assert_eq!(properties_end("---\ntitle: x\n"), None);
    assert_eq!(properties_end("---\n# Servers\n---\n"), None);
}

#[test]
fn a_share_signs_with_the_first_name_fit_to_sign() {
    let signature = |name: Option<&str>, alias: Option<&str>| tisty_core::event::Signature {
        name: name.map(str::to_owned),
        alias: alias.map(str::to_owned),
        ..Default::default()
    };
    assert_eq!(
        signing_name(&signature(Some("Ana"), Some("ana"))).as_deref(),
        Some("Ana")
    );
    assert_eq!(
        signing_name(&signature(Some("12345"), Some("ana"))).as_deref(),
        Some("ana")
    );
    assert_eq!(signing_name(&signature(None, Some("1234"))), None);
}
