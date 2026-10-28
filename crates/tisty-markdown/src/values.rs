#[cfg(feature = "tables")]
use std::cmp::Ordering;
use std::ops::Range;

#[cfg(feature = "tables")]
use unicode_normalization::UnicodeNormalization;

use crate::amounts::{Amount, convention, money, rounded};
use crate::inline::{atom, code_words, walls};
use crate::ranges::{Nearby, outside};
use crate::{Block, Column, Holds, Mark, Total};

/// How a cell of a typed column reads: what it shows, the key it sorts by, and whether it fits its type.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Seen {
    /// What is drawn in place of the words, for a type that draws its values its own way.
    pub shown: Option<String>,
    pub key: Option<String>,
    /// Written, but not as its type is written: drawn as it is, with a mark, never refused.
    pub amiss: bool,
    value: Option<Value>,
}

/// A table's typed cells, read once: each body row's cells, and the foot when a column adds up.
#[derive(Debug, Default)]
pub struct Read {
    pub rows: Vec<Vec<Seen>>,
    pub foot: Option<Vec<String>>,
}

impl Read {
    pub fn at(&self, row: usize, column: usize) -> Option<&Seen> {
        self.rows.get(row)?.get(column)
    }
}

pub fn read(columns: &[Column], block: &Block, text: &str) -> Read {
    if columns.iter().all(|one| one.typed.is_none()) {
        return Read::default();
    }
    let rows: Vec<Vec<Seen>> = block
        .inner
        .iter()
        .skip(1)
        .map(|row| {
            row.inner
                .iter()
                .zip(columns)
                .map(|(cell, column)| match column.typed {
                    Some(_) => seen(&column.holds, &worded(&column.holds, cell, text)),
                    None => Seen::default(),
                })
                .collect()
        })
        .collect();
    let foot = footed(columns, &rows);
    Read { rows, foot }
}

/// Where a cell's words sit in the text, from its first mark to its last.
pub fn spread(cell: &Block) -> Option<Range<usize>> {
    let start = cell.spans.iter().map(|one| one.at.start).min()?;
    let end = cell.spans.iter().map(|one| one.at.end).max()?;
    Some(start..end)
}

// A link is known by how it is written; any other value by the words a reader sees, marks aside.
/// A cell's words as a reader sees them, marks aside.
pub fn visible(cell: &Block, text: &str) -> String {
    worded(&Holds::Text, cell, text)
}

fn worded(holds: &Holds, cell: &Block, text: &str) -> String {
    if *holds == Holds::Link {
        return spread(cell)
            .and_then(|at| text.get(at))
            .unwrap_or_default()
            .trim()
            .to_string();
    }
    let mut silent: Vec<Range<usize>> = cell
        .spans
        .iter()
        .filter(|one| atom(&one.mark))
        .map(|one| one.at.clone())
        .collect();
    silent.extend(walls(&cell.spans, text));
    let silent = Nearby::merged(silent);
    let mut out = String::new();
    for span in &cell.spans {
        match &span.mark {
            Mark::Text => {
                for piece in outside(span.at.clone(), silent.near(&span.at)) {
                    out.push_str(&text[piece]);
                }
            }
            Mark::Code => out.push_str(&code_words(&text[span.at.clone()])),
            Mark::Ref { name } => out.push_str(name),
            Mark::Emoji { glyph } => out.push_str(glyph),
            Mark::Tag { name } => {
                out.push('#');
                out.push_str(name);
            }
            _ => {}
        }
    }
    out.trim().to_string()
}

/// The body rows in the order a column sorts them: by value as its type reads, then the empty and the amiss, ties as written.
#[cfg(feature = "tables")]
pub fn sorted(columns: &[Column], block: &Block, text: &str, at: usize, down: bool) -> Vec<usize> {
    let holds = columns
        .get(at)
        .map(|one| &one.holds)
        .unwrap_or(&Holds::Text);
    let seen: Vec<Seen> = block
        .inner
        .iter()
        .skip(1)
        .map(|row| {
            row.inner.get(at).map_or_else(Seen::default, |cell| {
                seen(holds, &worded(holds, cell, text))
            })
        })
        .collect();
    let ranks: Vec<Option<Value>> = seen
        .iter()
        .map(|one| match &one.value {
            Some(Value::Words(words)) => Some(Value::Words(folded(words))),
            value => value.clone(),
        })
        .collect();
    let mut order: Vec<usize> = (0..seen.len()).collect();
    order.sort_by(|&one, &two| match (&ranks[one], &ranks[two]) {
        (Some(one), Some(two)) if down => ranked(two, one),
        (Some(one), Some(two)) => ranked(one, two),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => seen[two].amiss.cmp(&seen[one].amiss),
    });
    order
}

#[cfg(feature = "tables")]
fn ranked(one: &Value, two: &Value) -> Ordering {
    match (one, two) {
        (Value::Amount(one), Value::Amount(two)) => one.against(two),
        (Value::Minutes(one), Value::Minutes(two)) => one.cmp(two),
        (Value::Date(one), Value::Date(two)) | (Value::Words(one), Value::Words(two)) => {
            one.cmp(two)
        }
        (Value::Yes(one), Value::Yes(two)) => one.cmp(two),
        _ => Ordering::Equal,
    }
}

// Accents come off for sorting, so «Ávila» sits with the A's and not after the Z's.
#[cfg(feature = "tables")]
fn folded(said: &str) -> String {
    said.nfd()
        .filter(|one| !unicode_normalization::char::is_combining_mark(*one))
        .collect()
}

pub fn seen(holds: &Holds, said: &str) -> Seen {
    if said.is_empty() {
        return Seen::default();
    }
    let Some(value) = valued(holds, said) else {
        return Seen {
            amiss: true,
            ..Seen::default()
        };
    };
    Seen {
        shown: shown(holds, &value),
        key: Some(keyed(&value)),
        amiss: false,
        value: Some(value),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Amount(Amount),
    Minutes(i128),
    Date(String),
    Yes(bool),
    Words(String),
}

fn valued(holds: &Holds, said: &str) -> Option<Value> {
    Some(match holds {
        Holds::Number | Holds::Percent => Value::Amount(Amount::read(said)?),
        Holds::Money { currency } => {
            let amount = Amount::read(said)?;
            if amount.scale > convention(currency.as_deref()).decimals {
                return None;
            }
            Value::Amount(amount)
        }
        Holds::Rating => {
            let amount = Amount::read(said)?;
            if amount.scale > 0 || !(0..=STARS).contains(&amount.units) {
                return None;
            }
            Value::Amount(amount)
        }
        Holds::Duration => Value::Minutes(minutes(said)?),
        Holds::Date => Value::Date(dated(said)?.to_string()),
        Holds::Check => Value::Yes(checked(said)?),
        Holds::Url => Value::Words(webbed(said)?),
        Holds::Email => Value::Words(mailed(said)?),
        Holds::Phone => Value::Words(dialled(said)?),
        Holds::Link => Value::Words(linked(said)?),
        Holds::Text | Holds::Select | Holds::Tags => Value::Words(said.to_lowercase()),
    })
}

fn shown(holds: &Holds, value: &Value) -> Option<String> {
    match (holds, value) {
        (Holds::Money { currency }, Value::Amount(amount)) => {
            Some(money(amount, currency.as_deref()))
        }
        (Holds::Percent, Value::Amount(amount)) => Some(format!("{}%", amount.plain())),
        (Holds::Rating, Value::Amount(amount)) => Some(stars(amount.units)),
        (Holds::Duration, Value::Minutes(minutes)) => Some(lasting(*minutes)),
        (Holds::Check, Value::Yes(yes)) => Some(ticked(*yes)),
        _ => None,
    }
}

fn keyed(value: &Value) -> String {
    match value {
        Value::Amount(amount) => amount.plain(),
        Value::Minutes(minutes) => minutes.to_string(),
        Value::Date(date) | Value::Words(date) => date.clone(),
        Value::Yes(yes) => u8::from(*yes).to_string(),
    }
}

// One cell per column, «Total» in the first when it adds nothing up itself.
fn footed(columns: &[Column], rows: &[Vec<Seen>]) -> Option<Vec<String>> {
    columns
        .iter()
        .any(|one| one.total.is_some())
        .then_some(())?;
    let mut foot: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(at, column)| {
            let Some(total) = column.total else {
                return String::new();
            };
            let cells: Vec<&Seen> = rows.iter().filter_map(|row| row.get(at)).collect();
            totalled(&column.holds, total, &cells)
        })
        .collect();
    if let Some(first) = foot.first_mut().filter(|one| one.is_empty()) {
        "Total".clone_into(first);
    }
    Some(foot)
}

fn totalled(holds: &Holds, total: Total, cells: &[&Seen]) -> String {
    let values: Vec<&Value> = cells.iter().filter_map(|one| one.value.as_ref()).collect();
    if total == Total::Count {
        let counted = if *holds == Holds::Check {
            values
                .iter()
                .filter(|one| matches!(one, Value::Yes(true)))
                .count()
        } else {
            cells
                .iter()
                .filter(|one| one.amiss || one.value.is_some())
                .count()
        };
        return counted.to_string();
    }
    match (values.first(), total) {
        (None, _) => String::new(),
        (Some(Value::Date(_)), Total::Min | Total::Max) => {
            let dates = values.iter().filter_map(|one| match one {
                Value::Date(date) => Some(date),
                _ => None,
            });
            let picked = if total == Total::Min {
                dates.min()
            } else {
                dates.max()
            };
            picked.cloned().unwrap_or_default()
        }
        (Some(Value::Minutes(_)), _) => {
            let all: Vec<i128> = values
                .iter()
                .filter_map(|one| match one {
                    Value::Minutes(minutes) => Some(*minutes),
                    _ => None,
                })
                .collect();
            lasting(gathered(&all, total))
        }
        _ => {
            let all: Vec<&Amount> = values
                .iter()
                .filter_map(|one| match one {
                    Value::Amount(amount) => Some(amount),
                    _ => None,
                })
                .collect();
            amounted(holds, total, &all)
        }
    }
}

fn gathered(all: &[i128], total: Total) -> i128 {
    let sum: i128 = all.iter().sum();
    match total {
        Total::Min => all.iter().copied().min().unwrap_or_default(),
        Total::Max => all.iter().copied().max().unwrap_or_default(),
        Total::Avg => rounded(sum, all.len() as i128),
        Total::Sum | Total::Count => sum,
    }
}

fn amounted(holds: &Holds, total: Total, all: &[&Amount]) -> String {
    let sum = all.iter().fold(Amount::default(), |one, two| one.plus(two));
    let picked = match total {
        Total::Min => all
            .iter()
            .min_by(|one, two| one.against(two))
            .map(|one| (*one).clone()),
        Total::Max => all
            .iter()
            .max_by(|one, two| one.against(two))
            .map(|one| (*one).clone()),
        Total::Avg => {
            let scale = match holds {
                Holds::Money { currency } => convention(currency.as_deref()).decimals,
                Holds::Rating => 1,
                _ => sum.scale.max(2),
            };
            Some(sum.shared(all.len() as i128, scale).trimmed(holds))
        }
        Total::Sum | Total::Count => Some(sum),
    };
    let Some(amount) = picked else {
        return String::new();
    };
    match holds {
        Holds::Money { currency } => money(&amount, currency.as_deref()),
        Holds::Percent => format!("{}%", amount.plain()),
        Holds::Rating if matches!(total, Total::Min | Total::Max) => stars(amount.units),
        _ => amount.plain(),
    }
}

const STARS: i128 = 5;

fn stars(many: i128) -> String {
    let lit = usize::try_from(many.clamp(0, STARS)).unwrap_or_default();
    format!("{}{}", "★".repeat(lit), "☆".repeat(5 - lit))
}

fn ticked(yes: bool) -> String {
    if yes { "✓" } else { "" }.to_string()
}

fn checked(said: &str) -> Option<bool> {
    match said {
        "[x]" | "[X]" => Some(true),
        "[ ]" => Some(false),
        _ => None,
    }
}

// Hours and minutes, written together: `1h30m`, `2h`, `45m`.
fn minutes(said: &str) -> Option<i128> {
    let (hours, rest) = match said.split_once('h') {
        Some((hours, rest)) => (Some(hours), rest),
        None => (None, said),
    };
    let minutes = rest.strip_suffix('m');
    if rest.is_empty() == minutes.is_some() || (hours.is_none() && minutes.is_none()) {
        return None;
    }
    let number = |one: &str| {
        (!one.is_empty() && one.len() < 10 && one.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| one.parse::<i128>().ok())
            .flatten()
    };
    let hours = hours.map_or(Some(0), number)?;
    let minutes = minutes.map_or(Some(0), number)?;
    Some(hours * 60 + minutes)
}

fn lasting(minutes: i128) -> String {
    let (hours, rest) = (minutes / 60, minutes % 60);
    match (hours, rest) {
        (0, rest) => format!("{rest} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, rest) => format!("{hours} h {rest} min"),
    }
}

// Always year, month and day, so the file sorts the way the calendar does.
fn dated(said: &str) -> Option<&str> {
    let mut parts = said.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return None;
    }
    let (year, month, day): (u32, u32, u32) =
        (year.parse().ok()?, month.parse().ok()?, day.parse().ok()?);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let last = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => return None,
    };
    (1..=last).contains(&day).then_some(said)
}

fn webbed(said: &str) -> Option<String> {
    let rest = said
        .strip_prefix("https://")
        .or_else(|| said.strip_prefix("http://"))?;
    (!rest.is_empty() && !said.contains(char::is_whitespace)).then(|| said.to_lowercase())
}

fn mailed(said: &str) -> Option<String> {
    let (name, place) = said.split_once('@')?;
    let fits = !name.is_empty()
        && place.contains('.')
        && !place.starts_with('.')
        && !place.ends_with('.')
        && !said.contains(char::is_whitespace)
        && !place.contains('@');
    fits.then(|| said.to_lowercase())
}

fn dialled(said: &str) -> Option<String> {
    let digits: String = said.chars().filter(char::is_ascii_digit).collect();
    let fits = digits.len() >= 3
        && said
            .chars()
            .all(|one| one.is_ascii_digit() || " +-().".contains(one));
    fits.then_some(digits)
}

// Another document, named as a reference, `[[Lease]]`, or linked as the `/` menu writes it, `![Lease](tisty:doc/…)`.
fn linked(said: &str) -> Option<String> {
    let name = match said.strip_prefix("[[") {
        Some(rest) => rest.strip_suffix("]]").filter(|one| !one.contains("]]"))?,
        None => {
            let (name, to) = said
                .strip_prefix('!')
                .unwrap_or(said)
                .strip_prefix('[')?
                .strip_suffix(')')?
                .split_once("](")?;
            let id = to.strip_prefix("tisty:doc/")?;
            if id.is_empty() || id.contains(char::is_whitespace) {
                return None;
            }
            name
        }
    };
    (!name.trim().is_empty()).then(|| name.trim().to_lowercase())
}

#[cfg(all(test, feature = "tables"))]
#[path = "values_test.rs"]
mod tests;
