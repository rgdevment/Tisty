use std::ops::Range;

use crate::inline::math_wall;
use crate::{Block, Kind, Mark};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    pub at: Range<usize>,
    pub source: String,
    inside: Option<Range<usize>>,
}

/// A piece a window of its own opens and writes back in its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Held {
    Diagram,
    Widget,
    Formula,
    InlineFormula,
    Table,
    Front,
}

impl Held {
    fn fences(self, tongue: &str) -> bool {
        match self {
            Self::Diagram => matches!(tongue, "mermaid" | "mmd"),
            Self::Widget => tongue == "widget",
            Self::Formula => tongue == "math",
            Self::InlineFormula | Self::Table | Self::Front => false,
        }
    }

    pub fn as_read(self, now: &str) -> String {
        match self {
            Self::InlineFormula => now.replace('\r', "").trim_end_matches('\n').to_string(),
            _ => as_read(now),
        }
    }
}

/// Why a window's words were not written: its piece is gone, or they would no longer read as it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unwritten {
    Moved,
    Unread,
}

pub fn diagrams(text: &str) -> Vec<Diagram> {
    held_in(text, Held::Diagram)
}

pub fn held_in(text: &str, held: Held) -> Vec<Diagram> {
    let doc = crate::read(text);
    let mut found = Vec::new();
    gather(doc.blocks(), text, held, &mut found);
    found
}

fn gather(blocks: &[Block], text: &str, held: Held, found: &mut Vec<Diagram>) {
    for block in blocks {
        match &block.kind {
            Kind::Code {
                tongue: Some(tongue),
                ..
            } if held.fences(tongue) => found.push(fence(block, text)),
            Kind::Table { .. } if held == Held::Table => {
                let end = block.at.start + text[block.at.clone()].trim_end().len();
                let inside =
                    block.at.start..line_end(text, end.saturating_sub(1).max(block.at.start));
                found.push(lined(text, block.at.clone(), inside));
            }
            Kind::Front if held == Held::Front => {
                let end = block.at.start + text[block.at.clone()].trim_end().len();
                let opened = line_end(text, block.at.start);
                let inside = opened..line_start(text, end).max(opened);
                found.push(lined(text, block.at.clone(), inside));
            }
            _ => {
                if held == Held::InlineFormula {
                    formulas(block, text, found);
                }
                gather(&block.inner, text, held, found);
            }
        }
    }
}

fn fence(block: &Block, text: &str) -> Diagram {
    Diagram {
        at: block.at.clone(),
        source: block
            .spans
            .iter()
            .map(|one| &text[one.at.clone()])
            .collect::<String>()
            .replace('\r', ""),
        inside: block
            .spans
            .first()
            .zip(block.spans.last())
            .map(|(first, last)| first.at.start..last.at.end),
    }
}

// Lines under a quote or a bullet are read without its marks, which are written back in front of each.
fn lined(text: &str, at: Range<usize>, inside: Range<usize>) -> Diagram {
    let lead = lead_before(text, inside.start);
    let source = text[inside.clone()]
        .replace('\r', "")
        .split_inclusive('\n')
        .enumerate()
        .map(|(index, line)| match index {
            0 => line,
            _ => line.strip_prefix(lead.as_str()).unwrap_or(line),
        })
        .collect();
    Diagram {
        at,
        source,
        inside: Some(inside),
    }
}

fn formulas(block: &Block, text: &str, found: &mut Vec<Diagram>) {
    for span in block.spans.iter().filter(|one| one.mark == Mark::Math) {
        let wall = math_wall(&text[span.at.clone()]);
        let inside = span.at.start + wall..span.at.end - wall;
        found.push(Diagram {
            at: span.at.clone(),
            source: text[inside.clone()].replace('\r', ""),
            inside: Some(inside),
        });
    }
}

/// Where the page put a diagram and which one it is counting from the top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub at: usize,
    pub nth: usize,
}

// Another diagram that merely says the same is never taken for this one: a copy is not a move.
pub fn still(text: &str, place: Place, was: &str) -> Option<(Place, Diagram)> {
    still_in(text, Held::Diagram, place, was)
}

pub fn still_in(text: &str, held: Held, place: Place, was: &str) -> Option<(Place, Diagram)> {
    let was = was.replace('\r', "");
    let found = held_in(text, held);
    let alike = found.iter().filter(|one| one.source == was).count();
    let nth = found
        .iter()
        .position(|one| one.at.start == place.at && one.source == was)
        .or_else(|| {
            found
                .get(place.nth)
                .filter(|one| alike == 1 && one.source == was)
                .map(|_| place.nth)
        })?;
    let chosen = found.into_iter().nth(nth)?;
    Some((
        Place {
            at: chosen.at.start,
            nth,
        },
        chosen,
    ))
}

// Blocks written above since a window opened move its diagram; the one alone in saying the same is still it.
pub fn order_of(text: &str, place: Place, was: &str) -> usize {
    let was = was.replace('\r', "");
    let found = diagrams(text);
    let alike: Vec<usize> = found
        .iter()
        .enumerate()
        .filter(|(_, one)| one.source == was)
        .map(|(nth, _)| nth)
        .collect();
    found
        .iter()
        .position(|one| one.at.start == place.at && one.source == was)
        .or_else(|| (alike.len() == 1).then(|| alike[0]))
        .unwrap_or(place.nth)
}

pub fn redrawn(text: &str, place: Place, was: &str, now: &str) -> Option<(String, Place)> {
    redrawn_in(text, Held::Diagram, place, was, now).ok()
}

pub fn redrawn_in(
    text: &str,
    held: Held,
    place: Place,
    was: &str,
    now: &str,
) -> Result<(String, Place), Unwritten> {
    let (place, chosen) = still_in(text, held, place, was).ok_or(Unwritten::Moved)?;
    let now = held.as_read(now);
    let body = written_into(text, held, &chosen, &now);
    let (place, _) = still_in(&body, held, place, &now).ok_or(Unwritten::Unread)?;
    Ok((body, place))
}

/// The diagram a window holds as the document says it now, whatever it says.
pub fn at_place(text: &str, place: Place) -> Option<(Place, String)> {
    at_place_in(text, Held::Diagram, place)
}

pub fn at_place_in(text: &str, held: Held, place: Place) -> Option<(Place, String)> {
    let found = held_in(text, held);
    let nth = found
        .iter()
        .position(|one| one.at.start == place.at)
        .or((place.nth < found.len()).then_some(place.nth))?;
    let one = &found[nth];
    Some((
        Place {
            at: one.at.start,
            nth,
        },
        one.source.clone(),
    ))
}

pub fn as_read(now: &str) -> String {
    now.replace('\r', "")
        .lines()
        .map(|line| format!("{line}\n"))
        .collect()
}

fn written_into(text: &str, held: Held, diagram: &Diagram, now: &str) -> String {
    if let (Held::InlineFormula, Some(inside)) = (held, &diagram.inside) {
        return format!("{}{now}{}", &text[..inside.start], &text[inside.end..]);
    }
    let (room, lead) = match &diagram.inside {
        Some(inside) => (inside.clone(), lead_before(text, inside.start)),
        None => {
            let after = line_end(text, diagram.at.start);
            (after..after, opening_lead(text, diagram.at.start))
        }
    };
    let opening = &text[diagram.at.start..line_end(text, diagram.at.start)];
    let ending = if opening.ends_with("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut said = String::new();
    for (index, line) in now.replace('\r', "").lines().enumerate() {
        if index > 0 || diagram.inside.is_none() {
            said.push_str(&lead);
        }
        said.push_str(line);
        said.push_str(ending);
    }
    if said.is_empty() && diagram.inside.is_some() {
        return format!("{}{}", &text[..room.start - lead.len()], &text[room.end..]);
    }
    format!("{}{said}{}", &text[..room.start], &text[room.end..])
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |found| found + 1)
}

fn line_end(text: &str, at: usize) -> usize {
    text[at..]
        .find('\n')
        .map_or(text.len(), |found| at + found + 1)
}

fn lead_before(text: &str, at: usize) -> String {
    text[line_start(text, at)..at].to_string()
}

// A fence opened on a list bullet keeps its lines under the bullet's words, not under the bullet.
fn opening_lead(text: &str, at: usize) -> String {
    let start = line_start(text, at);
    let line = &text[start..line_end(text, at)];
    let mark = line.find(['`', '~']).unwrap_or(0);
    line[..mark]
        .chars()
        .map(|one| {
            if one == '>' || one.is_whitespace() {
                one
            } else {
                ' '
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "diagrams_test.rs"]
mod tests;
