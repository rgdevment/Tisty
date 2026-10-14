use super::{PATH_AT_MOST, fitting};

#[test]
fn every_name_tisty_writes_up_there_fits() {
    for name in [
        "tisty.toml",
        "store/dev_46zdmrtt/2026-10/segment-000012.tisty",
        "attachments/ab/ab12cd34ef56.pdf",
        "keys/dev_46zdmrtt.join",
        "moved.json",
    ] {
        assert!(fitting(name).is_ok(), "{name}");
    }
}

#[test]
fn a_name_onedrive_refuses_is_refused_before_asking() {
    for name in [
        "store/a:b",
        "a*b",
        "what?",
        "pipe|d",
        "quote\"d",
        " leading",
        "trailing ",
        "dotted.",
        "CON",
        "folder/nul",
        "com1",
        "LPT9",
        ".lock",
        "Desktop.ini",
        "a/_vti_cnf/b",
        "~$draft.docx",
        "a//b",
        "../escape",
        "back\\slash",
    ] {
        assert!(fitting(name).is_err(), "{name}");
    }
}

#[test]
fn a_device_name_only_counts_with_one_digit() {
    assert!(fitting("com10").is_ok());
    assert!(fitting("compose").is_ok());
    assert!(fitting("lpt").is_ok());
}

#[test]
fn a_path_past_what_onedrive_takes_is_refused() {
    let long = "a/".repeat(PATH_AT_MOST / 2) + "b";
    assert!(fitting(&long).is_err());
    assert!(fitting(&"a".repeat(PATH_AT_MOST)).is_ok());
}
