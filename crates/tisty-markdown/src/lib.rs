mod amounts;
mod bare;
#[cfg(feature = "tables")]
pub mod canon;
#[cfg(feature = "details")]
mod details;
pub mod diagrams;
#[cfg(feature = "emoji")]
mod emoji;
pub mod fences;
pub mod front;
#[cfg(feature = "tables")]
pub mod grid;
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
mod outside;
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
#[cfg(feature = "tables")]
mod typed;
mod values;
pub mod vocab;
mod writing;

pub use inline::filed;
#[cfg(feature = "outline")]
pub use outline::{Head, outline};
pub use outside::{file_as_tisty, read_file};
pub use page::{html, html_placed, pieces};
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
    Aligned {
        how: String,
    },
    Definition {
        label: String,
        to: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Column {
    pub lean: Option<Lean>,
    pub wide: Option<u32>,
    pub holds: Holds,
    pub total: Option<Total>,
    /// Where ` {type}` sits at the end of the head, which only Tisty reads.
    pub typed: Option<Range<usize>>,
}

/// What a column holds, said as `{type}` at the end of its head; without one, words.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Holds {
    #[default]
    Text,
    Number,
    Money {
        currency: Option<String>,
    },
    Percent,
    Date,
    Duration,
    Check,
    Select,
    Tags,
    Rating,
    Link,
    Url,
    Email,
    Phone,
}

impl Column {
    /// Where its values lean: as the rule under the head says, or else as its type does.
    pub fn leaning(&self) -> Option<Lean> {
        self.lean.or_else(|| self.holds.lean())
    }
}

impl Holds {
    /// The type a word in `vocab::HOLDS` names; a money column's currency is read after it.
    pub fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "text" => Self::Text,
            "number" => Self::Number,
            "money" => Self::Money { currency: None },
            "percent" => Self::Percent,
            "date" => Self::Date,
            "duration" => Self::Duration,
            "check" => Self::Check,
            "select" => Self::Select,
            "tags" => Self::Tags,
            "rating" => Self::Rating,
            "link" => Self::Link,
            "url" => Self::Url,
            "email" => Self::Email,
            "phone" => Self::Phone,
            _ => return None,
        })
    }

    /// The word the type is written with in a column's head.
    pub fn word(&self) -> &'static str {
        let same = |one: &Self| std::mem::discriminant(one) == std::mem::discriminant(self);
        vocab::HOLDS
            .iter()
            .find(|word| Self::from_word(word).is_some_and(|one| same(&one)))
            .copied()
            .unwrap_or_default()
    }

    /// A value of this type reads on one line: a date or an amount split in two is misread.
    pub fn whole(&self) -> bool {
        matches!(
            self,
            Self::Number
                | Self::Money { .. }
                | Self::Percent
                | Self::Date
                | Self::Duration
                | Self::Rating
                | Self::Phone
        )
    }

    /// Where its values lean when the rule under the head does not say.
    pub fn lean(&self) -> Option<Lean> {
        match self {
            Self::Number | Self::Money { .. } | Self::Percent | Self::Duration => Some(Lean::Right),
            Self::Check => Some(Lean::Center),
            _ => None,
        }
    }
}

/// What a column adds up at its foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Total {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

impl Total {
    /// The total a word in `vocab::TOTALS` names.
    pub fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "sum" => Self::Sum,
            "avg" => Self::Avg,
            "min" => Self::Min,
            "max" => Self::Max,
            "count" => Self::Count,
            _ => return None,
        })
    }

    pub fn word(self) -> &'static str {
        vocab::TOTALS
            .iter()
            .find(|word| Self::from_word(word) == Some(self))
            .copied()
            .unwrap_or_default()
    }
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
    Idea,
}

pub const IDEA_HUES: [&str; 12] = [
    "pink", "coral", "orange", "yellow", "lime", "green", "teal", "cyan", "blue", "lavender",
    "violet", "magenta",
];

#[derive(Debug, Default, PartialEq, Eq)]
pub struct IdeaHead<'a> {
    pub when: Option<&'a str>,
    pub hue: Option<&'static str>,
    pub side: Option<&'static str>,
}

pub fn idea_words(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix('>')?.trim_start();
    rest.get(..7)
        .filter(|marker| marker.eq_ignore_ascii_case("[!idea]"))
        .map(|_| rest[7..].trim())
}

pub fn idea_head(source: &str) -> IdeaHead<'_> {
    source
        .lines()
        .next()
        .and_then(idea_words)
        .map(idea_parts)
        .unwrap_or_default()
}

pub fn idea_parts(words: &str) -> IdeaHead<'_> {
    let mut head = IdeaHead::default();
    for part in words.split(" · ").map(str::trim) {
        let lower = part.to_ascii_lowercase();
        if let Some(hue) = IDEA_HUES.iter().find(|one| **one == lower) {
            head.hue = Some(hue);
        } else if lower == "left" || lower == "right" {
            head.side = Some(if lower == "left" { "left" } else { "right" });
        } else if head.when.is_none() && stamped(part) {
            head.when = Some(part);
        }
    }
    head
}

impl<'a> IdeaHead<'a> {
    pub fn says_any(&self) -> bool {
        self.when.is_some() || self.hue.is_some() || self.side.is_some()
    }

    pub(crate) fn of(kind: Callout, source: &'a str) -> Self {
        match kind {
            Callout::Idea => idea_head(source),
            _ => Self::default(),
        }
    }

    pub(crate) fn said(&self) -> impl Iterator<Item = (&'static str, &'a str)> {
        [("when", self.when), ("hue", self.hue), ("side", self.side)]
            .into_iter()
            .filter_map(|(key, value)| value.map(|value| (key, value)))
    }
}

fn stamped(said: &str) -> bool {
    let shape = said
        .chars()
        .map(|c| if c.is_ascii_digit() { '9' } else { c });
    let shape: String = shape.collect();
    shape == "9999-99-99 99:99" || shape == "9999-99-99"
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
