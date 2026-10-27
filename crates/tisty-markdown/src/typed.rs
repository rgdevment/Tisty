use std::ops::Range;

use crate::{Block, Holds, Kind, Mark, Total};

// Past this a table is drawn as plain words, so one huge table never slows its page down.
const MOST_ROWS: usize = 2000;
const MOST_BYTES: usize = 256 * 1024;

/// A column's head may end in `{type}`; it is read only when every word in it is known.
pub fn typed(block: &mut Block, text: &str) {
    let Kind::Table { columns } = &mut block.kind else {
        return;
    };
    if block.inner.len() > MOST_ROWS + 1 || block.at.len() > MOST_BYTES {
        return;
    }
    let Some(head) = block.inner.first_mut() else {
        return;
    };
    for (column, cell) in columns.iter_mut().zip(head.inner.iter_mut()) {
        let Some((said, holds, total)) = braced(cell, text) else {
            continue;
        };
        cell.spans.retain_mut(|span| {
            span.at.end = span.at.end.min(said.start);
            span.at.start < span.at.end
        });
        column.holds = holds;
        column.total = total;
        column.typed = Some(said);
    }
}

fn braced(cell: &Block, text: &str) -> Option<(Range<usize>, Holds, Option<Total>)> {
    let end = cell.spans.iter().map(|one| one.at.end).max()?;
    let written = text.get(cell.at.start..end)?;
    let open = written.strip_suffix('}')?.rfind('{')?;
    let before = &written[..open];
    let kept = before.trim_end();
    // Glued to a word, as a pen's colour is, the braces belong to that word.
    if !kept.is_empty() && kept.len() == before.len() {
        return None;
    }
    let from = cell.at.start + open;
    if cell
        .spans
        .iter()
        .any(|one| one.at.end > from && one.mark != Mark::Text)
    {
        return None;
    }
    let (holds, total) = understood(&written[open + 1..written.len() - 1])?;
    Some((cell.at.start + kept.len()..end, holds, total))
}

pub(crate) fn understood(words: &str) -> Option<(Holds, Option<Total>)> {
    let mut words = words.split_whitespace();
    let mut holds = Holds::from_word(words.next()?)?;
    let mut total = None;
    for word in words {
        match (&mut holds, Total::from_word(word)) {
            (_, Some(one)) if total.is_none() => total = Some(one),
            (Holds::Money { currency }, None)
                if currency.is_none() && total.is_none() && coded(word) =>
            {
                *currency = Some(word.to_ascii_uppercase());
            }
            _ => return None,
        }
    }
    total
        .is_none_or(|one| adds_up(&holds, one))
        .then_some((holds, total))
}

// An ISO 4217 code: three letters.
fn coded(word: &str) -> bool {
    word.len() == 3 && word.bytes().all(|one| one.is_ascii_alphabetic())
}

fn adds_up(holds: &Holds, total: Total) -> bool {
    let measured = matches!(
        holds,
        Holds::Number | Holds::Money { .. } | Holds::Percent | Holds::Duration | Holds::Rating
    );
    match total {
        Total::Count => true,
        Total::Sum | Total::Avg => measured,
        Total::Min | Total::Max => measured || *holds == Holds::Date,
    }
}

#[cfg(test)]
#[path = "typed_test.rs"]
mod tests;
