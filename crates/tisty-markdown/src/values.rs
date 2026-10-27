use std::cmp::Ordering;

use crate::{Block, Column, Holds, Total};

/// How a cell of a typed column reads: what it shows, the key it sorts by, and whether it fits its type.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Seen {
    /// What is drawn in place of the words, for a type that draws its values its own way.
    pub shown: Option<String>,
    pub key: Option<String>,
    /// Written, but not as its type is written: drawn as it is, with a mark, never refused.
    pub amiss: bool,
}

/// The words of a cell as written, from its first mark to its last.
pub fn said<'a>(cell: &Block, text: &'a str) -> &'a str {
    let start = cell.spans.iter().map(|one| one.at.start).min();
    let end = cell.spans.iter().map(|one| one.at.end).max();
    match (start, end) {
        (Some(start), Some(end)) => text.get(start..end).unwrap_or_default().trim(),
        _ => "",
    }
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
    }
}

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
            (amount.scale <= convention(currency.as_deref()).decimals).then_some(())?;
            Value::Amount(amount)
        }
        Holds::Rating => {
            let amount = Amount::read(said)?;
            (amount.scale == 0 && (0..=STARS).contains(&amount.units)).then_some(())?;
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

/// The foot of a table some column adds up: one cell per column, «Total» in the first when it is free.
pub fn footed(columns: &[Column], block: &Block, text: &str) -> Option<Vec<String>> {
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
            let cells: Vec<&str> = block
                .inner
                .iter()
                .skip(1)
                .map(|row| row.inner.get(at).map_or("", |cell| said(cell, text)))
                .collect();
            totalled(&column.holds, total, &cells)
        })
        .collect();
    if let Some(first) = foot.first_mut().filter(|one| one.is_empty()) {
        "Total".clone_into(first);
    }
    Some(foot)
}

fn totalled(holds: &Holds, total: Total, cells: &[&str]) -> String {
    let filled = cells.iter().filter(|one| !one.is_empty());
    let values: Vec<Value> = filled
        .clone()
        .filter_map(|one| valued(holds, one))
        .collect();
    if total == Total::Count {
        let counted = if *holds == Holds::Check {
            values
                .iter()
                .filter(|one| matches!(one, Value::Yes(true)))
                .count()
        } else {
            filled.count()
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
            let all: Vec<Amount> = values
                .into_iter()
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

fn amounted(holds: &Holds, total: Total, all: &[Amount]) -> String {
    let sum = all.iter().fold(Amount::default(), |one, two| one.plus(two));
    let picked = match total {
        Total::Min => all.iter().min_by(|one, two| one.against(two)).cloned(),
        Total::Max => all.iter().max_by(|one, two| one.against(two)).cloned(),
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

/// A decimal kept exact: `units` counted in tenths to the power of `scale`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Amount {
    units: i128,
    scale: u32,
}

impl Amount {
    // The one way a number is written in a file: digits, a point for decimals, no thousands mark.
    fn read(said: &str) -> Option<Self> {
        let (sign, digits) = match said.strip_prefix('-') {
            Some(rest) => (-1, rest),
            None => (1, said),
        };
        let (whole, part) = digits.split_once('.').unwrap_or((digits, ""));
        let fits = |one: &str| one.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty()
            || !fits(whole)
            || !fits(part)
            || (digits.contains('.') && part.is_empty())
        {
            return None;
        }
        if whole.len() + part.len() > 30 {
            return None;
        }
        let units: i128 = format!("{whole}{part}").parse().ok()?;
        Some(Self {
            units: sign * units,
            scale: u32::try_from(part.len()).ok()?,
        })
    }

    fn at(&self, scale: u32) -> i128 {
        self.units * 10_i128.pow(scale.saturating_sub(self.scale))
    }

    fn plus(&self, other: &Self) -> Self {
        let scale = self.scale.max(other.scale);
        Self {
            units: self.at(scale) + other.at(scale),
            scale,
        }
    }

    fn against(&self, other: &Self) -> Ordering {
        let scale = self.scale.max(other.scale);
        self.at(scale).cmp(&other.at(scale))
    }

    // Shared out among `many`, kept to `scale` decimals.
    fn shared(&self, many: i128, scale: u32) -> Self {
        let units = if scale >= self.scale {
            rounded(self.at(scale), many)
        } else {
            rounded(self.units, many * 10_i128.pow(self.scale - scale))
        };
        Self { units, scale }
    }

    // A plain number drops the zeros an average leaves at its end; money keeps its cents.
    fn trimmed(mut self, holds: &Holds) -> Self {
        if matches!(holds, Holds::Money { .. }) {
            return self;
        }
        while self.scale > 0 && self.units % 10 == 0 {
            self.units /= 10;
            self.scale -= 1;
        }
        self
    }

    fn split(&self) -> (bool, String, String) {
        let digits = self.units.unsigned_abs().to_string();
        let scale = self.scale as usize;
        let digits = format!("{digits:0>width$}", width = scale + 1);
        let (whole, part) = digits.split_at(digits.len() - scale);
        (self.units < 0, whole.to_string(), part.to_string())
    }

    fn plain(&self) -> String {
        let (negative, whole, part) = self.split();
        let sign = if negative { "-" } else { "" };
        if part.is_empty() {
            format!("{sign}{whole}")
        } else {
            format!("{sign}{whole}.{part}")
        }
    }
}

// Halves go away from zero, as on a till.
fn rounded(units: i128, many: i128) -> i128 {
    if many == 0 {
        return 0;
    }
    let half = many / 2;
    if units >= 0 {
        (units + half) / many
    } else {
        (units - half) / many
    }
}

struct Convention {
    symbol: String,
    decimals: u32,
    group: &'static str,
    point: &'static str,
    after: bool,
    spaced: bool,
}

// Each currency as it is written where it is spent, so an export reads the same in any language.
#[rustfmt::skip]
const CURRENCIES: &[(&str, &str, u32, &str, &str, bool, bool)] = &[
    ("CLP", "$", 0, ".", ",", false, false),
    ("ARS", "$", 2, ".", ",", false, false),
    ("COP", "$", 2, ".", ",", false, false),
    ("MXN", "MX$", 2, ",", ".", false, false),
    ("PEN", "S/", 2, ",", ".", false, true),
    ("BRL", "R$", 2, ".", ",", false, true),
    ("UYU", "$U", 2, ".", ",", false, true),
    ("USD", "US$", 2, ",", ".", false, false),
    ("CAD", "CA$", 2, ",", ".", false, false),
    ("AUD", "A$", 2, ",", ".", false, false),
    ("EUR", "€", 2, ".", ",", true, true),
    ("GBP", "£", 2, ",", ".", false, false),
    ("CHF", "CHF", 2, "'", ".", false, true),
    ("JPY", "¥", 0, ",", ".", false, false),
    ("CNY", "CN¥", 2, ",", ".", false, false),
];

fn convention(currency: Option<&str>) -> Convention {
    let Some(code) = currency else {
        return Convention {
            symbol: String::new(),
            decimals: 2,
            group: "",
            point: ".",
            after: false,
            spaced: false,
        };
    };
    let known = CURRENCIES.iter().find(|one| one.0 == code);
    let (symbol, decimals, group, point, after, spaced) = known.map_or(
        (code, 2, ",", ".", false, true),
        |&(_, symbol, decimals, group, point, after, spaced)| {
            (symbol, decimals, group, point, after, spaced)
        },
    );
    Convention {
        symbol: symbol.to_string(),
        decimals,
        group,
        point,
        after,
        spaced,
    }
}

fn money(amount: &Amount, currency: Option<&str>) -> String {
    let way = convention(currency);
    let fixed = Amount {
        units: amount.at(way.decimals),
        scale: way.decimals,
    };
    let (negative, whole, part) = fixed.split();
    let mut number = grouped(&whole, way.group);
    if !part.is_empty() {
        number = format!("{number}{}{part}", way.point);
    }
    let sign = if negative { "-" } else { "" };
    let gap = if way.spaced { " " } else { "" };
    if way.symbol.is_empty() {
        format!("{sign}{number}")
    } else if way.after {
        format!("{sign}{number}{gap}{}", way.symbol)
    } else {
        format!("{sign}{}{gap}{number}", way.symbol)
    }
}

fn grouped(whole: &str, group: &str) -> String {
    let mut out = String::new();
    for (at, digit) in whole.chars().enumerate() {
        if at > 0 && (whole.len() - at).is_multiple_of(3) {
            out.push_str(group);
        }
        out.push(digit);
    }
    out
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

// Another document, named the way a reference is: `[[Lease contract]]`.
fn linked(said: &str) -> Option<String> {
    let name = said.strip_prefix("[[")?.strip_suffix("]]")?;
    (!name.trim().is_empty() && !name.contains("]]")).then(|| name.trim().to_lowercase())
}

#[cfg(all(test, feature = "tables"))]
#[path = "values_test.rs"]
mod tests;
