use std::ops::Range;

use crate::{Block, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    pub at: Range<usize>,
    pub source: String,
    inside: Option<Range<usize>>,
}

pub fn diagrams(text: &str) -> Vec<Diagram> {
    let doc = crate::read(text);
    let mut found = Vec::new();
    gather(doc.blocks(), text, &mut found);
    found
}

fn gather(blocks: &[Block], text: &str, found: &mut Vec<Diagram>) {
    for block in blocks {
        match &block.kind {
            Kind::Code {
                tongue: Some(tongue),
                ..
            } if matches!(tongue.as_str(), "mermaid" | "mmd") => found.push(Diagram {
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
            }),
            _ => gather(&block.inner, text, found),
        }
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
    let was = was.replace('\r', "");
    let found = diagrams(text);
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

pub fn redrawn(text: &str, place: Place, was: &str, now: &str) -> Option<(String, Place)> {
    let (place, chosen) = still(text, place, was)?;
    Some((written_into(text, &chosen, now), place))
}

/// The diagram a window holds as the document says it now, whatever it says.
pub fn at_place(text: &str, place: Place) -> Option<(Place, String)> {
    let found = diagrams(text);
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

fn written_into(text: &str, diagram: &Diagram, now: &str) -> String {
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
