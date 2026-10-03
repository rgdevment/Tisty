use crate::{
    State,
    event::{Op, TaskMove},
    model::{List, ListId},
};

impl State {
    pub fn shelved_lists(&self) -> Vec<&List> {
        let holding = self.lists_holding_open();
        let mut lists: Vec<_> = self
            .lists
            .values()
            .filter(|one| one.archived && !holding.contains(&one.id))
            .collect();
        lists.sort_by(|a, b| (&a.order, a.id).cmp(&(&b.order, b.id)));
        lists
    }

    /// An archived list that a racing writer filed into stays in sight until it is closed out.
    pub(crate) fn lists_holding_open(&self) -> std::collections::HashSet<ListId> {
        self.tasks
            .values()
            .filter(|task| task.is_open())
            .filter_map(|task| task.list)
            .collect()
    }

    pub fn list_holds_open(&self, id: ListId) -> bool {
        self.tasks
            .values()
            .any(|task| task.list == Some(id) && task.is_open())
    }

    /// A list with closed tasks is archived rather than deleted, so its history keeps its name;
    /// what is still open goes to the inbox either way.
    pub fn dropping_list(&self, id: ListId) -> Vec<Op> {
        let held: Vec<_> = self
            .tasks
            .values()
            .filter(|task| task.list == Some(id))
            .collect();
        if held.iter().all(|task| task.is_open()) {
            return vec![Op::ListDelete { id }];
        }
        let mut ops: Vec<Op> = held
            .iter()
            .filter(|task| task.is_open())
            .map(|task| Op::TaskMove {
                id: task.id,
                d: TaskMove {
                    list: Some(None),
                    order: None,
                },
            })
            .collect();
        ops.push(Op::ListArchive { id });
        ops
    }
}
