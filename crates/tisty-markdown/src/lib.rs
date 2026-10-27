mod bare;
#[cfg(feature = "details")]
mod details;
pub mod diagrams;
#[cfg(feature = "emoji")]
mod emoji;
pub mod fences;
pub mod front;
mod html;
mod inline;
pub mod kept;
#[cfg(any(feature = "tags", feature = "refs"))]
mod marks;
#[cfg(feature = "math")]
pub mod math;
mod old;
#[cfg(feature = "outline")]
mod outline;
mod page;
#[cfg(feature = "pens")]
mod pens;
mod printed;
mod ranges;
mod reading;
pub mod refs;
mod short;
mod standard;
#[cfg(feature = "tables")]
mod tables;
pub mod tags;
mod transform;
pub mod vocab;
mod writing;

#[cfg(feature = "outline")]
pub use outline::{Head, outline};
pub use page::html;
pub use printed::printed;
pub use reading::read;
#[doc(hidden)]
pub use reading::read_old;
pub use standard::{Toward, to_standard};
pub use transform::to_tisty;
pub use writing::on_disk;

use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc {
    text: String,
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    pub at: Range<usize>,
    pub inner: Vec<Block>,
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub mark: Mark,
    pub at: Range<usize>,
}

/// The whole language, written for whoever asks: the window shows it and the assistant door hands it over.
pub fn reference(spanish: bool) -> &'static str {
    if spanish {
        include_str!("../reference/es.md")
    } else {
        include_str!("../reference/en.md")
    }
}

/// A link with nowhere to go yet reads as its words, never as a way back to its own page.
pub(crate) fn nowhere(to: &str) -> bool {
    to.trim().is_empty()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Text,
    Emphasis,
    Strong,
    Strike,
    Code,
    Link { to: String },
    Image { src: String },
    Html,
    Break,
    Icon { name: String, hue: Option<String> },
    Pen { color: Option<String> },
    Underline,
    Math,
    Emoji { glyph: String },
    Tag { name: String },
    Ref { name: String },
    NoteRef { label: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Heading(u8),
    Paragraph,
    Quote,
    Callout(Callout),
    List {
        start: Option<u64>,
    },
    Item {
        done: Option<bool>,
    },
    Code {
        tongue: Option<String>,
        title: Option<String>,
    },
    Table {
        columns: Vec<Column>,
    },
    Row {
        head: bool,
    },
    Cell,
    Html,
    Front,
    Rule,
    Details {
        summary: String,
        open: bool,
    },
    Note {
        label: String,
    },
    Definition {
        label: String,
        to: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub lean: Option<Lean>,
    pub wide: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lean {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Callout {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl Doc {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn said(&self, block: &Block) -> &str {
        &self.text[block.at.clone()]
    }

    pub fn write(&self) -> String {
        on_disk(&self.text)
    }

    /// A range that does not fall on this text's characters is refused rather than guessed at.
    pub fn put(&mut self, at: Range<usize>, said: &str) -> bool {
        let fits = at.start <= at.end
            && at.end <= self.text.len()
            && self.text.is_char_boundary(at.start)
            && self.text.is_char_boundary(at.end);
        if !fits {
            return false;
        }
        let mut text = std::mem::take(&mut self.text);
        text.replace_range(at, said);
        *self = read(&text);
        true
    }
}

// What another crate draws must never take the window down with it.
#[cfg(feature = "math")]
fn guarded<T>(draw: impl FnOnce() -> Option<T>) -> Option<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(draw))
        .ok()
        .flatten()
}

// A block inside a quote or a callout carries the quote marks on every one of its lines.
fn unquoted(line: &str) -> &str {
    let mut said = line.trim_start();
    while let Some(rest) = said.strip_prefix('>') {
        said = rest.trim_start();
    }
    said
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
