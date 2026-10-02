use std::collections::BTreeMap;
use std::path::Path;

const KEPT: &str = ".turned-away";

/// What the round did with a machine's history, kept where the window can read it. Comparing a
/// published key against a confirmed one cannot stand in for this: the log keeps the first key a
/// machine ever published and never another, so the two agree even while the folder is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Away {
    Unreadable,
    Disowned,
    Unconfirmed,
}

impl Away {
    fn said(self) -> &'static str {
        match self {
            Away::Unreadable => "unreadable",
            Away::Disowned => "disowned",
            Away::Unconfirmed => "unconfirmed",
        }
    }

    fn read(said: &str) -> Option<Self> {
        match said {
            "unreadable" => Some(Away::Unreadable),
            "disowned" => Some(Away::Disowned),
            "unconfirmed" => Some(Away::Unconfirmed),
            _ => None,
        }
    }
}

pub fn of(data: &Path) -> BTreeMap<String, Away> {
    std::fs::read_to_string(data.join(KEPT))
        .map(|said| {
            said.lines()
                .filter_map(|line| {
                    let (whose, away) = line.trim_end_matches('\r').split_once('\t')?;
                    Some((whose.to_string(), Away::read(away)?))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Written whole on every round that read the folder, so a machine that heals stops being named
/// without anybody having to remember it was.
pub fn keep(data: &Path, away: &BTreeMap<String, Away>) {
    let said: Vec<String> = away
        .iter()
        .filter(|(whose, _)| tisty_core::store::is_device_name(whose))
        .map(|(whose, away)| format!("{whose}\t{}", away.said()))
        .collect();
    let at = data.join(KEPT);
    if said.is_empty() {
        let _ = std::fs::remove_file(&at);
        return;
    }
    let _ = tisty_core::store::write_atomic(&at, said.join("\n").as_bytes());
}

pub fn let_through(data: &Path, whose: &str) -> bool {
    let mut away = of(data);
    if away.remove(whose).is_none() {
        return false;
    }
    keep(data, &away);
    true
}

#[cfg(test)]
#[path = "turned_test.rs"]
mod tests;
