use jiff::ToSpan;
use jiff::civil::Date;
use tisty_core::{ListId, Priority, Tag, Task};

use crate::app::App;
use crate::i18n::{self, Lang};

#[derive(Debug, PartialEq)]
pub enum Window {
    /// Includes undated work: it is not waiting for a later day.
    Today,
    On(Date),
    Until(Date),
    Overdue,
}

#[derive(Debug, Default)]
pub struct Filter {
    pub archive: bool,
    inbox: bool,
    /// A task has one list, so several mean «any of».
    lists: Vec<ListId>,
    /// A task has many tags, so several mean «all of».
    tags: Vec<Tag>,
    priority: Option<Priority>,
    window: Option<Window>,
    heading: String,
}

impl Filter {
    /// Bare `ls` means today; naming any filter widens the scope to everything open.
    pub fn parse(tokens: &[String], app: &App, today: Date, lang: Lang) -> anyhow::Result<Self> {
        let mut f = Self {
            heading: heading(tokens, lang),
            ..Default::default()
        };
        if tokens.is_empty() {
            f.window = Some(Window::Today);
            return Ok(f);
        }

        for token in tokens {
            f.take(token, app, today, lang)?;
        }
        Ok(f)
    }

    fn take(&mut self, token: &str, app: &App, today: Date, lang: Lang) -> anyhow::Result<()> {
        if let Some(name) = token.strip_prefix('#') {
            return self.take_list(name, app, lang);
        }
        if let Some(name) = token.strip_prefix('@') {
            self.tags.push(Tag::new(name)?);
            return Ok(());
        }
        if let Some(digit) = token.strip_prefix('!') {
            self.priority = Some(Priority::try_from(digit.parse::<u8>()?)?);
            return Ok(());
        }

        match i18n::canonical_filter(token) {
            Some("all") => Ok(()),
            Some("inbox") => {
                self.inbox = true;
                Ok(())
            }
            Some("archive") => {
                self.archive = true;
                Ok(())
            }
            Some("today") => self.take_window(Window::Today, lang),
            Some("tomorrow") => self.take_window(Window::On(today.tomorrow()?), lang),
            Some("week") => self.take_window(Window::Until(today.checked_add(7.days())?), lang),
            Some("overdue") => self.take_window(Window::Overdue, lang),
            _ => self.take_date(token, lang),
        }
    }

    fn take_list(&mut self, name: &str, app: &App, lang: Lang) -> anyhow::Result<()> {
        match app.find_list(name).as_slice() {
            [one] => {
                self.lists.push(one.id);
                Ok(())
            }
            [] => anyhow::bail!("{}", lang.fill("no-such-list", &[("selector", name)])),
            _ => anyhow::bail!("{}", lang.fill("ambiguous-list", &[("selector", name)])),
        }
    }

    fn take_date(&mut self, token: &str, lang: Lang) -> anyhow::Result<()> {
        let now = jiff::Zoned::now();
        match tisty_nl::parse_date(token, &now, lang.code()) {
            Some(spec) => self.take_window(Window::On(spec.date()), lang),
            None => anyhow::bail!(
                "{}",
                lang.fill(
                    "unknown-filter",
                    &[("filter", token), ("known", i18n::FILTERS)]
                )
            ),
        }
    }

    /// Refuses a second one: «tomorrow overdue» has no answer to narrow.
    fn take_window(&mut self, window: Window, lang: Lang) -> anyhow::Result<()> {
        if self.window.is_some() {
            anyhow::bail!("{}", lang.get("one-window-only"));
        }
        self.window = Some(window);
        Ok(())
    }

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
            Some(Window::Overdue) => on.is_some_and(|d| d < today),
        }
    }

    pub fn heading(&self) -> &str {
        &self.heading
    }
}

/// One named filter reads back translated; anything composed, as it was typed.
fn heading(tokens: &[String], lang: Lang) -> String {
    match tokens {
        [] => lang.get("today").to_string(),
        [one] => match i18n::canonical_filter(one) {
            Some(name) => lang.get(name).to_string(),
            None => one.clone(),
        },
        many => many.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tisty_core::Status;
    use ulid::Ulid;

    fn today() -> Date {
        "2026-08-05".parse().unwrap()
    }

    fn task(title: &str) -> Task {
        Task::new(Ulid::generate(), title, "a0")
    }

    fn dated(title: &str, day: &str) -> Task {
        let mut t = task(title);
        t.date = Some(tisty_core::DateSpec::all_day(day.parse().unwrap(), "UTC"));
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
