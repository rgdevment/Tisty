use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

/// `floating` follows the user across time zones ("tomorrow at 10:00" stays
/// 10:00 anywhere); `has_time` keeps "due tomorrow" from meaning midnight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateSpec {
    pub at: DateTime,
    pub tz: String,
    pub floating: bool,
    pub has_time: bool,
}

impl DateSpec {
    pub fn floating(at: DateTime, tz: impl Into<String>) -> Self {
        Self {
            at,
            tz: tz.into(),
            floating: true,
            has_time: true,
        }
    }

    pub fn fixed(at: DateTime, tz: impl Into<String>) -> Self {
        Self {
            at,
            tz: tz.into(),
            floating: false,
            has_time: true,
        }
    }

    pub fn all_day(date: jiff::civil::Date, tz: impl Into<String>) -> Self {
        Self {
            at: date.to_datetime(jiff::civil::Time::midnight()),
            tz: tz.into(),
            floating: true,
            has_time: false,
        }
    }

    pub fn date(&self) -> jiff::civil::Date {
        self.at.date()
    }

    /// A floating spec resolves in `now_tz`; a fixed one keeps its own zone.
    pub fn instant(&self, now_tz: &jiff::tz::TimeZone) -> Result<jiff::Timestamp, jiff::Error> {
        let zone = if self.floating {
            now_tz.clone()
        } else {
            jiff::tz::TimeZone::get(&self.tz)?
        };
        Ok(self.at.to_zoned(zone)?.timestamp())
    }
}

#[cfg(test)]
mod tests {
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

    /// Chile moves its clock in September. A floating 10:00 must stay 10:00.
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
}
