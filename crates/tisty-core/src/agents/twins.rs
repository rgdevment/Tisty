use std::collections::BTreeSet;

use super::card::named_anew;
use super::{Card, GLOBAL, Kind};
use crate::model::DocId;

/// Two Global rules that took one name on two computers before they synced: the older keeps it, the newer takes the next free one.
// Only rules: a Global skill brought twice is one skill, and its name is the folder it lands in.
pub fn renamed_twins(docs: &[(DocId, String, String, bool)]) -> Vec<(String, String)> {
    let mut sorted: Vec<&(DocId, String, String, bool)> = docs.iter().collect();
    sorted.sort_by_key(|(id, ..)| *id);
    let mut taken: BTreeSet<String> = docs
        .iter()
        .filter_map(|(_, _, body, _)| Card::of(body).map(|card| card.address()))
        .collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (_, file, body, agreed) in sorted {
        let Some(card) =
            Card::of(body).filter(|card| card.level == GLOBAL && card.kind == Kind::Rule)
        else {
            continue;
        };
        if seen.insert(card.address()) || !agreed {
            continue;
        }
        let free = (2..).map(|n| numbered(&card.name, n)).find(|name| {
            !taken.contains(
                &Card {
                    name: name.clone(),
                    ..card.clone()
                }
                .address(),
            )
        });
        let Some(body) = free.and_then(|name| named_anew(body, &name)) else {
            continue;
        };
        if let Some(fresh) = Card::of(&body) {
            taken.insert(fresh.address());
        }
        out.push((file.clone(), body));
    }
    out
}

fn numbered(name: &str, n: u32) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !stem.ends_with('/') => {
            format!("{stem}-{n}.{ext}")
        }
        _ => format!("{name}-{n}"),
    }
}

#[cfg(test)]
#[path = "twins_test.rs"]
mod tests;
