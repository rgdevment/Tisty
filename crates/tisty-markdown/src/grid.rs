use std::fmt;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::{Block, Column, Kind, read, values};

/// One change to a table; `row` counts the rows under the head, and both count from 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Edit {
    Cell {
        row: usize,
        column: usize,
        said: String,
    },
    Rename {
        column: usize,
        said: String,
    },
    /// The words that go in the braces, `money CLP sum`; none takes the type away.
    Retype {
        column: usize,
        words: String,
    },
    /// A new type by its word alone, keeping the column's currency and total where they still apply.
    Recast {
        column: usize,
        word: String,
    },
    RowAdd {
        at: usize,
    },
    RowRemove {
        row: usize,
    },
    RowCopy {
        row: usize,
    },
    RowMove {
        from: usize,
        to: usize,
    },
    ColumnAdd {
        at: usize,
    },
    ColumnRemove {
        column: usize,
    },
    ColumnMove {
        from: usize,
        to: usize,
    },
    Sort {
        column: usize,
        down: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// No table starts and ends where it was asked for: the text changed since it was drawn.
    Moved,
    Beyond,
    UnknownType,
    LastColumn,
}

impl Refused {
    pub fn word(self) -> &'static str {
        match self {
            Self::Moved => "moved",
            Self::Beyond => "beyond",
            Self::UnknownType => "unknownType",
            Self::LastColumn => "lastColumn",
        }
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(self.word())
    }
}

/// The text with one table changed; the table is the one that spans `at`, or nothing is.
pub fn edit(text: &str, at: Range<usize>, asked: &Edit) -> Result<String, Refused> {
    located(text, &at, |block, columns| {
        let said = text[at.clone()].trim_end_matches(['\r', '\n']);
        let rest = &text[at.start + said.len()..];
        // The last line's `\r` sits in what follows the table, so it travels with whichever line ends up last.
        let carried = rest.starts_with("\r\n");
        let mut lines = Lines::read(said, carried);
        lines.changed(asked, columns, block, text)?;
        let written = lines.written();
        let written = if carried {
            written.strip_suffix('\r').unwrap_or(&written)
        } else {
            &written
        };
        Ok(format!("{}{written}{rest}", &text[..at.start]))
    })
}

/// Edits one after another on the same table, as one change: a cell written and its row moved.
pub fn edits(text: &str, at: Range<usize>, asked: &[Edit]) -> Result<String, Refused> {
    let mut now = text.to_string();
    let mut at = at;
    for one in asked {
        let next = edit(&now, at.clone(), one)?;
        // Only the table changed, so it still starts where it did and ends as much later as the text grew.
        let end = (at.end + next.len())
            .checked_sub(now.len())
            .ok_or(Refused::Moved)?;
        at = at.start..end;
        now = next;
    }
    Ok(now)
}

/// The rows under the head, by their place as written, in the order a column sorts them.
pub fn order(
    text: &str,
    at: Range<usize>,
    column: usize,
    down: bool,
) -> Result<Vec<usize>, Refused> {
    located(text, &at, |block, columns| {
        if column >= columns.len() {
            return Err(Refused::Beyond);
        }
        Ok(values::sorted(columns, block, text, column, down))
    })
}

/// A table as an assistant is told it: the heading over it, its columns with their types, its rows by their first cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Shown {
    #[serde(skip)]
    pub at: Range<usize>,
    pub heading: Option<String>,
    pub columns: Vec<Head>,
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Head {
    pub title: String,
    /// The words in its braces, `money CLP sum`; none for a column of words.
    #[serde(rename = "type")]
    pub kind: String,
}

/// Every table of a text in reading order, each named by the heading it sits under.
pub fn shown(text: &str) -> Vec<Shown> {
    fn walk(blocks: &[Block], text: &str, heading: &mut Option<String>, out: &mut Vec<Shown>) {
        for one in blocks {
            match &one.kind {
                Kind::Heading(_) => {
                    let said = text[one.at.clone()].trim().trim_start_matches('#').trim();
                    *heading = Some(said.to_string());
                }
                Kind::Table { columns } => out.push(listed(one, columns, text, heading.clone())),
                _ => walk(&one.inner, text, heading, out),
            }
        }
    }
    let doc = read(text);
    let mut out = Vec::new();
    walk(doc.blocks(), text, &mut None, &mut out);
    out
}

fn listed(block: &Block, columns: &[Column], text: &str, heading: Option<String>) -> Shown {
    let said = text[block.at.clone()].trim_end_matches(['\r', '\n']);
    let lines = Lines::read(said, false);
    let columns = columns
        .iter()
        .enumerate()
        .map(|(at, column)| {
            let cell = lines.head.cells.get(at).map_or("", String::as_str);
            let kind = column.typed.as_ref().and_then(|at| text.get(at.clone()));
            let kind = kind.map_or("", |one| {
                one.trim().trim_start_matches('{').trim_end_matches('}')
            });
            Head {
                title: titled(cell, column, text).0.to_string(),
                kind: kind.to_string(),
            }
        })
        .collect();
    let rows = block
        .inner
        .iter()
        .skip(1)
        .map(|row| {
            row.inner
                .first()
                .map(|cell| values::visible(cell, text))
                .unwrap_or_default()
        })
        .collect();
    Shown {
        at: block.at.clone(),
        heading,
        columns,
        rows,
    }
}

/// The cells of a table written not as their column's type asks, by row under the head and column.
pub fn amiss(text: &str, at: Range<usize>) -> Result<Vec<(usize, usize)>, Refused> {
    located(text, &at, |block, columns| {
        let read = values::read(columns, block, text);
        Ok(read
            .rows
            .iter()
            .enumerate()
            .flat_map(|(row, cells)| {
                cells
                    .iter()
                    .enumerate()
                    .filter(|(_, one)| one.amiss)
                    .map(move |(column, _)| (row, column))
            })
            .collect())
    })
}

/// A table's words as written, escapes kept, so a cell opens for writing just as it is in the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Written {
    /// Each head without its type, which the column's menu changes on its own.
    pub heads: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn written(text: &str, at: Range<usize>) -> Result<Written, Refused> {
    located(text, &at, |_, columns| {
        let said = text[at.clone()].trim_end_matches(['\r', '\n']);
        let lines = Lines::read(said, false);
        let heads = (0..columns.len())
            .map(|at| {
                let cell = lines.head.cells.get(at).map_or("", String::as_str);
                titled(cell, &columns[at], text).0.to_string()
            })
            .collect();
        let rows = lines
            .body
            .iter()
            .map(|line| {
                line.cells
                    .iter()
                    .map(|one| one.trim().to_string())
                    .collect()
            })
            .collect();
        Ok(Written { heads, rows })
    })
}

fn located<T>(
    text: &str,
    at: &Range<usize>,
    then: impl FnOnce(&Block, &[Column]) -> Result<T, Refused>,
) -> Result<T, Refused> {
    let doc = read(text);
    let block = found(doc.blocks(), at).ok_or(Refused::Moved)?;
    let Kind::Table { columns } = &block.kind else {
        return Err(Refused::Moved);
    };
    then(block, columns)
}

// The words of a new type for a column: its currency if it stays money, its total if that still adds up.
fn recast(column: &Column, word: &str) -> Option<String> {
    let holds = crate::Holds::from_word(word)?;
    let currency = match (&holds, &column.holds) {
        (crate::Holds::Money { .. }, crate::Holds::Money { currency }) => currency.clone(),
        _ => None,
    };
    let total = column
        .total
        .filter(|one| crate::typed::adds_up(&holds, *one))
        .map(crate::Total::word);
    let kept = [Some(word), currency.as_deref(), total]
        .into_iter()
        .flatten();
    let words = kept.collect::<Vec<_>>().join(" ");
    Some(if words == "text" {
        String::new()
    } else {
        words
    })
}

/// Edits on the table that holds a byte of the text, the caret's, say, found and changed in one ask.
pub fn edits_near(text: &str, byte: usize, asked: &[Edit]) -> Result<String, Refused> {
    let at = around(text, byte).ok_or(Refused::Moved)?;
    edits(text, at, asked)
}

/// Where the table that holds a byte of the text starts and ends, as `edit` wants it named.
pub fn around(text: &str, byte: usize) -> Option<Range<usize>> {
    fn within(blocks: &[Block], byte: usize) -> Option<Range<usize>> {
        blocks.iter().find_map(|one| {
            let inside = one.at.start <= byte && byte < one.at.end.max(one.at.start + 1);
            match &one.kind {
                Kind::Table { .. } if inside => Some(one.at.clone()),
                _ if inside => within(&one.inner, byte),
                _ => None,
            }
        })
    }
    within(read(text).blocks(), byte)
}

fn found<'a>(blocks: &'a [Block], at: &Range<usize>) -> Option<&'a Block> {
    blocks.iter().find_map(|one| {
        if one.at == *at && matches!(one.kind, Kind::Table { .. }) {
            Some(one)
        } else {
            found(&one.inner, at)
        }
    })
}

/// A table line kept as written, spaces and escapes included, so an untouched one returns byte for byte.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    prefix: String,
    lead: bool,
    cells: Vec<String>,
    /// The spaces after a closing pipe, when the line has one.
    trail: Option<String>,
    end: String,
}

impl Line {
    fn read(said: &str) -> Self {
        let (said, end) = match said.strip_suffix('\r') {
            Some(rest) => (rest, "\r"),
            None => (said, ""),
        };
        let start = said.len() - said.trim_start_matches(['>', ' ', '\t']).len();
        let (prefix, rest) = said.split_at(start);
        let lead = rest.starts_with('|');
        let rest = if lead { &rest[1..] } else { rest };
        let mut cells = split(rest);
        let closed = cells.len() > 1 && cells.last().is_some_and(|one| one.trim().is_empty());
        let trail = if closed { cells.pop() } else { None };
        Self {
            prefix: prefix.to_string(),
            lead,
            cells,
            trail,
            end: end.to_string(),
        }
    }

    fn written(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.prefix,
            if self.lead { "|" } else { "" },
            self.cells.join("|"),
            self.trail
                .as_ref()
                .map_or(String::new(), |after| format!("|{after}")),
            self.end
        )
    }

    // A row shorter than its head is filled out before a cell past its end is touched.
    fn padded(&mut self, many: usize) {
        while self.cells.len() < many {
            self.cells.push("  ".to_string());
        }
    }

    // An empty row shaped as this one: the same quote marks, outer pipes and line end.
    fn emptied(&self, many: usize) -> Self {
        Self {
            cells: vec!["  ".to_string(); many],
            ..self.clone()
        }
    }
}

// A backslash and the character after it are one piece, so what it escapes is never read on its own.
fn pieces(said: &str) -> impl Iterator<Item = &str> {
    let mut at = 0;
    std::iter::from_fn(move || {
        let mut chars = said[at..].chars();
        let first = chars.next()?;
        let wide = first.len_utf8()
            + chars
                .next()
                .filter(|_| first == '\\')
                .map_or(0, char::len_utf8);
        let piece = &said[at..at + wide];
        at += wide;
        Some(piece)
    })
}

// Cells split at each `|` a backslash does not escape; the pipe that would end the line stays a cell of its own.
fn split(said: &str) -> Vec<String> {
    let mut cells = vec![String::new()];
    for piece in pieces(said) {
        if piece == "|" {
            cells.push(String::new());
        } else if let Some(last) = cells.last_mut() {
            last.push_str(piece);
        }
    }
    cells
}

// Words put in a cell: on one line, its pipes escaped, its spaces kept as wide as the cell was.
fn put(cell: &str, said: &str) -> String {
    let words: String = pieces(said.trim())
        .map(|piece| match piece {
            "|" => r"\|",
            "\r" | "\n" => " ",
            _ => piece,
        })
        .collect();
    let lead = &cell[..cell.len() - cell.trim_start().len()];
    let lead = if lead.is_empty() || cell.trim().is_empty() {
        " "
    } else {
        lead
    };
    let made = format!("{lead}{words} ");
    let wide = cell.chars().count();
    let short = wide.saturating_sub(made.chars().count());
    format!("{made}{}", " ".repeat(short))
}

// The type in braces is where reading the table found it, so a head is never read by a second rule.
fn titled<'a>(cell: &'a str, column: &Column, text: &'a str) -> (&'a str, Option<&'a str>) {
    let said = cell.trim();
    let braces = column
        .typed
        .as_ref()
        .and_then(|at| text.get(at.clone()))
        .map(str::trim);
    match braces.and_then(|one| said.strip_suffix(one)) {
        Some(title) => (title.trim_end(), braces),
        None => (said, None),
    }
}

fn headed(title: &str, braces: Option<&str>) -> String {
    match (title.trim().is_empty(), braces) {
        (_, None) => title.trim().to_string(),
        (true, Some(braces)) => braces.to_string(),
        (false, Some(braces)) => format!("{} {braces}", title.trim()),
    }
}

struct Lines {
    head: Line,
    rule: Line,
    body: Vec<Line>,
}

impl Lines {
    fn read(said: &str, carried: bool) -> Self {
        let mut lines: Vec<Line> = said.split('\n').map(Line::read).collect();
        if let Some(last) = lines.last_mut().filter(|_| carried) {
            "\r".clone_into(&mut last.end);
        }
        let mut lines = lines.into_iter();
        let head = lines.next().unwrap_or_else(|| Line::read(""));
        let rule = lines.next().unwrap_or_else(|| head.emptied(0));
        Self {
            head,
            rule,
            body: lines.collect(),
        }
    }

    fn written(&self) -> String {
        std::iter::once(&self.head)
            .chain(std::iter::once(&self.rule))
            .chain(&self.body)
            .map(Line::written)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn all(&mut self) -> impl Iterator<Item = &mut Line> {
        std::iter::once(&mut self.head)
            .chain(std::iter::once(&mut self.rule))
            .chain(&mut self.body)
    }

    fn changed(
        &mut self,
        asked: &Edit,
        columns: &[Column],
        block: &Block,
        text: &str,
    ) -> Result<(), Refused> {
        let many = columns.len();
        let rows = self.body.len();
        let column_in = |at: usize| (at < many).then_some(()).ok_or(Refused::Beyond);
        let row_in = |at: usize| (at < rows).then_some(()).ok_or(Refused::Beyond);
        match asked {
            Edit::Cell { row, column, said } => {
                row_in(*row)?;
                column_in(*column)?;
                let line = &mut self.body[*row];
                line.padded(many);
                line.cells[*column] = put(&line.cells[*column], said);
            }
            Edit::Rename { column, said } => {
                column_in(*column)?;
                self.head.padded(many);
                let cell = &self.head.cells[*column];
                let (_, braces) = titled(cell, &columns[*column], text);
                self.head.cells[*column] = put(cell, &headed(said, braces));
            }
            Edit::Recast { column, word } => {
                column_in(*column)?;
                let words = recast(&columns[*column], word).ok_or(Refused::UnknownType)?;
                return self.changed(
                    &Edit::Retype {
                        column: *column,
                        words,
                    },
                    columns,
                    block,
                    text,
                );
            }
            Edit::Retype { column, words } => {
                column_in(*column)?;
                let words = words.split_whitespace().collect::<Vec<_>>().join(" ");
                if !words.is_empty() && crate::typed::understood(&words).is_none() {
                    return Err(Refused::UnknownType);
                }
                self.head.padded(many);
                let cell = &self.head.cells[*column];
                let (title, _) = titled(cell, &columns[*column], text);
                let braces = (!words.is_empty()).then(|| format!("{{{words}}}"));
                self.head.cells[*column] = put(cell, &headed(title, braces.as_deref()));
            }
            Edit::RowAdd { at } => {
                if *at > rows {
                    return Err(Refused::Beyond);
                }
                let empty = self.body.first().unwrap_or(&self.head).emptied(many);
                self.body.insert(*at, empty);
            }
            Edit::RowRemove { row } => {
                row_in(*row)?;
                self.body.remove(*row);
            }
            Edit::RowCopy { row } => {
                row_in(*row)?;
                let copy = self.body[*row].clone();
                self.body.insert(row + 1, copy);
            }
            Edit::RowMove { from, to } => {
                row_in(*from)?;
                row_in(*to)?;
                let line = self.body.remove(*from);
                self.body.insert(*to, line);
            }
            Edit::ColumnAdd { at } => {
                if *at > many {
                    return Err(Refused::Beyond);
                }
                for line in self.all() {
                    line.padded(many);
                    line.cells.insert(*at, "  ".to_string());
                }
                self.rule.cells[*at] = " --- ".to_string();
            }
            Edit::ColumnRemove { column } => {
                column_in(*column)?;
                if many == 1 {
                    return Err(Refused::LastColumn);
                }
                for line in self.all() {
                    line.padded(many);
                    line.cells.remove(*column);
                }
            }
            Edit::ColumnMove { from, to } => {
                column_in(*from)?;
                column_in(*to)?;
                for line in self.all() {
                    line.padded(many);
                    let cell = line.cells.remove(*from);
                    line.cells.insert(*to, cell);
                }
            }
            Edit::Sort { column, down } => {
                column_in(*column)?;
                let order = values::sorted(columns, block, text, *column, *down);
                if order.len() != rows {
                    return Err(Refused::Moved);
                }
                self.body = order.into_iter().map(|at| self.body[at].clone()).collect();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "grid_test.rs"]
mod tests;
