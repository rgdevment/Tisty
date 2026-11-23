use serde_json::{Map, Value, json};
use tisty_core::State;
use tisty_core::model::Task;
use tisty_core::view::Scope;

use super::super::looking::counted_tags;
use super::super::pressing::{pressed, pressing_among, tallied};

const TAGS_NAMED: usize = 10;

pub(super) struct Tally {
    open: usize,
    archived: usize,
    said: usize,
    said_left_out: usize,
    standings: Vec<(String, usize)>,
}

impl Tally {
    pub(super) fn of(tasks: &[&Task], said_left_out: usize, today: jiff::civil::Date) -> Self {
        let open = tasks.iter().filter(|one| one.is_open()).count();
        Self {
            open,
            archived: tasks.len() - open,
            said: tasks
                .iter()
                .filter(|one| one.is_open() && one.resolved.is_some())
                .count(),
            said_left_out,
            standings: tallied(&pressing_among(tasks.iter().copied(), today)),
        }
    }

    pub(super) fn worth_saying(&self, shown: usize) -> bool {
        self.said_left_out > 0 || self.open > 0 && (self.archived > 0 || self.open > shown)
    }

    pub(super) fn counts(&self) -> Value {
        let mut counts = Map::new();
        counts.insert("open".into(), json!(self.open));
        if self.archived > 0 {
            counts.insert("archived".into(), json!(self.archived));
        }
        for (how, many) in &self.standings {
            counts.insert(how.clone(), json!(many));
        }
        if self.said > 0 {
            counts.insert("to_confirm".into(), json!(self.said));
        }
        if self.said_left_out > 0 {
            counts.insert("to_confirm_left_out".into(), json!(self.said_left_out));
        }
        Value::Object(counts)
    }

    pub(super) fn told(&self) -> String {
        let archived = match self.archived {
            0 => String::new(),
            many => format!(", and {many} archived, which `scope` «open» leaves out"),
        };
        let said = match (self.said, self.said_left_out) {
            (0, 0) => String::new(),
            (0, many) => {
                format!("; {many} more an agent said done or not doing, left out by `said_done`")
            }
            (many, _) => format!("; {many} of them an agent said done or not doing"),
        };
        format!(
            "In all: {} open{}{said}{archived}.",
            self.open,
            pressed(&self.standings)
        )
    }
}

pub(super) fn what_to_try(
    state: &State,
    scope: Scope,
    beyond: Option<usize>,
    tag: Option<&str>,
) -> Option<String> {
    if scope != Scope::Either && beyond != Some(0) {
        let brings = beyond.map_or("searches the rest too".into(), |many| {
            format!("brings {many}")
        });
        return Some(format!("Nothing in that `scope`: «either» {brings}."));
    }
    let mut tags = counted_tags(state);
    let tag = tag.filter(|tag| !tags.iter().any(|(one, _)| one.to_lowercase() == **tag))?;
    let shared = |one: &str| {
        one.chars()
            .zip(tag.chars())
            .take_while(|(a, b)| a == b)
            .count()
    };
    tags.sort_by_key(|(one, _)| std::cmp::Reverse(shared(one)));
    let closest: Vec<String> = tags
        .iter()
        .take(TAGS_NAMED)
        .map(|(one, _)| format!("#{one}"))
        .collect();
    Some(format!(
        "Nothing carries #{tag}. These exist, the closest first: {}.",
        closest.join(", ")
    ))
}
