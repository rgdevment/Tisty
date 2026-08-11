use serde::{Deserialize, Deserializer, Serialize};

use crate::model::{DateSpec, ListId, LogId, Priority, Repeat, StepId, Tag, TaskId};

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
    #[serde(rename = "list.archive")]
    ListArchive { id: ListId },
    #[serde(rename = "list.unarchive")]
    ListUnarchive { id: ListId },
    #[serde(rename = "list.delete")]
    ListDelete { id: ListId },
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
