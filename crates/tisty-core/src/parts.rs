use crate::event::{Event, Op};
use crate::model::{Status, Task, TaskId};
use crate::state::State;

impl State {
    pub fn holds_parts(&self, id: TaskId) -> bool {
        self.tasks.values().any(|task| task.part_of == Some(id))
    }

    pub fn parts_of(&self, id: TaskId) -> impl Iterator<Item = &Task> {
        self.tasks
            .values()
            .filter(move |task| task.part_of == Some(id))
    }

    /// Judged at replay rather than where the event is written, so two machines that crossed a
    /// move agree on where the part landed, and a build that wrote it differently cannot nest.
    pub(crate) fn may_hold(&self, whole: TaskId, part: &Task, event: &Event) -> bool {
        let Some(holder) = self.tasks.get(&whole) else {
            return false;
        };
        whole != part.id
            && holder.part_of.is_none()
            && !self.holds_parts(part.id)
            && !repeats(holder)
            && !repeats(part)
            && (!self.assistants.contains(&event.device) || self.attended_by_agents(holder))
    }

    pub(crate) fn task_parted(&mut self, id: TaskId, whole: Option<TaskId>, event: &Event) {
        let landed = match whole {
            None => true,
            Some(whole) => self
                .tasks
                .get(&id)
                .is_some_and(|part| self.may_hold(whole, part, event)),
        };
        if landed && let Some(task) = self.tasks.get_mut(&id) {
            task.part_of = whole;
        }
    }

    pub(crate) fn part_turned_routine(&mut self, id: TaskId) {
        if let Some(task) = self.tasks.get_mut(&id)
            && repeats(task)
        {
            task.part_of = None;
        }
    }

    /// The parts still open are let go in the same breath as the whole is finished, so one
    /// undo takes all of it back.
    pub fn completing_with_parts(&self, id: TaskId, now: jiff::Zoned) -> Vec<Op> {
        let mut ops: Vec<Op> = self
            .parts_of(id)
            .filter(|part| part.status == Status::Open)
            .map(|part| Op::TaskDrop { id: part.id })
            .collect();
        ops.extend(self.completing(id, now));
        ops
    }
}

/// Something that comes back never ends, so it can neither hold an end nor be part of one.
fn repeats(task: &Task) -> bool {
    task.repeat.is_some() || task.after.is_some()
}

#[cfg(test)]
#[path = "parts_test.rs"]
mod tests;
