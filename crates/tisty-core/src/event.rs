mod op;

pub use op::{
    ALIAS_AT_MOST, Body, DeviceKind, DocAdd, Filed, Flag, FolderAdd, KNOWN_OPS, ListAdd, LogAdd,
    LogEdit, Look, Name, Op, Resolve, Said, Signature, StepAdd, StepRef, StepReorder, StepText,
    Stitch, TaskAdd, TaskMove, TaskPatch,
};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub const SCHEMA_VERSION: u32 = 15;

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
    /// The client an assistant spoke through — «claude-code», «codex» — as the MCP session
    /// named itself; sealed by the server that wrote, read by nothing that decides.
    #[serde(rename = "via", default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
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
            via: None,
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
            | Op::TaskResolve { id, .. }
            | Op::TaskUnresolve { id }
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
            | Op::FolderArchive { id }
            | Op::FolderUnarchive { id }
            | Op::DocAdd { id, .. }
            | Op::DocMove { id, .. }
            | Op::DocSaid { id, .. }
            | Op::DocDelete { id }
            | Op::DocSigned { id, .. }
            | Op::DocArchive { id }
            | Op::DocUnarchive { id }
            | Op::DocFlag { id, .. }
            | Op::DocUnflag { id }
            | Op::DocLock { id }
            | Op::DocUnlock { id } => Some(*id),
            Op::DeviceJoin { .. }
            | Op::DeviceHost { .. }
            | Op::Signed { .. }
            | Op::DeviceRemove { .. }
            | Op::AttachRetire { .. }
            | Op::StoresJoined { .. } => None,
        }
    }
}

#[cfg(test)]
#[path = "event_test.rs"]
mod tests;
