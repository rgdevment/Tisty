use std::collections::{BTreeMap, BTreeSet};

use crate::model::{DocId, Kept};
use crate::state::State;

// Each language is a document of its own; the log holds them around the anchor the others name.
impl State {
    /// The document its group gathers around: itself when it names none.
    pub fn anchor_of(&self, kept: &Kept) -> DocId {
        kept.translation_of.unwrap_or(kept.id)
    }

    /// The languages of its group in the order they joined; empty when it stands alone.
    pub fn tongues_of(&self, id: DocId) -> Vec<&Kept> {
        let Some(kept) = self.docs.get(&id).filter(|one| grouping(one)) else {
            return Vec::new();
        };
        let anchor = self.anchor_of(kept);
        let members = self
            .papers()
            .filter(|one| grouping(one) && self.anchor_of(one) == anchor);
        kept_by_language(members.collect())
    }

    /// Every group with two languages or more, by its anchor, worked out in one pass.
    pub fn families(&self) -> BTreeMap<DocId, Vec<&Kept>> {
        let mut gathered: BTreeMap<DocId, Vec<&Kept>> = BTreeMap::new();
        for one in self.papers().filter(|one| grouping(one)) {
            gathered.entry(self.anchor_of(one)).or_default().push(one);
        }
        gathered
            .into_iter()
            .map(|(anchor, members)| (anchor, kept_by_language(members)))
            .filter(|(_, tongues)| !tongues.is_empty())
            .collect()
    }

    /// The other languages that stand with it in its folder, which move, rest and go with it.
    pub fn kin(&self, id: DocId) -> Vec<DocId> {
        let Some(kept) = self.docs.get(&id) else {
            return Vec::new();
        };
        let tongues = self.tongues_of(id);
        if !tongues.iter().any(|one| one.id == id) {
            return Vec::new();
        }
        tongues
            .into_iter()
            .filter(|one| one.id != id && one.folder == kept.folder)
            .map(|one| one.id)
            .collect()
    }

    /// Joined to a group yet shown on its own: it says no language, or one an earlier member keeps.
    pub fn apart_in_tongues(&self, id: DocId) -> bool {
        let Some(kept) = self.docs.get(&id).filter(|one| grouping(one)) else {
            return false;
        };
        let anchor = self.anchor_of(kept);
        let members = self
            .papers()
            .filter(|one| grouping(one) && self.anchor_of(one) == anchor)
            .collect();
        left_apart(members).contains(&id)
    }

    /// Every document shown on its own although it is joined to others, worked out in one pass.
    pub fn apart(&self) -> BTreeSet<DocId> {
        let mut gathered: BTreeMap<DocId, Vec<&Kept>> = BTreeMap::new();
        for one in self.papers().filter(|one| grouping(one)) {
            gathered.entry(self.anchor_of(one)).or_default().push(one);
        }
        gathered.into_values().flat_map(left_apart).collect()
    }

    /// The page of another language that stands where this one does in its own document.
    pub fn twin_page(&self, page: DocId, tongue: DocId) -> Option<&Kept> {
        let up = self.docs.get(&page)?.page_of?;
        let at = self.pages_of(up).iter().position(|one| one.id == page)?;
        self.pages_of(tongue).get(at).copied()
    }
}

impl State {
    /// Whether the group a document is in, or the document alone, already says that language.
    pub fn tongue_taken(&self, id: DocId, lang: &str) -> bool {
        let tongues = self.tongues_of(id);
        match tongues.is_empty() {
            true => self.docs.get(&id).and_then(|one| one.lang.as_deref()) == Some(lang),
            false => tongues.iter().any(|one| one.lang.as_deref() == Some(lang)),
        }
    }

    /// What joins a document to another's languages: it moves to their folder, since a group stands in one.
    pub fn joining(&self, id: DocId, of: DocId) -> Vec<crate::Op> {
        let (Some(kept), Some(up)) = (self.docs.get(&id), self.docs.get(&of)) else {
            return Vec::new();
        };
        if self.anchor_of(kept) == self.anchor_of(up) {
            return Vec::new();
        }
        let mut ops: Vec<crate::Op> = std::iter::once(id)
            .chain(self.kin(id))
            .filter(|one| {
                self.docs
                    .get(one)
                    .is_some_and(|one| one.folder != up.folder)
            })
            .map(|id| crate::Op::DocMove {
                id,
                d: crate::event::Filed {
                    folder: Some(up.folder),
                    page_of: None,
                    order: None,
                },
            })
            .collect();
        ops.push(crate::Op::DocTongue {
            id,
            d: crate::event::Tongue {
                of: Some(self.anchor_of(up)),
            },
        });
        ops
    }

    /// What takes a document out of its languages; an anchor hands the others to the first of them.
    pub fn parting(&self, id: DocId) -> Vec<crate::Op> {
        let out = |id: DocId, of: Option<DocId>| crate::Op::DocTongue {
            id,
            d: crate::event::Tongue { of },
        };
        let Some(kept) = self.docs.get(&id) else {
            return Vec::new();
        };
        if kept.translation_of.is_some() {
            return vec![out(id, None)];
        }
        let mut others: Vec<&Kept> = self
            .papers()
            .filter(|one| one.translation_of == Some(id))
            .collect();
        others.sort_by_key(|one| (one.joined.or(one.made), one.id));
        let Some((next, rest)) = others.split_first() else {
            return Vec::new();
        };
        std::iter::once(out(next.id, None))
            .chain(rest.iter().map(|one| out(one.id, Some(next.id))))
            .collect()
    }
}

/// The property that carries a group out of Tisty and back, named as Hugo names it.
pub const TRANSLATION_KEY: &str = "translationKey";

impl State {
    /// Its words as they leave Tisty, carrying the key of the group they are one language of.
    pub fn keyed(&self, file: &str, body: &str) -> String {
        let Some(kept) = self.papers().find(|one| one.file == file) else {
            return body.to_string();
        };
        match self.tongues_of(kept.id).iter().any(|one| one.id == kept.id) {
            true => tisty_markdown::front::with_field(
                body,
                TRANSLATION_KEY,
                &self.anchor_of(kept).to_string(),
            ),
            false => body.to_string(),
        }
    }

    /// The anchor a body brought in joins: the one its key names, or one that came in under that key.
    pub fn keyed_to(&self, body: &str, but: DocId) -> Option<DocId> {
        let key = key_of(body)?;
        let lang = tisty_markdown::front::lang(body)?;
        let named = key
            .parse::<DocId>()
            .ok()
            .and_then(|id| self.docs.get(&id))
            .filter(|one| grouping(one) && one.id != but);
        let found = named.or_else(|| {
            self.papers()
                .filter(|one| grouping(one) && one.id != but)
                .find(|one| one.key.as_deref() == Some(key.as_str()))
        })?;
        let kept: Vec<&Kept> = match self.tongues_of(found.id) {
            none if none.is_empty() => vec![found],
            some => some,
        };
        let taken = kept
            .iter()
            .any(|one| one.lang.as_deref() == Some(lang.as_str()));
        (!taken).then(|| self.anchor_of(found))
    }
}

pub(crate) fn key_of(body: &str) -> Option<String> {
    tisty_markdown::front::fields(body)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case(TRANSLATION_KEY))
        .and_then(|one| one.values.into_iter().next())
        .filter(|one| !one.trim().is_empty())
}

// Only a document of its own takes part: a page goes with its document, a task's words with the task.
fn grouping(kept: &Kept) -> bool {
    kept.page_of.is_none() && kept.task_of.is_none()
}

// The first to join keeps a language, so a copy brought back later never takes it from the one in use.
fn kept_by_language(mut members: Vec<&Kept>) -> Vec<&Kept> {
    members.sort_by_key(|one| (one.joined.or(one.made), one.id));
    let mut said = BTreeSet::new();
    let tongues: Vec<&Kept> = members
        .into_iter()
        .filter(|one| {
            one.lang
                .as_ref()
                .is_some_and(|lang| said.insert(lang.clone()))
        })
        .collect();
    match tongues.len() > 1 {
        true => tongues,
        false => Vec::new(),
    }
}

// In a group of two or more, those that say no language or one an earlier member already says.
fn left_apart(mut members: Vec<&Kept>) -> Vec<DocId> {
    if members.len() < 2 {
        return Vec::new();
    }
    members.sort_by_key(|one| (one.joined.or(one.made), one.id));
    let mut said = BTreeSet::new();
    members
        .into_iter()
        .filter(|one| {
            !one.lang
                .as_ref()
                .is_some_and(|lang| said.insert(lang.clone()))
        })
        .map(|one| one.id)
        .collect()
}

#[cfg(test)]
#[path = "tongues_test.rs"]
mod tests;
