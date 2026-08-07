use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{DateSpec, ListId, Tag};

pub type TaskId = Ulid;
pub type StepId = Ulid;
pub type LogId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Done,
    /// Kept apart from `Done`: costs nothing now, unreconstructable later.
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("priority must be between 1 and 4")]
pub struct InvalidPriority;

/// Ordered so that `P1 < P4`: the most urgent sorts first.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum Priority {
    P1,
    P2,
    P3,
    #[default]
    P4,
}

impl TryFrom<u8> for Priority {
    type Error = InvalidPriority;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::P1),
            2 => Ok(Self::P2),
            3 => Ok(Self::P3),
            4 => Ok(Self::P4),
            _ => Err(InvalidPriority),
        }
    }
}

impl From<Priority> for u8 {
    fn from(p: Priority) -> Self {
        match p {
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
            Priority::P4 => 4,
        }
    }
}

/// No notes of its own: a failed step *happened*, so it belongs in the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub text: String,
    pub done: bool,
    pub order: String,
}

/// Entries accumulate; they are never overwritten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: LogId,
    pub at: Timestamp,
    /// The author's zone, or the entry renders on the reader's day instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    pub body: String,
}

impl LogEntry {
    /// Falls back to the reader's zone so older entries still render.
    pub fn zoned(&self) -> jiff::Zoned {
        let zone = self
            .tz
            .as_deref()
            .and_then(|name| jiff::tz::TimeZone::get(name).ok())
            .unwrap_or_else(jiff::tz::TimeZone::system);
        self.at.to_zoned(zone)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub order: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log: Vec<LogEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<ListId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reminders: Vec<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,

    /// Held apart from the vectors, not derived from them: a summary loaded
    /// without its body still has to know how much body there is.
    #[serde(default, skip_serializing_if = "Volume::is_empty")]
    pub volume: Volume,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub steps: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub steps_done: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub journal: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub described: bool,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

impl Volume {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl Task {
    pub fn new(id: TaskId, title: impl Into<String>, order: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            status: Status::Open,
            priority: Priority::default(),
            order: order.into(),
            description: None,
            log: Vec::new(),
            steps: Vec::new(),
            date: None,
            deadline: None,
            list: None,
            tags: Vec::new(),
            reminders: Vec::new(),
            completed_at: None,
            volume: Volume::default(),
        }
    }

    /// Derived from the ULID rather than stored: the id already carries it.
    pub fn created_at(&self) -> Timestamp {
        Timestamp::from_millisecond(self.id.timestamp_ms() as i64).unwrap_or(Timestamp::UNIX_EPOCH)
    }

    pub fn is_open(&self) -> bool {
        self.status == Status::Open
    }

    /// Archived rather than finished: it stays searchable forever.
    pub fn is_archived(&self) -> bool {
        !self.is_open()
    }

    pub fn steps_done(&self) -> (usize, usize) {
        (self.volume.steps_done, self.volume.steps)
    }

    pub fn journal_count(&self) -> usize {
        self.volume.journal
    }

    /// Recomputed from the vectors after any change to them.
    pub fn retally(&mut self) {
        self.volume = Volume {
            steps: self.steps.len(),
            steps_done: self.steps.iter().filter(|s| s.done).count(),
            journal: self
                .log
                .iter()
                .filter(|e| !e.body.trim().is_empty())
                .count(),
            described: self.description.is_some(),
        };
    }

    pub fn step(&self, id: StepId) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }

    pub fn entry(&self, id: LogId) -> Option<&LogEntry> {
        self.log.iter().find(|e| e.id == id)
    }

    /// Skips entries emptied by an undo, which are kept so a later edit finds them.
    pub fn journal(&self) -> impl Iterator<Item = &LogEntry> {
        self.log.iter().filter(|e| !e.body.trim().is_empty())
    }

    /// Always recomputed: drives empty-section skipping and search ranking.
    pub fn weight(&self) -> usize {
        usize::from(self.volume.described)
            + self.volume.journal
            + self.volume.steps
            + self.tags.len()
            + usize::from(self.date.is_some())
            + usize::from(self.deadline.is_some())
            + usize::from(self.list.is_some())
            + self.reminders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> Task {
        Task::new(Ulid::generate(), "ship it", "a0")
    }

    #[test]
    fn priority_sorts_most_urgent_first() {
        let mut all = vec![Priority::P3, Priority::P1, Priority::P4, Priority::P2];
        all.sort();
        assert_eq!(
            all,
            [Priority::P1, Priority::P2, Priority::P3, Priority::P4]
        );
    }

    #[test]
    fn priority_serialises_as_its_number() {
        assert_eq!(serde_json::to_string(&Priority::P1).unwrap(), "1");
        assert_eq!(serde_json::from_str::<Priority>("4").unwrap(), Priority::P4);
    }

    #[test]
    fn priority_outside_the_range_is_rejected() {
        assert!(serde_json::from_str::<Priority>("0").is_err());
        assert!(serde_json::from_str::<Priority>("5").is_err());
    }

    #[test]
    fn a_bare_task_serialises_to_the_minimum() {
        let json = serde_json::to_string(&task()).unwrap();
        for absent in [
            "description",
            "log",
            "steps",
            "date",
            "deadline",
            "list",
            "tags",
            "reminders",
            "completed_at",
        ] {
            assert!(
                !json.contains(absent),
                "'{absent}' should not appear in {json}"
            );
        }
    }

    #[test]
    fn round_trips() {
        let mut t = task();
        t.tags = vec![Tag::new("work").unwrap()];
        t.description = Some("check the gateway".into());
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(t, serde_json::from_str::<Task>(&json).unwrap());
    }

    #[test]
    fn created_at_comes_from_the_id() {
        let t = task();
        assert_eq!(t.created_at().as_millisecond(), t.id.timestamp_ms() as i64);
    }

    #[test]
    fn weight_separates_the_trivial_from_the_documented() {
        let trivial = task();

        let mut rich = task();
        rich.description = Some("…".into());
        rich.date = Some(DateSpec::all_day("2026-08-05".parse().unwrap(), "UTC"));
        rich.tags = vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()];
        rich.log.push(LogEntry {
            id: Ulid::generate(),
            at: Timestamp::UNIX_EPOCH,
            tz: None,
            body: "first attempt failed".into(),
        });

        assert_eq!(trivial.weight(), 0);
        assert!(rich.weight() > trivial.weight());
    }

    #[test]
    fn a_completed_task_is_archived_not_gone() {
        let mut t = task();
        t.status = Status::Done;
        assert!(t.is_archived());
        assert!(!t.is_open());
    }
}
