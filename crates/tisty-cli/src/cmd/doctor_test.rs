use super::{ended, lasting};
use crate::i18n::Lang;

fn round(code: &str, left: usize) -> tisty_carrier::LastRound {
    tisty_carrier::LastRound {
        at: 1_791_892_800,
        code: code.into(),
        name: None,
        left,
        names: Vec::new(),
        ..Default::default()
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
fn a_round_put_off_by_the_provider_or_by_attachments_waits_rather_than_stops() {
    let en = Lang::from_code("en");

    let troubled = ended(&round("deferred-troubled", 0), en);
    assert!(troubled.contains("put off: your cloud is having trouble on its side"));
    assert!(!troubled.contains("stopped"));
    let unsent = ended(&round("deferred-unsent", 2), en);
    assert!(unsent.contains("put off: some attachments could not travel"));
    assert!(unsent.ends_with("(2 left for later)"));
}

#[test]
fn a_round_that_keeps_ending_the_same_way_says_why_and_since_when() {
    let en = Lang::from_code("en");
    let blocked = tisty_carrier::LastRound {
        cause: Some("blocked".into()),
        rounds: 4,
        since: 1_791_878_400,
        ..round("deferred-offline", 0)
    };

    let lasted = lasting(&blocked, en).unwrap();

    assert!(lasted.starts_with("something on this network blocks the way"));
    assert!(lasted.ends_with("4 rounds in a row like this since 2026-10-13 08:00 UTC"));
}

#[test]
fn a_round_that_ended_well_or_once_has_nothing_more_to_say() {
    let en = Lang::from_code("en");
    let once = round("deferred-offline", 0);
    let well = tisty_carrier::LastRound {
        rounds: 5,
        since: 1_791_878_400,
        ..round("ok", 0)
    };
    let unknown = tisty_carrier::LastRound {
        cause: Some("someday".into()),
        ..round("cloudLost", 0)
    };

    assert_eq!(lasting(&once, en), None);
    assert_eq!(lasting(&well, en), None);
    assert_eq!(lasting(&unknown, en), None);
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
    let slow = tisty_carrier::LastRound {
        cause: Some("slow".into()),
        rounds: 2,
        since: 1_791_878_400,
        ..round("deferred-offline", 0)
    };
    assert_eq!(
        lasting(&slow, es).unwrap(),
        "tu nube tarda en responder desde esta red; 2 rondas seguidas así desde 2026-10-13 08:00 UTC"
    );
}
