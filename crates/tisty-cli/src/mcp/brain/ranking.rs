use std::collections::{HashMap, HashSet};

use tisty_core::docs::Sighting;
use tisty_core::model::{Task, TaskId};
use tisty_core::view::Scope;
use tisty_core::{Paths, State};

use super::project::last_moved;

pub(super) struct Ranked<'a> {
    pub(super) tasks: Vec<&'a Task>,
    pub(super) lined: HashMap<TaskId, String>,
    pub(super) papers: Vec<Sighting>,
    pub(super) every_word: bool,
}

struct Held {
    sighting: Sighting,
    found: usize,
    named: usize,
}

pub(super) fn ranked<'a>(
    paths: &Paths,
    state: &'a State,
    query: &str,
    scope: Scope,
    keeps: &dyn Fn(&Task) -> bool,
    paper_kept: &dyn Fn(&str) -> bool,
) -> Ranked<'a> {
    let terms = tisty_core::text::terms(query);
    let whole = gathered(
        paths,
        state,
        &[(query, terms.len())],
        scope,
        keeps,
        paper_kept,
        false,
    );
    let none = whole.0.is_empty() && whole.2.is_empty();
    let (found, lined, papers) = match none && terms.len() > 1 {
        true => {
            let each: Vec<(&str, usize)> = terms.iter().map(|one| (one.as_str(), 1)).collect();
            nearest(gathered(
                paths, state, &each, scope, keeps, paper_kept, true,
            ))
        }
        false => whole,
    };
    let every_word = found.is_empty() && papers.is_empty() || !(none && terms.len() > 1);
    let mut tasks: Vec<(&Task, usize)> = found.into_values().collect();
    tasks.sort_by_cached_key(|(task, found)| {
        let named = terms.len() - tisty_core::view::unnamed(task, &terms).len();
        (
            std::cmp::Reverse(*found),
            std::cmp::Reverse(named),
            std::cmp::Reverse(last_moved(task)),
            task.id,
        )
    });
    let written: HashMap<&str, Option<jiff::Timestamp>> = state
        .docs
        .values()
        .map(|kept| (kept.file.as_str(), kept.wrote))
        .collect();
    let mut papers: Vec<Held> = papers.into_values().collect();
    papers.sort_by_cached_key(|one| {
        let wrote = written.get(one.sighting.id.as_str()).copied().flatten();
        (
            std::cmp::Reverse(one.found),
            std::cmp::Reverse(one.named),
            std::cmp::Reverse(wrote),
            one.sighting.id.clone(),
        )
    });
    Ranked {
        tasks: tasks.into_iter().map(|(task, _)| task).collect(),
        lined,
        papers: papers.into_iter().map(|one| one.sighting).collect(),
        every_word,
    }
}

// One common word alone would bring half the store, so only what comes within a word of the best stays.
fn nearest(gathered: Gathered<'_>) -> Gathered<'_> {
    let (mut tasks, lined, mut papers) = gathered;
    let best = tasks
        .values()
        .map(|(_, found)| *found)
        .chain(papers.values().map(|one| one.found))
        .max()
        .unwrap_or(0);
    let least = best.saturating_sub(1).max(1);
    tasks.retain(|_, (_, found)| *found >= least);
    papers.retain(|_, one| one.found >= least);
    (tasks, lined, papers)
}

type Gathered<'a> = (
    HashMap<TaskId, (&'a Task, usize)>,
    HashMap<TaskId, String>,
    HashMap<String, Held>,
);

fn gathered<'a>(
    paths: &Paths,
    state: &'a State,
    asks: &[(&str, usize)],
    scope: Scope,
    keeps: &dyn Fn(&Task) -> bool,
    paper_kept: &dyn Fn(&str) -> bool,
    by_word: bool,
) -> Gathered<'a> {
    let mut tasks: HashMap<TaskId, (&Task, usize)> = HashMap::new();
    let mut lined = HashMap::new();
    let mut papers: HashMap<String, Held> = HashMap::new();
    let behind = tisty_core::task_docs::behind(state);
    if by_word {
        let words: Vec<String> = asks.iter().map(|(ask, _)| (*ask).to_string()).collect();
        for task in state.tasks.values().filter(|task| keeps(task)) {
            let held = tisty_core::view::terms_held(task, &words);
            if held > 0 {
                tasks.insert(task.id, (task, held));
            }
        }
    }
    let seeded: HashSet<TaskId> = tasks.keys().copied().collect();
    for (ask, worth) in asks {
        let mut counted: HashSet<TaskId> = HashSet::new();
        if !by_word {
            for task in state.searching(ask, scope, usize::MAX).0 {
                if keeps(task) && counted.insert(task.id) {
                    tasks.entry(task.id).or_insert((task, 0)).1 += worth;
                }
            }
        }
        for one in super::search::sightings(paths, ask) {
            if let Some(id) = behind.get(&one.id) {
                let Some(task) = state.tasks.get(id).filter(|task| keeps(task)) else {
                    continue;
                };
                if !seeded.contains(id) && counted.insert(*id) {
                    tasks.entry(*id).or_insert((task, 0)).1 += worth;
                    lined.entry(*id).or_insert_with(|| one.line.clone());
                }
                continue;
            }
            if !paper_kept(&one.id) {
                continue;
            }
            let named = if one.line.is_empty() { *worth } else { 0 };
            let held = papers.entry(one.id.clone()).or_insert_with(|| Held {
                sighting: one.clone(),
                found: 0,
                named: 0,
            });
            held.found += worth;
            held.named += named;
            if held.sighting.line.is_empty() {
                held.sighting.line = one.line;
            }
        }
    }
    (tasks, lined, papers)
}

// The newest note that still holds, so a decision taken back is never quoted as standing.
pub(super) fn newest_line(task: &Task, terms: &[String]) -> Option<(&'static str, String)> {
    let unnamed = tisty_core::view::unnamed(task, terms);
    if unnamed.is_empty() {
        return None;
    }
    let replaced = task.replaced();
    let mut notes: Vec<&tisty_core::model::LogEntry> = task
        .log
        .iter()
        .filter(|one| !replaced.contains(&one.id))
        .collect();
    notes.sort_by_key(|one| std::cmp::Reverse(one.at));
    let said = notes
        .into_iter()
        .map(|one| ("journal", one.body.as_str()))
        .chain(task.description.as_deref().map(|one| ("description", one)))
        .chain(task.steps.iter().map(|one| ("step", one.text.as_str())));
    let mut best: Option<(usize, &'static str, &str)> = None;
    for (kind, text) in said {
        for line in text.lines().map(str::trim) {
            let saying = counted(line, &unnamed);
            if saying == unnamed.len() {
                return Some((kind, line.to_string()));
            }
            if saying > best.map_or(0, |(most, _, _)| most) {
                best = Some((saying, kind, line));
            }
        }
    }
    best.map(|(_, kind, line)| (kind, line.to_string()))
}

// A log grows at its end, so the last line saying the most is the newest word on it.
pub(super) fn newest_paper_line(paths: &Paths, file: &str, terms: &[String]) -> Option<String> {
    let body = tisty_core::docs::read(&paths.docs(), file).ok()?;
    let mut best: Option<(usize, bool, &str)> = None;
    for line in prose(&body) {
        let saying = counted(line, terms);
        if saying == 0 {
            continue;
        }
        let standing = !line.contains("~~");
        let better = best.is_none_or(|(most, stood, _)| (saying, standing) >= (most, stood));
        if better {
            best = Some((saying, standing, line));
        }
    }
    best.map(|(_, _, line)| tisty_core::docs::bare(line))
}

fn prose(body: &str) -> Vec<&str> {
    let mut lines = body.lines().peekable();
    if lines.peek().is_some_and(|one| one.trim_end() == "---") {
        lines.next();
        for one in lines.by_ref() {
            if one.trim_end() == "---" {
                break;
            }
        }
    }
    let mut fenced = false;
    lines
        .filter(|one| {
            let said = one.trim_start();
            if said.starts_with("```") {
                fenced = !fenced;
                return false;
            }
            !fenced && !said.starts_with('#')
        })
        .collect()
}

fn counted(line: &str, terms: &[String]) -> usize {
    let flat = tisty_core::text::folded(line);
    terms
        .iter()
        .filter(|term| flat.contains(term.as_str()))
        .count()
}
