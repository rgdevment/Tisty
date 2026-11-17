use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use tisty_markdown::refs::{DOC, TASK, link_spots};
use tisty_markdown::{Kind, outline};

use super::linked::paper_named;
use super::{Brain, FILE, SCHEMA, WAITS_FOR_OTHERS, schema_of};
use crate::model::DocId;
use crate::witness::{self, Fact, channel};

const NAMED: &str = "SELECT id, uid FROM node WHERE id = ?1
    UNION ALL SELECT id, uid FROM node WHERE uid = ?1 LIMIT 1";
const CITING: &str = "SELECT n.id, n.kind, n.title, n.state, e.kind FROM edge e
    JOIN node n ON n.id = e.source
    WHERE e.kind IN ('cite', 'mention') AND e.target IN (?1, ?2)
    ORDER BY n.kind = 'task', n.title COLLATE NOCASE, n.id";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Citing {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub card: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub under: Vec<Under>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Under {
    pub id: String,
    pub title: String,
}

impl Brain {
    pub fn looked_at(cache_dir: &Path) -> Option<Self> {
        let at = cache_dir.join(FILE);
        let opened = || -> rusqlite::Result<Option<Self>> {
            let db = Connection::open_with_flags(
                &at,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?;
            db.busy_timeout(WAITS_FOR_OTHERS)?;
            Ok((schema_of(&db)? == Some(SCHEMA)).then_some(Self { db }))
        };
        opened()
            .inspect_err(|e| unread("the brain could not be read", &e.to_string()))
            .ok()
            .flatten()
    }

    pub fn cited_by(&self, id: &str, docs: &Path) -> rusqlite::Result<Option<Vec<Citing>>> {
        if self.seen().is_none() {
            return Ok(None);
        }
        let id = id
            .parse::<DocId>()
            .map_or_else(|_| id.to_string(), |one| one.to_string());
        let tx = self.db.unchecked_transaction()?;
        let (file, uid) = tx
            .query_row(NAMED, [&id], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional()?
            .unwrap_or_else(|| (id.clone(), None::<String>));
        let aimed = [file.clone(), uid.unwrap_or(file)];
        let mut all: Vec<Citing> = Vec::new();
        {
            let mut asked = tx.prepare(CITING)?;
            let rows = asked.query_map([aimed[0].as_str(), aimed[1].as_str()], |row| {
                Ok(Citing {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    title: row.get(2)?,
                    state: row.get(3)?,
                    card: row.get::<_, String>(4)? == "cite",
                    under: Vec::new(),
                })
            })?;
            for row in rows {
                let one = row?;
                match all.iter_mut().find(|had| had.id == one.id) {
                    Some(had) => had.card |= one.card,
                    None => all.push(one),
                }
            }
        }
        drop(tx);
        for one in all.iter_mut().filter(|one| one.kind != "task") {
            if let Ok(body) = crate::docs::read(docs, &one.id) {
                one.under = under(&body, &aimed);
            }
        }
        Ok(Some(all))
    }
}

pub fn cited_by(paths: &crate::Paths, id: &str) -> Option<Vec<Citing>> {
    let brain = Brain::looked_at(paths.cache())?;
    brain
        .cited_by(id, &paths.docs())
        .inspect_err(|e| unread("the brain could not say who cites it", &e.to_string()))
        .ok()
        .flatten()
}

// Places and headings are both read from the translator's text, so front matter and CRLF shift neither.
fn under(body: &str, aimed: &[String; 2]) -> Vec<Under> {
    let doc = tisty_markdown::read(body);
    let heads = outline(&doc);
    let title = doc
        .blocks()
        .iter()
        .find(|block| block.kind != Kind::Front)
        .filter(|block| block.kind == Kind::Heading(1))
        .map(|block| block.at.start);
    let mut found: Vec<Under> = Vec::new();
    for (link, at) in link_spots(doc.text()) {
        let named = link
            .target
            .strip_prefix(DOC)
            .or_else(|| link.target.strip_prefix(TASK));
        if !named.is_some_and(|one| aimed.contains(&paper_named(one))) {
            continue;
        }
        let Some(head) = heads.iter().rev().find(|head| head.at.start <= at) else {
            continue;
        };
        if Some(head.at.start) == title || found.iter().any(|one| one.id == head.id) {
            continue;
        }
        found.push(Under {
            id: head.id.clone(),
            title: head.said.clone(),
        });
    }
    found
}

fn unread(said: &'static str, why: &str) {
    witness::trace(channel::BRAIN, said, &[("why", Fact::Why(why.to_string()))]);
}

#[cfg(test)]
#[path = "reading_test.rs"]
mod tests;
