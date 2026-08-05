mod op;

pub use op::{
    Body, ListAdd, LogAdd, LogEdit, Name, Op, StepAdd, StepRef, StepReorder, StepText, TaskAdd,
    TaskMove, TaskPatch,
};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub const SCHEMA_VERSION: u32 = 1;

/// Lives in local config and is never synced: two machines sharing an id would
/// write the same segment file, which is what makes conflicts impossible.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "v")]
    pub version: u32,
    #[serde(rename = "ts")]
    pub timestamp: jiff::Timestamp,
    #[serde(rename = "by")]
    pub device: DeviceId,
    #[serde(flatten)]
    pub op: Op,
}

impl Event {
    pub fn new(device: DeviceId, timestamp: jiff::Timestamp, op: Op) -> Self {
        Self {
            version: SCHEMA_VERSION,
            timestamp,
            device,
            op,
        }
    }

    /// Device tiebreak guarantees every machine replays into the same state.
    pub fn sort_key(&self) -> (jiff::Timestamp, &DeviceId) {
        (self.timestamp, &self.device)
    }

    pub fn entity_id(&self) -> Ulid {
        match &self.op {
            Op::TaskAdd { id, .. }
            | Op::TaskUpdate { id, .. }
            | Op::TaskDone { id }
            | Op::TaskReopen { id }
            | Op::TaskDrop { id }
            | Op::TaskDelete { id }
            | Op::TaskMove { id, .. }
            | Op::TaskDescribe { id, .. }
            | Op::TaskLog { id, .. }
            | Op::TaskLogEdit { id, .. }
            | Op::StepAdd { id, .. }
            | Op::StepDone { id, .. }
            | Op::StepUndone { id, .. }
            | Op::StepText { id, .. }
            | Op::StepRemove { id, .. }
            | Op::StepReorder { id, .. } => *id,
            Op::ListAdd { id, .. }
            | Op::ListRename { id, .. }
            | Op::ListArchive { id }
            | Op::ListDelete { id } => *id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DateSpec, Priority, Tag};

    fn at(ms: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_millisecond(ms).unwrap()
    }

    fn id() -> Ulid {
        "01J8F2K3XQ0000000000000000".parse().unwrap()
    }

    fn add_event() -> Event {
        let mut d = TaskAdd::new("issue en redirecciones istio para registration en BR", "a0");
        d.date = Some(DateSpec::floating(
            "2026-08-05T10:00:00".parse().unwrap(),
            "America/Santiago",
        ));
        d.priority = Some(Priority::P1);
        d.tags = vec![Tag::new("istio").unwrap(), Tag::new("brasil").unwrap()];

        Event::new(
            DeviceId("dev_a3f1".into()),
            at(1_754_320_931_482),
            Op::TaskAdd { id: id(), d },
        )
    }

    #[test]
    fn an_event_fits_on_one_line() {
        let json = serde_json::to_string(&add_event()).unwrap();
        assert!(!json.contains('\n'), "{json}");
        eprintln!("{json}");
    }

    #[test]
    fn round_trips() {
        let ev = add_event();
        let json = serde_json::to_string(&ev).unwrap();
        assert_eq!(ev, serde_json::from_str::<Event>(&json).unwrap());
    }

    #[test]
    fn skips_empty_fields() {
        let ev = Event::new(
            DeviceId("dev_a3f1".into()),
            at(1),
            Op::TaskAdd {
                id: id(),
                d: TaskAdd::new("agendar reunión con Pepe", "a0"),
            },
        );
        let json = serde_json::to_string(&ev).unwrap();
        for absent in ["date", "deadline", "priority", "list", "tags", "reminders"] {
            assert!(
                !json.contains(absent),
                "'{absent}' should not appear in {json}"
            );
        }
    }

    #[test]
    fn every_op_carries_its_entity_id() {
        let ops = [
            Op::TaskDone { id: id() },
            Op::TaskLog {
                id: id(),
                d: LogAdd {
                    entry: Ulid::generate(),
                    body: "the sidecar failed".into(),
                },
            },
            Op::StepDone {
                id: id(),
                d: StepRef {
                    step: Ulid::generate(),
                },
            },
            Op::ListArchive { id: id() },
        ];
        for op in ops {
            let ev = Event::new(DeviceId("dev_a3f1".into()), at(1), op);
            assert_eq!(ev.entity_id(), id());
        }
    }

    #[test]
    fn patch_distinguishes_absent_from_null() {
        let untouched: TaskPatch = serde_json::from_str("{}").unwrap();
        assert_eq!(untouched.date, None);

        let cleared: TaskPatch = serde_json::from_str(r#"{"date":null}"#).unwrap();
        assert_eq!(cleared.date, Some(None));
    }

    #[test]
    fn moving_out_of_a_list_is_not_the_same_as_not_moving() {
        let reordered: TaskMove = serde_json::from_str(r#"{"order":"a1"}"#).unwrap();
        assert_eq!(reordered.list, None);

        let to_inbox: TaskMove = serde_json::from_str(r#"{"list":null}"#).unwrap();
        assert_eq!(to_inbox.list, Some(None));
    }

    #[test]
    fn sort_key_breaks_ties_by_device() {
        let a = Event::new(
            DeviceId("dev_0001".into()),
            at(1),
            Op::TaskDone { id: id() },
        );
        let b = Event::new(
            DeviceId("dev_0002".into()),
            at(1),
            Op::TaskDone { id: id() },
        );
        assert!(a.sort_key() < b.sort_key());
    }

    #[test]
    fn op_names_are_the_documented_ones() {
        let cases = [
            (Op::TaskDone { id: id() }, "task.done"),
            (Op::TaskDrop { id: id() }, "task.drop"),
            (Op::ListArchive { id: id() }, "list.archive"),
            (
                Op::StepUndone {
                    id: id(),
                    d: StepRef {
                        step: Ulid::generate(),
                    },
                },
                "task.step.undone",
            ),
        ];
        for (op, name) in cases {
            let json = serde_json::to_string(&op).unwrap();
            assert!(json.contains(&format!(r#""op":"{name}""#)), "{json}");
        }
    }

    /// A log entry is an event, never an edited field, so the log is
    /// chronological and immutable by construction.
    #[test]
    fn the_log_grows_by_appending() {
        let entries = ["the sidecar failed", "it was X-Forwarded-Proto"];
        let events: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(i, body)| {
                Event::new(
                    DeviceId("dev_a3f1".into()),
                    at(i as i64 + 1),
                    Op::TaskLog {
                        id: id(),
                        d: LogAdd {
                            entry: Ulid::generate(),
                            body: (*body).into(),
                        },
                    },
                )
            })
            .collect();

        let mut sorted = events.clone();
        sorted.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        assert_eq!(events, sorted);
    }
}
