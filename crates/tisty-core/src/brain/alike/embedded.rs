use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};

use super::super::linked::Ground;
use super::super::{Brain, Docs, nodes, warned};
use super::meant::Meant;
use super::{said_of, turn_of};
use crate::Paths;

pub(super) const MEANT: &str = "meant_turn";
pub(super) const BYTES_PER_PASS: usize = 2 * 1024 * 1024;
const TEXT_AT_MOST: usize = 512 * 1024;

// Only the app holds one, so the door never loads the model.
pub trait Meaning {
    fn version(&self) -> &str;
    fn meaning(&mut self, text: &str) -> Option<Vec<f32>>;
    fn refused(&self) -> bool {
        false
    }
    fn rest(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Embedded {
    pub(crate) written: usize,
    pub(crate) left: usize,
}

struct Fresh {
    id: String,
    stamp: String,
    meant: Meant,
}

impl Brain {
    pub(super) fn unmeant(&self, version: &str) -> rusqlite::Result<Vec<(String, String)>> {
        let mut asked = self.db.prepare(
            "SELECT w.id, w.stamp FROM words w LEFT JOIN vector v ON v.id = w.id
             WHERE v.id IS NULL OR v.stamp <> w.stamp OR v.model <> ?1 ORDER BY w.id",
        )?;
        let rows = asked.query_map([version], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    pub(super) fn embed(
        &mut self,
        paths: &Paths,
        meaning: &mut dyn Meaning,
        budget: usize,
    ) -> rusqlite::Result<Option<Embedded>> {
        if meaning.refused() {
            return Ok(None);
        }
        let version = meaning.version().to_string();
        let pending = self.unmeant(&version)?;
        let orphans: i64 = self.db.query_row(
            "SELECT count(*) FROM vector WHERE id NOT IN (SELECT id FROM words)",
            [],
            |row| row.get(0),
        )?;
        if pending.is_empty() && orphans == 0 {
            return Ok(None);
        }
        let (fresh, left) = match pending.is_empty() {
            true => (Vec::new(), 0),
            false => meant(paths, meaning, &pending, budget),
        };
        if left == 0 {
            meaning.rest();
        }
        let written = self.kept_meant(&fresh, &version)?;
        Ok(Some(Embedded { written, left }))
    }

    // Read outside the lock: a vector is kept only if its node still has words with that stamp.
    fn kept_meant(&mut self, fresh: &[Fresh], version: &str) -> rusqlite::Result<usize> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut moved = tx.execute(
            "DELETE FROM vector WHERE id NOT IN (SELECT id FROM words)",
            [],
        )? > 0;
        let mut written = 0;
        {
            let mut worded = tx.prepare("SELECT stamp FROM words WHERE id = ?")?;
            let mut held = tx.prepare("SELECT said, model FROM vector WHERE id = ?")?;
            let mut put = tx.prepare("INSERT OR REPLACE INTO vector VALUES (?, ?, ?, ?)")?;
            for one in fresh {
                let stamp: Option<String> =
                    worded.query_row([&one.id], |row| row.get(0)).optional()?;
                if stamp.as_deref() != Some(one.stamp.as_str()) {
                    continue;
                }
                let said = one.meant.said();
                let before: Option<(Vec<u8>, String)> = held
                    .query_row([&one.id], |row| Ok((row.get(0)?, row.get(1)?)))
                    .optional()?;
                put.execute(rusqlite::params![one.id, one.stamp, version, said])?;
                written += 1;
                moved |= before != Some((said, version.to_string()));
            }
        }
        if moved {
            let turn = turn_of(&tx, MEANT)?.unwrap_or(0) + 1;
            tx.execute(
                "INSERT OR REPLACE INTO meta VALUES (?, ?)",
                [MEANT, &turn.to_string()],
            )?;
        }
        tx.commit()?;
        Ok(written)
    }
}

fn meant(
    paths: &Paths,
    meaning: &mut dyn Meaning,
    pending: &[(String, String)],
    budget: usize,
) -> (Vec<Fresh>, usize) {
    let state = match crate::cache::project(&paths.store(), paths.cache()) {
        Ok(state) => state,
        Err(e) => {
            warned(
                "the brain could not read the store for what things mean",
                &paths.store(),
                &e.to_string(),
            );
            return (Vec::new(), 0);
        }
    };
    let docs = Docs::on_disk(&paths.docs());
    let ground = Ground::new(&state, &docs);
    let wanted = nodes::nodes(&state, &docs, &ground);
    let mut fresh = Vec::new();
    let mut spent = 0;
    for (at, (id, stamp)) in pending.iter().enumerate() {
        if spent >= budget {
            return (fresh, pending.len() - at);
        }
        let Some(node) = wanted.get(id).filter(|node| &node.stamp == stamp) else {
            continue;
        };
        let read = ground.read(id, &node.holder);
        let whole = said_of(&node.holder, read.body.as_deref()).join("\n");
        let text = cut(&whole);
        spent += text.len().max(1);
        let Some(vector) = meaning.meaning(text) else {
            return (fresh, 0);
        };
        fresh.push(Fresh {
            id: id.clone(),
            stamp: stamp.clone(),
            meant: Meant::of(&vector),
        });
    }
    (fresh, 0)
}

fn cut(text: &str) -> &str {
    if text.len() <= TEXT_AT_MOST {
        return text;
    }
    let mut end = TEXT_AT_MOST;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

pub(super) fn vectors(db: &Connection, version: &str) -> rusqlite::Result<HashMap<String, Meant>> {
    let mut asked = db.prepare(
        "SELECT v.id, v.said FROM vector v JOIN words w ON w.id = v.id
         WHERE v.stamp = w.stamp AND v.model = ?1",
    )?;
    let rows = asked.query_map([version], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
    })?;
    let mut all = HashMap::new();
    for row in rows {
        let (id, said) = row?;
        if let Some(meant) = Meant::read(&said) {
            all.insert(id, meant);
        }
    }
    Ok(all)
}

#[cfg(test)]
#[path = "embedded_test.rs"]
mod tests;
