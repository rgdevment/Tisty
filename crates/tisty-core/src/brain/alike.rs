use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::Serialize;

use super::linked::{Ground, Holder, TITLED};
use super::nodes::{Kind, Node};
use super::{Brain, Docs, said, warned};
use crate::model::DocId;
use crate::witness::{self, Fact, channel};
use crate::{Paths, State};

mod dictionary;
mod likeness;
mod spoken;
mod tracks;
mod typos;
mod words;

use dictionary::Dictionary;
use likeness::{CAPS, Caps, Item};
pub(super) use tracks::SHOWN;
use tracks::Tracked;

const TURN: &str = "words_turn";
const PASSED: &str = "alike_turn";
const KIN_BOOK: &str = "book:";
const KIN_TONGUE: &str = "tongue:";
const PATIENCE: u32 = 10;
pub const TISTY_NOTICES: &[&str] = &[
    "Aquí hay algo que parece una credencial de verdad",
    "Something here reads like a live credential",
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Alike {
    pub id: String,
    pub score: f64,
    pub shared: Vec<String>,
    pub method: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Passed {
    Idle,
    Settling,
    Overtaken,
    Ran {
        nodes: usize,
        rows: usize,
        written: usize,
        steps: u64,
        common_from: usize,
        typos: usize,
        translated: usize,
    },
}

pub(super) struct Counted {
    lang: Option<String>,
    guess: Option<&'static str>,
    kin: Vec<String>,
    said: String,
}

pub(super) fn counted(
    state: &State,
    ground: &Ground,
    holder: &Holder,
    body: Option<&str>,
) -> Option<Counted> {
    match holder {
        Holder::Doc(kept) => {
            let text = body.map_or_else(|| kept.title.clone().unwrap_or_default(), prose);
            let text = text.as_str();
            let mut kin = Vec::new();
            if let Some(book) = ground.book_of(kept) {
                kin.push(format!("{KIN_BOOK}{book}"));
            }
            if !ground.tongues_of(kept).is_empty() {
                kin.push(format!("{KIN_TONGUE}{}", state.anchor_of(kept)));
            }
            let lang = kept.lang.as_deref().and_then(primary);
            let declared = match lang.as_deref() {
                Some(spoken::SPANISH) => Some(spoken::SPANISH),
                Some(spoken::ENGLISH) => Some(spoken::ENGLISH),
                _ => None,
            };
            Some(Counted {
                guess: declared.or_else(|| spoken::guessed(&[text])),
                lang,
                kin,
                said: words::said(&words::counted(&[text])),
            })
        }
        // A note tells how the task went, often in the same few words for any task, not what it is about.
        Holder::Task(task) => {
            let written = task
                .description
                .as_deref()
                .into_iter()
                .chain(task.steps.iter().map(|step| step.text.as_str()))
                .chain(body)
                .map(prose);
            let said: Vec<String> = std::iter::once(task.title.clone()).chain(written).collect();
            let texts: Vec<&str> = said.iter().map(String::as_str).collect();
            Some(Counted {
                lang: None,
                guess: spoken::guessed(&texts),
                kin: Vec::new(),
                said: words::said(&words::counted(&texts)),
            })
        }
        Holder::Nothing => None,
    }
}

// Tisty's own notice over a credential reads the same in every document it lands in.
fn prose(markdown: &str) -> String {
    if !TISTY_NOTICES.iter().any(|notice| markdown.contains(notice)) {
        return tisty_markdown::prose(markdown);
    }
    let unnoticed: Vec<&str> = markdown
        .lines()
        .filter(|line| {
            let said = line.trim_start().trim_start_matches('>').trim_start();
            !TISTY_NOTICES.iter().any(|notice| said.starts_with(notice))
        })
        .collect();
    tisty_markdown::prose(&unnoticed.join("\n"))
}

fn primary(lang: &str) -> Option<String> {
    let first = lang.split(['-', '_']).next().unwrap_or_default().trim();
    (!first.is_empty()).then(|| first.to_lowercase())
}

// The diary stays out whether or not agents may read it: its words are the person's alone.
pub(super) fn kept_out(state: &State, docs: &Docs) -> HashSet<String> {
    let books = crate::diary::books(&docs.root, state);
    let diary = state
        .papers()
        .filter(|kept| crate::diary::within(&books, kept))
        .map(|kept| kept.file.clone());
    let folded = state
        .tasks
        .values()
        .filter(|task| task.folded())
        .map(|task| task.id.to_string());
    diary.chain(folded).chain(agents_own(state)).collect()
}

// What the agents keep is their own words and code, mirrored from their files: it guesses nothing about the person's.
fn agents_own(state: &State) -> Vec<String> {
    let held = crate::agents::holding(state);
    state
        .papers()
        .filter(|kept| held(kept))
        .map(|kept| kept.file.clone())
        .collect()
}

pub(super) struct Rewritten<'a> {
    pub(super) id: &'a str,
    pub(super) stamp: &'a str,
    pub(super) counted: Counted,
}

// What the base lacks words for though it is in sight: a diary that opened again, or a row lost.
pub(super) fn unworded<'w>(
    tx: &Transaction,
    wanted: &'w BTreeMap<String, Node>,
    out: &HashSet<String>,
    rewritten: &[Rewritten],
) -> rusqlite::Result<Vec<&'w String>> {
    let held = ids(tx, "SELECT id FROM words")?;
    let fresh: HashSet<&str> = rewritten.iter().map(|one| one.id).collect();
    Ok(wanted
        .iter()
        .filter(|(id, node)| {
            matches!(node.kind, Kind::Doc | Kind::Page | Kind::Task)
                && !out.contains(*id)
                && !held.contains(*id)
                && !fresh.contains(id.as_str())
        })
        .map(|(id, _)| id)
        .collect())
}

pub(super) fn kept_in_round(
    tx: &Transaction,
    rewritten: &[Rewritten],
    out: &HashSet<String>,
    gone: &[&String],
) -> rusqlite::Result<()> {
    let mut moved = false;
    let mut put = tx.prepare("INSERT OR REPLACE INTO words VALUES (?, ?, ?, ?, ?, ?)")?;
    for one in rewritten.iter().filter(|one| !out.contains(one.id)) {
        put.execute(rusqlite::params![
            one.id,
            one.stamp,
            one.counted.lang,
            one.counted.guess,
            one.counted.kin.join("\u{1}"),
            one.counted.said
        ])?;
        moved = true;
    }
    let held = ids(tx, "SELECT id FROM words")?;
    moved |= gone.iter().any(|id| held.contains(*id));
    let mut unworded = tx.prepare("DELETE FROM words WHERE id = ?")?;
    let mut unlike = tx.prepare("DELETE FROM alike WHERE a = ?1 OR b = ?1")?;
    for id in held.iter().filter(|one| out.contains(*one)) {
        unworded.execute([id])?;
        unlike.execute([id])?;
        moved = true;
    }
    if moved {
        let turn = turn_of(tx, TURN)?.unwrap_or(0) + 1;
        tx.execute(
            "INSERT OR REPLACE INTO meta VALUES (?, ?)",
            [TURN, &turn.to_string()],
        )?;
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Weighing {
    heard: Option<u64>,
    waited: u32,
    dictionary: Option<Dictionary>,
    missed: u32,
}

impl Weighing {
    // A turn that moved since the last look means somebody is still writing, so the pass waits, but never past its patience.
    pub fn kept(&mut self, paths: &Paths, dictionary: Option<&Path>) -> Option<Passed> {
        if !paths.store().is_dir() {
            return None;
        }
        let mut brain = Brain::open(paths.cache())?;
        let turn = turn_of(&brain.db, TURN).ok().flatten();
        let settled = turn == self.heard;
        self.heard = turn;
        if !settled && self.waited < PATIENCE {
            self.waited += 1;
            return Some(Passed::Settling);
        }
        self.waited = 0;
        if let Some(at) = dictionary
            && self.dictionary.is_none()
        {
            match Dictionary::read(at) {
                Ok(read) => self.dictionary = Some(read),
                Err(_) if self.missed < PATIENCE => {
                    self.missed += 1;
                    return Some(Passed::Settling);
                }
                Err(why) => {
                    if self.missed == PATIENCE {
                        self.missed += 1;
                        warned_dictionary(at, why);
                    }
                }
            }
        }
        weighed_now(&mut brain, paths, self.dictionary.as_ref())
    }
}

// A table that cannot be read gives up its guesses only after the pass's patience, so one bad read drops none.
fn warned_dictionary(at: &Path, why: String) {
    witness::warn(
        channel::BRAIN,
        "the dictionary could not be read, so no guess crosses from Spanish to English",
        &[
            ("at", Fact::Path(at.to_path_buf())),
            ("why", Fact::Why(why)),
        ],
    );
}

fn weighed_now(
    brain: &mut Brain,
    paths: &Paths,
    dictionary: Option<&Dictionary>,
) -> Option<Passed> {
    let started = Instant::now();
    match brain.weigh_alike(&CAPS, dictionary) {
        Ok(passed) => {
            if let Passed::Ran {
                nodes,
                rows,
                written,
                steps,
                common_from,
                typos,
                translated,
            } = passed
            {
                witness::trace(
                    channel::BRAIN,
                    "the brain weighed what is alike",
                    &[
                        ("nodes", Fact::Count(nodes)),
                        ("rows", Fact::Count(rows)),
                        ("written", Fact::Count(written)),
                        ("steps", Fact::Count(steps.try_into().unwrap_or(usize::MAX))),
                        ("common_from", Fact::Count(common_from)),
                        ("typos", Fact::Count(typos)),
                        ("translated", Fact::Count(translated)),
                        (
                            "ms",
                            Fact::Count(started.elapsed().as_millis().try_into().unwrap_or(0)),
                        ),
                    ],
                );
            }
            Some(passed)
        }
        Err(e) => {
            warned(
                "the brain could not weigh what is alike",
                paths.cache(),
                &e.to_string(),
            );
            None
        }
    }
}

struct Worded {
    ids: Vec<String>,
    items: Vec<Item>,
    guesses: Vec<Option<String>>,
}

pub(super) struct Weighed {
    turn: u64,
    ids: Vec<String>,
    found: Tracked,
}

impl Brain {
    pub(super) fn weigh_alike(
        &mut self,
        caps: &Caps,
        dictionary: Option<&Dictionary>,
    ) -> rusqlite::Result<Passed> {
        match self.weighed(caps, dictionary)? {
            Some(weighed) => self.kept_alike(weighed),
            None => Ok(Passed::Idle),
        }
    }

    pub(super) fn weighed(
        &self,
        caps: &Caps,
        dictionary: Option<&Dictionary>,
    ) -> rusqlite::Result<Option<Weighed>> {
        let snapshot = self.db.unchecked_transaction()?;
        let Some(turn) = turn_of(&snapshot, TURN)? else {
            return Ok(None);
        };
        if turn_of(&snapshot, PASSED)? == Some(turn) {
            return Ok(None);
        }
        let worded = worded(&snapshot)?;
        let tied = tied(&snapshot, &worded.ids)?;
        drop(snapshot);
        Ok(Some(Weighed {
            turn,
            found: tracks::alike(&worded.items, &worded.guesses, &tied, caps, dictionary),
            ids: worded.ids,
        }))
    }

    // A round that wrote words meanwhile leaves this result stale, so it is dropped for the next pass.
    pub(super) fn kept_alike(&mut self, weighed: Weighed) -> rusqlite::Result<Passed> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if turn_of(&tx, TURN)? != Some(weighed.turn) {
            return Ok(Passed::Overtaken);
        }
        let wanted: Held = weighed
            .found
            .found
            .iter()
            .map(|pair| {
                (
                    (weighed.ids[pair.a].clone(), weighed.ids[pair.b].clone()),
                    (
                        thousandths(f64::from(pair.score)),
                        pair.shared.join(" "),
                        pair.method.to_string(),
                    ),
                )
            })
            .collect();
        let held = held_alike(&tx)?;
        let mut written = 0;
        {
            let mut unlike = tx.prepare("DELETE FROM alike WHERE a = ? AND b = ?")?;
            for (a, b) in held.keys().filter(|one| !wanted.contains_key(*one)) {
                unlike.execute([a, b])?;
                written += 1;
            }
            let mut like = tx.prepare("INSERT OR REPLACE INTO alike VALUES (?, ?, ?, ?, ?)")?;
            for ((a, b), told) in wanted
                .iter()
                .filter(|(key, told)| held.get(*key) != Some(told))
            {
                let (score, shared, method) = told;
                like.execute(rusqlite::params![
                    a,
                    b,
                    *score as f64 / 1000.0,
                    shared,
                    method
                ])?;
                written += 1;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO meta VALUES (?, ?)",
            [PASSED, &weighed.turn.to_string()],
        )?;
        tx.commit()?;
        Ok(Passed::Ran {
            nodes: weighed.ids.len(),
            rows: wanted.len(),
            written,
            steps: weighed.found.steps,
            common_from: weighed.found.common_from,
            typos: weighed.found.typos,
            translated: weighed.found.translated,
        })
    }

    // The base keeps everything, so the reader brings its gate: both ends pass it or nothing shows.
    pub fn alike(&self, id: &str, seen: impl Fn(&str) -> bool) -> rusqlite::Result<Vec<Alike>> {
        let tx = self.db.unchecked_transaction()?;
        let asked = id
            .parse::<DocId>()
            .map_or_else(|_| id.to_string(), |one| one.to_string());
        let own: Option<String> = tx
            .query_row(
                "SELECT id FROM node WHERE id = ?1 UNION ALL SELECT id FROM node WHERE uid = ?1 LIMIT 1",
                [&asked],
                |row| row.get(0),
            )
            .optional()?;
        let Some(own) = own.filter(|one| seen(one)) else {
            return Ok(Vec::new());
        };
        let mut rows = tx.prepare(
            "SELECT b, score, shared, method FROM alike WHERE a = ?1
             UNION ALL SELECT a, score, shared, method FROM alike WHERE b = ?1
             ORDER BY 2 DESC, 1",
        )?;
        let all = rows
            .query_map([&own], |row| {
                let shared: String = row.get(2)?;
                Ok(Alike {
                    id: row.get(0)?,
                    score: row.get(1)?,
                    shared: shared.split(' ').map(str::to_string).collect(),
                    method: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<Alike>>>()?;
        Ok(all.into_iter().filter(|one| seen(&one.id)).collect())
    }
}

fn thousandths(score: f64) -> i64 {
    (score * 1000.0).round() as i64
}

fn turn_of(db: &Connection, key: &str) -> rusqlite::Result<Option<u64>> {
    Ok(said(db, key)?.and_then(|one| one.parse().ok()))
}

fn ids(db: &Connection, sql: &str) -> rusqlite::Result<HashSet<String>> {
    let mut asked = db.prepare(sql)?;
    let rows = asked.query_map([], |row| row.get(0))?;
    rows.collect()
}

fn worded(db: &Connection) -> rusqlite::Result<Worded> {
    let mut asked = db.prepare("SELECT id, lang, kin, said, guess FROM words ORDER BY id")?;
    let rows = asked.query_map([], |row| {
        let kin: String = row.get(2)?;
        let said: String = row.get(3)?;
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(4)?,
            Item {
                lang: row.get(1)?,
                kin: kin
                    .split('\u{1}')
                    .filter(|one| !one.is_empty())
                    .map(str::to_string)
                    .collect(),
                words: words::read(&said),
            },
        ))
    })?;
    let mut worded = Worded {
        ids: Vec::new(),
        items: Vec::new(),
        guesses: Vec::new(),
    };
    for row in rows {
        let (id, guess, item) = row?;
        worded.ids.push(id);
        worded.guesses.push(guess);
        worded.items.push(item);
    }
    Ok(worded)
}

// An edge names its target by id, by the document's uid or by `[[title`, so each is read back to a node.
fn tied(db: &Connection, ids: &[String]) -> rusqlite::Result<HashSet<(usize, usize)>> {
    let at: HashMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(n, id)| (id.as_str(), n))
        .collect();
    let mut by_uid: HashMap<String, usize> = HashMap::new();
    let mut by_title: HashMap<String, Option<usize>> = HashMap::new();
    let mut nodes = db.prepare("SELECT id, uid, title, kind FROM node")?;
    let rows = nodes.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (id, uid, title, kind) = row?;
        // Titles are read among what every reader may see, so a pair any card shows tied is never guessed.
        let Some(&n) = at.get(id.as_str()) else {
            continue;
        };
        if let Some(uid) = uid {
            by_uid.insert(uid, n);
        }
        if kind == Kind::Doc.as_str() || kind == Kind::Page.as_str() {
            by_title
                .entry(title)
                .and_modify(|one| *one = None)
                .or_insert(Some(n));
        }
    }
    let node_of = |target: &str| {
        at.get(target)
            .copied()
            .or_else(|| by_uid.get(target).copied())
            .or_else(|| {
                let title = target.strip_prefix(TITLED)?;
                by_title.get(title).copied().flatten()
            })
    };
    let mut edges = db.prepare(
        "SELECT source, target FROM edge WHERE kind IN ('cite', 'mention', 'structure')",
    )?;
    let rows = edges.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut tied = HashSet::new();
    for row in rows {
        let (source, target) = row?;
        if let (Some(a), Some(b)) = (at.get(source.as_str()).copied(), node_of(&target))
            && a != b
        {
            tied.insert((a.min(b), a.max(b)));
        }
    }
    Ok(tied)
}

type Held = HashMap<(String, String), (i64, String, String)>;

fn held_alike(db: &Connection) -> rusqlite::Result<Held> {
    let mut asked = db.prepare("SELECT a, b, score, shared, method FROM alike")?;
    let rows = asked.query_map([], |row| {
        Ok((
            (row.get(0)?, row.get(1)?),
            (thousandths(row.get(2)?), row.get(3)?, row.get(4)?),
        ))
    })?;
    rows.collect()
}

#[cfg(test)]
#[path = "alike_test.rs"]
mod tests;
