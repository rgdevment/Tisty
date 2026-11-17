use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::Brain;
use super::linked::paper_named;
use super::own::Own;
use crate::model::Priority;
use crate::standing::{Standing, standing_on};

const ROW: &str = "SELECT id, kind, title, grp, state, uid FROM node WHERE id = ?1
    UNION ALL SELECT id, kind, title, grp, state, uid FROM node WHERE uid = ?1 LIMIT 1";
const OPEN: [&str; 3] = ["open", "said_done", "said_dropped"];

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

// Lines take both ends as `read_doc` does; bytes are half-open so `&body[from..to]` is the section.
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
pub struct Near {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ideas_from: Option<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ideas: Vec<Near>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
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
    // The base keeps everything, so each reader brings its own gate in `seen`.
    pub fn card(
        &self,
        id: &str,
        today: Date,
        seen: impl Fn(&str) -> bool,
    ) -> rusqlite::Result<Option<Card>> {
        let reader = Reader { db: &self.db, seen };
        let Some(row) = reader.row(id)? else {
            return Ok(None);
        };
        let said: Option<String> = self
            .db
            .query_row("SELECT said FROM card WHERE id = ?", [&row.id], |one| {
                one.get(0)
            })
            .optional()?;
        let Some(said) = said else {
            return Ok(None);
        };
        let zone = match &row.grp {
            Some(grp) => self
                .db
                .query_row("SELECT name FROM zone WHERE id = ?", [grp], |one| {
                    one.get(0)
                })
                .optional()?,
            None => None,
        };
        match row.kind.as_str() {
            "task" => reader.task(row, &said, zone, today),
            "doc" | "page" => reader.doc(row, &said, zone),
            _ => Ok(None),
        }
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
        let placed = [own.book.as_deref()];
        self.ties(&row, &own, &placed, &mut around)?;
        Ok(Some(Card {
            folder: zone,
            list: None,
            standing: None,
            facts: Facts::Doc(own.facts),
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
        let Some(own) = parsed::<TaskFacts>(said) else {
            return Ok(None);
        };
        let origin = own
            .facts
            .source
            .as_deref()
            .and_then(|one| one.split_once('#'))
            .map(|(named, _)| paper_named(named.trim()));
        let mut around = Around {
            part_of: own
                .part_of
                .as_deref()
                .map(|one| self.near(one))
                .transpose()?
                .flatten(),
            ..Around::default()
        };
        let placed = [own.part_of.as_deref(), origin.as_deref()];
        self.ties(&row, &own, &placed, &mut around)?;
        if let Some(origin) = origin.as_deref() {
            around.origin = self.tied_out(&row.id, origin)?;
        }
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

    // Book, whole and origin come from the node's own record, so the edges that say them again are passed over.
    fn ties<T>(
        &self,
        row: &Row,
        own: &Own<T>,
        placed: &[Option<&str>],
        around: &mut Around,
    ) -> rusqlite::Result<()> {
        let mut out = self.db.prepare_cached(
            "SELECT target, kind FROM edge WHERE source = ?1 ORDER BY kind, target",
        )?;
        let outgoing: Vec<(String, String)> = out
            .query_map([&row.id], |one| Ok((one.get(0)?, one.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (target, kind) in outgoing {
            let Some(near) = self.near(&target)? else {
                continue;
            };
            match kind.as_str() {
                "cite" => added(&mut around.cites, near),
                "mention" => added(&mut around.mentions, near),
                "tag" => around.tags.push(near.title),
                "uses" => around.files.push(near.id),
                "structure" if !placed.contains(&Some(target.as_str())) => {
                    around.ideas_from = Some(near);
                }
                _ => {}
            }
        }
        let mut into = self.db.prepare_cached(
            "SELECT source, kind FROM edge WHERE target IN (?1, ?2) ORDER BY kind, source",
        )?;
        let uid = row.uid.as_deref().unwrap_or(&row.id);
        let incoming: Vec<(String, String)> = into
            .query_map([row.id.as_str(), uid], |one| Ok((one.get(0)?, one.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (source, kind) in incoming {
            let Some(near) = self.near(&source)? else {
                continue;
            };
            match kind.as_str() {
                "cite" => added(&mut around.cited_by, near),
                "mention" => added(&mut around.mentioned_by, near),
                "structure" if own.pages.contains(&source) => {}
                "structure" if near.kind == "task" && row.kind == "task" => {
                    added(&mut around.parts, near);
                }
                "structure" => added(&mut around.ideas, near),
                _ => {}
            }
        }
        Ok(())
    }

    fn tied_out(&self, source: &str, target: &str) -> rusqlite::Result<Option<Near>> {
        let tied = self
            .db
            .query_row(
                "SELECT 1 FROM edge WHERE source = ?1 AND target = ?2 AND kind = 'structure'",
                [source, target],
                |_| Ok(()),
            )
            .optional()?;
        match tied {
            Some(()) => self.near(target),
            None => Ok(None),
        }
    }
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
