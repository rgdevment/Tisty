use crate::State;
use crate::event::{Event, Op, PageChange};
use crate::model::TaskId;

fn grows(op: &Op) -> Option<TaskId> {
    match op {
        Op::StepAdd { id, .. } => Some(*id),
        Op::TaskLog { id, d } if !d.body.trim().is_empty() => Some(*id),
        Op::TaskDescribe { id, d }
            if d.body.as_ref().is_some_and(|body| !body.trim().is_empty()) =>
        {
            Some(*id)
        }
        Op::TaskPaged { id, d } if matches!(d.change, PageChange::Added | PageChange::Written) => {
            Some(*id)
        }
        _ => None,
    }
}

impl State {
    pub(crate) fn grown(&mut self, event: &Event) {
        if event.undo || event.redo {
            return;
        }
        if let Some(id) = grows(&event.op) {
            self.with_task(id, |task| task.grew_at(event.timestamp));
        }
    }
}

#[cfg(test)]
#[path = "growing_test.rs"]
mod tests;
