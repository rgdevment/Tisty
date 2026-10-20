use std::collections::BTreeMap;
use std::sync::Mutex;

use tisty_core::Op;
use tisty_core::attach::{canonical, leaf_of};
use tisty_core::joining::{Copy, keeper, relinked_all, shelved};
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

#[derive(Default)]
struct Naming {
    links: Vec<Linked>,
    shelved: bool,
    locked: Option<String>,
}

fn namings(session: &Session) -> BTreeMap<String, Naming> {
    let mut found: BTreeMap<String, Naming> = BTreeMap::new();
    let mut note = |at: String, linked: Linked, body: &str, locked: bool| {
        let Some(leaf) = leaf_of(&at).map(str::to_string) else {
            return;
        };
        let one = found.entry(at).or_default();
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
        let whole: Vec<&str> = task
            .description
            .as_deref()
            .into_iter()
            .chain(task.log.iter().map(|entry| entry.body.as_str()))
            .collect();
        let whole = whole.join("\n");
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

fn since(data: &std::path::Path, at: &str) -> i64 {
    tisty_core::attach::resolve(at, data)
        .ok()
        .and_then(|held| std::fs::metadata(held).ok()?.modified().ok())
        .and_then(|when| when.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(i64::MAX, |gone| gone.as_secs() as i64)
}

#[tauri::command(async)]
pub fn twinned(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Pair>> {
    let data = held(&session).paths.data().to_path_buf();
    // Hashing goes on without the session, which is held only to read who links what.
    let groups = tisty_core::attach::twins(&data);
    if groups.is_empty() {
        return Ok(Vec::new());
    }
    Ok(seen(&held(&session), groups))
}

fn seen(session: &Session, groups: Vec<tisty_core::attach::Twins>) -> Vec<Pair> {
    let named = namings(session);
    let data = session.paths.data();
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
                    since: since(data, at),
                })
                .collect();
            Pair {
                bytes: group.bytes,
                stays: group.at[keeper(&copies)].clone(),
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

fn names_any(body: &str, going: &[String]) -> bool {
    tisty_core::refs::extract(body)
        .iter()
        .any(|one| going.contains(&one.target))
}

struct Plan {
    docs: Vec<(String, String)>,
    ops: Vec<Op>,
    links: usize,
}

// Rewritten in memory first, so nothing is written while any link to a going copy would be left.
fn planned(session: &Session, go: &[String], going: &[(String, String)]) -> Answer<Plan> {
    let root = session.paths.docs();
    let mut plan = Plan {
        docs: Vec::new(),
        ops: Vec::new(),
        links: 0,
    };
    for doc in tisty_core::docs::all(&root) {
        let Ok(body) = tisty_core::docs::read(&root, &doc.id) else {
            return Err(Refusal::about("retireUnread", doc.title));
        };
        let now = relinked_all(&body, going);
        let after = now
            .as_ref()
            .map_or(body.as_str(), |(text, _)| text.as_str());
        if names_any(after, go) {
            return Err(Refusal::about("twinsStill", doc.title));
        }
        let Some((now, moved)) = now else {
            continue;
        };
        if session.state.bolted(&doc.id) {
            return Err(Refusal::about("twinsLocked", doc.title));
        }
        plan.links += moved;
        plan.docs.push((doc.id, now));
    }
    for task in session.state.tasks.values() {
        let mut left = false;
        if let Some(body) = task.description.as_deref() {
            match relinked_all(body, going) {
                Some((now, moved)) => {
                    left |= names_any(&now, go);
                    plan.links += moved;
                    plan.ops.push(Op::TaskDescribe {
                        id: task.id,
                        d: tisty_core::event::Body { body: Some(now) },
                    });
                }
                None => left |= names_any(body, go),
            }
        }
        for entry in &task.log {
            match relinked_all(&entry.body, going) {
                Some((now, moved)) => {
                    left |= names_any(&now, go);
                    plan.links += moved;
                    plan.ops.push(Op::TaskLogEdit {
                        id: task.id,
                        d: tisty_core::event::LogEdit {
                            entry: entry.id,
                            body: now,
                        },
                    });
                }
                None => left |= names_any(&entry.body, go),
            }
        }
        if left {
            return Err(Refusal::about("twinsStill", task.title.clone()));
        }
        let too_long = plan.ops.iter().any(|op| match op {
            Op::TaskDescribe { id, d } if *id == task.id => d
                .body
                .as_deref()
                .is_some_and(|body| tisty_core::state::short_enough(body).is_err()),
            Op::TaskLogEdit { id, d } if *id == task.id => {
                tisty_core::state::short_enough(&d.body).is_err()
            }
            _ => false,
        });
        if too_long {
            return Err(Refusal::about("twinsTooLong", task.title.clone()));
        }
    }
    Ok(plan)
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
    let mut all = vec![keep.clone()];
    all.extend(go.iter().cloned());
    // Read again from the files: only copies the same bytes through and through become one.
    let each = tisty_core::attach::same_bytes(session.paths.data(), &all)
        .ok()
        .flatten()
        .filter(|_| !go.is_empty())
        .ok_or_else(|| Refusal::about("twinsGone", keep.clone()))?;
    let to = leaf_of(&keep).unwrap_or_default().to_string();
    let going: Vec<(String, String)> = go
        .iter()
        .filter_map(|one| leaf_of(one).map(|from| (from.to_string(), to.clone())))
        .collect();
    let plan = planned(session, &go, &going)?;

    tisty_core::attach::stays_listed(session.paths.data(), &keep).map_err(|e| {
        blamed(
            channel::ATTACH,
            "the copy that stays could not be named in the ledger",
            e,
        )
    })?;
    let root = session.paths.docs();
    let hand = crate::signing(&session.state);
    for (id, body) in &plan.docs {
        tisty_core::docs::write(&root, id, body).map_err(|e| {
            blamed(
                channel::ATTACH,
                "a document could not be pointed at the copy that stays",
                e,
            )
        })?;
        // Left unminded, so an editor holding the old body is told the file moved under it.
        session.corpus.forget(id);
        session.retell(id, body, hand.clone());
    }
    if !plan.ops.is_empty() {
        session.commit_all(plan.ops).map_err(|e| {
            blamed(
                channel::ATTACH,
                "the tasks could not be pointed at the copy that stays",
                e,
            )
        })?;
    }
    let gone = session.retire(&go)?;
    Ok(Joined {
        freed: each * gone as u64,
        relinked: plan.links,
    })
}

#[cfg(test)]
#[path = "twinning_test.rs"]
mod tests;
