use std::sync::Mutex;

use tisty_core::Task;
use tisty_core::view::Scope;

use crate::{Answer, Session, answers, held, summing, today};

#[tauri::command(async)]
pub fn search(
    session: tauri::State<'_, Mutex<Session>>,
    query: String,
    scope: Option<String>,
) -> Answer<Found> {
    let mut session = held(&session);
    session.reload()?;

    let scope = match scope.as_deref() {
        Some("open") => Scope::Open,
        Some("archived") => Scope::Archived,
        _ => Scope::Either,
    };
    let behind = tisty_core::task_docs::behind(&session.state);
    let listed: std::collections::BTreeMap<String, bool> = session
        .state
        .papers()
        .filter(|one| session.state.doc_kept_by(scope, one))
        .map(|one| (one.file.clone(), session.state.held_away(one)))
        .collect();
    let root = session.paths.docs();
    let worded = session
        .corpus
        .searching(&root, &query, usize::MAX, |id| behind.contains_key(id));
    let (mut found, _) = session.state.searching(&query, scope, usize::MAX);
    let mut seen: std::collections::HashSet<_> = found.iter().map(|one| one.id).collect();
    for task in worded
        .iter()
        .filter_map(|one| session.state.tasks.get(behind.get(&one.id)?))
        .filter(|task| scope.keeps(task))
    {
        if seen.insert(task.id) {
            found.push(task);
        }
    }
    let total = found.len();
    let tasks: Vec<Task> = found.into_iter().take(MOST).cloned().collect();
    let sighted = session
        .corpus
        .searching(&root, &query, PAPERS_MOST, |id| listed.contains_key(id));
    let papers = answers::tongues::once_each(&session.state, sighted, |one| &one.id)
        .into_iter()
        .map(|one| Paper {
            archived: listed.get(&one.id).copied().unwrap_or(false),
            id: one.id,
            title: one.title,
            line: one.line,
        })
        .collect();
    let wholes = answers::parts::wholes(&session.state);
    Ok(Found {
        standings: summing::standings(&session.state, today(), &tasks),
        paces: summing::paces(&wholes, today(), &tasks),
        tasks,
        total,
        papers,
        wholes,
    })
}

const MOST: usize = 200;
const PAPERS_MOST: usize = 40;

#[derive(serde::Serialize)]
pub struct Paper {
    id: String,
    title: String,
    line: String,
    archived: bool,
}

#[derive(serde::Serialize)]
pub struct Found {
    tasks: Vec<Task>,
    papers: Vec<Paper>,
    total: usize,
    wholes: std::collections::BTreeMap<String, answers::parts::Whole>,
    standings: std::collections::BTreeMap<String, tisty_core::standing::Standing>,
    paces: std::collections::BTreeMap<String, tisty_core::pace::Paced>,
}
