use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::Brain;
use super::decided::Decision;
use super::linked::{TITLED, paper_named};
use super::own::Own;
use crate::model::{DocId, Priority};
use crate::standing::{Standing, standing_on};

const ROW: &str = "SELECT id, kind, title, grp, state, uid FROM node WHERE id = ?1
    UNION ALL SELECT id, kind, title, grp, state, uid FROM node WHERE uid = ?1 LIMIT 1";
const OUTGOING: &str = "SELECT e.kind, e.target, COALESCE(a.id, b.id), COALESCE(a.kind, b.kind),
        COALESCE(a.title, b.title), CASE WHEN a.id IS NULL THEN b.state ELSE a.state END
    FROM edge e
    LEFT JOIN node a ON a.id = e.target
    LEFT JOIN node b ON a.id IS NULL AND b.uid = e.target
    WHERE e.source = ?1 AND COALESCE(a.id, b.id) IS NOT NULL
    ORDER BY e.kind, e.target";
const INCOMING: &str = "SELECT e.kind, n.id, n.kind, n.title, n.state, c.said
    FROM edge e JOIN node n ON n.id = e.source
    LEFT JOIN card c ON c.id = e.source AND e.kind = 'structure' AND n.kind = 'task'
    WHERE e.target IN (?1, ?2, ?3)
    ORDER BY e.kind, e.source";
const TITLES_AIMED: &str = "SELECT DISTINCT substr(target, 3) FROM edge
    WHERE source = ?1 AND kind = 'mention' AND substr(target, 1, 2) = '[['
    ORDER BY 1";
const GUESSED: &str = "SELECT b, score, shared FROM alike WHERE a = ?1 AND method = 'words'
    UNION ALL SELECT a, score, shared FROM alike WHERE b = ?1 AND method = 'words'
    ORDER BY 2 DESC, 1";
pub(super) const CARRYING: &str =
    "SELECT id, kind, title, state FROM node WHERE kind IN ('doc', 'page') AND title = ?1";

type Incoming = (String, Near, Option<String>);
const OPEN: [&str; 3] = ["open", "said_done", "said_dropped"];
const DECISIONS_SHOWN: usize = 60;

// Fields serialize in this order, so what every agent needs comes first and the longest last.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Card {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standing: Option<Standing>,
    #[serde(flatten)]
    pub facts: Facts,
    #[serde(flatten)]
    pub around: Around,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub guessed: Vec<Guessed>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<Decision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decisions_more: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Facts {
    Doc(DocFacts),
    Task(TaskFacts),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DocFacts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub made: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrote: Option<jiff::Timestamp>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskFacts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<Date>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<[usize; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<String>,
}

// Lines take both ends as `read` does; bytes are half-open so `&body[from..to]` is the section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub level: usize,
    pub lines: [usize; 2],
    pub bytes: [usize; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tongue {
    pub lang: String,
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Used {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shows: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Near {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

// Tisty's guess from the words both share, apart from what anyone wrote.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Guessed {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub score: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shared: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Around {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book: Option<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<Tongue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of: Option<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<Near>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub born: Vec<Near>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ideas_from: Option<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ideas: Vec<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<Used>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cites: Vec<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mentions: Vec<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cited_by: Vec<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mentioned_by: Vec<Near>,
}

struct Row {
    id: String,
    kind: String,
    title: String,
    grp: Option<String>,
    state: Option<String>,
    uid: Option<String>,
}

impl Row {
    fn near(self) -> Near {
        Near {
            id: self.id,
            kind: self.kind,
            title: self.title,
            state: self.state,
        }
    }
}

struct Reader<'a, F> {
    db: &'a Connection,
    seen: F,
}

impl Brain {
    // The base keeps everything, so each reader brings its gate in `seen`: true for what it does not know, as a source may name a chat.
    pub fn card(
        &self,
        id: &str,
        today: Date,
        seen: impl Fn(&str) -> bool,
    ) -> rusqlite::Result<Option<Card>> {
        // One snapshot, so a round that commits meanwhile never mixes two versions in one card.
        let tx = self.db.unchecked_transaction()?;
        let id = id
            .parse::<DocId>()
            .map_or_else(|_| id.to_string(), |one| one.to_string());
        let reader = Reader { db: &tx, seen };
        let Some(row) = reader.row(&id)? else {
            return Ok(None);
        };
        let Some(said) = reader.said(&row.id)? else {
            return Ok(None);
        };
        let zone = match &row.grp {
            Some(grp) => tx
                .query_row("SELECT name FROM zone WHERE id = ?", [grp], |one| {
                    one.get(0)
                })
                .optional()?,
            None => None,
        };
        let card = match row.kind.as_str() {
            "task" => reader.task(row, &said, zone, today)?,
            "doc" | "page" => reader.doc(row, &said, zone)?,
            _ => None,
        };
        card.map(|mut card| {
            card.guessed = reader.guessed(&card.id)?;
            Ok(card)
        })
        .transpose()
    }
}

impl<F: Fn(&str) -> bool> Reader<'_, F> {
    fn row(&self, id: &str) -> rusqlite::Result<Option<Row>> {
        let mut asked = self.db.prepare_cached(ROW)?;
        let row = asked
            .query_row([id], |one| {
                Ok(Row {
                    id: one.get(0)?,
                    kind: one.get(1)?,
                    title: one.get(2)?,
                    grp: one.get(3)?,
                    state: one.get(4)?,
                    uid: one.get(5)?,
                })
            })
            .optional()?;
        Ok(row.filter(|one| (self.seen)(&one.id)))
    }

    fn near(&self, id: &str) -> rusqlite::Result<Option<Near>> {
        Ok(self.row(id)?.map(Row::near))
    }

    fn said(&self, id: &str) -> rusqlite::Result<Option<String>> {
        let mut asked = self
            .db
            .prepare_cached("SELECT said FROM card WHERE id = ?")?;
        asked.query_row([id], |one| one.get(0)).optional()
    }

    fn doc(&self, row: Row, said: &str, zone: Option<String>) -> rusqlite::Result<Option<Card>> {
        let Some(own) = parsed::<DocFacts>(said) else {
            return Ok(None);
        };
        let mut around = Around {
            book: own
                .book
                .as_deref()
                .map(|one| self.near(one))
                .transpose()?
                .flatten(),
            ..Around::default()
        };
        for page in &own.pages {
            if let Some(near) = self.near(page)? {
                around.pages.push(near);
            }
        }
        for tongue in &own.languages {
            if self.row(&tongue.doc)?.is_some() {
                around.languages.push(tongue.clone());
            }
        }
        self.ties(&row, &own, own.book.as_deref(), None, &mut around)?;
        let mut decisions = own.decisions;
        decisions.sort_by_key(|one| one.replaced);
        let more = decisions.len().saturating_sub(DECISIONS_SHOWN);
        decisions.truncate(DECISIONS_SHOWN);
        Ok(Some(Card {
            folder: zone,
            list: None,
            standing: None,
            facts: Facts::Doc(own.facts),
            decisions,
            decisions_more: (more > 0).then_some(more),
            sections: own.sections,
            ..carded(row, around)
        }))
    }

    fn task(
        &self,
        row: Row,
        said: &str,
        zone: Option<String>,
        today: Date,
    ) -> rusqlite::Result<Option<Card>> {
        let Some(mut own) = parsed::<TaskFacts>(said) else {
            return Ok(None);
        };
        let origin = origin_of(&own.facts);
        // Asked of the gate, not the base, so a hidden document not yet indexed is not named either.
        if origin.as_deref().is_some_and(|one| !(self.seen)(one)) {
            own.facts.source = None;
        }
        let mut around = Around {
            part_of: own
                .part_of
                .as_deref()
                .map(|one| self.near(one))
                .transpose()?
                .flatten(),
            ..Around::default()
        };
        self.ties(
            &row,
            &own,
            own.part_of.as_deref(),
            origin.as_deref(),
            &mut around,
        )?;
        let open = row.state.as_deref().is_some_and(|one| OPEN.contains(&one));
        let standing = standing_on(own.facts.date, own.facts.deadline, today);
        Ok(Some(Card {
            folder: None,
            list: zone,
            standing: (open && standing != Standing::Undated).then_some(standing),
            facts: Facts::Task(own.facts),
            sections: Vec::new(),
            ..carded(row, around)
        }))
    }

    // One `structure` edge means a book, a whole, an origin or an idea, so the record tells them apart.
    fn ties<T>(
        &self,
        row: &Row,
        own: &Own<T>,
        placed: Option<&str>,
        origin: Option<&str>,
        around: &mut Around,
    ) -> rusqlite::Result<()> {
        for (kind, target, near) in self.outgoing(&row.id)? {
            match kind.as_str() {
                "cite" => added(&mut around.cites, near),
                "mention" => added(&mut around.mentions, near),
                "tag" => around.tags.push(near.title),
                "uses" => around.files.push(Used {
                    shows: own.shows.get(&near.id).cloned(),
                    file: near.id,
                }),
                "structure" if origin == Some(target.as_str()) => around.origin = Some(near),
                "structure" if placed != Some(target.as_str()) => around.ideas_from = Some(near),
                _ => {}
            }
        }
        for title in self.titles_aimed(&row.id)? {
            if let Some(near) = self.carrying(&title)?.filter(|one| one.id != row.id) {
                added(&mut around.mentions, near);
            }
        }
        let uid = row.uid.as_deref().unwrap_or(&row.id);
        let titled = match row.kind.as_str() {
            "doc" | "page"
                if self
                    .carrying(&row.title)?
                    .is_some_and(|one| one.id == row.id) =>
            {
                format!("{TITLED}{}", row.title)
            }
            _ => uid.to_string(),
        };
        let others = self.incoming(&row.id, uid, &titled)?.into_iter();
        for (kind, near, said) in others.filter(|(_, near, _)| near.id != row.id) {
            match kind.as_str() {
                "cite" => added(&mut around.cited_by, near),
                "mention" => added(&mut around.mentioned_by, near),
                "structure" if own.pages.contains(&near.id) => {}
                "structure" => match hung(said.as_deref(), row) {
                    Some(Hung::Part) => added(&mut around.parts, near),
                    Some(Hung::Born) => added(&mut around.born, near),
                    None => added(&mut around.ideas, near),
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn outgoing(&self, id: &str) -> rusqlite::Result<Vec<(String, String, Near)>> {
        let mut asked = self.db.prepare_cached(OUTGOING)?;
        let all: Vec<(String, String, Near)> = asked
            .query_map([id], |one| {
                let near = Near {
                    id: one.get(2)?,
                    kind: one.get(3)?,
                    title: one.get(4)?,
                    state: one.get(5)?,
                };
                Ok((one.get(0)?, one.get(1)?, near))
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(all
            .into_iter()
            .filter(|(_, _, near)| (self.seen)(&near.id))
            .collect())
    }

    // Both ends pass the gate, or the guess is not there: a hidden task is never named through it.
    fn guessed(&self, id: &str) -> rusqlite::Result<Vec<Guessed>> {
        let mut asked = self.db.prepare_cached(GUESSED)?;
        let rows: Vec<(String, f64, String)> = asked
            .query_map([id], |one| Ok((one.get(0)?, one.get(1)?, one.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut all = Vec::new();
        for (other, score, shared) in rows {
            let Some(near) = self.near(&other)? else {
                continue;
            };
            all.push(Guessed {
                id: near.id,
                kind: near.kind,
                title: near.title,
                state: near.state,
                score: (score * 100.0).round() / 100.0,
                shared: shared
                    .split(' ')
                    .filter(|one| !one.is_empty())
                    .map(str::to_string)
                    .collect(),
            });
        }
        Ok(all)
    }

    fn titles_aimed(&self, id: &str) -> rusqlite::Result<Vec<String>> {
        let mut asked = self.db.prepare_cached(TITLES_AIMED)?;
        asked.query_map([id], |one| one.get(0))?.collect()
    }

    // Only what this reader may see is counted, so a hidden twin neither shows nor gives itself away.
    fn carrying(&self, title: &str) -> rusqlite::Result<Option<Near>> {
        let mut asked = self.db.prepare_cached(CARRYING)?;
        let mut all: Vec<Near> = asked
            .query_map([title], |one| {
                Ok(Near {
                    id: one.get(0)?,
                    kind: one.get(1)?,
                    title: one.get(2)?,
                    state: one.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<Near>>>()?
            .into_iter()
            .filter(|near| (self.seen)(&near.id))
            .collect();
        Ok(match all.len() {
            1 => all.pop(),
            _ => None,
        })
    }

    fn incoming(&self, id: &str, uid: &str, titled: &str) -> rusqlite::Result<Vec<Incoming>> {
        let mut asked = self.db.prepare_cached(INCOMING)?;
        let all: Vec<Incoming> = asked
            .query_map([id, uid, titled], |one| {
                let near = Near {
                    id: one.get(1)?,
                    kind: one.get(2)?,
                    title: one.get(3)?,
                    state: one.get(4)?,
                };
                Ok((one.get(0)?, near, one.get(5)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(all
            .into_iter()
            .filter(|(_, near, _)| (self.seen)(&near.id))
            .collect())
    }
}

enum Hung {
    Part,
    Born,
}

// One edge ties a task to its whole, its origin or an idea's sheet; only its record tells which.
fn hung(said: Option<&str>, row: &Row) -> Option<Hung> {
    let own = parsed::<TaskFacts>(said?)?;
    let names = |one: &str| one == row.id || row.uid.as_deref() == Some(one);
    if own.part_of.as_deref().is_some_and(names) {
        return Some(Hung::Part);
    }
    origin_of(&own.facts)
        .is_some_and(|one| names(&one))
        .then_some(Hung::Born)
}

fn origin_of(facts: &TaskFacts) -> Option<String> {
    let (named, _) = facts.source.as_deref()?.split_once('#')?;
    Some(paper_named(named.trim()))
}

fn carded(row: Row, around: Around) -> Card {
    Card {
        id: row.id,
        kind: row.kind,
        title: row.title,
        folder: None,
        list: None,
        state: row.state.unwrap_or_default(),
        standing: None,
        facts: Facts::Doc(DocFacts::default()),
        around,
        guessed: Vec::new(),
        decisions: Vec::new(),
        decisions_more: None,
        sections: Vec::new(),
    }
}

fn parsed<T: DeserializeOwned>(said: &str) -> Option<Own<T>> {
    serde_json::from_str(said).ok()
}

fn added(all: &mut Vec<Near>, near: Near) {
    if !all.iter().any(|one| one.id == near.id) {
        all.push(near);
    }
}

#[cfg(test)]
#[path = "card_test.rs"]
mod tests;
