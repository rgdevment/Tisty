use std::ops::Range;

use pulldown_cmark::{BlockQuoteKind, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::{Block, Callout, Doc, Kind};

pub fn read(text: &str) -> Doc {
    let mut open: Vec<Block> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    for (event, at) in tokens(text) {
        match event {
            Event::Start(tag) => {
                if let Some(kind) = kind_of(&tag) {
                    open.push(Block {
                        kind,
                        at,
                        inner: Vec::new(),
                    });
                }
            }
            Event::End(end) if ends_block(end) => {
                if let Some(done) = open.pop() {
                    shelve(&mut open, &mut blocks, done);
                }
            }
            Event::Rule => shelve(&mut open, &mut blocks, leaf(Kind::Rule, at)),
            Event::TaskListMarker(ticked) => {
                if let Some(Block {
                    kind: Kind::Item { done },
                    ..
                }) = open.last_mut()
                {
                    *done = Some(ticked);
                }
            }
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

fn kind_of(tag: &Tag<'_>) -> Option<Kind> {
    Some(match tag {
        Tag::Paragraph => Kind::Paragraph,
        Tag::Heading { level, .. } => Kind::Heading(*level as u8),
        Tag::BlockQuote(None) => Kind::Quote,
        Tag::BlockQuote(Some(kind)) => Kind::Callout(callout(*kind)),
        Tag::CodeBlock(CodeBlockKind::Fenced(info)) => code(info),
        Tag::CodeBlock(CodeBlockKind::Indented) => Kind::Code {
            tongue: None,
            title: None,
        },
        Tag::HtmlBlock => Kind::Html,
        Tag::List(start) => Kind::List {
            ordered: start.is_some(),
            start: start.unwrap_or(1),
        },
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

/// A fence names its tongue first and may carry one `title="…"`, the way Tisty writes it.
fn code(info: &str) -> Kind {
    let title = titled(info);
    let rest = match &title {
        Some((at, _)) => format!("{}{}", &info[..at.start], &info[at.end..]),
        None => info.to_string(),
    };
    let tongue = rest.split_whitespace().next().map(str::to_string);
    Kind::Code {
        tongue,
        title: title.map(|(_, said)| said),
    }
}

fn titled(info: &str) -> Option<(Range<usize>, String)> {
    let from = info.find("title=\"")?;
    let body = from + "title=\"".len();
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
