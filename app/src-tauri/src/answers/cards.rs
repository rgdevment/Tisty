use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use tisty_core::State;
use tisty_core::model::{DateSpec, Status, Task, TaskId};
use tisty_markdown::cards::{self, DocCard, Paper, Standing, Step, TaskCard, When, Words};

use crate::{Answer, Session, caught, held};

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Named {
    Doc,
    Task,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Asked {
    pub kind: Named,
    pub id: String,
    pub words: String,
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Seen {
    pub here: Option<String>,
    pub acting: bool,
    pub more: bool,
    pub examples: bool,
}

pub enum Card {
    Read {
        card: DocCard,
        stamp: Option<String>,
    },
    Shown(DocCard),
    Task(TaskCard),
}

const GLIMPSES_AT_MOST: usize = 1024;

#[derive(Default)]
pub struct Glimpses(Mutex<HashMap<String, (String, String)>>);

impl Glimpses {
    fn held(&self) -> std::sync::MutexGuard<'_, HashMap<String, (String, String)>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn of(&self, root: &Path, file: &str, stamp: Option<&str>) -> String {
        if let Some(stamp) = stamp
            && let Some((was, said)) = self.held().get(file)
            && was == stamp
        {
            return said.clone();
        }
        let said = tisty_core::docs::read(root, file)
            .map(|body| cards::blurb(&body))
            .unwrap_or_default();
        if let Some(stamp) = stamp {
            let mut held = self.held();
            if held.len() >= GLIMPSES_AT_MOST {
                held.clear();
            }
            held.insert(file.to_string(), (stamp.to_string(), said.clone()));
        }
        said
    }
}

#[tauri::command(async)]
pub fn cards_drawn(
    session: tauri::State<'_, Mutex<Session>>,
    glimpses: tauri::State<'_, Glimpses>,
    asked: Vec<Asked>,
    seen: Seen,
) -> Answer<Vec<String>> {
    caught("cards_drawn", || drawn(&session, &glimpses, &asked, &seen))
}

// The store is read under the lock and the documents' words after it, so a long page holds nobody up.
pub fn drawn(
    session: &Mutex<Session>,
    glimpses: &Glimpses,
    asked: &[Asked],
    seen: &Seen,
) -> Vec<String> {
    let (known, root, words) = {
        let session = held(session);
        let root = session.paths.docs();
        let spoken = tisty_core::model::spoken(session.locale.as_deref());
        let known = known(&session.state, asked, seen);
        (known, root, cards::words(&spoken))
    };
    said(known, &root, glimpses, words)
}

pub fn said(known: Vec<Card>, root: &Path, glimpses: &Glimpses, words: &Words) -> Vec<String> {
    known
        .into_iter()
        .map(|one| match one {
            Card::Read { mut card, stamp } => {
                if written(root, &card.file) {
                    card.blurb = glimpses.of(root, &card.file, stamp.as_deref());
                } else {
                    card.paper = Paper::Gone;
                }
                cards::doc_card(&card, words)
            }
            Card::Shown(card) => cards::doc_card(&card, words),
            Card::Task(card) => cards::task_card(&card, words),
        })
        .collect()
}

pub fn known(state: &State, asked: &[Asked], seen: &Seen) -> Vec<Card> {
    let named: Vec<String> = asked
        .iter()
        .filter(|one| one.kind == Named::Doc)
        .map(|one| one.id.clone())
        .collect();
    let pages = pages_here(state, seen.here.as_deref(), &named);
    asked
        .iter()
        .map(|one| match one.kind {
            Named::Doc => paper(state, one, seen, &pages),
            Named::Task => Card::Task(cited(state, one, seen)),
        })
        .collect()
}

fn pages_here(state: &State, here: Option<&str>, named: &[String]) -> Vec<String> {
    let Some(mine) = here.and_then(|file| {
        state
            .papers()
            .find(|one| one.file == file && one.page_of.is_none())
    }) else {
        return Vec::new();
    };
    state
        .pages_named(mine.id, named)
        .into_iter()
        .filter(|one| !state.held_away(one))
        .map(|one| one.file.clone())
        .collect()
}

fn paper(state: &State, one: &Asked, seen: &Seen, pages: &[String]) -> Card {
    let card = |paper| DocCard {
        file: one.id.clone(),
        words: one.words.clone(),
        paper,
        blurb: String::new(),
        more: seen.more,
    };
    if seen.examples {
        return Card::Shown(DocCard {
            more: seen.more,
            ..DocCard::example(&one.id, &one.words)
        });
    }
    let Some(doc) = state.papers().find(|doc| doc.file == one.id) else {
        return Card::Shown(card(Paper::Gone));
    };
    let title = doc.title.clone().unwrap_or_default();
    if seen.here.as_deref() == Some(doc.file.as_str()) {
        return Card::Shown(card(Paper::Itself { title }));
    }
    let shown = match pages.iter().position(|page| *page == doc.file) {
        _ if state.held_away(doc) => Paper::Away { title },
        Some(at) => Paper::Page {
            title,
            at: at + 1,
            of: pages.len(),
        },
        None => Paper::Doc { title },
    };
    let stamp = doc
        .print
        .clone()
        .or_else(|| doc.wrote.map(|at| at.to_string()));
    Card::Read {
        card: card(shown),
        stamp,
    }
}

fn written(root: &Path, file: &str) -> bool {
    tisty_core::docs::resolve(root, file).is_ok_and(|at| at.is_file())
}

fn cited(state: &State, one: &Asked, seen: &Seen) -> TaskCard {
    let task = one
        .id
        .parse::<TaskId>()
        .ok()
        .and_then(|id| state.tasks.get(&id))
        .filter(|_| !seen.examples)
        .map(|task| facts(state, task, seen.here.as_deref()));
    TaskCard {
        id: one.id.clone(),
        words: one.words.clone(),
        task,
        acting: seen.acting,
    }
}

fn when(spec: &DateSpec) -> When {
    When {
        at: spec.at.to_string(),
        timed: spec.has_time,
    }
}

// A part on its own whole's page sits in the whole's list, so naming the list there says nothing.
fn facts(state: &State, task: &Task, here: Option<&str>) -> cards::Task {
    let at_home = task
        .part_of
        .is_some_and(|whole| here == Some(whole.to_string().as_str()));
    cards::Task {
        title: task.title.clone(),
        standing: match task.status {
            Status::Open => Standing::Open,
            Status::Done => Standing::Done,
            Status::Dropped => Standing::Dropped,
        },
        list: task
            .list
            .and_then(|list| state.lists.get(&list))
            .map(|list| list.name.clone())
            .filter(|_| !at_home),
        date: task.date.as_ref().map(when),
        deadline: task.deadline.as_ref().map(when),
        closed: task.completed_at.map(|at| at.to_string()),
        steps: tisty_core::stages::ordered(task)
            .into_iter()
            .map(|step| Step {
                id: step.id.to_string(),
                text: step.text.clone(),
                done: step.done,
                said: step.said && !step.done && task.is_open(),
            })
            .collect(),
        parts: counted(state, task.id),
    }
}

fn counted(state: &State, whole: TaskId) -> Option<(usize, usize)> {
    let (closed, all) = state
        .parts_of(whole)
        .filter(|part| !part.folded())
        .fold((0, 0), |(closed, all), part| {
            (closed + usize::from(!part.is_open()), all + 1)
        });
    (all > 0).then_some((closed, all))
}

#[cfg(test)]
#[path = "cards_test.rs"]
mod tests;
