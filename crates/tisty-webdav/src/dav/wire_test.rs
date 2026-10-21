use tisty_carrier::Hitch;

use super::{given, refusal, strong};

#[test]
fn each_refusal_reads_as_what_the_round_does_with_it() {
    assert!(matches!(refusal(401, None), Hitch::Lost));
    assert!(matches!(refusal(404, None), Hitch::Missing(_)));
    assert!(matches!(refusal(409, None), Hitch::Missing(_)));
    assert!(matches!(refusal(412, None), Hitch::Changed(_)));
    assert!(matches!(refusal(507, None), Hitch::Full));
    assert!(matches!(refusal(423, None), Hitch::Limited { .. }));
    assert!(matches!(refusal(429, None), Hitch::Limited { .. }));
    assert!(matches!(refusal(503, None), Hitch::Troubled(_)));
    assert!(matches!(
        refusal(503, Some(std::time::Duration::from_secs(5))),
        Hitch::Limited { .. }
    ));
    assert!(matches!(refusal(403, None), Hitch::Broke(_)));
}

#[test]
fn only_a_strong_etag_can_guard_a_write() {
    assert!(strong("\"abc\""));
    assert!(!strong("W/\"abc\""));
    assert!(!strong("modified:Tue:3"));
}

#[test]
fn a_bigger_transfer_is_given_longer() {
    assert!(given(1 << 30) > given(1 << 10));
}
