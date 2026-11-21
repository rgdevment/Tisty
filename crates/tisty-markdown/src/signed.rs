use std::collections::HashMap;
use std::ops::Range;

use crate::{Doc, Kind};

pub const ASSISTANT_NOTES: [&str; 2] = ["## Assistant notes", "## Notas del asistente"];

pub fn signer(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("**")?;
    let (day, by) = rest.split_once("** · ")?;
    let stamped = day.len() == 16
        && day.as_bytes()[4] == b'-'
        && day.as_bytes()[10] == b' '
        && day.as_bytes()[13] == b':';
    (stamped && !by.trim().is_empty()).then_some(by.trim())
}

// Only under the assistant's heading: a person's own `**date** · name` elsewhere stays as written.
pub(crate) fn signers(doc: &Doc) -> HashMap<usize, Range<usize>> {
    let text = doc.text();
    let mut found = HashMap::new();
    let mut under: Option<u8> = None;
    for block in doc.blocks() {
        match block.kind {
            Kind::Heading(level) => {
                if under.is_some_and(|own| level <= own) {
                    under = None;
                }
                if under.is_none() && ASSISTANT_NOTES.contains(&text[block.at.clone()].trim()) {
                    under = Some(level);
                }
            }
            Kind::Paragraph if under.is_some() => {
                if let Some(name) = named_at(text, &block.at) {
                    found.insert(block.at.start, name);
                }
            }
            _ => {}
        }
    }
    found
}

fn named_at(text: &str, at: &Range<usize>) -> Option<Range<usize>> {
    let said = &text[at.clone()];
    let line = said.split('\n').next().unwrap_or(said);
    let name = signer(line)?;
    let from = at.start + (name.as_ptr() as usize - said.as_ptr() as usize);
    Some(from..from + name.len())
}

#[cfg(test)]
#[path = "signed_test.rs"]
mod tests;
