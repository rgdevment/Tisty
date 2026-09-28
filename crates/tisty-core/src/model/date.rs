use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

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

    pub fn moved(&self, at: DateTime) -> Self {
        Self {
            at: if self.has_time {
                at
            } else {
                at.date().to_datetime(jiff::civil::Time::midnight())
            },
            ..self.clone()
        }
    }

    pub fn date(&self) -> jiff::civil::Date {
        self.at.date()
    }

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
#[path = "date_test.rs"]
mod tests;
