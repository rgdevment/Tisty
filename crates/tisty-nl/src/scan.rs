use jiff::civil::{Date, Time};

use crate::vocab::Vocabulary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Today,
    Tomorrow,
    DayAfterTomorrow,
    Weekday(usize),
    NextWeekday(usize),
    InDays(i64),
    InWeeks(i64),
    InMonths(i64),
    OnDate(u8, Option<u8>, Option<i16>),
    EndOfWeek,
    EndOfMonth,
    Weekend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Date,
    Deadline,
}

#[derive(Debug, Clone)]
pub struct Found {
    pub anchor: Option<Anchor>,
    pub time: Option<Time>,
    pub role: Role,
    /// One range per piece taken out: the day and the clock can sit apart.
    pub spans: Vec<(usize, usize)>,
}

pub struct Token {
    pub word: String,
    pub start: usize,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if let Some(s) = start.take() {
                tokens.push(make(input, s, i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        tokens.push(make(input, s, input.len()));
    }
    tokens
}

fn make(input: &str, start: usize, end: usize) -> Token {
    Token {
        word: input[start..end]
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase(),
        start,
    }
}

/// Right to left: a phrase that means the date sits at the end, not mid-sentence.
pub fn scan(tokens: &[Token], v: &Vocabulary) -> Option<Found> {
    (0..=tokens.len()).rev().find_map(|end| at(tokens, end, v))
}

fn at(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<Found> {
    if end == 0 {
        return None;
    }
    let trailing = end == tokens.len();

    let mut cursor = end;
    let mut time = None;

    if let Some((t, from)) = match_time(tokens, cursor, v) {
        time = Some(t);
        cursor = from;
    }

    let anchor = match_anchor(tokens, cursor, v).map(|(a, from)| {
        cursor = from;
        a
    });

    if anchor.is_none() && time.is_none() {
        return None;
    }

    let role = role_before(tokens, cursor, v);
    if anchor.is_some() && time.is_none() && is_descriptive(tokens, cursor, v, role) {
        return None;
    }
    // «por la mañana» is a part of the day; only the bare word means the day
    // after. The article is what tells them apart.
    if anchor.is_some() && names_part_of_day(tokens, cursor, end, v) {
        return None;
    }
    // Mid-sentence, «monday» is usually naming something — «the monday report»,
    // «lunes santo». Only a clock or a temporal preposition says it is a date.
    if !trailing && time.is_none() && !prefixed(tokens, cursor, v) {
        return None;
    }
    // And a noun right behind it means it was modifying that noun: «the monday
    // list» names a list, while «el viernes el informe» just carries on.
    if !trailing && qualifies_next(tokens, end, v) {
        return None;
    }

    let mut spans = vec![(skip_particles(tokens, cursor, v, role), end)];
    let mut anchor = anchor;

    // A clock is signal enough to go looking for the day it belongs to: «the
    // meeting on tuesday in room 3c at 16:00» means tuesday, not tomorrow.
    if anchor.is_none()
        && time.is_some()
        && let Some((found, from, to)) = anchor_before(tokens, cursor, v)
    {
        anchor = Some(found);
        spans.insert(0, (from, to));
    }

    Some(Found {
        anchor,
        time,
        role,
        spans,
    })
}

fn anchor_before(tokens: &[Token], limit: usize, v: &Vocabulary) -> Option<(Anchor, usize, usize)> {
    (0..limit).rev().find_map(|end| {
        let (anchor, from) = match_anchor(tokens, end, v)?;
        let role = role_before(tokens, from, v);
        if is_descriptive(tokens, from, v, role) || qualifies_next(tokens, end, v) {
            return None;
        }
        Some((anchor, skip_particles(tokens, from, v, role), end))
    })
}

/// A bare noun straight after the phrase means the phrase was describing it.
fn qualifies_next(tokens: &[Token], end: usize, v: &Vocabulary) -> bool {
    let Some(next) = tokens.get(end) else {
        return false;
    };
    let word = next.word.as_str();
    !(v.article.contains(&word)
        || v.linker.contains(&word)
        || v.date_prep.contains(&word)
        || v.deadline_prep.contains(&word)
        || v.time_prep.contains(&word)
        || v.in_prep.contains(&word)
        || v.spans_prep.contains(&word))
}

/// "el informe del lunes" may be its name; only an action preposition dates it.
fn is_descriptive(tokens: &[Token], cursor: usize, v: &Vocabulary, role: Role) -> bool {
    if role == Role::Deadline || cursor == 0 {
        return false;
    }
    if !matches!(tokens[cursor - 1].word.as_str(), "de" | "del") {
        return false;
    }
    !(cursor >= 2 && v.date_prep.contains(&tokens[cursor - 2].word.as_str()))
}

fn names_part_of_day(tokens: &[Token], cursor: usize, end: usize, v: &Vocabulary) -> bool {
    end == cursor + 1
        && v.day_part.contains(&tokens[cursor].word.as_str())
        && cursor > 0
        && v.article.contains(&tokens[cursor - 1].word.as_str())
}

fn prefixed(tokens: &[Token], cursor: usize, v: &Vocabulary) -> bool {
    (0..cursor).rev().take(3).any(|i| {
        let word = tokens[i].word.as_str();
        v.deadline_prep.contains(&word) || v.date_prep.contains(&word)
    })
}

fn role_before(tokens: &[Token], cursor: usize, v: &Vocabulary) -> Role {
    for i in (0..cursor).rev().take(3) {
        if v.deadline_prep.contains(&tokens[i].word.as_str()) {
            return Role::Deadline;
        }
        if v.date_prep.contains(&tokens[i].word.as_str()) {
            return Role::Date;
        }
        if !v.article.contains(&tokens[i].word.as_str()) {
            break;
        }
    }
    Role::Date
}

fn skip_particles(tokens: &[Token], cursor: usize, v: &Vocabulary, role: Role) -> usize {
    let mut i = cursor;
    while i > 0 {
        let w = tokens[i - 1].word.as_str();
        let droppable = v.article.contains(&w)
            || v.time_prep.contains(&w)
            || v.date_prep.contains(&w)
            || (role == Role::Deadline && v.deadline_prep.contains(&w));
        if !droppable {
            break;
        }
        i -= 1;
    }
    i
}

fn match_time(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<(Time, usize)> {
    if end == 0 {
        return None;
    }
    let last = &tokens[end - 1].word;

    if v.noon.contains(&last.as_str()) {
        return Some((
            Time::constant(12, 0, 0, 0),
            skip_time_preps(tokens, end - 1, v),
        ));
    }

    // «10 am» arrives as two words; alone, «am» is not a clock.
    if matches!(last.as_str(), "am" | "pm") && end >= 2 {
        let joined = format!("{}{last}", tokens[end - 2].word);
        let preceded = end >= 3 && v.clock_prep.contains(&tokens[end - 3].word.as_str());
        if let Some(t) = parse_clock(&joined, preceded) {
            return Some((t, skip_time_preps(tokens, end - 2, v)));
        }
    }

    let preceded = end >= 2 && v.clock_prep.contains(&tokens[end - 2].word.as_str());
    let t = parse_clock(last, preceded)?;
    Some((t, skip_time_preps(tokens, end - 1, v)))
}

fn skip_time_preps(tokens: &[Token], mut from: usize, v: &Vocabulary) -> usize {
    while from > 0 && v.time_prep.contains(&tokens[from - 1].word.as_str()) {
        from -= 1;
    }
    from
}

/// A bare integer is a clock only behind its preposition, or versions get eaten.
fn parse_clock(word: &str, preceded: bool) -> Option<Time> {
    let (digits, suffix) = split_suffix(word);

    if let Some((h, m)) = digits.split_once(':') {
        let h: i8 = h.parse().ok()?;
        let m: i8 = m.parse().ok()?;
        return Time::new(apply_suffix(h, suffix)?, m, 0, 0).ok();
    }

    if suffix.is_some() || preceded {
        let h: i8 = digits.parse().ok()?;
        return Time::new(apply_suffix(h, suffix)?, 0, 0, 0).ok();
    }
    None
}

fn split_suffix(word: &str) -> (&str, Option<&str>) {
    for s in ["am", "pm"] {
        if let Some(rest) = word.strip_suffix(s) {
            return (rest, Some(s));
        }
    }
    (word, None)
}

fn apply_suffix(hour: i8, suffix: Option<&str>) -> Option<i8> {
    match suffix {
        Some("pm") if hour < 12 => Some(hour + 12),
        Some("am") if hour == 12 => Some(0),
        _ if (0..=23).contains(&hour) => Some(hour),
        _ => None,
    }
}

fn match_anchor(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<(Anchor, usize)> {
    if end == 0 {
        return None;
    }

    for phrases in [v.weekend, v.end_of_month, v.this_week] {
        if let Some(from) = match_phrase(tokens, end, phrases) {
            let anchor = if std::ptr::eq(phrases, v.weekend) {
                Anchor::Weekend
            } else if std::ptr::eq(phrases, v.end_of_month) {
                Anchor::EndOfMonth
            } else {
                Anchor::EndOfWeek
            };
            return Some((anchor, from));
        }
    }

    let last = tokens[end - 1].word.as_str();

    if v.today.contains(&last) {
        return Some((Anchor::Today, end - 1));
    }
    if v.tomorrow.contains(&last) {
        let after = end >= 2 && v.day_after.contains(&tokens[end - 2].word.as_str());
        return Some(if after {
            (Anchor::DayAfterTomorrow, end - 2)
        } else {
            (Anchor::Tomorrow, end - 1)
        });
    }
    if let Some(day) = v.weekday_index(last) {
        let is_next = (1..=2)
            .any(|back| end > back && v.next.contains(&tokens[end - 1 - back].word.as_str()));
        let from = if is_next { end - 2 } else { end - 1 };
        return Some((
            if is_next {
                Anchor::NextWeekday(day)
            } else {
                Anchor::Weekday(day)
            },
            from,
        ));
    }

    if let Some(found) = match_offset(tokens, end, v) {
        return Some(found);
    }
    match_explicit_date(tokens, end, v)
}

fn match_phrase(tokens: &[Token], end: usize, phrases: &[&[&str]]) -> Option<usize> {
    for phrase in phrases {
        if end < phrase.len() {
            continue;
        }
        let from = end - phrase.len();
        if tokens[from..end]
            .iter()
            .zip(phrase.iter())
            .all(|(t, w)| t.word == *w)
        {
            return Some(from);
        }
    }
    None
}

fn match_offset(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<(Anchor, usize)> {
    if end < 2 {
        return None;
    }
    let unit = tokens[end - 1].word.as_str();
    let amount_word = tokens[end - 2].word.as_str();

    let amount: i64 = if v.one.contains(&amount_word) {
        1
    } else {
        amount_word.parse().ok()?
    };

    let anchor = if v.days_unit.contains(&unit) {
        Anchor::InDays(amount)
    } else if v.weeks_unit.contains(&unit) {
        Anchor::InWeeks(amount)
    } else if v.months_unit.contains(&unit) {
        Anchor::InMonths(amount)
    } else {
        return None;
    };

    let mut from = end - 2;
    let mut asked = false;
    while from > 0 {
        let w = tokens[from - 1].word.as_str();
        if v.in_prep.contains(&w) {
            asked = true;
            from -= 1;
        } else if v.article.contains(&w) {
            from -= 1;
        } else {
            break;
        }
    }

    // «en 30 días» is a date; «por 30 días» is how long it lasts and «hace 3
    // días» points backwards. Neither has anywhere to go: guessing is worse.
    if from > 0 {
        let before = tokens[from - 1].word.as_str();
        if v.past_prep.contains(&before) || (!asked && v.spans_prep.contains(&before)) {
            return None;
        }
    }
    Some((anchor, from))
}

fn match_explicit_date(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<(Anchor, usize)> {
    let last = tokens[end - 1].word.as_str();

    if let Ok(date) = last.parse::<Date>() {
        return Some((
            Anchor::OnDate(
                date.day() as u8,
                Some(date.month() as u8),
                Some(date.year()),
            ),
            end - 1,
        ));
    }

    if let Some((a, b)) = last.split_once('/')
        && let (Ok(d), Ok(m)) = (a.parse::<u8>(), b.parse::<u8>())
        && (1..=31).contains(&d)
        && (1..=12).contains(&m)
    {
        return Some((Anchor::OnDate(d, Some(m), None), end - 1));
    }

    // "15 de agosto" and "august 15" are the same date in either order.
    if let Some(month) = v.month_index(last) {
        for back in 2..=3 {
            if end >= back
                && let Ok(day) = tokens[end - back].word.parse::<u8>()
                && (1..=31).contains(&day)
            {
                return Some((Anchor::OnDate(day, Some(month), None), end - back));
            }
        }
        return None;
    }

    if let Ok(day) = last.parse::<u8>()
        && (1..=31).contains(&day)
    {
        for back in 2..=3 {
            if end >= back
                && let Some(month) = v.month_index(tokens[end - back].word.as_str())
            {
                return Some((Anchor::OnDate(day, Some(month), None), end - back));
            }
        }
    }
    None
}
