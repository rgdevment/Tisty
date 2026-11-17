use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use tisty_markdown::refs::{DOC, TASK, link_spots};
use tisty_markdown::sections;

use super::linked::paper_named;
use super::{Brain, FILE, SCHEMA, WAITS_FOR_OTHERS, schema_of};
use crate::model::DocId;
use crate::witness::{self, Fact, channel};

const NAMED: &str = "SELECT id, uid FROM node WHERE id = ?1
    UNION ALL SELECT id, uid FROM node WHERE uid = ?1 LIMIT 1";
const CITING: &str = "SELECT n.id, n.kind, n.title, n.state, e.kind, n.stamp FROM edge e
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

const HEADED_AT_MOST: usize = 4096;

// Kept by the node's stamp, so a body is read again only once a round has seen it move.
#[derive(Debug, Default)]
pub struct Headed {
    bodies: HashMap<String, (String, Vec<(String, Under)>)>,
}

impl Headed {
    fn of(&mut self, id: &str, stamp: &str, docs: &Path) -> &[(String, Under)] {
        let fresh = self.bodies.get(id).is_some_and(|(had, _)| had == stamp);
        if !fresh {
            if self.bodies.len() >= HEADED_AT_MOST {
                self.bodies.clear();
            }
            let said = crate::docs::read(docs, id)
                .map(|body| headed(&body))
                .unwrap_or_default();
            self.bodies
                .insert(id.to_string(), (stamp.to_string(), said));
        }
        self.bodies.get(id).map_or(&[], |(_, said)| said.as_slice())
    }
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

    pub fn cited_by(
        &self,
        id: &str,
        docs: &Path,
        headed: &mut Headed,
    ) -> rusqlite::Result<Option<Vec<Citing>>> {
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
        let mut all: Vec<(Citing, String)> = Vec::new();
        {
            let mut asked = tx.prepare(CITING)?;
            let rows = asked.query_map([aimed[0].as_str(), aimed[1].as_str()], |row| {
                let one = Citing {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    title: row.get(2)?,
                    state: row.get(3)?,
                    card: row.get::<_, String>(4)? == "cite",
                    under: Vec::new(),
                };
                Ok((one, row.get(5)?))
            })?;
            for row in rows {
                let (one, stamp) = row?;
                match all.iter_mut().find(|(had, _)| had.id == one.id) {
                    Some((had, _)) => had.card |= one.card,
                    None => all.push((one, stamp)),
                }
            }
        }
        drop(tx);
        for (one, stamp) in all.iter_mut().filter(|(one, _)| one.kind != "task") {
            one.under = under(headed.of(&one.id, stamp, docs), &aimed);
        }
        Ok(Some(all.into_iter().map(|(one, _)| one).collect()))
    }
}

pub fn cited_by(paths: &crate::Paths, id: &str, headed: &mut Headed) -> Option<Vec<Citing>> {
    let brain = Brain::looked_at(paths.cache())?;
    brain
        .cited_by(id, &paths.docs(), headed)
        .inspect_err(|e| unread("the brain could not say who cites it", &e.to_string()))
        .ok()
        .flatten()
}

// Places and headings are both read from the translator's text, so front matter and CRLF shift neither.
fn headed(body: &str) -> Vec<(String, Under)> {
    let doc = tisty_markdown::read(body);
    let heads = sections(&doc);
    link_spots(doc.text())
        .into_iter()
        .filter_map(|(link, at)| {
            let named = link
                .target
                .strip_prefix(DOC)
                .or_else(|| link.target.strip_prefix(TASK))?;
            let head = heads.iter().rev().find(|head| head.at.start <= at)?;
            let under = Under {
                id: head.id.clone(),
                title: head.said.clone(),
            };
            Some((paper_named(named), under))
        })
        .collect()
}

fn under(headed: &[(String, Under)], aimed: &[String; 2]) -> Vec<Under> {
    let mut found: Vec<Under> = Vec::new();
    for (named, under) in headed {
        if aimed.contains(named) && !found.iter().any(|one| one.id == under.id) {
            found.push(under.clone());
        }
    }
    found
}

fn unread(said: &'static str, why: &str) {
    witness::trace(channel::BRAIN, said, &[("why", Fact::Why(why.to_string()))]);
}

#[cfg(test)]
#[path = "reading_test.rs"]
mod tests;
