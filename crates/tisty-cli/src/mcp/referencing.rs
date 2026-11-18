use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use serde_json::{Value, json};
use tisty_core::model::TaskId;
use tisty_core::{Paths, State};
use tisty_markdown::named::{Named, spots};
use tisty_markdown::refs::{DOC, TASK, labelled, pictures};

use super::catalogue::Effect;
use super::jsonrpc::{with_ask, with_said};

const LINKED_SHOWN: usize = 10;
const MAYBE_SHOWN: usize = 5;
const COULD_BE_SHOWN: usize = 3;
const SAID_SHOWN: usize = 3;
const NESTED: [&str; 2] = ["tasks", "parts"];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Target {
    task: bool,
    title: String,
    id: String,
}

impl Target {
    fn link(&self, words: &str, card: bool) -> String {
        let scheme = if self.task { TASK } else { DOC };
        let bang = if card { "!" } else { "" };
        format!("{bang}[{}]({scheme}{})", labelled(words), self.id)
    }

    fn json(&self) -> Value {
        let key = if self.task { "task" } else { "doc" };
        json!({ key: self.id, "title": self.title })
    }

    fn said(&self) -> String {
        let what = if self.task { "task" } else { "document" };
        format!("the {what} {:?} [{}]", self.title, self.id)
    }
}

#[derive(Default)]
pub(super) struct Known {
    ids: BTreeMap<String, Target>,
    titles: BTreeMap<String, Vec<Target>>,
    loosely: BTreeMap<String, Vec<Target>>,
}

impl Known {
    pub(super) fn of(state: &State) -> Self {
        let tasks = state
            .tasks
            .values()
            .filter(|task| !task.folded())
            .map(|task| Target {
                task: true,
                title: task.title.clone(),
                id: task.id.to_string(),
            });
        let docs = state
            .docs
            .values()
            .filter(|kept| kept.task_of.is_none())
            .map(|kept| Target {
                task: false,
                title: kept.title.clone().unwrap_or_else(|| kept.file.clone()),
                id: kept.file.clone(),
            });
        let mut known = Self::default();
        for target in tasks.chain(docs) {
            known.add(target);
        }
        known
    }

    fn add(&mut self, target: Target) {
        let title = target.title.trim().to_string();
        if !title.is_empty() {
            self.loosely
                .entry(loose(&title))
                .or_default()
                .push(target.clone());
            self.titles.entry(title).or_default().push(target.clone());
        }
        self.ids.insert(target.id.clone(), target);
    }

    fn by_title(&self, words: &str, only_docs: bool) -> (Vec<&Target>, bool) {
        let exact = kept(self.titles.get(words), only_docs);
        match exact.is_empty() {
            false => (exact, true),
            true => (kept(self.loosely.get(&loose(words)), only_docs), false),
        }
    }
}

fn kept(all: Option<&Vec<Target>>, only_docs: bool) -> Vec<&Target> {
    let mut found: Vec<&Target> = all
        .into_iter()
        .flatten()
        .filter(|one| !(only_docs && one.task))
        .collect();
    found.sort();
    found
}

fn loose(words: &str) -> String {
    tisty_core::text::folded(words)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct Heard {
    linked: Vec<(String, Target)>,
    maybe: Vec<(String, Vec<Target>)>,
    unshown: Vec<String>,
}

enum Found<'a> {
    Sure(&'a Target),
    Doubt(Vec<&'a Target>),
    Nothing,
}

pub(super) fn linked(
    text: &str,
    known: &Known,
    own: Option<&str>,
    kept: &BTreeSet<&str>,
    heard: &mut Heard,
) -> String {
    let mut done: BTreeSet<String> = heard.linked.iter().map(|(_, one)| one.id.clone()).collect();
    let mut changes: Vec<(Range<usize>, String)> = Vec::new();
    for spot in spots(text) {
        if settled(text, spot.at.start, kept) {
            continue;
        }
        let words = &text[spot.at.clone()];
        let found = match &spot.named {
            Named::Id(id) => known.ids.get(id).map_or(Found::Nothing, Found::Sure),
            Named::Quoted(said) => titled(known, said, false, true),
            Named::Wiki(said) => titled(known, said, true, false),
        };
        match found {
            Found::Sure(target) if own != Some(target.id.as_str()) => {
                let first = done.insert(target.id.clone());
                if !first && matches!(spot.named, Named::Quoted(_)) {
                    continue;
                }
                let link = match &spot.named {
                    Named::Id(_) => {
                        target.link(&target.title, target.task && alone(text, &spot.at))
                    }
                    Named::Quoted(said) | Named::Wiki(said) => target.link(said, false),
                };
                if first {
                    heard.linked.push((words.to_string(), target.clone()));
                }
                changes.push((spot.at.clone(), link));
            }
            Found::Doubt(could_be) => {
                let could_be: Vec<Target> = could_be
                    .into_iter()
                    .filter(|one| own != Some(one.id.as_str()))
                    .cloned()
                    .collect();
                if !could_be.is_empty() && !heard.maybe.iter().any(|(had, _)| had == words) {
                    heard.maybe.push((words.to_string(), could_be));
                }
            }
            _ => {}
        }
    }
    unshown_pictures(text, kept, &mut heard.unshown);
    let mut said = text.to_string();
    for (at, link) in changes.into_iter().rev() {
        said.replace_range(at, &link);
    }
    said
}

fn unshown_pictures(text: &str, kept: &BTreeSet<&str>, unshown: &mut Vec<String>) {
    for (target, label) in pictures(text) {
        let Some(leaf) = tisty_core::attach::leaf_of(&target) else {
            continue;
        };
        let new = text
            .lines()
            .any(|line| line.contains(&target) && !kept.contains(line.trim()));
        let unsaid =
            tisty_core::attach::pictured(leaf) && tisty_core::attach::shows(&label, leaf).is_none();
        if new && unsaid && !unshown.iter().any(|one| one == leaf) {
            unshown.push(leaf.to_string());
        }
    }
}

fn titled<'a>(known: &'a Known, said: &str, only_docs: bool, quoted: bool) -> Found<'a> {
    let said = said.trim();
    let (found, exact) = known.by_title(said, only_docs);
    let one_word = !said.contains(char::is_whitespace);
    match found.as_slice() {
        [] if only_docs => match known.by_title(said, false).0 {
            tasks if tasks.is_empty() => Found::Nothing,
            tasks => Found::Doubt(tasks),
        },
        [] => Found::Nothing,
        [one] if exact && !(quoted && one_word) => Found::Sure(one),
        _ => Found::Doubt(found),
    }
}

fn line_around(text: &str, at: &Range<usize>) -> Range<usize> {
    let start = text[..at.start].rfind('\n').map_or(0, |n| n + 1);
    let end = text[at.end..].find('\n').map_or(text.len(), |n| at.end + n);
    start..end
}

// A heading names a section and a table cell its row, so linking either would move what points at them.
fn settled(text: &str, at: usize, kept: &BTreeSet<&str>) -> bool {
    let around = line_around(text, &(at..at));
    let line = text[around.clone()].trim();
    let next = text[around.end..].lines().nth(1).unwrap_or_default().trim();
    let underlined = !next.is_empty()
        && (next.chars().all(|c| c == '=') || next.len() >= 2 && next.chars().all(|c| c == '-'));
    let hashes = line.trim_start_matches('#');
    let heading =
        hashes.len() < line.len() && (hashes.is_empty() || hashes.starts_with([' ', '\t']));
    kept.contains(line)
        || underlined
        || heading
        || tisty_markdown::idea_words(line).is_some()
        || in_a_table(text, &around)
}

fn in_a_table(text: &str, line: &Range<usize>) -> bool {
    let delimits = |one: &str| {
        let one = one.trim();
        one.contains('-')
            && one.contains('|')
            && one
                .chars()
                .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'))
    };
    let above = text[..line.start]
        .lines()
        .rev()
        .take_while(|one| !one.trim().is_empty());
    let below = text[line.end..]
        .lines()
        .skip(1)
        .take_while(|one| !one.trim().is_empty());
    let said = &text[line.clone()];
    said.trim_start().starts_with('|')
        || said.contains('|') && (above.chain(below).any(delimits) || delimits(said))
}

fn alone(text: &str, at: &Range<usize>) -> bool {
    let around = line_around(text, at);
    let blank = |line: Option<&str>| line.is_none_or(|one| one.trim().is_empty());
    text[around.clone()].trim() == &text[at.clone()]
        && blank(text[..around.start].lines().next_back())
        && blank(text[around.end..].lines().nth(1))
}

impl Heard {
    pub(super) fn told(self, mut answer: Value) -> Value {
        if let Some(references) = self.references() {
            with_said(&mut answer, "references", references);
        }
        if let Some(ask) = self.ask() {
            with_ask(&mut answer, ask);
        }
        answer
    }

    fn references(&self) -> Option<Value> {
        if self.linked.is_empty() && self.maybe.is_empty() {
            return None;
        }
        let mut sentences = Vec::new();
        let mut shown = json!({});
        if !self.linked.is_empty() {
            let named: Vec<String> = self
                .linked
                .iter()
                .take(SAID_SHOWN)
                .map(|(said, target)| format!("{said:?} to {}", target.said()))
                .collect();
            sentences.push(format!(
                "Tisty linked what this names by id or exact title: {}{}.",
                named.join("; "),
                more(self.linked.len(), SAID_SHOWN)
            ));
            let linked: Vec<Value> = self
                .linked
                .iter()
                .take(LINKED_SHOWN)
                .map(|(said, target)| with_words(target.json(), said))
                .collect();
            shown["linked"] = json!(linked);
            if self.linked.len() > LINKED_SHOWN {
                shown["linked_more"] = json!(self.linked.len() - LINKED_SHOWN);
            }
        }
        if !self.maybe.is_empty() {
            let doubts: Vec<String> = self
                .maybe
                .iter()
                .take(SAID_SHOWN)
                .map(|(said, could_be)| {
                    let named: Vec<String> = could_be
                        .iter()
                        .take(COULD_BE_SHOWN)
                        .map(Target::said)
                        .collect();
                    format!(
                        "{said:?} could be {}{}",
                        named.join(" or "),
                        more(could_be.len(), COULD_BE_SHOWN)
                    )
                })
                .collect();
            sentences.push(format!(
                "Not linked, for you to judge: {}{}. If you meant one, link it as \
                 [its words](tisty:task/<id>) or [its words](tisty:doc/<id>).",
                doubts.join("; "),
                more(self.maybe.len(), SAID_SHOWN)
            ));
            let maybe: Vec<Value> = self
                .maybe
                .iter()
                .take(MAYBE_SHOWN)
                .map(|(said, could_be)| {
                    let could_be: Vec<Value> = could_be
                        .iter()
                        .take(COULD_BE_SHOWN)
                        .map(Target::json)
                        .collect();
                    json!({ "said": said, "could_be": could_be })
                })
                .collect();
            shown["maybe"] = json!(maybe);
        }
        shown["said"] = json!(sentences.join(" "));
        Some(shown)
    }

    fn ask(&self) -> Option<Value> {
        let first = self.unshown.first()?;
        Some(pictures_asked(
            &self.unshown,
            &format!("write what it shows between the brackets, as ![What it shows]({first})"),
        ))
    }
}

pub(super) fn pictures_asked(unshown: &[String], fix: &str) -> Value {
    let named: Vec<String> = unshown
        .iter()
        .take(SAID_SHOWN)
        .map(|one| format!("{one:?}"))
        .collect();
    let said = format!(
        "What does {}{} show? The words in a picture's brackets are all an agent or a screen \
         reader knows of it, and a file's name says nothing: {fix}.",
        named.join(", "),
        more(unshown.len(), SAID_SHOWN)
    );
    let pictures: Vec<&String> = unshown.iter().take(MAYBE_SHOWN).collect();
    json!({ "pictures": pictures, "said": said })
}

fn with_words(mut target: Value, said: &str) -> Value {
    target["said"] = json!(said);
    target
}

fn more(all: usize, shown: usize) -> String {
    match all.saturating_sub(shown) {
        0 => String::new(),
        left => format!(" and {left} more"),
    }
}

pub(super) fn link_all(paths: &Paths, tool: &str, args: &mut Value, field: &str) -> Heard {
    let mut heard = Heard::default();
    if !names_anything(args, field) {
        return heard;
    }
    let Some(mut state) = seen(paths, false) else {
        return heard;
    };
    let rewrites = !only_adds(tool);
    if rewrites && bodies_needed(&state, args) {
        let Some(whole) = seen(paths, true) else {
            return heard;
        };
        state = whole;
    }
    let known = Known::of(&state);
    let own = own_of(&state, args);
    let mut before = match rewrites {
        true => current(paths, &state, args),
        false => Vec::new(),
    };
    before.extend(title_line(args, field));
    let kept: BTreeSet<&str> = before
        .iter()
        .flat_map(|one| one.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    put_back(args, field, &mut |text| {
        linked(text, &known, own.as_deref(), &kept, &mut heard)
    });
    heard
}

// A tool that only adds writes no line that was there before, so it has nothing of the person's to keep.
fn only_adds(tool: &str) -> bool {
    super::catalogue::EFFECTS
        .iter()
        .any(|(name, effect)| *name == tool && matches!(effect, Effect::Adds | Effect::Sets))
}

fn names_anything(value: &Value, field: &str) -> bool {
    let here = value
        .get(field)
        .and_then(Value::as_str)
        .is_some_and(|text| !spots(text).is_empty() || !pictures(text).is_empty());
    here || NESTED.iter().any(|nested| {
        value
            .get(nested)
            .and_then(Value::as_array)
            .is_some_and(|all| all.iter().any(|one| names_anything(one, field)))
    })
}

// A whole document's first line is its title even without `#`, and a link there would be in its name.
fn title_line(args: &Value, field: &str) -> Option<String> {
    let whole = field == "body"
        && args.get("task").is_none()
        && (args.get("doc").is_none() || args.get("print").is_some());
    let body = args.get(field).and_then(Value::as_str).filter(|_| whole)?;
    let front = tisty_markdown::front::span(body).map_or(0, |one| one.end);
    body[front..]
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::to_string)
}

fn put_back(value: &mut Value, field: &str, link: &mut impl FnMut(&str) -> String) {
    if let Some(said) = value.get(field).and_then(Value::as_str) {
        value[field] = Value::String(link(said));
    }
    for nested in NESTED {
        if let Some(Value::Array(all)) = value.get_mut(nested) {
            all.iter_mut().for_each(|one| put_back(one, field, link));
        }
    }
}

// A task's page holds its description, so only a note reworded or a task with no page needs its body.
fn bodies_needed(state: &State, args: &Value) -> bool {
    let task = args
        .get("task")
        .and_then(Value::as_str)
        .and_then(named_task);
    task.is_some_and(|id| {
        args.get("note").is_some() || tisty_core::task_docs::of(state, id).is_none()
    })
}

fn seen(paths: &Paths, bodies: bool) -> Option<State> {
    let projected = match bodies {
        true => tisty_core::cache::project(&paths.store(), paths.cache()),
        false => tisty_core::cache::summarised(&paths.store(), paths.cache()),
    };
    let mut state = projected.ok()?;
    tisty_core::diary::out_of_sight(&paths.docs(), &mut state);
    Some(state)
}

fn named_task(said: &str) -> Option<TaskId> {
    let said = said.trim();
    said.parse()
        .ok()
        .or_else(|| tisty_core::task_docs::task_of_file(said))
}

fn own_of(state: &State, args: &Value) -> Option<String> {
    if let Some(task) = args
        .get("task")
        .and_then(Value::as_str)
        .and_then(named_task)
    {
        return Some(task.to_string());
    }
    let doc = args.get("doc").and_then(Value::as_str)?.trim();
    match named_task(doc) {
        Some(task) => Some(task.to_string()),
        None => state
            .docs
            .values()
            .any(|kept| kept.file == doc)
            .then(|| doc.to_string()),
    }
}

fn current(paths: &Paths, state: &State, args: &Value) -> Vec<String> {
    let task = args
        .get("task")
        .or_else(|| args.get("doc"))
        .and_then(Value::as_str)
        .and_then(named_task)
        .and_then(|id| state.tasks.get(&id));
    let page = match task {
        Some(task) => tisty_core::task_docs::of(state, task.id).map(|kept| kept.file.clone()),
        None => args
            .get("doc")
            .and_then(Value::as_str)
            .map(|one| one.trim().to_string()),
    };
    let mut said: Vec<String> = page
        .and_then(|file| tisty_core::docs::read(&paths.docs(), &file).ok())
        .into_iter()
        .collect();
    if let Some(task) = task {
        said.extend(task.description.clone());
        said.extend(task.log.iter().map(|entry| entry.body.clone()));
    }
    said
}

#[cfg(test)]
#[path = "referencing_test.rs"]
mod tests;
