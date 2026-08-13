use serde::{Deserialize, Deserializer, Serialize};

use crate::model::{
    DateSpec, DocId, FolderId, ListId, LogId, Priority, Repeat, StepId, Tag, TaskId,
};

/// Serde folds `null` into `None`, making "clear" and "leave alone" the same.
mod null_clears {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<T, S>(v: &Option<Option<T>>, s: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        v.serialize(s)
    }

    pub fn deserialize<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Option::deserialize(d).map(Some)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Op {
    #[serde(rename = "task.add")]
    TaskAdd { id: TaskId, d: TaskAdd },
    #[serde(rename = "task.update")]
    TaskUpdate { id: TaskId, d: TaskPatch },
    #[serde(rename = "task.done")]
    TaskDone { id: TaskId },
    #[serde(rename = "task.reopen")]
    TaskReopen { id: TaskId },
    #[serde(rename = "task.drop")]
    TaskDrop { id: TaskId },
    #[serde(rename = "task.delete")]
    TaskDelete { id: TaskId },
    #[serde(rename = "task.hide")]
    TaskHide { id: TaskId },
    #[serde(rename = "task.show")]
    TaskShow { id: TaskId },
    #[serde(rename = "task.move")]
    TaskMove { id: TaskId, d: TaskMove },

    #[serde(rename = "task.describe")]
    TaskDescribe { id: TaskId, d: Body },
    #[serde(rename = "task.log")]
    TaskLog { id: TaskId, d: LogAdd },
    #[serde(rename = "task.log.edit")]
    TaskLogEdit { id: TaskId, d: LogEdit },

    #[serde(rename = "task.step.add")]
    StepAdd { id: TaskId, d: StepAdd },
    #[serde(rename = "task.step.done")]
    StepDone { id: TaskId, d: StepRef },
    #[serde(rename = "task.step.undone")]
    StepUndone { id: TaskId, d: StepRef },
    #[serde(rename = "task.step.text")]
    StepText { id: TaskId, d: StepText },
    #[serde(rename = "task.step.remove")]
    StepRemove { id: TaskId, d: StepRef },
    #[serde(rename = "task.step.reorder")]
    StepReorder { id: TaskId, d: StepReorder },

    #[serde(rename = "list.add")]
    ListAdd { id: ListId, d: ListAdd },
    #[serde(rename = "list.rename")]
    ListRename { id: ListId, d: Name },
    #[serde(rename = "list.look")]
    ListLook { id: ListId, d: Look },
    #[serde(rename = "list.archive")]
    ListArchive { id: ListId },
    #[serde(rename = "list.unarchive")]
    ListUnarchive { id: ListId },
    #[serde(rename = "list.delete")]
    ListDelete { id: ListId },

    #[serde(rename = "folder.add")]
    FolderAdd { id: FolderId, d: FolderAdd },
    #[serde(rename = "folder.rename")]
    FolderRename { id: FolderId, d: Name },
    #[serde(rename = "folder.look")]
    FolderLook { id: FolderId, d: Look },
    #[serde(rename = "folder.move")]
    FolderMove { id: FolderId, d: Filed },
    #[serde(rename = "folder.delete")]
    FolderDelete { id: FolderId },

    #[serde(rename = "doc.add")]
    DocAdd { id: DocId, d: DocAdd },
    #[serde(rename = "doc.move")]
    DocMove { id: DocId, d: Filed },
    #[serde(rename = "doc.delete")]
    DocDelete { id: DocId },
    #[serde(rename = "doc.archive")]
    DocArchive { id: DocId },
    #[serde(rename = "doc.unarchive")]
    DocUnarchive { id: DocId },
}

impl Op {
    /// The same operation, aimed at another entity. Only a redo needs this: it
    /// has to rebuild what its own undo buried under a tombstone.
    pub fn about(self, id: TaskId) -> Self {
        match self {
            Op::TaskAdd { d, .. } => Op::TaskAdd { id, d },
            Op::TaskUpdate { d, .. } => Op::TaskUpdate { id, d },
            Op::TaskDone { .. } => Op::TaskDone { id },
            Op::TaskReopen { .. } => Op::TaskReopen { id },
            Op::TaskDrop { .. } => Op::TaskDrop { id },
            Op::TaskDelete { .. } => Op::TaskDelete { id },
            Op::TaskHide { .. } => Op::TaskHide { id },
            Op::TaskShow { .. } => Op::TaskShow { id },
            Op::TaskMove { d, .. } => Op::TaskMove { id, d },
            Op::TaskDescribe { d, .. } => Op::TaskDescribe { id, d },
            Op::TaskLog { d, .. } => Op::TaskLog { id, d },
            Op::TaskLogEdit { d, .. } => Op::TaskLogEdit { id, d },
            Op::StepAdd { d, .. } => Op::StepAdd { id, d },
            Op::StepDone { d, .. } => Op::StepDone { id, d },
            Op::StepUndone { d, .. } => Op::StepUndone { id, d },
            Op::StepText { d, .. } => Op::StepText { id, d },
            Op::StepRemove { d, .. } => Op::StepRemove { id, d },
            Op::StepReorder { d, .. } => Op::StepReorder { id, d },
            Op::ListAdd { d, .. } => Op::ListAdd { id, d },
            Op::ListRename { d, .. } => Op::ListRename { id, d },
            Op::ListLook { d, .. } => Op::ListLook { id, d },
            Op::ListArchive { .. } => Op::ListArchive { id },
            Op::ListUnarchive { .. } => Op::ListUnarchive { id },
            Op::ListDelete { .. } => Op::ListDelete { id },
            Op::FolderAdd { d, .. } => Op::FolderAdd { id, d },
            Op::FolderRename { d, .. } => Op::FolderRename { id, d },
            Op::FolderLook { d, .. } => Op::FolderLook { id, d },
            Op::FolderMove { d, .. } => Op::FolderMove { id, d },
            Op::FolderDelete { .. } => Op::FolderDelete { id },
            Op::DocAdd { d, .. } => Op::DocAdd { id, d },
            Op::DocMove { d, .. } => Op::DocMove { id, d },
            Op::DocDelete { .. } => Op::DocDelete { id },
            Op::DocArchive { .. } => Op::DocArchive { id },
            Op::DocUnarchive { .. } => Op::DocUnarchive { id },
        }
    }

    pub fn composed(self) -> Self {
        use crate::text::composed;
        let one = |text: String| composed(&text);
        let maybe = |text: Option<String>| text.map(|one| composed(&one));

        match self {
            Op::TaskAdd { id, mut d } => {
                d.title = one(d.title);
                Op::TaskAdd { id, d }
            }
            Op::TaskUpdate { id, mut d } => {
                d.title = maybe(d.title);
                Op::TaskUpdate { id, d }
            }
            Op::TaskDescribe { id, mut d } => {
                d.body = maybe(d.body);
                Op::TaskDescribe { id, d }
            }
            Op::TaskLog { id, mut d } => {
                d.body = one(d.body);
                Op::TaskLog { id, d }
            }
            Op::TaskLogEdit { id, mut d } => {
                d.body = one(d.body);
                Op::TaskLogEdit { id, d }
            }
            Op::StepAdd { id, mut d } => {
                d.text = one(d.text);
                Op::StepAdd { id, d }
            }
            Op::StepText { id, mut d } => {
                d.text = one(d.text);
                Op::StepText { id, d }
            }
            Op::ListAdd { id, mut d } => {
                d.name = one(d.name);
                Op::ListAdd { id, d }
            }
            Op::FolderAdd { id, mut d } => {
                d.name = one(d.name);
                Op::FolderAdd { id, d }
            }
            Op::FolderRename { id, mut d } => {
                d.name = one(d.name);
                Op::FolderRename { id, d }
            }
            Op::ListRename { id, mut d } => {
                d.name = one(d.name);
                Op::ListRename { id, d }
            }
            plain => plain,
        }
    }

    pub fn about_whom(&self) -> TaskId {
        match self {
            Op::TaskAdd { id, .. }
            | Op::TaskUpdate { id, .. }
            | Op::TaskDone { id }
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
            | Op::StepReorder { id, .. }
            | Op::ListAdd { id, .. }
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
            | Op::DocUnarchive { id } => *id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAdd {
    pub title: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<ListId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reminders: Vec<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Repeat>,
    /// The occurrence this one was born from, so undoing that completion can
    /// find its successor instead of leaving two of the series alive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<TaskId>,
}

impl TaskAdd {
    pub fn new(title: impl Into<String>, order: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            order: order.into(),
            date: None,
            deadline: None,
            priority: None,
            list: None,
            tags: Vec::new(),
            reminders: Vec::new(),
            repeat: None,
            after: None,
        }
    }
}

/// Only changed fields travel: last-write-wins per field, not per entity.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub date: Option<Option<DateSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub deadline: Option<Option<DateSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reminders: Option<Vec<DateSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub repeat: Option<Option<Repeat>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskMove {
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub list: Option<Option<ListId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Body {
    #[serde(deserialize_with = "explicit_option")]
    pub body: Option<String>,
}

fn explicit_option<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(d)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogAdd {
    pub entry: LogId,
    /// The author's zone, or the entry reads as another hour elsewhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    pub body: String,
}

impl LogAdd {
    pub fn new(entry: LogId, body: impl Into<String>) -> Self {
        Self {
            entry,
            tz: None,
            body: body.into(),
        }
    }

    pub fn in_zone(mut self, tz: Option<String>) -> Self {
        self.tz = tz;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEdit {
    pub entry: LogId,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepAdd {
    pub step: StepId,
    pub text: String,
    pub order: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepRef {
    pub step: StepId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepText {
    pub step: StepId,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepReorder {
    pub step: StepId,
    pub order: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAdd {
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderAdd {
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<FolderId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocAdd {
    pub file: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<FolderId>,
}

/// `null` files it at the root, absent leaves it where it was.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Filed {
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub folder: Option<Option<FolderId>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Look {
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub icon: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "null_clears")]
    pub color: Option<Option<String>>,
}
