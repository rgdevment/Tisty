use std::ops::Range;

use pulldown_cmark::{BlockQuoteKind, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::{Block, Callout, Doc, Kind};

pub fn read(text: &str) -> Doc {
    // The core and the window both read past a byte-order mark; the tokenizer does not.
    let skip = text.len() - text.trim_start_matches('\u{feff}').len();
    let mut open: Vec<Block> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    for (event, at) in tokens(&text[skip..]) {
        let at = at.start + skip..at.end + skip;
        match event {
            Event::Start(tag) => {
                if let Some(kind) = kind_of(&tag, &text[at.clone()]) {
                    open.push(leaf(kind, at));
                }
            }
            Event::End(end) if ends_block(end) => {
                if let Some(mut done) = open.pop() {
                    if done.kind == Kind::Quote
                        && let Some(said) = marked(&text[done.at.clone()])
                    {
                        done.kind = Kind::Callout(said);
                    }
                    shelve(&mut open, &mut blocks, done);
                }
            }
            Event::Rule => shelve(&mut open, &mut blocks, leaf(Kind::Rule, at)),
            Event::TaskListMarker(ticked) => tick(&mut open, ticked),
            _ => {}
        }
    }
    Doc {
        text: text.to_string(),
        blocks,
    }
}

/// The only place the tokenizer is named: swapping it for another touches nothing else.
fn tokens(text: &str) -> impl Iterator<Item = (Event<'_>, Range<usize>)> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_GFM;
    Parser::new_ext(text, options).into_offset_iter()
}

fn leaf(kind: Kind, at: Range<usize>) -> Block {
    Block {
        kind,
        at,
        inner: Vec::new(),
    }
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
        Tag::Table(_) => Kind::Table,
        _ => return None,
    })
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
