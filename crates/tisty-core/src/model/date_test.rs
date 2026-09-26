use super::*;

fn tz(name: &str) -> jiff::tz::TimeZone {
    jiff::tz::TimeZone::get(name).unwrap()
}

#[test]
fn floating_time_follows_the_traveller() {
    let spec = DateSpec::floating("2026-08-05T10:00:00".parse().unwrap(), "America/Santiago");

    let at_home = spec.instant(&tz("America/Santiago")).unwrap();
    let abroad = spec.instant(&tz("Europe/Berlin")).unwrap();

    assert_ne!(at_home, abroad, "10:00 local is a different instant abroad");
}

#[test]
fn fixed_time_keeps_its_zone() {
    let spec = DateSpec::fixed("2026-08-05T15:00:00".parse().unwrap(), "Europe/Berlin");

    assert_eq!(
        spec.instant(&tz("America/Santiago")).unwrap(),
        spec.instant(&tz("Europe/Berlin")).unwrap(),
        "a fixed instant is the same no matter where it is read"
    );
}

#[test]
fn all_day_carries_no_time() {
    let spec = DateSpec::all_day("2026-08-05".parse().unwrap(), "America/Santiago");
    assert!(!spec.has_time);
}

#[test]
fn floating_time_survives_a_dst_change() {
    let zone = tz("America/Santiago");
    let before = DateSpec::floating("2026-09-05T10:00:00".parse().unwrap(), "America/Santiago");
    let after = DateSpec::floating("2026-09-12T10:00:00".parse().unwrap(), "America/Santiago");

    let a = before.instant(&zone).unwrap().to_zoned(zone.clone());
    let b = after.instant(&zone).unwrap().to_zoned(zone);

    assert_eq!(a.hour(), 10);
    assert_eq!(b.hour(), 10);
}

#[test]
fn round_trips() {
    let spec = DateSpec::floating("2026-08-05T10:00:00".parse().unwrap(), "America/Santiago");
    let json = serde_json::to_string(&spec).unwrap();
    assert_eq!(spec, serde_json::from_str::<DateSpec>(&json).unwrap());
}
