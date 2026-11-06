use std::ops::Range;

use crate::inline::{paired, safe};
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
    rejoined_html(blocks, text);
    for block in blocks {
        if column_cut(&text[block.at.clone()]) {
            rejoined(block, text);
        }
        let kept = paired(&block.spans, text);
        let drawn = html_drawn(&block.spans, text);
        block.spans.retain(|one| match &one.mark {
            Mark::Html => kept.contains(&one.at.start) || text[one.at.clone()].starts_with("<!--"),
            Mark::Tag { name } => !numbered(name),
            _ => true,
        });
        block.spans.extend(drawn.iter().flat_map(HtmlDrawn::spans));
        block
            .spans
            .sort_by_key(|one| (one.at.start, std::cmp::Reverse(one.at.end)));
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

fn opened_divs(said: &str) -> isize {
    let lowered = said.to_ascii_lowercase();
    let opens = lowered.matches("<div").count();
    let shuts = lowered.matches("</div").count();
    opens as isize - shuts as isize
}

// A blank line ends an HTML block, and each block is cleaned on its own, so a centred header would stop at its first gap.
fn rejoined_html(blocks: &mut Vec<Block>, text: &str) {
    let mut at = 0;
    while at < blocks.len() {
        let mut open = html_opened(&blocks[at], text);
        let mut last = at;
        while open > 0
            && blocks
                .get(last + 1)
                .is_some_and(|one| one.kind == Kind::Html)
        {
            last += 1;
            open += html_opened(&blocks[last], text);
        }
        if open == 0 && last > at {
            blocks[at].at.end = blocks[last].at.end;
            blocks.drain(at + 1..=last);
        }
        at += 1;
    }
}

fn html_opened(block: &Block, text: &str) -> isize {
    match block.kind {
        Kind::Html => opened_divs(&text[block.at.clone()]),
        _ => 0,
    }
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
    let drawn = html_drawn(&block.spans, text);
    let rewritten: Vec<usize> = drawn.iter().flat_map(HtmlDrawn::tags).collect();
    edits.extend(drawn.iter().flat_map(|one| one.written(text)));
    for span in &block.spans {
        let said = &text[span.at.clone()];
        match &span.mark {
            Mark::Break if cut && said.trim_matches(['\r', '\n']).is_empty() => {
                edits.push((with_its_margin(span.at.clone(), text), " ".to_string()));
            }
            Mark::Html
                if !kept.contains(&span.at.start)
                    && !rewritten.contains(&span.at.start)
                    && !said.starts_with("<!--") =>
            {
                edits.push((span.at.clone(), String::new()));
            }
            Mark::Tag { name } if numbered(name) => {
                edits.push((span.at.start..span.at.start + 1, "&#35;".to_string()));
            }
            Mark::Link { to } | Mark::Image { src: to } => {
                edits.extend(linked_in_place(span, text, to));
            }
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

fn destination(to: &str) -> String {
    if to.contains([' ', '(', ')']) {
        format!("<{to}>")
    } else {
        to.to_string()
    }
}

fn last_group(said: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (at, one) in said.bytes().enumerate().rev() {
        match one {
            b']' => depth += 1,
            b'[' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
    }
    None
}

fn linked_in_place(span: &Span, text: &str, to: &str) -> Option<(Range<usize>, String)> {
    let said = &text[span.at.clone()];
    let first = usize::from(said.starts_with('!'));
    if !said[first..].starts_with('[') || !said.ends_with(']') {
        return None;
    }
    let inline = format!("]({})", destination(to));
    let label = last_group(said)?;
    if label == first {
        let collapsed = text[span.at.end..].starts_with("[]");
        let end = span.at.end + if collapsed { 2 } else { 0 };
        return Some((span.at.end - 1..end, inline));
    }
    Some((span.at.start + label - 1..span.at.end, inline))
}

// What an inline tag means and Markdown can say, so neither the page nor the copy loses it.
enum HtmlDrawn {
    Picture {
        at: Range<usize>,
        src: String,
        alt: Option<Range<usize>>,
    },
    Link {
        open: Range<usize>,
        shut: Range<usize>,
        to: String,
    },
}

impl HtmlDrawn {
    fn spans(&self) -> Vec<Span> {
        match self {
            Self::Picture { at, src, alt } => std::iter::once(Span {
                mark: Mark::Image { src: src.clone() },
                at: at.clone(),
            })
            .chain(alt.iter().map(|at| Span {
                mark: Mark::Text,
                at: at.clone(),
            }))
            .collect(),
            Self::Link { open, shut, to } => vec![Span {
                mark: Mark::Link { to: to.clone() },
                at: open.start..shut.end,
            }],
        }
    }

    fn tags(&self) -> Vec<usize> {
        match self {
            Self::Picture { at, .. } => vec![at.start],
            Self::Link { open, shut, .. } => vec![open.start, shut.start],
        }
    }

    fn written(&self, text: &str) -> Vec<(Range<usize>, String)> {
        match self {
            Self::Picture { at, src, alt } => {
                let alt = alt.as_ref().map_or("", |at| &text[at.clone()]);
                vec![(at.clone(), format!("![{alt}]({})", destination(src)))]
            }
            Self::Link { open, shut, to } => vec![
                (open.clone(), "[".to_string()),
                (shut.clone(), format!("]({})", destination(to))),
            ],
        }
    }
}

fn html_drawn(spans: &[Span], text: &str) -> Vec<HtmlDrawn> {
    let mut drawn = Vec::new();
    let mut open: Vec<(Range<usize>, String)> = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Html) {
        let tag = &text[span.at.clone()];
        let lowered = tag.to_ascii_lowercase();
        if lowered.starts_with("<img") {
            let found = attributes(tag, span.at.start);
            if let Some(src) = wanted_address(&found, "src", text) {
                drawn.push(HtmlDrawn::Picture {
                    at: span.at.clone(),
                    src,
                    alt: found
                        .iter()
                        .find(|(name, _)| name == "alt")
                        .map(|(_, at)| at.clone()),
                });
            }
        } else if lowered.starts_with("<a ") || lowered.starts_with("<a\t") {
            if let Some(to) = wanted_address(&attributes(tag, span.at.start), "href", text) {
                open.push((span.at.clone(), to));
            }
        } else if lowered == "</a>"
            && let Some((tag, to)) = open.pop()
        {
            drawn.push(HtmlDrawn::Link {
                open: tag,
                shut: span.at.clone(),
                to,
            });
        }
    }
    drawn
}

fn wanted_address(found: &[(String, Range<usize>)], name: &str, text: &str) -> Option<String> {
    let (_, at) = found.iter().find(|(one, _)| one == name)?;
    let said = text[at.clone()].trim().replace("&amp;", "&");
    (!said.is_empty() && safe(&said) != "#").then_some(said)
}

fn attributes(tag: &str, offset: usize) -> Vec<(String, Range<usize>)> {
    let bytes = tag.as_bytes();
    let mut found = Vec::new();
    let mut at = tag
        .find(|one: char| one.is_ascii_whitespace())
        .unwrap_or(tag.len());
    while at < bytes.len() {
        while at < bytes.len() && (bytes[at].is_ascii_whitespace() || bytes[at] == b'/') {
            at += 1;
        }
        let name_at = at;
        while at < bytes.len()
            && !matches!(bytes[at], b'=' | b'>' | b'/')
            && !bytes[at].is_ascii_whitespace()
        {
            at += 1;
        }
        let name = tag[name_at..at].to_ascii_lowercase();
        if at >= bytes.len() || bytes[at] != b'=' {
            if at < bytes.len() && bytes[at] == b'>' {
                break;
            }
            at += usize::from(name.is_empty());
            continue;
        }
        at += 1;
        let quote = bytes
            .get(at)
            .copied()
            .filter(|one| matches!(one, b'"' | b'\''));
        at += usize::from(quote.is_some());
        let value_at = at;
        while at < bytes.len()
            && match quote {
                Some(quote) => bytes[at] != quote,
                None => !bytes[at].is_ascii_whitespace() && bytes[at] != b'>',
            }
        {
            at += 1;
        }
        found.push((name, offset + value_at..offset + at));
        at += usize::from(quote.is_some());
    }
    found
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
