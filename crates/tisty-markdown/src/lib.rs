//! Tisty's own Markdown: what a document says, read once and shared by every part of Tisty.
//!
//! A document is kept as the text it was written in, with each block knowing where it sits in
//! that text. Writing an unchanged document gives back its bytes, and changing a block touches
//! only that block's bytes.

mod reading;
mod writing;

pub use reading::read;
pub use writing::settled;

use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc {
    pub text: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    pub at: Range<usize>,
    pub inner: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Heading(u8),
    Paragraph,
    Quote,
    Callout(Callout),
    List {
        ordered: bool,
        start: u64,
    },
    Item {
        done: Option<bool>,
    },
    Code {
        tongue: Option<String>,
        title: Option<String>,
    },
    Table,
    Html,
    Rule,
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
    pub fn said(&self, block: &Block) -> &str {
        &self.text[block.at.clone()]
    }

    /// The document as it goes to disk.
    pub fn write(&self) -> String {
        settled(&self.text).into_owned()
    }

    /// Puts `said` where `at` was and reads the document again; nothing outside `at` moves.
    pub fn put(&mut self, at: Range<usize>, said: &str) {
        let mut text = std::mem::take(&mut self.text);
        text.replace_range(at, said);
        *self = read(&text);
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
