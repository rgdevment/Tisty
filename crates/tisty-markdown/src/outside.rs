use std::ops::Range;

use crate::inline::paired;
use crate::{Block, Doc, Kind, Mark, Span};

const COLUMN_CUT: usize = 60;

pub fn read_file(text: &str) -> Doc {
    let mut doc = crate::reading::reading(text, crate::reading::FILE_PASSES);
    let Doc { text, blocks } = &mut doc;
    settled(blocks, text);
    doc
}

fn settled(blocks: &mut Vec<Block>, text: &str) {
    blocks.retain(|one| !matches!(one.kind, Kind::Definition { .. }));
    for block in blocks {
        if column_cut(&text[block.at.clone()]) {
            rejoined(block, text);
        }
        let kept = paired(&block.spans, text);
        block.spans.retain(|one| match &one.mark {
            Mark::Html => kept.contains(&one.at.start) || text[one.at.clone()].starts_with("<!--"),
            Mark::Tag { name } => !numbered(name),
            _ => true,
        });
        settled(&mut block.inner, text);
    }
}

// Short lines were broken on purpose, as an address or a poem is; a long one followed by another was cut by an editor.
fn column_cut(said: &str) -> bool {
    let mut lines = said.lines().peekable();
    while let Some(line) = lines.next() {
        if lines.peek().is_some() && line.chars().count() >= COLUMN_CUT {
            return true;
        }
    }
    false
}

fn rejoined(block: &mut Block, text: &str) {
    for span in &mut block.spans {
        let only_a_newline = text[span.at.clone()].trim_matches(['\r', '\n']).is_empty();
        if span.mark == Mark::Break && only_a_newline {
            span.mark = Mark::Text;
        }
    }
}

// What read_file showed, written so that Tisty's own reading shows the same; its tags and references pass over a backslash.
pub fn file_as_tisty(text: &str) -> String {
    let doc = crate::reading::reading(text, crate::reading::FILE_PASSES);
    let mut edits = Vec::new();
    turned(&doc.blocks, text, &mut edits);
    edits.sort_by_key(|(at, _): &(Range<usize>, String)| std::cmp::Reverse(at.start));
    let mut out = text.to_string();
    for (at, now) in edits {
        out.replace_range(at, &now);
    }
    out
}

fn turned(blocks: &[Block], text: &str, edits: &mut Vec<(Range<usize>, String)>) {
    for block in blocks {
        if matches!(block.kind, Kind::Definition { .. }) {
            edits.push((block.at.clone(), String::new()));
            continue;
        }
        if !matches!(block.kind, Kind::Code { .. }) {
            spans_turned(block, text, edits);
            brackets_kept(block, text, edits);
        }
        turned(&block.inner, text, edits);
    }
}

fn spans_turned(block: &Block, text: &str, edits: &mut Vec<(Range<usize>, String)>) {
    let cut = column_cut(&text[block.at.clone()]);
    let kept = paired(&block.spans, text);
    for span in &block.spans {
        let said = &text[span.at.clone()];
        match &span.mark {
            Mark::Break if cut && said.trim_matches(['\r', '\n']).is_empty() => {
                edits.push((with_its_margin(span.at.clone(), text), " ".to_string()));
            }
            Mark::Html if !kept.contains(&span.at.start) && !said.starts_with("<!--") => {
                edits.push((span.at.clone(), String::new()));
            }
            Mark::Tag { name } if numbered(name) => {
                edits.push((span.at.start..span.at.start + 1, "&#35;".to_string()));
            }
            Mark::Link { to } => edits.extend(linked_in_place(span, text, to)),
            _ => {}
        }
    }
}

// The next line of a quote or an item starts with its margin, which would be read as words once on this line.
fn with_its_margin(at: Range<usize>, text: &str) -> Range<usize> {
    let margin = text[at.end..]
        .find(|one| !matches!(one, ' ' | '\t' | '>'))
        .unwrap_or(0);
    at.start..at.end + margin
}

fn linked_in_place(span: &Span, text: &str, to: &str) -> Option<(Range<usize>, String)> {
    let said = &text[span.at.clone()];
    if !said.starts_with('[') || said.contains("](") {
        return None;
    }
    let inline = format!("]({to})");
    match said.rfind("][") {
        Some(from) => Some((span.at.start + from..span.at.end, inline)),
        None => said.ends_with(']').then(|| {
            let collapsed = text[span.at.end..].starts_with("[]");
            (
                span.at.end - 1..span.at.end + if collapsed { 2 } else { 0 },
                inline,
            )
        }),
    }
}

fn brackets_kept(block: &Block, text: &str, edits: &mut Vec<(Range<usize>, String)>) {
    for span in block.spans.iter().filter(|one| one.mark == Mark::Text) {
        let said = &text[span.at.clone()];
        let closes = text[span.at.end..].starts_with('[');
        for (at, _) in said.char_indices().filter(|(at, one)| {
            *one == '[' && (said[at + 1..].starts_with('[') || (closes && at + 1 == said.len()))
        }) {
            edits.push((
                span.at.start + at..span.at.start + at + 1,
                "&#91;".to_string(),
            ));
        }
    }
}

fn numbered(name: &str) -> bool {
    let digits = name.chars().all(|one| one.is_ascii_digit());
    let colour = matches!(name.len(), 6 | 8) && name.chars().all(|one| one.is_ascii_hexdigit());
    digits || colour
}

#[cfg(test)]
#[path = "outside_test.rs"]
mod tests;
