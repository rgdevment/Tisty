use std::fmt;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::{Block, Column, Kind, read, values};

/// One change to a table, as the page, the editor and an assistant ask for it.
/// `row` counts the rows under the head from 0; `column` counts from 0.
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
    let doc = read(text);
    let block = found(doc.blocks(), &at).ok_or(Refused::Moved)?;
    let Kind::Table { columns } = &block.kind else {
        return Err(Refused::Moved);
    };
    let said = text[at.clone()].trim_end_matches(['\r', '\n']);
    let mut lines = Lines::read(said);
    lines.changed(asked, columns, block, text)?;
    Ok(format!(
        "{}{}{}",
        &text[..at.start],
        lines.written(),
        &text[at.start + said.len()..]
    ))
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

/// A table's lines as written, cell by cell with their spaces and escapes, so what is not touched is
/// written back byte for byte.
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

// Cells split at each `|` a backslash does not escape; the pipe that would end the line stays a cell of its own.
fn split(said: &str) -> Vec<String> {
    let mut cells = vec![String::new()];
    let mut chars = said.chars();
    while let Some(one) = chars.next() {
        let last = cells.last_mut().expect("one cell at least");
        match one {
            '\\' => {
                last.push(one);
                if let Some(next) = chars.next() {
                    last.push(next);
                }
            }
            '|' => cells.push(String::new()),
            _ => last.push(one),
        }
    }
    cells
}

// Words put in a cell: on one line, its pipes escaped, its spaces kept as wide as the cell was.
fn put(cell: &str, said: &str) -> String {
    let mut words = String::new();
    let mut chars = said.trim().chars();
    while let Some(one) = chars.next() {
        match one {
            '\\' => {
                words.push(one);
                if let Some(next) = chars.next() {
                    words.push(next);
                }
            }
            '|' => words.push_str("\\|"),
            '\r' | '\n' => words.push(' '),
            _ => words.push(one),
        }
    }
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

// A head's title and the type in braces after it, both as written.
fn titled(cell: &str) -> (&str, Option<&str>) {
    let said = cell.trim();
    let braced = said.strip_suffix('}').and_then(|inner| {
        let open = inner.rfind('{')?;
        let before = &said[..open];
        let glued = !before.is_empty() && before.trim_end().len() == before.len();
        (!glued && crate::typed::understood(&inner[open + 1..]).is_some())
            .then(|| (before.trim_end(), &said[open..]))
    });
    match braced {
        Some((title, braces)) => (title, Some(braces)),
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
    fn read(said: &str) -> Self {
        let mut lines = said.split('\n').map(Line::read);
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
                let (_, braces) = titled(cell);
                self.head.cells[*column] = put(cell, &headed(said, braces));
            }
            Edit::Retype { column, words } => {
                column_in(*column)?;
                let words = words.split_whitespace().collect::<Vec<_>>().join(" ");
                if !words.is_empty() && crate::typed::understood(&words).is_none() {
                    return Err(Refused::UnknownType);
                }
                self.head.padded(many);
                let cell = &self.head.cells[*column];
                let (title, _) = titled(cell);
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
