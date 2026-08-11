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
    /// Kept apart from `Done` — unreconstructable once merged.
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
    pub repeat: Option<crate::model::Repeat>,
    /// Noise the user put away by hand. Never removed, only folded.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,

    /// Stored, not derived — a summary loaded without its body must still know how much body there is.
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
    /// Substance of what was written, capped. A summary is loaded without its
    /// bodies, so the weight has to be stored rather than recomputed from them.
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
            hidden: false,
            completed_at: None,
            volume: Volume::default(),
        }
    }

    /// Derived from the ULID's embedded timestamp; not stored separately.
    pub fn created_at(&self) -> Timestamp {
        Timestamp::from_millisecond(self.id.timestamp_ms() as i64).unwrap_or(Timestamp::UNIX_EPOCH)
    }

    pub fn is_open(&self) -> bool {
        self.status == Status::Open
    }

    /// True for both `Done` and `Dropped` — archived tasks stay searchable.
    pub fn is_archived(&self) -> bool {
        !self.is_open()
    }

    pub fn is_dropped(&self) -> bool {
        self.status == Status::Dropped
    }

    /// Put away by hand, or decided against: the archive is what you did.
    pub fn folded(&self) -> bool {
        self.hidden || self.is_dropped()
    }

    pub fn steps_done(&self) -> (usize, usize) {
        (self.volume.steps_done, self.volume.steps)
    }

    pub fn journal_count(&self) -> usize {
        self.volume.journal
    }

    /// Pulled from the prose of the description and the journal, in that order;
    /// a step is one line and holds no references of its own.
    pub fn references(&self) -> Vec<crate::refs::Ref> {
        let bodies = self
            .description
            .as_deref()
            .into_iter()
            .chain(self.log.iter().map(|entry| entry.body.as_str()));

        let mut all: Vec<crate::refs::Ref> = Vec::new();
        for one in bodies.flat_map(crate::refs::extract) {
            if !all.contains(&one) {
                all.push(one);
            }
        }
        all
    }

    /// Recomputed from the vectors after any change to them.
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

    /// Skips entries emptied by an undo, which are kept so a later edit finds them.
    pub fn journal(&self) -> impl Iterator<Item = &LogEntry> {
        self.log.iter().filter(|e| !e.body.trim().is_empty())
    }

    pub fn weight(&self) -> usize {
        self.volume.weight()
    }
}

const PROSE_CAP: usize = 8;

/// Words, not entries: twelve «ok» say nothing and two paragraphs say a lot.
/// Blind to scripts that do not space their words, which only ever costs a
/// place in the ranking and never touches stored data.
fn substance(body: &str) -> usize {
    match body.split_whitespace().count() {
        0..=2 => 0,
        3..=29 => 1,
        _ => 2,
    }
}

impl Volume {
    /// What a task is worth being found by, months later. Date, deadline, list,
    /// tags, priority and reminders say WHEN and WHERE, and say nothing at all
    /// by then, so none of them count.
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
mod tests {
    use super::*;

    fn task() -> Task {
        Task::new(Ulid::generate(), "ship it", "a0")
    }

    fn entry(body: &str) -> LogEntry {
        LogEntry {
            id: Ulid::generate(),
            at: Timestamp::UNIX_EPOCH,
            tz: None,
            body: body.into(),
        }
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
        let mut trivial = task();
        trivial.retally();

        let mut rich = task();
        rich.description = Some("el redirect de registration se cae en Brasil".into());
        rich.log
            .push(entry("el proxy lateral no arrancaba con la config nueva"));
        rich.retally();

        assert_eq!(trivial.weight(), 0);
        assert!(rich.weight() > trivial.weight());
    }

    /// The old formula counted date, list and tags, so «reunión con Pepe mañana»
    /// outranked a task with three journal entries — backwards.
    #[test]
    fn the_agenda_never_outweighs_the_history() {
        let mut agenda = task();
        agenda.date = Some(DateSpec::all_day("2026-08-05".parse().unwrap(), "UTC"));
        agenda.deadline = Some(DateSpec::all_day("2026-08-09".parse().unwrap(), "UTC"));
        agenda.tags = vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()];
        agenda.list = Some(Ulid::generate());
        agenda.reminders = vec![DateSpec::all_day("2026-08-04".parse().unwrap(), "UTC")];
        agenda.retally();

        let mut history = task();
        history
            .log
            .push(entry("el gateway no propagaba la cabecera"));
        history.retally();

        assert_eq!(agenda.weight(), 0);
        assert!(history.weight() > agenda.weight());
    }

    /// Twelve «ok» are not documentation; two paragraphs are.
    #[test]
    fn weight_counts_substance_and_not_entries() {
        let mut noisy = task();
        for _ in 0..12 {
            noisy.log.push(entry("ok"));
        }
        noisy.retally();

        let mut written = task();
        written.log.push(entry(
            "el proxy lateral no arrancaba con la configuración nueva",
        ));
        written.retally();

        assert_eq!(noisy.weight(), 0);
        assert!(written.weight() > noisy.weight());
    }

    #[test]
    fn weight_has_a_ceiling_so_a_disaster_cannot_top_everything() {
        let mut endless = task();
        for _ in 0..40 {
            endless
                .log
                .push(entry("volvió a fallar el despliegue del proxy lateral"));
        }
        endless.retally();
        assert_eq!(endless.weight(), 8);
    }

    #[test]
    fn references_are_gathered_across_the_description_and_the_journal() {
        let mut t = task();
        t.description = Some("sale del ticket [[CUSLEG-3465]]".into());
        t.log.push(entry("el MR está en https://gl.example/mr/7"));
        t.log.push(entry("sigue siendo [[CUSLEG-3465]]"));
        t.retally();

        assert_eq!(
            t.references()
                .into_iter()
                .map(|one| one.target)
                .collect::<Vec<_>>(),
            ["CUSLEG-3465", "https://gl.example/mr/7"],
            "the same target twice is one reference, and the description comes first"
        );
        assert_eq!(t.volume.refs, 2);
    }

    /// A task that points somewhere carries more than one that points nowhere.
    #[test]
    fn a_reference_adds_to_the_weight() {
        let mut bare = task();
        bare.description = Some("mirar el despliegue del proxy lateral".into());
        bare.retally();

        let mut pointed = task();
        pointed.description = Some("mirar el despliegue del proxy lateral [[CUSLEG-3465]]".into());
        pointed.retally();

        assert!(pointed.weight() > bare.weight());
    }

    #[test]
    fn a_reference_that_leaves_the_prose_stops_counting() {
        let mut t = task();
        t.description = Some("sale de [[CUSLEG-3465]]".into());
        t.retally();
        assert_eq!(t.volume.refs, 1);

        t.description = Some("sale de otra parte".into());
        t.retally();
        assert_eq!(
            t.volume.refs, 0,
            "the index outlived the prose it came from"
        );
    }

    #[test]
    fn a_completed_task_is_archived_not_gone() {
        let mut t = task();
        t.status = Status::Done;
        assert!(t.is_archived());
        assert!(!t.is_open());
    }
}
