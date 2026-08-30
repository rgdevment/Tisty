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

/// Every word has to land somewhere, but not all in the same field: what someone remembers of a
/// task is half its title and half a line of its journal.
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

    fn due(title: &str, day: &str) -> Task {
        let mut t = task(title);
        t.deadline = Some(DateSpec::all_day(day.parse().unwrap(), "UTC"));
        t
    }

    fn filter(f: impl FnOnce(&mut Filter)) -> Filter {
        let mut filter = Filter::default();
        f(&mut filter);
        filter
    }

    fn asked(task: &Task, query: &str) -> Option<Hit> {
        matches_query(task, &crate::text::terms(query))
    }

    #[test]
    fn a_word_is_found_whether_or_not_the_accent_was_typed() {
        let mut task = task("Llamar al médico");
        task.description = Some("y pedir la analítica".into());

        assert!(asked(&task, "medico").is_some(), "sin tilde no aparece");
        assert!(asked(&task, "médico").is_some());
        assert!(asked(&task, "MEDICO").is_some());
        assert!(asked(&task, "analitica").is_some());
        assert!(asked(&task, "nada de eso").is_none());
    }

    #[test]
    fn the_words_do_not_have_to_be_written_together_or_in_order() {
        let mut task = task("Llamar al médico por la analítica");
        task.description = Some("del control anual".into());

        assert!(asked(&task, "analitica llamar").is_some());
        assert!(asked(&task, "medico anual").is_some(), "titulo y cuerpo");
        assert!(asked(&task, "medico dentista").is_none(), "todas o nada");
    }

    #[test]
    fn a_phrase_in_quotes_keeps_its_order() {
        let task = task("Llamar al médico por la analítica");

        assert!(asked(&task, "\"al médico\"").is_some());
        assert!(asked(&task, "\"médico al\"").is_none());
    }

    #[test]
    fn what_the_title_holds_outranks_what_the_journal_mentions() {
        let mut named = task("presupuesto del condominio");
        named.description = Some("nada".into());
        let mut aside = task("llamar a la administradora");
        aside.description = Some("preguntar por el presupuesto".into());

        assert_eq!(asked(&named, "presupuesto"), Some(Hit::Named));
        assert_eq!(asked(&aside, "presupuesto"), Some(Hit::Mentioned));
    }

    #[test]
    fn repeating_keeps_only_what_comes_back() {
        let f = filter(|f| f.repeating = true);
        let mut habit = dated("regar", "2026-08-20");
        habit.repeat = Some(crate::model::Repeat::due(crate::model::Cadence {
            every: 1,
            unit: crate::model::Unit::Week,
        }));

        assert!(f.matches(&habit, today()));
        assert!(!f.matches(&dated("una vez", "2026-08-20"), today()));
        assert!(!f.matches(&task("sin fecha"), today()));
    }

    #[test]
    fn repeating_says_nothing_about_the_day() {
        let f = filter(|f| f.repeating = true);
        let mut far = dated("anual", "2027-01-01");
        far.repeat = Some(crate::model::Repeat::due(crate::model::Cadence {
            every: 1,
            unit: crate::model::Unit::Year,
        }));

        assert!(f.matches(&far, today()));
    }

    #[test]
    fn undated_is_everything_waiting_for_no_day() {
        let f = filter(|f| f.window = Some(Window::Undated));

        assert!(f.matches(&task("no date"), today()));
        assert!(!f.matches(&dated("today", "2026-08-05"), today()));
        assert!(!f.matches(&dated("overdue", "2026-08-01"), today()));
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
    fn a_deadline_that_passed_is_overdue_whatever_day_you_meant_to_do_it() {
        let f = filter(|f| f.window = Some(Window::Overdue));

        assert!(
            f.matches(&due("the paperwork", "2026-08-04"), today()),
            "a limit that ran out yesterday is the whole point of having one"
        );
        assert!(!f.matches(&due("still has time", "2026-08-06"), today()));

        let mut planned_later = due("filed late on purpose", "2026-08-04");
        planned_later.date = Some(DateSpec::all_day("2026-08-09".parse().unwrap(), "UTC"));
        assert!(
            f.matches(&planned_later, today()),
            "meaning to get to it next week does not un-expire the limit"
        );
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

    #[test]
    fn a_layer_filter_keeps_only_what_reads_that_way() {
        let mut errand = task("buy bread");
        errand.status = Status::Done;
        errand.retally();

        let mut told = task("renew the certificate");
        told.status = Status::Done;
        told.description = Some("the authority took nine days to issue it".into());
        told.retally();

        let mut turn = task("water the plants");
        turn.status = Status::Done;
        turn.after = Some(Ulid::generate());
        turn.retally();

        let stories = filter(|f| {
            f.scope = Scope::Archived;
            f.reading = Some(Reading::Story);
        });
        let traces = filter(|f| {
            f.scope = Scope::Archived;
            f.reading = Some(Reading::Trace);
        });

        assert!(stories.matches(&told, today()));
        assert!(!stories.matches(&errand, today()));
        assert!(!stories.matches(&turn, today()));
        assert!(traces.matches(&errand, today()));
    }
}
