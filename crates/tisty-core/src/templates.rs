use std::io::Read;
use std::path::Path;

use crate::model::Task;
use crate::state::State;

const KEY: &str = "template";
const FOR: &str = "template_for";
pub const EVERY_TASK: &str = "tasks";
pub const EVERY_ROUTINE: &str = "routines";
// The front matter sits at the top, so a template is told without reading a long body whole.
const HEAD_AT_MOST: u64 = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Tasks,
    Docs,
    Both,
}

impl Scope {
    pub fn takes_tasks(self) -> bool {
        self != Self::Docs
    }

    pub fn takes_docs(self) -> bool {
        self != Self::Tasks
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Template {
    pub file: String,
    pub title: Option<String>,
    pub scope: Scope,
    pub r#for: Option<String>,
}

pub fn told(text: &str) -> Option<(Scope, Option<String>)> {
    let fields = tisty_markdown::front::fields(text);
    let said = |key: &str| {
        fields
            .iter()
            .find(|one| one.key.eq_ignore_ascii_case(key))
            .and_then(|one| one.values.first())
            .map(|one| one.trim().to_lowercase())
    };
    let scope = match said(KEY)?.as_str() {
        "tasks" => Scope::Tasks,
        "docs" => Scope::Docs,
        "both" => Scope::Both,
        _ => return None,
    };
    let r#for = said(FOR)
        .map(|one| one.trim_start_matches('@').to_string())
        .filter(|one| !one.is_empty());
    Some((scope, r#for))
}

pub fn body(text: &str) -> String {
    let rest = match tisty_markdown::front::span(text) {
        Some(at) => &text[at.end..],
        None => text,
    };
    rest.replace("\r\n", "\n")
        .trim_start_matches('\n')
        .to_string()
}

fn head(root: &Path, file: &str) -> Option<String> {
    let at = crate::docs::resolve(root, file).ok()?;
    let mut read = Vec::new();
    std::fs::File::open(at)
        .ok()?
        .take(HEAD_AT_MOST)
        .read_to_end(&mut read)
        .ok()?;
    // The cut may fall inside a letter, and only the front matter at the top is wanted.
    Some(String::from_utf8_lossy(&read).into_owned())
}

pub fn found(root: &Path, state: &State) -> Vec<Template> {
    let mut out: Vec<Template> = state
        .docs
        .values()
        .filter(|one| one.task_of.is_none() && !state.held_away(one))
        .filter_map(|one| {
            let (scope, r#for) = told(&head(root, &one.file)?)?;
            Some(Template {
                file: one.file.clone(),
                title: one.title.clone(),
                scope,
                r#for,
            })
        })
        .collect();
    out.sort_by(|a, b| a.title.cmp(&b.title).then(a.file.cmp(&b.file)));
    out
}

pub fn picked<'a>(all: &'a [Template], state: &State, task: &Task) -> Option<&'a Template> {
    let list = task
        .list
        .and_then(|id| state.lists.get(&id))
        .map(|one| one.name.to_lowercase())
        .filter(|name| name != EVERY_TASK && name != EVERY_ROUTINE);
    let routine = task.repeat.is_some() || task.after.is_some();
    let named = |name: &str| {
        all.iter()
            .find(|one| one.scope.takes_tasks() && one.r#for.as_deref() == Some(name))
    };
    list.as_deref()
        .and_then(named)
        .or_else(|| routine.then(|| named(EVERY_ROUTINE)).flatten())
        .or_else(|| named(EVERY_TASK))
}

#[cfg(test)]
#[path = "templates_test.rs"]
mod tests;
