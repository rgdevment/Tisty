use serde::{Deserialize, Serialize};

use super::card::{DocFacts, Section, TaskFacts, Tongue};
use super::linked::{Ground, Holder, Read, Ties};
use super::nodes::Node;
use crate::model::Kept;
use crate::{State, Task};

pub(super) struct Fresh {
    pub(super) ties: Ties,
    pub(super) said: Option<String>,
}

// What the round keeps of a card: the node's own facts and the bare ids it names, joined on `node` when read.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(super) struct Own<F> {
    #[serde(flatten)]
    pub(super) facts: F,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) book: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) pages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) languages: Vec<Tongue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) part_of: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) sections: Vec<Section>,
}

pub(super) fn fresh(state: &State, ground: &Ground, id: &str, node: &Node) -> Fresh {
    let read = ground.read(id, &node.holder);
    let said = match node.holder {
        Holder::Doc(kept) => serde_json::to_string(&of_doc(state, ground, kept, &read)).ok(),
        Holder::Task(task) => serde_json::to_string(&of_task(task, read.away)).ok(),
        Holder::Nothing => None,
    };
    Fresh {
        ties: read.ties,
        said,
    }
}

fn of_doc(state: &State, ground: &Ground, kept: &Kept, read: &Read) -> Own<DocFacts> {
    let body = read.body.as_deref();
    let pages = match (ground.has_pages(kept), body) {
        (true, Some(body)) => state.pages_read(kept.id, body),
        (true, None) => state.pages_of(kept.id),
        (false, _) => Vec::new(),
    };
    Own {
        facts: DocFacts {
            lang: kept.lang.clone(),
            bytes: body.map_or(kept.bytes, |one| Some(one.len() as u64)),
            lines: body.map(|one| one.lines().count()),
            words: body.map(|one| one.split_whitespace().count()),
            made: kept.made,
            wrote: kept.wrote,
        },
        book: ground.book_of(kept).map(str::to_string),
        pages: pages.iter().map(|one| one.file.clone()).collect(),
        languages: ground
            .tongues_of(kept)
            .iter()
            .filter(|one| one.file != kept.file)
            .map(|one| Tongue {
                lang: one.lang.clone().unwrap_or_default(),
                doc: one.file.clone(),
            })
            .collect(),
        part_of: None,
        sections: body.map(sections).unwrap_or_default(),
    }
}

fn of_task(task: &Task, away: Vec<String>) -> Own<TaskFacts> {
    let (done, all) = task.steps_done();
    Own {
        facts: TaskFacts {
            priority: task.priority.set().then_some(task.priority),
            date: task.date.as_ref().map(|one| one.date()),
            deadline: task.deadline.as_ref().map(|one| one.date()),
            steps: (all > 0).then_some([done, all]),
            source: task.source.clone(),
            links: away,
        },
        part_of: task.part_of.map(|whole| whole.to_string()),
        ..Own::default()
    }
}

// Byte places count every line as `lines_between` cuts it, so a section reads back the same either way.
fn sections(body: &str) -> Vec<Section> {
    let mut starts = vec![0];
    starts.extend(body.split_inclusive('\n').scan(0, |at, line| {
        *at += line.len();
        Some(*at)
    }));
    let at_line = |line: usize| {
        starts
            .get(line.saturating_sub(1))
            .copied()
            .unwrap_or(body.len())
    };
    crate::docs::outlined(body)
        .into_iter()
        .map(|head| Section {
            title: head.title,
            level: head.level,
            lines: [head.line, head.to],
            bytes: [at_line(head.line), at_line(head.to + 1)],
        })
        .collect()
}

#[cfg(test)]
#[path = "own_test.rs"]
mod tests;
