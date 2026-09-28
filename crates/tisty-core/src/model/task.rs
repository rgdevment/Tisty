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
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reading {
    Story,
    Routine,
    Trace,
}

/// Why a task is not for erasing: only a closed trace goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stays {
    Open,
    Story,
    Routine,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("priority must be do, decide, delegate or minor")]
pub struct InvalidPriority;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "Wire", into = "Wire")]
pub enum Priority {
    Do,
    Decide,
    Delegate,
    #[default]
    Unset,
    Minor,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Wire {
    Named(String),
    Level(u8),
}

impl Priority {
    pub fn name(self) -> &'static str {
        match self {
            Self::Do => "do",
            Self::Decide => "decide",
            Self::Delegate => "delegate",
            Self::Minor => "minor",
            Self::Unset => "unset",
        }
    }

    pub fn set(self) -> bool {
        self != Self::Unset
    }
}

impl std::str::FromStr for Priority {
    type Err = InvalidPriority;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "do" => Ok(Self::Do),
            "decide" => Ok(Self::Decide),
            "delegate" => Ok(Self::Delegate),
            "minor" | "wont" => Ok(Self::Minor),
            "unset" => Ok(Self::Unset),
            _ => Err(InvalidPriority),
        }
    }
}

impl TryFrom<Wire> for Priority {
    type Error = InvalidPriority;

    fn try_from(wire: Wire) -> Result<Self, Self::Error> {
        match wire {
            Wire::Named(name) => name.parse(),
            Wire::Level(1..=4) => Ok(Self::Unset),
            Wire::Level(_) => Err(InvalidPriority),
        }
    }
}

impl From<Priority> for Wire {
    fn from(p: Priority) -> Self {
        Wire::Named(p.name().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub text: String,
    pub done: bool,
    pub order: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: LogId,
    pub at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    pub body: String,
    /// Projected from the event's `by`, never written to the log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<crate::event::DeviceId>,
    /// The client the assistant spoke through, from the event's `via`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

impl LogEntry {
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
pub struct Resolved {
    pub at: Timestamp,
    pub by: crate::event::DeviceId,
    pub entry: LogId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
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
    pub repeat: Option<crate::model::Repeat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<TaskId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
    /// Projected from the event's `by`, never written to the log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<crate::event::DeviceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<Resolved>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The zone the closing event was written in, so an hour reads back where it happened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_in: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub filled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,
    /// The layer the person chose, story or trace; absent, the weight decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_as: Option<Reading>,
    /// The person let an assistant fill this one in: say it is done, describe it, plan and
    /// tick its steps — what an assistant may do on a task it filed itself.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub open_to_agents: bool,
    /// The client the assistant that filed this task spoke through, from the event's `via`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_via: Option<String>,

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
    #[serde(default, skip_serializing_if = "is_zero")]
    pub prose: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub refs: usize,
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
    /// The date a closure is attributed to. A backfilled turn was stamped the day it was marked,
    /// and belongs to the day it covered instead.
    pub fn counted_on(&self, zone: &jiff::tz::TimeZone) -> Option<jiff::civil::Date> {
        match (self.filled, self.date.as_ref()) {
            (true, Some(due)) => Some(due.at.date()),
            _ => Some(self.completed_at?.to_zoned(zone.clone()).date()),
        }
    }

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
            repeat: None,
            after: None,
            hidden: false,
            created_by: None,
            resolved: None,
            source: None,
            closed_in: None,
            filled: false,
            completed_at: None,
            read_as: None,
            open_to_agents: false,
            created_via: None,
            volume: Volume::default(),
        }
    }

    pub fn created_at(&self) -> Timestamp {
        Timestamp::from_millisecond(self.id.timestamp_ms() as i64).unwrap_or(Timestamp::UNIX_EPOCH)
    }

    pub fn is_open(&self) -> bool {
        self.status == Status::Open
    }

    pub fn is_archived(&self) -> bool {
        !self.is_open()
    }

    pub fn is_dropped(&self) -> bool {
        self.status == Status::Dropped
    }

    pub fn folded(&self) -> bool {
        self.hidden || self.is_dropped()
    }

    pub fn steps_done(&self) -> (usize, usize) {
        (self.volume.steps_done, self.volume.steps)
    }

    pub fn journal_count(&self) -> usize {
        self.volume.journal
    }

    pub fn references(&self) -> Vec<crate::refs::Ref> {
        let bodies = self
            .description
            .as_deref()
            .into_iter()
            .chain(self.log.iter().map(|entry| entry.body.as_str()));

        let mut all: Vec<crate::refs::Ref> = Vec::new();
        for one in bodies.flat_map(crate::refs::extract) {
            // A step anchor points inside this very task, so it is not something the task left.
            if one.target.starts_with('#') {
                continue;
            }
            // Two labels for one target are one trace: the label is how it was written, not what it is.
            if all
                .iter()
                .any(|held| held.kind == one.kind && held.target == one.target)
            {
                continue;
            }
            all.push(one);
        }
        all
    }

    pub fn retally(&mut self) {
        let written: usize = self
            .description
            .iter()
            .map(|body| substance(body))
            .chain(self.log.iter().map(|entry| substance(&entry.body)))
            .sum();

        self.volume = Volume {
            steps: self.steps.len(),
            steps_done: self.steps.iter().filter(|s| s.done).count(),
            journal: self
                .log
                .iter()
                .filter(|e| !e.body.trim().is_empty())
                .count(),
            described: self.description.is_some(),
            prose: written.min(PROSE_CAP),
            refs: self.references().len(),
        };
    }

    pub fn step(&self, id: StepId) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }

    pub fn entry(&self, id: LogId) -> Option<&LogEntry> {
        self.log.iter().find(|e| e.id == id)
    }

    pub fn journal(&self) -> impl Iterator<Item = &LogEntry> {
        self.log.iter().filter(|e| !e.body.trim().is_empty())
    }

    pub fn weight(&self) -> usize {
        self.volume.weight()
    }

    pub fn reading(&self) -> Reading {
        if self.repeat.is_some() || self.after.is_some() {
            return Reading::Routine;
        }
        match self.read_as {
            Some(Reading::Story) => Reading::Story,
            Some(Reading::Trace) => Reading::Trace,
            _ if self.weight() >= STORY_AT => Reading::Story,
            _ => Reading::Trace,
        }
    }

    /// The weight as the person reads it: a conversion lands on its layer's side of the
    /// threshold, and everything else weighs what it wrote. For ordering, never for counting.
    pub fn heft(&self) -> usize {
        match self.reading() {
            Reading::Story => self.weight().max(STORY_AT),
            Reading::Trace => self.weight().min(STORY_AT - 1),
            Reading::Routine => self.weight(),
        }
    }

    /// Only a closed trace is erased; a story is hidden, and converting it is the person's
    /// deliberate step. Whether it is folded away plays no part.
    pub fn erasable(&self) -> Result<(), Stays> {
        if self.is_open() {
            return Err(Stays::Open);
        }
        match self.reading() {
            Reading::Trace => Ok(()),
            Reading::Story => Err(Stays::Story),
            Reading::Routine => Err(Stays::Routine),
        }
    }
}

const PROSE_CAP: usize = 8;

pub const STORY_AT: usize = 3;

fn substance(body: &str) -> usize {
    match body.split_whitespace().count() {
        0..=7 => 0,
        8..=29 => 1,
        30..=99 => 2,
        _ => 3,
    }
}

impl Volume {
    pub fn weight(&self) -> usize {
        let plan = match self.steps {
            0..=2 => 0,
            3..=7 => 1,
            _ => 2,
        };
        let refs = match self.refs {
            0 => 0,
            1..=2 => 1,
            _ => 2,
        };
        self.prose + plan + refs
    }
}

#[cfg(test)]
#[path = "task_test.rs"]
mod tests;

#[cfg(test)]
#[path = "task_trace_tests.rs"]
mod trace_tests;
