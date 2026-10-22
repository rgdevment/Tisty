use super::fitting;

#[test]
fn every_name_tisty_writes_up_there_fits() {
    for name in [
        "tisty.toml",
        "store/dev_46zdmrtt/2026-10/segment-000012.tisty",
        "attachments/ab12cd34ef56.pdf",
        "keys/dev_46zdmrtt.join",
        "moved.json",
    ] {
        assert!(fitting(name).is_ok(), "{name}");
    }
}

#[test]
fn a_name_some_server_would_refuse_is_refused_before_asking() {
    for name in [
        "store/a:b",
        "a*b",
        "what?",
        "pipe|d",
        "quote\"d",
        "less<more>",
        " leading",
        "trailing ",
        "dotted.",
        "",
        "a//b",
        "../up",
    ] {
        assert!(fitting(name).is_err(), "{name}");
    }
}
