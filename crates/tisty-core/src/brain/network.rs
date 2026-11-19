use std::collections::{HashMap, HashSet};

use jiff::Timestamp;
use serde::Serialize;

use super::Brain;
use crate::witness::{self, Fact, channel};
use crate::{Paths, State, Task};

// A tie travels as its place in this list, so the window reads the numbers in this order.
pub const TIES: [&str; 5] = ["cite", "mention", "structure", "tag", "uses"];
pub const NODES_AT_MOST: usize = 12_000;
pub const EDGES_AT_MOST: usize = 40_000;

const HELD_BY_OTHERS: [&str; 2] = ["tag", "file"];
const UNDER_WAY: [&str; 3] = ["open", "said_done", "said_dropped"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Neuron {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LeftOut {
    pub nodes: usize,
    pub edges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Network {
    pub nodes: Vec<Neuron>,
    pub edges: Vec<(u32, u32, u8)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub near: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_out: Option<LeftOut>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alike: Vec<Guess>,
}

// Both ends, the score in thousandths and the shared words: Tisty's guess, apart from what anyone wrote.
pub type Guess = (u32, u32, u16, String);

type Guessed = (String, String, f64, String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Caps {
    nodes: usize,
    edges: usize,
}

const CAPS: Caps = Caps {
    nodes: NODES_AT_MOST,
    edges: EDGES_AT_MOST,
};

type Held = (Neuron, Option<String>);

impl Brain {
    pub fn network(
        &self,
        shown: impl Fn(&str) -> bool,
        near: &[String],
    ) -> rusqlite::Result<Option<Network>> {
        if self.seen().is_none() {
            return Ok(None);
        }
        // One snapshot, so a round that commits meanwhile never leaves an edge without its node.
        let tx = self.db.unchecked_transaction()?;
        let mut held: Vec<Held> = Vec::new();
        {
            let mut asked =
                tx.prepare("SELECT id, kind, title, grp, state, uid FROM node ORDER BY id")?;
            let rows = asked.query_map([], |row| {
                let neuron = Neuron {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    title: row.get(2)?,
                    group: row.get(3)?,
                    state: row.get(4)?,
                    when: None,
                };
                Ok((neuron, row.get(5)?))
            })?;
            for row in rows {
                let one = row?;
                if shown(&one.0.id) {
                    held.push(one);
                }
            }
        }
        let mut ties: Vec<(String, String, String)> = Vec::new();
        {
            let mut asked = tx.prepare("SELECT source, target, kind FROM edge")?;
            let rows = asked.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            for row in rows {
                ties.push(row?);
            }
        }
        let mut guessed: Vec<Guessed> = Vec::new();
        {
            let mut asked =
                tx.prepare("SELECT a, b, score, shared FROM alike WHERE method = 'words'")?;
            let rows = asked.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;
            for row in rows {
                guessed.push(row?);
            }
        }
        drop(tx);
        let mut network = woven(held, &ties, near, CAPS);
        network.alike = guesses(&network.nodes, &guessed);
        Ok(Some(network))
    }
}

pub fn network(paths: &Paths, shown: impl Fn(&str) -> bool, near: &[String]) -> Option<Network> {
    let brain = Brain::looked_at(paths.cache())?;
    brain
        .network(shown, near)
        .inspect_err(|e| {
            witness::trace(
                channel::BRAIN,
                "the brain could not draw its network",
                &[("why", Fact::Why(e.to_string()))],
            );
        })
        .ok()
        .flatten()
}

pub fn card_of(
    paths: &Paths,
    id: &str,
    today: jiff::civil::Date,
    shown: impl Fn(&str) -> bool,
) -> Option<super::Card> {
    let brain = Brain::looked_at(paths.cache())?;
    brain
        .card(id, today, shown)
        .inspect_err(|e| {
            witness::trace(
                channel::BRAIN,
                "the brain could not read a card",
                &[("why", Fact::Why(e.to_string()))],
            );
        })
        .ok()
        .flatten()
}

pub fn kept_out(state: &State) -> HashSet<String> {
    state
        .tasks
        .values()
        .filter(|task| task.hidden)
        .map(|task| task.id.to_string())
        .collect()
}

impl Network {
    pub fn dated(&mut self, dates: &HashMap<String, String>) {
        for one in &mut self.nodes {
            one.when = dates.get(&one.id).cloned();
        }
    }
}

pub fn dates(state: &State) -> HashMap<String, String> {
    let day = |at: Timestamp| at.to_string().chars().take(10).collect::<String>();
    let docs = state
        .papers()
        .filter_map(|kept| Some((kept.file.clone(), day(kept.wrote.or(kept.made)?))));
    let tasks = state
        .tasks
        .values()
        .map(|task| (task.id.to_string(), day(touched(task))));
    docs.chain(tasks).collect()
}

pub fn touched_last(state: &State) -> Option<String> {
    let docs = state
        .papers()
        .filter_map(|kept| Some((kept.wrote.or(kept.made)?, kept.file.clone())));
    let tasks = state
        .tasks
        .values()
        .filter(|task| !task.hidden)
        .map(|task| (touched(task), task.id.to_string()));
    docs.chain(tasks)
        .max_by_key(|(at, _)| *at)
        .map(|(_, id)| id)
}

fn touched(task: &Task) -> Timestamp {
    let born = i64::try_from(task.id.timestamp_ms())
        .ok()
        .and_then(|ms| Timestamp::from_millisecond(ms).ok())
        .unwrap_or_default();
    task.log
        .iter()
        .map(|entry| entry.at)
        .chain(task.completed_at)
        .chain(task.stated_at)
        .chain(task.steps_moved_at)
        .chain(task.grown_at)
        .fold(born, Timestamp::max)
}

type Line = (usize, usize, u8);

fn woven(
    held: Vec<Held>,
    ties: &[(String, String, String)],
    near: &[String],
    caps: Caps,
) -> Network {
    let at = places(&held);
    let mut edges = lines(&at, ties);
    let kinds: Vec<&str> = held.iter().map(|(one, _)| one.kind.as_str()).collect();
    let mut keep = vec![true; held.len()];
    drop_unheld(&kinds, &edges, &mut keep);
    let near = near
        .iter()
        .find_map(|one| at.get(one.as_str()).copied())
        .filter(|&n| keep[n]);
    let whole = (keep.iter().filter(|one| **one).count(), edges.len());
    if whole.0 > caps.nodes {
        keep = kept_within(&held, &kinds, &edges, &keep, near, caps.nodes);
    }
    edges.retain(|(from, to, _)| keep[*from] && keep[*to]);
    if edges.len() > caps.edges {
        edges.sort_by_key(|(_, _, tie)| tie_rank(*tie));
        edges.truncate(caps.edges);
    }
    drop_unheld(&kinds, &edges, &mut keep);
    let near = near.filter(|n| keep[*n]);
    renumbered(held, &edges, &keep, near, whole)
}

// A node is named by its own id first, so another's document ULID never takes its place.
fn places(held: &[Held]) -> HashMap<&str, usize> {
    let mut at: HashMap<&str, usize> = HashMap::new();
    for (n, (neuron, _)) in held.iter().enumerate() {
        at.insert(neuron.id.as_str(), n);
    }
    for (n, (_, uid)) in held.iter().enumerate() {
        if let Some(uid) = uid {
            at.entry(uid.as_str()).or_insert(n);
        }
    }
    at
}

fn lines(at: &HashMap<&str, usize>, ties: &[(String, String, String)]) -> Vec<Line> {
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for (source, target, kind) in ties {
        let (Some(&from), Some(&to), Some(tie)) = (
            at.get(source.as_str()),
            at.get(target.as_str()),
            TIES.iter().position(|one| one == kind),
        ) else {
            continue;
        };
        let tie = u8::try_from(tie).unwrap_or(u8::MAX);
        if from != to && seen.insert((from, to, tie)) {
            edges.push((from, to, tie));
        }
    }
    edges
}

fn kept_within(
    held: &[Held],
    kinds: &[&str],
    edges: &[Line],
    keep: &[bool],
    near: Option<usize>,
    most: usize,
) -> Vec<bool> {
    let mut beside: Vec<Vec<usize>> = vec![Vec::new(); held.len()];
    for (from, to, _) in edges {
        beside[*from].push(*to);
        beside[*to].push(*from);
    }
    let mut kept = vec![false; held.len()];
    let mut waiting = vec![false; held.len()];
    let mut taken = 0;
    for n in ranked(held, edges, keep, near) {
        if taken == most {
            break;
        }
        if HELD_BY_OTHERS.contains(&kinds[n]) && !beside[n].iter().any(|one| kept[*one]) {
            waiting[n] = true;
            continue;
        }
        kept[n] = true;
        taken += 1;
        for &held_here in &beside[n] {
            if waiting[held_here] && taken < most {
                waiting[held_here] = false;
                kept[held_here] = true;
                taken += 1;
            }
        }
    }
    kept
}

fn renumbered(
    held: Vec<Held>,
    edges: &[Line],
    keep: &[bool],
    near: Option<usize>,
    whole: (usize, usize),
) -> Network {
    let mut placed = vec![u32::MAX; held.len()];
    let mut nodes = Vec::new();
    for (n, (neuron, _)) in held.into_iter().enumerate() {
        if keep[n] {
            placed[n] = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
            nodes.push(neuron);
        }
    }
    let edges: Vec<(u32, u32, u8)> = edges
        .iter()
        .map(|(from, to, tie)| (placed[*from], placed[*to], *tie))
        .collect();
    let left_out = (whole.0 > nodes.len() || whole.1 > edges.len()).then_some(LeftOut {
        nodes: whole.0 - nodes.len(),
        edges: whole.1 - edges.len(),
    });
    Network {
        near: near.map(|n| placed[n]),
        nodes,
        edges,
        left_out,
        alike: Vec::new(),
    }
}

// A guess shows only between two nodes in sight, so the gate and the caps that kept them decide it too.
fn guesses(nodes: &[Neuron], guessed: &[Guessed]) -> Vec<Guess> {
    let at: HashMap<&str, u32> = nodes
        .iter()
        .enumerate()
        .filter_map(|(n, one)| Some((one.id.as_str(), u32::try_from(n).ok()?)))
        .collect();
    guessed
        .iter()
        .filter_map(|(a, b, score, shared)| {
            let (&from, &to) = (at.get(a.as_str())?, at.get(b.as_str())?);
            let thousandths = (score.clamp(0.0, 1.0) * 1000.0).round() as u16;
            (from != to).then(|| (from, to, thousandths, shared.clone()))
        })
        .collect()
}

// A tag or a file stands for what holds it, so one whose holders are all out of sight goes too.
fn drop_unheld(kinds: &[&str], edges: &[Line], keep: &mut [bool]) {
    let mut tied = vec![false; kinds.len()];
    for (from, to, _) in edges {
        if keep[*from] && keep[*to] {
            tied[*from] = true;
            tied[*to] = true;
        }
    }
    for (n, kind) in kinds.iter().enumerate() {
        if HELD_BY_OTHERS.contains(kind) && !tied[n] {
            keep[n] = false;
        }
    }
}

fn ranked(held: &[Held], edges: &[Line], keep: &[bool], near: Option<usize>) -> Vec<usize> {
    let around: HashSet<usize> = near.map_or_else(HashSet::new, |near| {
        edges
            .iter()
            .filter_map(|(from, to, _)| match (*from == near, *to == near) {
                (true, _) => Some(*to),
                (_, true) => Some(*from),
                _ => None,
            })
            .collect()
    });
    let rank = |n: usize| -> u8 {
        let (neuron, _) = &held[n];
        if Some(n) == near {
            return 0;
        }
        if around.contains(&n) {
            return 1;
        }
        match neuron.kind.as_str() {
            "doc" | "page" => 2,
            "task"
                if neuron
                    .state
                    .as_deref()
                    .is_some_and(|one| UNDER_WAY.contains(&one)) =>
            {
                3
            }
            "tag" => 4,
            "file" => 5,
            _ => 6,
        }
    };
    let mut all: Vec<usize> = (0..held.len()).filter(|n| keep[*n]).collect();
    all.sort_by_key(|n| rank(*n));
    all
}

fn tie_rank(tie: u8) -> u8 {
    match TIES.get(usize::from(tie)).copied() {
        Some("structure") => 0,
        Some("cite") => 1,
        Some("mention") => 2,
        Some("uses") => 3,
        _ => 4,
    }
}

#[cfg(test)]
#[path = "network_test.rs"]
mod tests;
