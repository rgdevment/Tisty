use std::collections::{BTreeMap, BTreeSet};

use crate::model::{DocId, Kept};
use crate::state::State;

/// One document told in several languages: each language is a document of its own, and the
/// log holds them together around the anchor the others name (`doc.tongue`).
impl State {
    /// The document its group gathers around: itself when it names none.
    pub fn anchor_of(&self, kept: &Kept) -> DocId {
        kept.translation_of.unwrap_or(kept.id)
    }

    /// The languages of the group a document is in, one document each, in the order they joined.
    /// Empty when it stands alone, since one language is no choice to make.
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

    /// Joined to a group yet shown on its own: it says no language, or one an earlier member keeps.
    pub fn apart_in_tongues(&self, id: DocId) -> bool {
        let Some(kept) = self.docs.get(&id).filter(|one| grouping(one)) else {
            return false;
        };
        let anchor = self.anchor_of(kept);
        let grouped = kept.translation_of.is_some()
            || self.papers().any(|one| one.translation_of == Some(anchor));
        grouped && !self.tongues_of(id).iter().any(|one| one.id == id)
    }

    /// The page of another language that stands where this one does in its own document.
    pub fn twin_page(&self, page: DocId, tongue: DocId) -> Option<&Kept> {
        let up = self.docs.get(&page)?.page_of?;
        let at = self.pages_of(up).iter().position(|one| one.id == page)?;
        self.pages_of(tongue).get(at).copied()
    }
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

#[cfg(test)]
#[path = "tongues_test.rs"]
mod tests;
