use tisty_core::view::{Filter, Scope, Window};
use tisty_core::{Reading, Tag, Task};

use crate::{Refusal, answers, today, zone};

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct View {
    #[serde(default)]
    pub(crate) archive: bool,
    #[serde(default)]
    pub(crate) everything: bool,
    #[serde(default)]
    pub(crate) inbox: bool,
    #[serde(default)]
    pub(crate) list: Option<String>,
    #[serde(default)]
    pub(crate) lists: Vec<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) tagged: bool,
    #[serde(default)]
    pub(crate) hidden: bool,
    #[serde(default)]
    pub(crate) window: Option<String>,
    #[serde(default)]
    pub(crate) repeating: bool,
    #[serde(default)]
    pub(crate) most: Option<usize>,
    #[serde(default)]
    pub(crate) reading: Option<String>,
}

pub(crate) fn ahead(
    spec: &tisty_core::DateSpec,
    now: &jiff::Zoned,
    code: &'static str,
) -> Result<(), Refusal> {
    let passed = if spec.has_time {
        spec.at < now.datetime()
    } else {
        spec.date() < now.date()
    };
    if passed {
        return Err(Refusal::of(code));
    }
    Ok(())
}

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Change {
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) date: Option<String>,
    #[serde(default)]
    pub(crate) no_date: bool,
    #[serde(default)]
    pub(crate) deadline: Option<String>,
    #[serde(default)]
    pub(crate) no_deadline: bool,
    #[serde(default)]
    pub(crate) priority: Option<String>,
    #[serde(default)]
    pub(crate) add_tag: Option<String>,
    #[serde(default)]
    pub(crate) untag: Option<String>,
    #[serde(default)]
    pub(crate) list: Option<String>,
    #[serde(default)]
    pub(crate) list_named: Option<String>,
    #[serde(default)]
    pub(crate) inbox: bool,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) remind: Option<String>,
    #[serde(default)]
    pub(crate) unremind: Option<String>,
    #[serde(default)]
    pub(crate) repeat: Option<tisty_core::model::Repeat>,
    #[serde(default)]
    pub(crate) no_repeat: bool,
}

pub(crate) fn tagged(task: &Task, change: &Change) -> Result<Option<Vec<Tag>>, Refusal> {
    if change.add_tag.is_none() && change.untag.is_none() {
        return Ok(None);
    }
    let mut tags = task.tags.clone();
    if let Some(name) = &change.untag {
        let gone = Tag::new(name).map_err(|_| Refusal::about("badTag", name))?;
        tags.retain(|kept| *kept != gone);
    }
    if let Some(name) = &change.add_tag {
        let one = Tag::written(name).map_err(|_| Refusal::about("badTag", name))?;
        if !tags.contains(&one) {
            tags.push(one);
        }
    }
    Ok(Some(tags))
}

pub(crate) fn repeated(
    change: &Change,
    now: &jiff::Zoned,
) -> Result<Option<Option<tisty_core::model::Repeat>>, Refusal> {
    if change.no_repeat {
        return Ok(Some(None));
    }
    let Some(over) = change.repeat else {
        return Ok(None);
    };
    let every = over.cadence().every;
    if every == 0 || every > 999 {
        return Err(Refusal::of("notACadence"));
    }
    if over.ended(now.date()) {
        return Err(Refusal::of("pastEnd"));
    }
    Ok(Some(Some(over)))
}

pub(crate) fn recalled(
    task: &Task,
    change: &Change,
    now: &jiff::Zoned,
) -> Result<Option<Vec<tisty_core::DateSpec>>, Refusal> {
    if change.remind.is_none() && change.unremind.is_none() {
        return Ok(None);
    }
    let civil = |raw: &String| {
        raw.parse::<jiff::civil::DateTime>()
            .map_err(|_| Refusal::about("notADate", raw))
    };
    let mut at = task.reminders.clone();
    if let Some(raw) = &change.unremind {
        let gone = civil(raw)?;
        at.retain(|kept| kept.at != gone);
    }
    if let Some(raw) = &change.remind {
        let when = civil(raw)?;
        if when < now.datetime() {
            return Err(Refusal::of("pastReminder"));
        }
        if !at.iter().any(|kept| kept.at == when) {
            at.push(tisty_core::DateSpec::floating(when, zone()));
        }
    }
    at.sort_by_key(|one| one.at);
    Ok(Some(at))
}

pub(crate) fn dated_field(
    raw: Option<&str>,
    cleared: bool,
    now: &jiff::Zoned,
    spoken: &str,
) -> Result<Option<Option<tisty_core::DateSpec>>, Refusal> {
    match (raw, cleared) {
        (Some(raw), _) => Ok(Some(Some(answers::tasks::dated(raw, now, spoken)?))),
        (None, true) => Ok(Some(None)),
        _ => Ok(None),
    }
}

impl View {
    pub(crate) fn resolve(self) -> Result<Filter, Refusal> {
        Ok(Filter {
            scope: match (self.everything, self.archive) {
                (true, _) => Scope::Either,
                (_, true) => Scope::Archived,
                _ => Scope::Open,
            },
            inbox: self.inbox,
            lists: self
                .list
                .into_iter()
                .chain(self.lists)
                .map(|id| id.parse().map_err(|_| Refusal::of("notAListId")))
                .collect::<Result<_, _>>()?,
            tags: self
                .tags
                .iter()
                .map(|t| Tag::new(t).map_err(|_| Refusal::about("badTag", t)))
                .collect::<Result<_, _>>()?,
            tagged: self.tagged,
            hidden: self.hidden,
            priority: None,
            repeating: self.repeating,
            reading: match self.reading.as_deref() {
                Some("story") => Some(Reading::Story),
                Some("routine") => Some(Reading::Routine),
                Some("trace") => Some(Reading::Trace),
                _ => None,
            },
            window: match self.window.as_deref() {
                Some("today") => Some(Window::Today),
                Some("upcoming") => Some(Window::After(today())),
                Some("overdue") => Some(Window::Overdue),
                Some("undated") => Some(Window::Undated),
                _ => None,
            },
        })
    }
}
