use std::cell::LazyCell;
use std::ops::Range;

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, LinkType, Options, Parser, Tag, TagEnd,
};

use crate::inline::Nearby;
use crate::{Block, Callout, Column, Doc, Kind, Lean, Mark, Span};

pub fn read(text: &str) -> Doc {
    reading(text, PASSES)
}

// What the transformer reads: Tisty's own words and, besides, the tags it wrote before them.
pub fn read_old(text: &str) -> Doc {
    reading(text, OLD)
}

fn reading(text: &str, passes: &[fn(&mut Vec<Span>, &str)]) -> Doc {
    // The core and the window both read past a byte-order mark; the tokenizer does not.
    let mut skip = text.len() - text.trim_start_matches('\u{feff}').len();
    let mut open: Vec<Block> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    // The tokenizer would take any fenced run of lines for front matter; only the top one is.
    if let Some(front) = crate::front::span(text) {
        skip = front.end;
        blocks.push(leaf(Kind::Front, front));
    }
    for (event, at) in tokens(&text[skip..]) {
        let at = at.start + skip..at.end + skip;
        match event {
            Event::Start(tag) => {
                if let Some(kind) = kind_of(&tag, &text[at.clone()]) {
                    open.push(leaf(kind, at));
                } else if let Some(mark) = mark_of(&tag) {
                    spot(&mut open, mark, at);
                }
            }
            Event::End(end) if ends_block(end) => {
                if let Some(mut done) = open.pop() {
                    if done.kind == Kind::Quote
                        && let Some(said) = marked(&text[done.at.clone()])
                    {
                        done.kind = Kind::Callout(said);
                        unmark(&mut done, text);
                    }
                    shelve(&mut open, &mut blocks, done);
                }
            }
            Event::Rule => shelve(&mut open, &mut blocks, leaf(Kind::Rule, at)),
            Event::TaskListMarker(ticked) => tick(&mut open, ticked),
            Event::Text(_) => spot(&mut open, Mark::Text, at),
            Event::Code(_) => spot(&mut open, Mark::Code, at),
            Event::FootnoteReference(label) => spot(
                &mut open,
                Mark::NoteRef {
                    label: label.to_string(),
                },
                at,
            ),
            Event::InlineHtml(_) => spot(&mut open, Mark::Html, at),
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                let mark = if priced(text, &at) {
                    Mark::Text
                } else {
                    Mark::Math
                };
                spot(&mut open, mark, at);
            }
            // A single newline breaks the line in Tisty, the way the window has always shown it.
            Event::SoftBreak | Event::HardBreak => spot(&mut open, Mark::Break, at),
            _ => {}
        }
    }
    defined(text, skip, &mut blocks);
    polish_all(&mut blocks, text, passes);
    #[cfg(feature = "tags")]
    crate::marks::tagged(&mut blocks, text);
    Doc {
        text: text.to_string(),
        blocks,
    }
}

// The order matters: a later pass reads the marks an earlier one has already folded.
const PASSES: &[fn(&mut Vec<Span>, &str)] = &[
    crate::bare::fold,
    crate::short::fold,
    #[cfg(feature = "pens")]
    crate::pens::fold,
    #[cfg(feature = "emoji")]
    crate::emoji::fold,
    #[cfg(feature = "refs")]
    crate::marks::named,
];

// The same passes after the old tags are folded, which only the transformer asks for.
const OLD: &[fn(&mut Vec<Span>, &str)] = &[
    crate::old::fold,
    crate::short::fold,
    #[cfg(feature = "pens")]
    crate::pens::fold,
    #[cfg(feature = "emoji")]
    crate::emoji::fold,
    #[cfg(feature = "refs")]
    crate::marks::named,
];

const SHAPES: &[fn(&mut Block, &str)] = &[
    #[cfg(feature = "tables")]
    crate::tables::widen,
];

const GATHERS: &[fn(&mut Vec<Block>, &str)] = &[
    #[cfg(feature = "details")]
    crate::details::gather,
];

fn polish_all(blocks: &mut Vec<Block>, text: &str, passes: &[fn(&mut Vec<Span>, &str)]) {
    for gather in GATHERS {
        gather(blocks, text);
    }
    for block in blocks {
        for shape in SHAPES {
            shape(block, text);
        }
        // Code keeps its words as written: nothing in it is a pen, an emoji or a reference.
        if !matches!(block.kind, Kind::Code { .. }) {
            for pass in passes {
                pass(&mut block.spans, text);
            }
        }
        block
            .spans
            .sort_by_key(|one| (one.at.start, std::cmp::Reverse(one.at.end)));
        polish_all(&mut block.inner, text, passes);
    }
}

// The only place the tokenizer is named: swapping it for another touches nothing else.
fn tokens(text: &str) -> impl Iterator<Item = (Event<'_>, Range<usize>)> {
    Parser::new_ext(text, options()).into_offset_iter()
}

// The tokenizer drops a link's definition from what it hands over, so the page would have nowhere to show or edit it.
fn defined(text: &str, skip: usize, blocks: &mut Vec<Block>) {
    // Built only once a definition turns up, so a document with none pays nothing for it.
    let held = LazyCell::new(|| Nearby::of(blocks.iter().map(|one| one.at.clone()).collect()));
    let parser = Parser::new_ext(&text[skip..], options());
    let mut found: Vec<Block> = parser
        .reference_definitions()
        .iter()
        .map(|(label, one)| {
            let at = one.span.start + skip..one.span.end + skip;
            let kind = Kind::Definition {
                label: label.to_string(),
                to: one.dest.to_string(),
            };
            leaf(kind, at)
        })
        .filter(|one| held.touching(&one.at).next().is_none())
        .collect();
    found.extend(repeated(text, skip, blocks, &found));
    if found.is_empty() {
        return;
    }
    blocks.append(&mut found);
    blocks.sort_by_key(|one| one.at.start);
}

// The parser keeps one definition per label, so a label written twice leaves its later lines unclaimed.
fn repeated(text: &str, skip: usize, blocks: &[Block], found: &[Block]) -> Vec<Block> {
    let held = LazyCell::new(|| {
        Nearby::of(
            blocks
                .iter()
                .chain(found)
                .map(|one| one.at.clone())
                .collect(),
        )
    });
    let claimed = |at: usize| held.holds(at);
    let mut out = Vec::new();
    let mut at = skip;
    for line in text[skip..].split_inclusive('\n') {
        let start = at;
        at += line.len();
        let said = line.trim();
        // The cheap look first: asking every block about every line made a long document quadratic.
        if !said.starts_with('[') || said.starts_with("[^") || claimed(start) {
            continue;
        }
        let Some((label, to)) = said[1..].split_once("]:") else {
            continue;
        };
        let kind = Kind::Definition {
            label: label.to_string(),
            to: to.split_whitespace().next().unwrap_or_default().to_string(),
        };
        out.push(leaf(kind, start..start + line.trim_end().len()));
    }
    out
}

fn options() -> Options {
    let mut options = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_GFM
        | Options::ENABLE_FOOTNOTES;
    if cfg!(feature = "math") {
        options |= Options::ENABLE_MATH;
    }
    options
}

// A digit right after the closing dollar is a price written next to another one, not a formula.
fn priced(text: &str, at: &Range<usize>) -> bool {
    text[at.end..].starts_with(|c: char| c.is_ascii_digit())
}

#[cfg(feature = "outline")]
pub(crate) fn plain(source: &str) -> String {
    let mut said = String::new();
    for (event, _) in tokens(source) {
        match event {
            Event::Text(words)
            | Event::Code(words)
            | Event::InlineMath(words)
            | Event::DisplayMath(words) => said.push_str(&words),
            Event::SoftBreak | Event::HardBreak => said.push(' '),
            Event::FootnoteReference(label) => {
                said.push_str("[^");
                said.push_str(&label);
                said.push(']');
            }
            _ => {}
        }
    }
    said.trim().to_string()
}

fn leaf(kind: Kind, at: Range<usize>) -> Block {
    Block {
        kind,
        at,
        inner: Vec::new(),
        spans: Vec::new(),
    }
}

fn spot(open: &mut [Block], mark: Mark, at: Range<usize>) {
    if let Some(block) = open.last_mut()
        && block.kind != Kind::Html
    {
        block.spans.push(Span { mark, at });
    }
}

fn mark_of(tag: &Tag<'_>) -> Option<Mark> {
    Some(match tag {
        Tag::Emphasis => Mark::Emphasis,
        Tag::Strong => Mark::Strong,
        Tag::Strikethrough => Mark::Strike,
        Tag::Link {
            link_type: LinkType::Email,
            dest_url,
            ..
        } => Mark::Link {
            to: format!("mailto:{dest_url}"),
        },
        Tag::Link { dest_url, .. } => Mark::Link {
            to: dest_url.to_string(),
        },
        Tag::Image { dest_url, .. } => Mark::Image {
            src: dest_url.to_string(),
        },
        _ => return None,
    })
}

fn shelve(open: &mut [Block], blocks: &mut Vec<Block>, done: Block) {
    match open.last_mut() {
        Some(parent) => parent.inner.push(done),
        None => blocks.push(done),
    }
}

// In a loose list the marker arrives inside the item's paragraph, not right under the item.
fn tick(open: &mut [Block], ticked: bool) {
    let item = open
        .iter_mut()
        .rev()
        .find(|one| matches!(one.kind, Kind::Item { .. }));
    if let Some(Block {
        kind: Kind::Item { done },
        ..
    }) = item
    {
        *done = Some(ticked);
    }
}

fn kind_of(tag: &Tag<'_>, source: &str) -> Option<Kind> {
    Some(match tag {
        Tag::Paragraph => Kind::Paragraph,
        Tag::Heading { level, .. } => Kind::Heading(*level as u8),
        Tag::BlockQuote(None) => Kind::Quote,
        Tag::BlockQuote(Some(kind)) => Kind::Callout(callout(*kind)),
        Tag::CodeBlock(CodeBlockKind::Fenced(_)) => code(fence_info(source)),
        Tag::CodeBlock(CodeBlockKind::Indented) => Kind::Code {
            tongue: None,
            title: None,
        },
        Tag::HtmlBlock => Kind::Html,
        Tag::List(start) => Kind::List { start: *start },
        Tag::Item => Kind::Item { done: None },
        Tag::Table(aligned) => Kind::Table {
            columns: aligned.iter().map(column).collect(),
        },
        Tag::TableHead => Kind::Row { head: true },
        Tag::TableRow => Kind::Row { head: false },
        Tag::TableCell => Kind::Cell,
        Tag::FootnoteDefinition(label) => Kind::Note {
            label: label.to_string(),
        },
        _ => return None,
    })
}

fn column(aligned: &Alignment) -> Column {
    let lean = match aligned {
        Alignment::None => None,
        Alignment::Left => Some(Lean::Left),
        Alignment::Center => Some(Lean::Center),
        Alignment::Right => Some(Lean::Right),
    };
    Column { lean, wide: None }
}

fn ends_block(end: TagEnd) -> bool {
    matches!(
        end,
        TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::HtmlBlock
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::Table
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell
            | TagEnd::FootnoteDefinition
    )
}

fn callout(kind: BlockQuoteKind) -> Callout {
    match kind {
        BlockQuoteKind::Note => Callout::Note,
        BlockQuoteKind::Tip => Callout::Tip,
        BlockQuoteKind::Important => Callout::Important,
        BlockQuoteKind::Warning => Callout::Warning,
        BlockQuoteKind::Caution => Callout::Caution,
    }
}

// GitHub wants the marker alone on its line; the window also takes words after it.
fn marked(source: &str) -> Option<Callout> {
    let line = source
        .lines()
        .next()?
        .trim_start()
        .strip_prefix('>')?
        .trim_start();
    let rest = line.strip_prefix("[!")?;
    let (word, after) = rest.split_once(']')?;
    if !after.is_empty() && !after.starts_with(char::is_whitespace) {
        return None;
    }
    Some(match word.to_ascii_lowercase().as_str() {
        "note" => Callout::Note,
        "tip" => Callout::Tip,
        "important" => Callout::Important,
        "warning" => Callout::Warning,
        "caution" => Callout::Caution,
        _ => return None,
    })
}

// The tokenizer only knows a marker alone on its line, so one with words after it is cut here.
fn unmark(quote: &mut Block, text: &str) {
    let Some(first) = quote.inner.first_mut() else {
        return;
    };
    if first.kind != Kind::Paragraph {
        return;
    }
    let Some(start) = first.spans.iter().map(|one| one.at.start).min() else {
        return;
    };
    let Some(close) = text[start..]
        .strip_prefix("[!")
        .and_then(|rest| rest.find(']'))
    else {
        return;
    };
    let after = start + 2 + close + 1;
    let rest = &text[after..];
    let cut = after + (rest.len() - rest.trim_start_matches([' ', '\t']).len());
    first.spans.retain(|one| one.at.end > cut);
    for one in &mut first.spans {
        one.at.start = one.at.start.max(cut);
    }
    first.spans.sort_by_key(|one| one.at.start);
    if first
        .spans
        .first()
        .is_some_and(|one| one.mark == Mark::Break)
    {
        first.spans.remove(0);
    }
    if first.spans.is_empty() {
        quote.inner.remove(0);
    }
}

// The tokenizer hands the info string back unescaped, and a title's own escapes need the raw one.
fn fence_info(source: &str) -> &str {
    let line = source.lines().next().unwrap_or_default().trim_start();
    let wall = line.chars().next().unwrap_or('`');
    line.trim_start_matches(wall).trim()
}

fn code(info: &str) -> Kind {
    let title = titled(info);
    let rest = match &title {
        Some((at, _)) => format!("{}{}", &info[..at.start], &info[at.end..]),
        None => info.to_string(),
    };
    let tongue = rest.split_whitespace().next().map(unescaped);
    Kind::Code {
        tongue,
        title: title.map(|(_, said)| said),
    }
}

fn unescaped(raw: &str) -> String {
    let mut said = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match chars.peek() {
            Some(next) if c == '\\' && next.is_ascii_punctuation() => {}
            _ => said.push(c),
        }
    }
    said
}

fn titled(info: &str) -> Option<(Range<usize>, String)> {
    const OPEN: &str = "title=\"";
    let from = info.match_indices(OPEN).map(|(at, _)| at).find(|at| {
        info[..*at]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric() && before != '_')
    })?;
    let body = from + OPEN.len();
    let mut said = String::new();
    let mut escaped = false;
    for (step, c) in info[body..].char_indices() {
        match (escaped, c) {
            (true, _) => {
                said.push(c);
                escaped = false;
            }
            (false, '\\') => escaped = true,
            (false, '"') => return Some((from..body + step + 1, said)),
            (false, _) => said.push(c),
        }
    }
    None
}
