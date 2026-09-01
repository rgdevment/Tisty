mod op;

pub use op::{
    Body, DeviceKind, DocAdd, Filed, FolderAdd, KNOWN_OPS, ListAdd, LogAdd, LogEdit, Look, Name,
    Op, StepAdd, StepRef, StepReorder, StepText, Stitch, TaskAdd, TaskMove, TaskPatch,
};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub const SCHEMA_VERSION: u32 = 8;

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
    #[serde(rename = "tx", default, skip_serializing_if = "Option::is_none")]
    pub batch: Option<Ulid>,
    #[serde(rename = "un", default, skip_serializing_if = "std::ops::Not::not")]
    pub undo: bool,
    #[serde(rename = "re", default, skip_serializing_if = "std::ops::Not::not")]
    pub redo: bool,
    #[serde(rename = "n", default, skip_serializing_if = "is_zero")]
    pub seq: u64,
    /// A reader that does not know this operation skips it instead of refusing the whole store.
    #[serde(rename = "opt", default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    #[serde(rename = "tz", default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(flatten)]
    pub op: Op,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

impl Event {
    pub fn new(device: DeviceId, timestamp: jiff::Timestamp, op: Op) -> Self {
        Self {
            version: SCHEMA_VERSION,
            timestamp,
            device,
            batch: None,
            undo: false,
            redo: false,
            seq: 0,
            optional: false,
            zone: None,
            op: op.composed(),
        }
    }

    pub fn zoned(&self) -> Option<jiff::Zoned> {
        let zone = jiff::tz::TimeZone::get(self.zone.as_deref()?).ok()?;
        Some(self.timestamp.to_zoned(zone))
    }

    pub fn in_batch(mut self, batch: Ulid) -> Self {
        self.batch = Some(batch);
        self
    }

    pub fn sort_key(&self) -> (jiff::Timestamp, &DeviceId, u64) {
        (self.timestamp, &self.device, self.seq)
    }

    pub fn entity_id(&self) -> Option<Ulid> {
        match &self.op {
            Op::TaskAdd { id, .. }
            | Op::TaskUpdate { id, .. }
            | Op::TaskDone { id, .. }
            | Op::TaskReopen { id }
            | Op::TaskDrop { id }
            | Op::TaskDelete { id }
            | Op::TaskHide { id }
            | Op::TaskShow { id }
            | Op::TaskMove { id, .. }
            | Op::TaskDescribe { id, .. }
            | Op::TaskLog { id, .. }
            | Op::TaskLogEdit { id, .. }
            | Op::StepAdd { id, .. }
            | Op::StepDone { id, .. }
            | Op::StepUndone { id, .. }
            | Op::StepText { id, .. }
            | Op::StepRemove { id, .. }
            | Op::StepReorder { id, .. } => Some(*id),
            Op::ListAdd { id, .. }
            | Op::ListRename { id, .. }
            | Op::ListLook { id, .. }
            | Op::ListArchive { id }
            | Op::ListUnarchive { id }
            | Op::ListDelete { id }
            | Op::FolderAdd { id, .. }
            | Op::FolderRename { id, .. }
            | Op::FolderLook { id, .. }
            | Op::FolderMove { id, .. }
            | Op::FolderDelete { id }
            | Op::DocAdd { id, .. }
            | Op::DocMove { id, .. }
            | Op::DocDelete { id }
            | Op::DocArchive { id }
            | Op::DocUnarchive { id } => Some(*id),
            Op::DeviceJoin { .. }
            | Op::DeviceRemove { .. }
            | Op::AttachRetire { .. }
            | Op::StoresJoined { .. } => None,
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
        let mut d = TaskAdd::new("fix the failing checkout", "a0");
        d.date = Some(DateSpec::floating(
            "2026-08-05T10:00:00".parse().unwrap(),
            "America/Santiago",
        ));
        d.priority = Some(Priority::Do);
        d.tags = vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()];

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
                d: TaskAdd::new("book a haircut", "a0"),
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
            Op::TaskDone {
                id: id(),
                filled: false,
            },
            Op::TaskLog {
                id: id(),
                d: LogAdd::new(Ulid::generate(), "first attempt failed"),
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
            assert_eq!(ev.entity_id(), Some(id()));
        }
    }

    #[test]
    fn the_list_of_known_operations_is_the_one_serde_accepts() {
        let stranger = r#"{"v":1,"ts":"2026-08-28T10:00:00Z","by":"dev_a","op":"task.bless"}"#;
        let complaint = serde_json::from_str::<Event>(stranger)
            .expect_err("an operation that does not exist cannot parse")
            .to_string();

        for name in KNOWN_OPS {
            assert!(
                complaint.contains(&format!("`{name}`")),
                "serde does not know `{name}`, but the reader forgives it as if it did"
            );
        }
        assert_eq!(
            complaint.matches('`').count() / 2,
            KNOWN_OPS.len() + 1,
            "serde accepts an operation the reader would treat as corruption"
        );
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
            Op::TaskDone {
                id: id(),
                filled: false,
            },
        );
        let b = Event::new(
            DeviceId("dev_0002".into()),
            at(1),
            Op::TaskDone {
                id: id(),
                filled: false,
            },
        );
        assert!(a.sort_key() < b.sort_key());
    }

    #[test]
    fn op_names_are_the_documented_ones() {
        let cases = [
            (
                Op::TaskDone {
                    id: id(),
                    filled: false,
                },
                "task.done",
            ),
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

    #[test]
    fn the_log_grows_by_appending() {
        let entries = ["first attempt failed", "an index was missing"];
        let events: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(i, body)| {
                Event::new(
                    DeviceId("dev_a3f1".into()),
                    at(i as i64 + 1),
                    Op::TaskLog {
                        id: id(),
                        d: LogAdd::new(Ulid::generate(), *body),
                    },
                )
            })
            .collect();

        let mut sorted = events.clone();
        sorted.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        assert_eq!(events, sorted);
    }

    fn made(op: Op) -> Op {
        Event::new(DeviceId("dev_a3f1".into()), at(1), op).op
    }

    #[test]
    fn every_kind_of_prose_reaches_the_log_in_one_spelling() {
        let apart = "disen\u{0303}o";
        let together = "diseño";

        let kinds: Vec<(Op, String)> = vec![
            (
                Op::TaskAdd {
                    id: id(),
                    d: TaskAdd::new(apart, "a0"),
                },
                together.into(),
            ),
            (
                Op::TaskUpdate {
                    id: id(),
                    d: TaskPatch {
                        title: Some(apart.into()),
                        ..Default::default()
                    },
                },
                together.into(),
            ),
            (
                Op::TaskDescribe {
                    id: id(),
                    d: Body {
                        body: Some(apart.into()),
                    },
                },
                together.into(),
            ),
            (
                Op::TaskLog {
                    id: id(),
                    d: LogAdd::new(Ulid::generate(), apart),
                },
                together.into(),
            ),
            (
                Op::TaskLogEdit {
                    id: id(),
                    d: LogEdit {
                        entry: Ulid::generate(),
                        body: apart.into(),
                    },
                },
                together.into(),
            ),
            (
                Op::StepAdd {
                    id: id(),
                    d: StepAdd {
                        step: Ulid::generate(),
                        text: apart.into(),
                        order: "a0".into(),
                    },
                },
                together.into(),
            ),
            (
                Op::StepText {
                    id: id(),
                    d: StepText {
                        step: Ulid::generate(),
                        text: apart.into(),
                    },
                },
                together.into(),
            ),
            (
                Op::ListAdd {
                    id: id(),
                    d: ListAdd {
                        name: apart.into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
                together.into(),
            ),
            (
                Op::ListRename {
                    id: id(),
                    d: Name { name: apart.into() },
                },
                together.into(),
            ),
        ];

        for (op, wanted) in kinds {
            let written = serde_json::to_string(&made(op)).unwrap();
            assert!(
                written.contains(&wanted),
                "a decomposed accent survived into {written}"
            );
            assert!(
                !written.contains("\\u0303"),
                "a bare combining mark reached the log: {written}"
            );
        }
    }

    #[test]
    fn an_event_that_arrived_from_another_machine_is_written_untouched() {
        let theirs = Event {
            version: SCHEMA_VERSION,
            timestamp: at(1),
            device: DeviceId("dev_b7c2".into()),
            batch: None,
            undo: false,
            redo: false,
            seq: 0,
            op: Op::TaskAdd {
                id: id(),
                d: TaskAdd::new("disen\u{0303}o", "a0"),
            },
            optional: false,
            zone: None,
        };

        let read: Event = serde_json::from_str(&serde_json::to_string(&theirs).unwrap()).unwrap();

        assert_eq!(read, theirs);
    }
}
