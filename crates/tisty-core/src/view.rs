use jiff::civil::Date;

use crate::model::{ListId, Priority, Tag, Task};

#[derive(Debug, Clone, PartialEq)]
pub enum Window {
    /// Includes undated work: it is not waiting for a later day.
    Today,
    On(Date),
    Until(Date),
    /// What is still ahead, so it never repeats what «today» already showed.
    After(Date),
    Overdue,
}

/// `Either` exists for tag views, which cross the open/archived boundary.
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
    /// A task has one list, so several mean «any of».
    pub lists: Vec<ListId>,
    /// A task has many tags, so several mean «all of».
    pub tags: Vec<Tag>,
    /// True for any tag; empty `tags` then means «what I filed», not «everything».
    pub tagged: bool,
    /// Folded noise stays out unless it is asked for by name.
    pub hidden: bool,
    pub priority: Option<Priority>,
    pub window: Option<Window>,
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

        let on = task.date.as_ref().map(|d| d.date());
        match &self.window {
            None => true,
            Some(Window::Today) => on.is_none_or(|d| d <= today),
            Some(Window::On(day)) => on == Some(*day),
            Some(Window::Until(day)) => on.is_some_and(|d| d <= *day),
            Some(Window::After(day)) => on.is_some_and(|d| d > *day),
            Some(Window::Overdue) => on.is_some_and(|d| d < today),
        }
    }
}

/// Naming it beats mentioning it: a heavy task that says «brasil» in passing
/// must not outrank one that is called «…en BR».
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hit {
    Named,
    Mentioned,
}

/// Searches both open and archived tasks by default.
pub fn matches_query(task: &Task, query: &str) -> Option<Hit> {
    let contains = |text: &str| text.to_lowercase().contains(query);

    if contains(&task.title) || task.tags.iter().any(|t| contains(t.as_str())) {
        return Some(Hit::Named);
    }
    // A reference is a declared pointer, so naming one whole names the task —
    // «OPS-3465» is what the task is about, not something it says in passing.
    // The label counts as much as the target: a ticket is written as a link, and
    // what gets searched is its code, never the address behind it.
    if task.volume.refs > 0
        && task.references().iter().any(|one| {
            one.target.to_lowercase() == query
                || one
                    .label
                    .as_deref()
                    .is_some_and(|l| l.to_lowercase() == query)
        })
    {
        return Some(Hit::Named);
    }
    let body = task.description.as_deref().is_some_and(contains)
        || task.log.iter().any(|e| contains(&e.body))
        || task.steps.iter().any(|s| contains(&s.text));

    body.then_some(Hit::Mentioned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DateSpec, Status, Task};
    use ulid::Ulid;

    fn today() -> Date {
        "2026-08-05".parse().unwrap()
    }

    fn task(title: &str) -> Task {
        Task::new(Ulid::generate(), title, "a0")
    }

    fn dated(title: &str, day: &str) -> Task {
        let mut t = task(title);
        t.date = Some(DateSpec::all_day(day.parse().unwrap(), "UTC"));
        t
    }

    fn filter(f: impl FnOnce(&mut Filter)) -> Filter {
        let mut filter = Filter::default();
        f(&mut filter);
        filter
    }

    #[test]
    fn today_keeps_undated_work_in_sight() {
        let f = filter(|f| f.window = Some(Window::Today));

        assert!(f.matches(&task("no date"), today()));
        assert!(f.matches(&dated("overdue", "2026-08-01"), today()));
        assert!(f.matches(&dated("today", "2026-08-05"), today()));
        assert!(!f.matches(&dated("later", "2026-08-09"), today()));
    }

    #[test]
    fn a_day_means_that_day_and_not_everything_before_it() {
        let f = filter(|f| f.window = Some(Window::On("2026-08-06".parse().unwrap())));

        assert!(f.matches(&dated("tomorrow", "2026-08-06"), today()));
        assert!(!f.matches(&dated("today", "2026-08-05"), today()));
        assert!(!f.matches(&task("no date"), today()));
    }

    #[test]
    fn a_week_reaches_forward_but_still_shows_what_is_late() {
        let f = filter(|f| f.window = Some(Window::Until("2026-08-12".parse().unwrap())));

        assert!(f.matches(&dated("late", "2026-07-30"), today()));
        assert!(f.matches(&dated("within", "2026-08-11"), today()));
        assert!(!f.matches(&dated("beyond", "2026-08-20"), today()));
        assert!(
            !f.matches(&task("no date"), today()),
            "undated is not a week"
        );
    }

    #[test]
    fn what_is_ahead_never_repeats_what_today_already_showed() {
        let f = filter(|f| f.window = Some(Window::After(today())));

        assert!(f.matches(&dated("later", "2026-08-09"), today()));
        assert!(!f.matches(&dated("today", "2026-08-05"), today()));
        assert!(!f.matches(&dated("overdue", "2026-08-01"), today()));
        assert!(
            !f.matches(&task("no date"), today()),
            "undated work is not ahead: today already claims it"
        );
    }

    #[test]
    fn overdue_is_only_what_has_a_date_and_missed_it() {
        let f = filter(|f| f.window = Some(Window::Overdue));

        assert!(f.matches(&dated("late", "2026-08-04"), today()));
        assert!(!f.matches(&dated("today", "2026-08-05"), today()));
        assert!(!f.matches(&task("no date"), today()));
    }

    #[test]
    fn tags_narrow_and_do_not_widen() {
        let mut t = task("both");
        t.tags = vec![Tag::new("backend").unwrap(), Tag::new("urgent").unwrap()];
        let mut one = task("one");
        one.tags = vec![Tag::new("backend").unwrap()];

        let f = filter(|f| {
            f.tags = vec![Tag::new("backend").unwrap(), Tag::new("urgent").unwrap()];
        });

        assert!(f.matches(&t, today()));
        assert!(!f.matches(&one, today()));
    }

    #[test]
    fn several_lists_mean_any_of_them() {
        let (a, b) = (Ulid::generate(), Ulid::generate());
        let mut first = task("in a");
        first.list = Some(a);
        let mut other = task("in neither");
        other.list = Some(Ulid::generate());

        let f = filter(|f| f.lists = vec![a, b]);

        assert!(f.matches(&first, today()));
        assert!(!f.matches(&other, today()));
        assert!(!f.matches(&task("loose"), today()));
    }

    #[test]
    fn the_archive_and_the_open_work_never_mix() {
        let mut done = task("finished");
        done.status = Status::Done;

        let open = filter(|_| {});
        let archive = filter(|f| f.scope = Scope::Archived);

        assert!(open.matches(&task("open"), today()));
        assert!(!open.matches(&done, today()));
        assert!(archive.matches(&done, today()));
        assert!(!archive.matches(&task("open"), today()));
    }

    #[test]
    fn the_inbox_is_what_no_list_claimed() {
        let mut filed = task("filed");
        filed.list = Some(Ulid::generate());

        let f = filter(|f| f.inbox = true);

        assert!(f.matches(&task("loose"), today()));
        assert!(!f.matches(&filed, today()));
    }
}
