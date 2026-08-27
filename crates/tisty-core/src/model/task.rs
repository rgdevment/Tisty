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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,

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
            completed_at: None,
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
            Reading::Routine
        } else if self.weight() > 0 {
            Reading::Story
        } else {
            Reading::Trace
        }
    }
}

const PROSE_CAP: usize = 8;

fn substance(body: &str) -> usize {
    match body.split_whitespace().count() {
        0..=2 => 0,
        3..=29 => 1,
        _ => 2,
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
    fn what_will_not_happen_sorts_last_of_all() {
        let mut all = vec![
            Priority::Minor,
            Priority::Unset,
            Priority::Do,
            Priority::Delegate,
            Priority::Decide,
        ];
        all.sort();
        assert_eq!(
            all,
            [
                Priority::Do,
                Priority::Decide,
                Priority::Delegate,
                Priority::Unset,
                Priority::Minor
            ]
        );
    }

    #[test]
    fn priority_serialises_as_the_word_it_is() {
        assert_eq!(serde_json::to_string(&Priority::Do).unwrap(), "\"do\"");
        assert_eq!(
            serde_json::from_str::<Priority>("\"delegate\"").unwrap(),
            Priority::Delegate
        );
    }

    #[test]
    fn the_old_levels_come_back_unclassified() {
        for level in ["1", "2", "3", "4"] {
            assert_eq!(
                serde_json::from_str::<Priority>(level).unwrap(),
                Priority::Unset
            );
        }
    }

    #[test]
    fn what_the_first_matrix_wrote_is_still_read() {
        assert_eq!(
            serde_json::from_str::<Priority>("\"wont\"").unwrap(),
            Priority::Minor
        );
        assert_eq!(
            serde_json::to_string(&Priority::Minor).unwrap(),
            "\"minor\""
        );
    }

    #[test]
    fn a_priority_nobody_defined_is_rejected() {
        assert!(serde_json::from_str::<Priority>("0").is_err());
        assert!(serde_json::from_str::<Priority>("5").is_err());
        assert!(serde_json::from_str::<Priority>("\"urgent\"").is_err());
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

    fn daily() -> crate::model::Repeat {
        crate::model::Repeat::due(crate::model::Cadence {
            every: 1,
            unit: crate::model::Unit::Day,
        })
    }

    #[test]
    fn a_repeating_task_is_a_routine_even_when_it_carries_a_journal() {
        let mut chore = task();
        chore.repeat = Some(daily());
        chore.log.push(entry("the pharmacy was shut so it waited"));
        chore.retally();

        assert!(chore.weight() > 0, "the note is real substance");
        assert_eq!(
            chore.reading(),
            Reading::Routine,
            "the series outranks any single turn"
        );
    }

    #[test]
    fn a_turn_is_a_routine_through_its_chain_alone() {
        let mut turn = task();
        turn.after = Some(Ulid::generate());
        turn.retally();

        assert_eq!(turn.reading(), Reading::Routine);
    }

    #[test]
    fn a_task_with_nothing_written_is_a_trace() {
        let mut errand = task();
        errand.retally();

        assert_eq!(errand.reading(), Reading::Trace);
    }

    #[test]
    fn the_agenda_alone_never_lifts_a_trace() {
        let mut errand = task();
        errand.date = Some(DateSpec::all_day("2026-08-05".parse().unwrap(), "UTC"));
        errand.deadline = Some(DateSpec::all_day("2026-08-09".parse().unwrap(), "UTC"));
        errand.tags = vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()];
        errand.list = Some(Ulid::generate());
        errand.reminders = vec![DateSpec::all_day("2026-08-04".parse().unwrap(), "UTC")];
        errand.retally();

        assert_eq!(
            errand.reading(),
            Reading::Trace,
            "dates and labels are not something learnt"
        );
    }

    #[test]
    fn writing_a_note_lifts_a_trace_into_a_story() {
        let mut one = task();
        one.retally();
        assert_eq!(one.reading(), Reading::Trace);

        one.log
            .push(entry("the courier leaves the parcel with the neighbour"));
        one.retally();

        assert_eq!(
            one.reading(),
            Reading::Story,
            "the layer is read from what is there, never stored"
        );
    }
}
#[cfg(test)]
mod trace_tests {
    use super::*;

    fn told(body: &str, log: &[&str]) -> Vec<crate::refs::Ref> {
        let mut task = Task::new(ulid::Ulid::generate(), "x", "a0");
        task.description = Some(body.to_string());
        task.log = log
            .iter()
            .map(|one| LogEntry {
                id: ulid::Ulid::generate(),
                at: Timestamp::from_second(0).unwrap(),
                tz: None,
                body: (*one).to_string(),
            })
            .collect();
        task.references()
    }

    #[test]
    fn one_target_written_twice_with_two_labels_is_one_trace() {
        let all = told(
            "[the report](https://x.example/1)",
            &["[final report](https://x.example/1)"],
        );

        assert_eq!(all.len(), 1, "the same link came back twice: {all:?}");
    }

    #[test]
    fn a_step_anchor_is_not_something_the_task_left_behind() {
        let all = told("see [[#3]] and [[CUSLEG-1]]", &[]);

        assert_eq!(all.len(), 1);
        assert_eq!(all[0].target, "CUSLEG-1");
    }
}
