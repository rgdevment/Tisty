use std::collections::BTreeMap;
use std::sync::Mutex;

use tisty_core::Op;
use tisty_core::attach::{canonical, leaf_of};
use tisty_core::joining::{Candidate, keeper, relinked_all, shelved};
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
    /// A document that could not be read, so who links what is not known for sure.
    unread: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joined {
    freed: u64,
    relinked: usize,
    /// Copies that could not be set aside and are still on disk.
    left: usize,
}

#[derive(Default)]
struct Naming {
    links: Vec<Linked>,
    shelved: bool,
    locked: Option<String>,
}

struct Read {
    id: String,
    title: String,
    body: String,
}

fn read_all(root: &std::path::Path) -> (Vec<Read>, Option<String>) {
    let mut unread = None;
    let mut all = Vec::new();
    for doc in tisty_core::docs::all(root) {
        match tisty_core::docs::read(root, &doc.id) {
            Ok(body) => all.push(Read {
                id: doc.id,
                title: doc.title,
                body,
            }),
            Err(_) => unread = unread.or(Some(doc.title)),
        }
    }
    (all, unread)
}

fn namings(session: &Session, docs: &[Read]) -> BTreeMap<String, Naming> {
    let mut found: BTreeMap<String, Naming> = BTreeMap::new();
    let mut note = |at: String, linked: Linked, body: &str, locked: bool| {
        let Some(leaf) = leaf_of(&at).map(str::to_string) else {
            return;
        };
        let one = found.entry(at).or_default();
        one.shelved = one.shelved || shelved(body, &leaf);
        if locked && one.locked.is_none() {
            one.locked = Some(linked.title.clone());
        }
        if !one.links.iter().any(|seen| seen.id == linked.id) {
            one.links.push(linked);
        }
    };
    for doc in docs {
        let locked = session.state.bolted(&doc.id);
        for one in tisty_core::refs::extract(&doc.body) {
            let linked = Linked {
                kind: "doc",
                id: doc.id.clone(),
                title: doc.title.clone(),
            };
            note(one.target, linked, &doc.body, locked);
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
        .map_or(i64::MAX, tisty_core::attach::since_epoch)
}

#[tauri::command(async)]
pub fn twinned(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Pair>> {
    let paths = held(&session).paths.clone();
    // Hashing and reading go on without the session, which is held only to ask the tasks.
    let groups = tisty_core::attach::twins(paths.data());
    if groups.is_empty() {
        return Ok(Vec::new());
    }
    let (docs, unread) = read_all(&paths.docs());
    Ok(seen(&held(&session), groups, &docs, unread))
}

fn seen(
    session: &Session,
    groups: Vec<tisty_core::attach::Twins>,
    docs: &[Read],
    unread: Option<String>,
) -> Vec<Pair> {
    let named = namings(session, docs);
    let data = session.paths.data();
    groups
        .into_iter()
        .map(|group| {
            let copies: Vec<Candidate> = group
                .at
                .iter()
                .map(|at| Candidate {
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
                unread: unread.clone(),
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
    docs: Vec<(String, String, String, String)>,
    ops: Vec<Op>,
    read: Vec<(
        tisty_core::model::TaskId,
        Option<tisty_core::model::LogId>,
        String,
    )>,
    links: usize,
}

fn fits(body: &str) -> bool {
    tisty_core::state::short_enough(body).is_ok()
}

// Rewritten in memory first, so nothing is written while any link to a going copy would be left.
fn planned(session: &Session, go: &[String], going: &[(String, String)]) -> Answer<Plan> {
    let root = session.paths.docs();
    let mut plan = Plan {
        docs: Vec::new(),
        ops: Vec::new(),
        read: Vec::new(),
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
        if tisty_core::docs::settled(&now).len() as u64 > tisty_core::docs::BODY_AT_MOST {
            return Err(Refusal::about("twinsDocTooLong", doc.title));
        }
        let print = tisty_core::attach::printed(tisty_core::docs::settled(&body).as_bytes());
        plan.links += moved;
        plan.docs.push((doc.id, doc.title, now, print));
    }
    for task in session.state.tasks.values() {
        let mut left = false;
        let mut long = false;
        if let Some(body) = task.description.as_deref() {
            match relinked_all(body, going) {
                Some((now, moved)) => {
                    left |= names_any(&now, go);
                    long |= !fits(&now);
                    plan.links += moved;
                    plan.read.push((task.id, None, body.to_string()));
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
                    long |= !fits(&now);
                    plan.links += moved;
                    plan.read
                        .push((task.id, Some(entry.id), entry.body.clone()));
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
        if long {
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
    let data = held(&session).paths.data().to_path_buf();
    // Read with the session let go: two large videos take seconds the rest of the window would wait.
    let (keep, go, same) = alike(&data, &keep, &go)?;
    joined(&mut held(&session), keep, go, same)
}

fn alike(
    data: &std::path::Path,
    keep: &str,
    go: &[String],
) -> Answer<(String, Vec<String>, (String, u64))> {
    let gone = || Refusal::about("twinsGone", keep.to_string());
    let keep = canonical(keep).ok_or_else(gone)?;
    let mut go: Vec<String> = go
        .iter()
        .filter_map(|one| canonical(one))
        .filter(|one| *one != keep)
        .collect();
    go.sort();
    go.dedup();
    let mut all = vec![keep.clone()];
    all.extend(go.iter().cloned());
    let same = tisty_core::attach::same_bytes(data, &all)
        .ok()
        .flatten()
        .filter(|_| !go.is_empty())
        .ok_or_else(gone)?;
    Ok((keep, go, same))
}

// The log may have moved under the plan, from another process, while documents were written.
fn unmoved(session: &mut Session, plan: &Plan) -> Answer<()> {
    let _ = session.reload();
    for (id, entry, was) in &plan.read {
        let Some(task) = session.state.tasks.get(id) else {
            return Err(Refusal::of("twinsGone"));
        };
        let now = match entry {
            None => task.description.clone().unwrap_or_default(),
            Some(entry) => task
                .entry(*entry)
                .map(|one| one.body.clone())
                .unwrap_or_default(),
        };
        if &now != was {
            return Err(Refusal::about("twinsEdited", task.title.clone()));
        }
    }
    Ok(())
}

fn joined(
    session: &mut Session,
    keep: String,
    go: Vec<String>,
    (sha256, each): (String, u64),
) -> Answer<Joined> {
    let _ = session.reload();
    let to = leaf_of(&keep).unwrap_or_default().to_string();
    let going: Vec<(String, String)> = go
        .iter()
        .filter_map(|one| leaf_of(one).map(|from| (from.to_string(), to.clone())))
        .collect();
    let plan = planned(session, &go, &going)?;

    let root = session.paths.docs();
    let hand = crate::signing(&session.state);
    for (id, title, body, print) in &plan.docs {
        // Written only over the body it was read from, and what it said is kept to go back to.
        let made = tisty_core::docs::rewrite(&root, session.paths.data(), id, body, print)
            .map_err(|e| match e {
                tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
                e => blamed(
                    channel::ATTACH,
                    "a document could not be pointed at the copy that stays",
                    e,
                ),
            })?;
        // What went before stays pointed at the copy that stays: joining again finishes the rest.
        let tisty_core::docs::Rewrite::Made { whole, .. } = made else {
            return Err(Refusal::about("twinsEdited", title.clone()));
        };
        // Left unminded, so an editor holding the old body is told the file moved under it.
        session.corpus.forget(id);
        session.retell(id, &whole, hand.clone());
    }
    if !plan.ops.is_empty() {
        unmoved(session, &plan)?;
        session.commit_all(plan.ops).map_err(|e| {
            blamed(
                channel::ATTACH,
                "the tasks could not be pointed at the copy that stays",
                e,
            )
        })?;
    }
    let gone = session.retire(&go)?;
    // A copy retire passed over is still on disk, so its ledger line stays and it is said.
    let data = session.paths.data();
    let (still, went): (Vec<String>, Vec<String>) = go
        .into_iter()
        .partition(|one| tisty_core::attach::resolve(one, data).is_ok_and(|at| at.is_file()));
    tisty_core::attach::joined_in_ledger(session.paths.data(), &keep, &went, &sha256, each)
        .map_err(|e| {
            blamed(
                channel::ATTACH,
                "the ledger could not name the copy that stays",
                e,
            )
        })?;
    Ok(Joined {
        freed: each * gone as u64,
        relinked: plan.links,
        left: still.len(),
    })
}

#[cfg(test)]
#[path = "twinning_test.rs"]
mod tests;
