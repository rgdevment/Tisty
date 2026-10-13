use super::ended;
use crate::i18n::Lang;

fn round(code: &str, left: usize) -> tisty_carrier::LastRound {
    tisty_carrier::LastRound {
        at: 1_791_892_800,
        code: code.into(),
        name: None,
        left,
        names: Vec::new(),
    }
}

#[test]
fn the_last_round_says_when_and_how_it_ended() {
    let en = Lang::from_code("en");

    assert_eq!(
        ended(&round("ok", 0), en),
        "2026-10-13 12:00 UTC, ended well"
    );
    assert!(
        ended(&round("deferred-budget", 3), en)
            .contains("put off: your cloud reached its limit for today")
    );
    assert!(ended(&round("deferred-budget", 3), en).ends_with("(3 left for later)"));
    assert!(ended(&round("deferred-someday", 0), en).contains("stopped (deferred-someday)"));
    assert!(ended(&round("cloudLost", 0), en).contains("stopped (cloudLost)"));
}

#[test]
fn the_last_round_is_told_in_spanish_too() {
    let es = Lang::from_code("es");

    assert_eq!(
        ended(&round("ok", 0), es),
        "2026-10-13 12:00 UTC, terminó bien"
    );
    assert!(
        ended(&round("deferred-offline", 0), es)
            .contains("quedó para después: no se pudo llegar a tu nube")
    );
}
