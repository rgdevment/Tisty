use jiff::civil::Date;

use crate::model::{ListId, Priority, Reading, Tag, Task};

#[derive(Debug, Clone, PartialEq)]
pub enum Window {
    Today,
    On(Date),
    Until(Date),
    After(Date),
    Overdue,
    Undated,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Scope {
    #[default]
    Open,
    Archived,
    Either,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Filter {
    pub scope: Scope,
    pub inbox: bool,
    pub lists: Vec<ListId>,
    pub tags: Vec<Tag>,
    pub tagged: bool,
    pub hidden: bool,
    pub priority: Option<Priority>,
    pub window: Option<Window>,
    pub repeating: bool,
    pub reading: Option<Reading>,
}

impl Filter {
    pub fn matches(&self, task: &Task, today: Date) -> bool {
        let fits = match self.scope {
            Scope::Open => !task.is_archived(),
            Scope::Archived => task.is_archived(),
            Scope::Either => true,
        };
        if !fits {
            return false;
        }
        if task.folded() != self.hidden {
            return false;
        }
        if self.inbox && task.list.is_some() {
            return false;
        }
        if !self.lists.is_empty() && !task.list.is_some_and(|l| self.lists.contains(&l)) {
            return false;
        }
        if self.tagged && task.tags.is_empty() {
            return false;
        }
        if !self.tags.iter().all(|t| task.tags.contains(t)) {
            return false;
        }
        if self.priority.is_some_and(|p| task.priority != p) {
            return false;
        }

        if self.repeating && task.repeat.is_none() {
            return false;
        }

        if self.reading.is_some_and(|how| task.reading() != how) {
            return false;
        }

        let on = task.date.as_ref().map(|d| d.date());
        match &self.window {
            None => true,
            Some(Window::Today) => on.is_none_or(|d| d <= today),
            Some(Window::On(day)) => on == Some(*day),
            Some(Window::Until(day)) => on.is_some_and(|d| d <= *day),
            Some(Window::After(day)) => on.is_some_and(|d| d > *day),
            // A deadline that passed is overdue whatever day you meant to get to it, and until
            // now nothing in the product read `deadline` at all.
            Some(Window::Overdue) => {
                let due = task.deadline.as_ref().map(|d| d.date());
                on.is_some_and(|d| d < today) || due.is_some_and(|d| d < today)
            }
            Some(Window::Undated) => on.is_none(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hit {
    Named,
    Mentioned,
}

pub fn matches_query(task: &Task, terms: &[String]) -> Option<Hit> {
    if terms.is_empty() {
        return None;
    }
    let folded = crate::text::folded;
    let named: Vec<String> = std::iter::once(folded(&task.title))
        .chain(task.tags.iter().map(|t| folded(t.as_str())))
        .collect();
    // A link matches whole or not at all: half a URL is in every other link on the list.
    let linked: Vec<String> = if task.volume.refs > 0 {
        task.references()
            .iter()
            .flat_map(|one| [Some(folded(&one.target)), one.label.as_deref().map(folded)])
            .flatten()
            .collect()
    } else {
        Vec::new()
    };
    let by_name = |term: &String| {
        named.iter().any(|one| one.contains(term.as_str())) || linked.iter().any(|one| one == term)
    };

    if terms.iter().all(by_name) {
        return Some(Hit::Named);
    }
    let body: Vec<String> = task
        .description
        .iter()
        .map(|one| folded(one))
        .chain(task.log.iter().map(|e| folded(&e.body)))
        .chain(task.steps.iter().map(|s| folded(&s.text)))
        .collect();

    terms
        .iter()
        .all(|term| by_name(term) || body.iter().any(|one| one.contains(term.as_str())))
        .then_some(Hit::Mentioned)
}

#[cfg(test)]
#[path = "view_test.rs"]
mod tests;
