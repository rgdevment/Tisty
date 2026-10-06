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

    /// Judged at replay, so two machines that crossed a move agree on where the part landed.
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

    /// Reachable only from a build that knew no parts, or a move crossed on another machine.
    pub(crate) fn turned_routine(&mut self, id: TaskId) {
        if !self.tasks.get(&id).is_some_and(repeats) {
            return;
        }
        for task in self
            .tasks
            .values_mut()
            .filter(|task| task.id == id || task.part_of == Some(id))
        {
            task.part_of = None;
        }
    }

    /// What comes back never ends, and a whole is there to end.
    pub fn repeat_refused(&self, id: TaskId) -> Option<&'static str> {
        let task = self.tasks.get(&id)?;
        match () {
            () if task.part_of.is_some() => Some("partRepeats"),
            () if self.holds_parts(id) => Some("wholeRepeats"),
            () => None,
        }
    }

    pub(crate) fn open_parts_dropped(&self, id: TaskId) -> Vec<Op> {
        self.parts_of(id)
            .filter(|part| part.status == Status::Open)
            .map(|part| Op::TaskDrop { id: part.id })
            .collect()
    }
}

/// Something that comes back never ends, so it can neither hold an end nor be part of one.
fn repeats(task: &Task) -> bool {
    task.repeat.is_some() || task.after.is_some()
}

#[cfg(test)]
#[path = "parts_test.rs"]
mod tests;
