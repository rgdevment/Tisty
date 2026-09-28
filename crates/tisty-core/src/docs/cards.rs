use std::path::Path;

use super::text::{Heading, outlined, pointed_at, standing_out, titled};
use super::{Sighting, all, bared, read, resolve, shown_around, stamped};

/// What can be worked out from a body without anybody writing it down, and so can never be
/// stale: every field here is read back out of the text each time the file changes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Card {
    pub print: String,
    pub title: String,
    pub chars: usize,
    pub lines: usize,
    pub words: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outline: Vec<Heading>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "none_at_all")]
    pub pictures: usize,
    #[serde(default, skip_serializing_if = "none_at_all")]
    pub links: usize,
}

fn none_at_all(many: &usize) -> bool {
    *many == 0
}

/// What a card cannot work out because nobody can: somebody read the document and said what it
/// was about. It is kept beside the card, on this machine only, and carries the print of the
/// body it was written against — so a reader can be told it is describing an older text.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Gist {
    pub print: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    pub at: jiff::Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
}

/// Searching without opening anything: the cards already hold the text, folded the same way a
/// query is, so the database names the few documents worth reading and only those are read.
/// Without a cache there is nothing to ask, and the caller falls back to walking the files.
pub fn sighted(
    root: &Path,
    cache: Option<&crate::cache::Cache>,
    query: &str,
    most: usize,
    wanted: impl Fn(&str) -> bool,
) -> Option<Vec<Sighting>> {
    let terms = crate::text::terms(query);
    if terms.is_empty() {
        return Some(Vec::new());
    }
    let cache = cache?;
    // A card missing for a file that is there means the text was never read into the database,
    // and answering from what it holds would quietly leave that document out of every search.
    let all = all(root);
    for one in &all {
        card_of(root, Some(cache), &one.id)?;
    }

    let mut found = cache.holding(&terms)?;
    found.sort();
    let mut out = Vec::new();
    for id in found {
        if out.len() >= most || !wanted(&id) {
            continue;
        }
        let Ok(body) = read(root, &id) else { continue };
        let title = titled(&body);
        let line = match crate::text::folded(&title)
            .split_whitespace()
            .collect::<String>()
            .is_empty()
        {
            _ if terms
                .iter()
                .all(|term| crate::text::folded(&title).contains(term.as_str())) =>
            {
                String::new()
            }
            _ => match shown_around(&body, &terms) {
                Some(line) => line,
                None => continue,
            },
        };
        out.push(Sighting { id, title, line });
    }
    Some(out)
}

/// Worked out once per version of a file and remembered locally, because reading two hundred
/// bodies to answer "which of these is about the roof" is a cost nobody should pay twice.
pub fn card_of(root: &Path, cache: Option<&crate::cache::Cache>, id: &str) -> Option<Card> {
    let at = resolve(root, id).ok()?;
    let stamp = stamped(&at)?;
    if let Some(cache) = cache
        && let Some(card) = cache.card(id, stamp)
    {
        return Some(card);
    }
    let body = read(root, id).ok()?;
    let card = Card::read_from(&body);
    if let Some(cache) = cache {
        cache.note_card(id, stamp, &card, &crate::text::folded(&bared(&body)));
    }
    Some(card)
}

/// The cards of many, in one pass. It forgets nothing: the caller asks for a page at a time,
/// and throwing away every card outside that page would leave the cache colder each time.
pub fn cards_of(
    root: &Path,
    cache: Option<&crate::cache::Cache>,
    ids: &[String],
) -> std::collections::BTreeMap<String, Card> {
    ids.iter()
        .filter_map(|id| card_of(root, cache, id).map(|card| (id.clone(), card)))
        .collect()
}

/// What is remembered about documents that are no longer on disk, weighed against the files
/// themselves rather than against whatever somebody happened to ask for.
pub fn forget_stray_cards(root: &Path, cache: Option<&crate::cache::Cache>) {
    let Some(cache) = cache else {
        return;
    };
    cache.forget_cards(&all(root).into_iter().map(|one| one.id).collect());
}

impl Card {
    pub fn read_from(body: &str) -> Self {
        let outline = outlined(body);
        let (pictures, links) = pointed_at(body);
        Self {
            print: crate::attach::printed(body.as_bytes()),
            title: titled(body),
            chars: body.chars().count(),
            lines: body.lines().count(),
            words: body.split_whitespace().count(),
            keywords: standing_out(body),
            outline,
            pictures,
            links,
        }
    }
}

#[cfg(test)]
#[path = "cards_test.rs"]
mod tests;
