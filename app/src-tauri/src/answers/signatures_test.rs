use super::name_to_sign;

#[test]
fn a_name_to_sign_is_trimmed_short_enough_and_never_mostly_digits() {
    assert_eq!(name_to_sign(None).unwrap(), None);
    assert_eq!(name_to_sign(Some("   ".into())).unwrap(), None);
    assert_eq!(
        name_to_sign(Some("  Ana Pérez 2 ".into()))
            .unwrap()
            .as_deref(),
        Some("Ana Pérez 2")
    );
    for unfit in ["12345", "A1234", "ab12 3"] {
        assert_eq!(
            name_to_sign(Some(unfit.into())).unwrap_err().code,
            "nameUnfit",
            "{unfit}"
        );
    }
    let long = "a".repeat(tisty_core::event::NAME_AT_MOST + 1);
    assert_eq!(name_to_sign(Some(long)).unwrap_err().code, "nameTooLong");
}
