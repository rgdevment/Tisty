//! Natural language capture. Deterministic and local: nothing is sent to a
//! model, so the result is reproducible and testable.

use jiff::Zoned;
use serde::{Deserialize, Serialize};
use tisty_core::{DateSpec, Priority, Tag};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Parsed {
    pub title: String,
    pub date: Option<DateSpec>,
    pub deadline: Option<DateSpec>,
    pub priority: Option<Priority>,
    pub tags: Vec<Tag>,
}

pub trait Parser {
    fn locales(&self) -> &[&str];
    fn parse(&self, input: &str, now: &Zoned) -> Parsed;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Es,
    En,
}
