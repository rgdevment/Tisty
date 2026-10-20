use std::collections::BTreeMap;
use std::sync::Mutex;

use tisty_core::Op;
use tisty_core::attach::{canonical, leaf_of};
use tisty_core::joining::{Copy, keeper, relinked, shelved};
use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Linked {
    kind: &'static str,
    id: String,
    title: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Twin {
    at: String,
    links: Vec<Linked>,
    /// A locked or archived document names it, so its links cannot be changed from here.
    locked: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    bytes: u64,
    stays: String,
    copies: Vec<Twin>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joined {
    freed: u64,
    relinked: usize,
}

struct Naming {
    links: Vec<Linked>,
    shelved: bool,
    locked: Option<String>,
}

// What names each attachment, read from every document and task body once.
fn namings(session: &Session) -> BTreeMap<String, Naming> {
    let mut found: BTreeMap<String, Naming> = BTreeMap::new();
    let mut note = |at: String, linked: Linked, body: &str, locked: bool| {
        let leaf = leaf_of(&at).unwrap_or_default().to_string();
        let one = found.entry(at).or_insert(Naming {
            links: Vec::new(),
            shelved: false,
            locked: None,
        });
        one.shelved |= shelved(body, &leaf);
        if locked && one.locked.is_none() {
            one.locked = Some(linked.title.clone());
        }
        if !one.links.iter().any(|seen| seen.id == linked.id) {
            one.links.push(linked);
        }
    };
    let root = session.paths.docs();
    for doc in tisty_core::docs::all(&root) {
        let Ok(body) = tisty_core::docs::read(&root, &doc.id) else {
            continue;
        };
        let locked = session.state.bolted(&doc.id);
        for one in tisty_core::refs::extract(&body) {
            let linked = Linked {
                kind: "doc",
                id: doc.id.clone(),
                title: doc.title.clone(),
            };
            note(one.target, linked, &body, locked);
        }
    }
    for task in session.state.tasks.values() {
        let bodies: Vec<&str> = task
            .description
            .as_deref()
            .into_iter()
            .chain(task.log.iter().map(|entry| entry.body.as_str()))
            .collect();
        let whole = bodies.join("\n");
        for one in task.references() {
            let linked = Linked {
                kind: "task",
                id: task.id.to_string(),
                title: task.title.clone(),
            };
            note(one.target, linked, &whole, false);
        }
    }
    found
}

fn since(session: &Session, at: &str) -> i64 {
    tisty_core::attach::resolve(at, session.paths.data())
        .ok()
        .and_then(|held| std::fs::metadata(held).ok()?.modified().ok())
        .and_then(|when| when.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(i64::MAX, |gone| gone.as_secs() as i64)
}

#[tauri::command(async)]
pub fn twinned(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Pair>> {
    Ok(seen(&held(&session)))
}

fn seen(session: &Session) -> Vec<Pair> {
    let named = namings(session);
    let groups = tisty_core::attach::twins(session.paths.data());
    groups
        .into_iter()
        .map(|group| {
            let copies: Vec<Copy> = group
                .at
                .iter()
                .map(|at| Copy {
                    at: at.clone(),
                    links: named.get(at).map_or(0, |one| one.links.len()),
                    shelved: named.get(at).is_some_and(|one| one.shelved),
                    since: since(session, at),
                })
                .collect();
            let stays = group.at[keeper(&copies)].clone();
            Pair {
                bytes: group.bytes,
                stays,
                copies: group
                    .at
                    .iter()
                    .map(|at| Twin {
                        at: at.clone(),
                        links: named
                            .get(at)
                            .map(|one| one.links.clone())
                            .unwrap_or_default(),
                        locked: named.get(at).and_then(|one| one.locked.clone()),
                    })
                    .collect(),
            }
        })
        .collect()
}

// The task bodies that name a copy going, each written again to name the one that stays.
fn tasks_relinked(session: &Session, going: &[(String, String)]) -> Vec<Op> {
    let swapped = |body: &str| {
        let mut now: Option<String> = None;
        for (from, to) in going {
            if let Some(changed) = relinked(now.as_deref().unwrap_or(body), from, to) {
                now = Some(changed);
            }
        }
        now
    };
    let mut ops = Vec::new();
    for task in session.state.tasks.values() {
        if let Some(body) = task.description.as_deref().and_then(swapped) {
            ops.push(Op::TaskDescribe {
                id: task.id,
                d: tisty_core::event::Body { body: Some(body) },
            });
        }
        for entry in &task.log {
            if let Some(body) = swapped(&entry.body) {
                ops.push(Op::TaskLogEdit {
                    id: task.id,
                    d: tisty_core::event::LogEdit {
                        entry: entry.id,
                        body,
                    },
                });
            }
        }
    }
    ops
}

#[tauri::command(async)]
pub fn twins_join(
    session: tauri::State<'_, Mutex<Session>>,
    keep: String,
    go: Vec<String>,
) -> Answer<Joined> {
    joined(&mut held(&session), keep, go)
}

fn joined(session: &mut Session, keep: String, go: Vec<String>) -> Answer<Joined> {
    let _ = session.reload();
    let keep = canonical(&keep).ok_or_else(|| Refusal::about("twinsGone", keep.clone()))?;
    let go: Vec<String> = go
        .iter()
        .filter_map(|one| canonical(one))
        .filter(|one| *one != keep)
        .collect();
    // Asked again now: only copies the same bytes through and through become one.
    let group = tisty_core::attach::twins(session.paths.data())
        .into_iter()
        .find(|group| group.at.contains(&keep) && go.iter().all(|one| group.at.contains(one)))
        .filter(|_| !go.is_empty())
        .ok_or_else(|| Refusal::about("twinsGone", keep.clone()))?;
    let to = leaf_of(&keep).unwrap_or_default().to_string();
    let going: Vec<(String, String)> = go
        .iter()
        .filter_map(|one| leaf_of(one).map(|from| (from.to_string(), to.clone())))
        .collect();

    let root = session.paths.docs();
    let mut written = Vec::new();
    for doc in tisty_core::docs::all(&root) {
        let Ok(body) = tisty_core::docs::read(&root, &doc.id) else {
            return Err(Refusal::about("retireUnread", doc.title));
        };
        let mut now: Option<String> = None;
        for (from, to) in &going {
            if let Some(changed) = relinked(now.as_deref().unwrap_or(&body), from, to) {
                now = Some(changed);
            }
        }
        let Some(now) = now else {
            continue;
        };
        if session.state.bolted(&doc.id) {
            return Err(Refusal::about("twinsLocked", doc.title));
        }
        written.push((doc.id, now));
    }

    let hand = crate::signing(&session.state);
    for (id, body) in &written {
        tisty_core::docs::write(&root, id, body).map_err(|e| {
            blamed(
                channel::ATTACH,
                "a document could not be pointed at the copy that stays",
                e,
            )
        })?;
        session.mind_body(id, &tisty_core::docs::settled(body));
        session.corpus.forget(id);
        session.retell(id, body, hand.clone());
    }
    let ops = tasks_relinked(session, &going);
    let relinked = written.len() + ops.len();
    if !ops.is_empty() {
        session.commit_all(ops).map_err(|e| {
            blamed(
                channel::ATTACH,
                "the tasks could not be pointed at the copy that stays",
                e,
            )
        })?;
    }
    let gone = session.retire(&go)?;
    let each = group.bytes / (group.at.len() as u64 - 1).max(1);
    Ok(Joined {
        freed: each * gone as u64,
        relinked,
    })
}

#[cfg(test)]
#[path = "twinning_test.rs"]
mod tests;
