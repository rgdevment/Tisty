//! Which tasks a view shows. Shared, or each client answers it differently.

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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Filter {
    pub archive: bool,
    pub inbox: bool,
    /// A task has one list, so several mean «any of».
    pub lists: Vec<ListId>,
    /// A task has many tags, so several mean «all of».
    pub tags: Vec<Tag>,
    pub priority: Option<Priority>,
    pub window: Option<Window>,
}

impl Filter {
    pub fn matches(&self, task: &Task, today: Date) -> bool {
        if task.is_archived() != self.archive {
            return false;
        }
        if self.inbox && task.list.is_some() {
            return false;
        }
        if !self.lists.is_empty() && !task.list.is_some_and(|l| self.lists.contains(&l)) {
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
        let archive = filter(|f| f.archive = true);

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
