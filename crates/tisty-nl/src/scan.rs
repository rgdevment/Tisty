use jiff::civil::{Date, Time};

use crate::{Certainty, vocab::Vocabulary};

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
    EndOfNextWeek,
    EndOfNextMonth,
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
    pub certainty: Certainty,
}

pub struct Token {
    pub word: String,
    pub start: usize,
    pub end: usize,
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
        word: composed(
            input[start..end]
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase(),
        ),
        start,
        end,
    }
}

/// Composes decomposed accents (e.g. n + U+0303) so they match the vocabulary; the title keeps what was typed.
fn composed(word: String) -> String {
    if !word.chars().any(is_mark) {
        return word;
    }

    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        match out.pop() {
            Some(base) if is_mark(c) => match join(base, c) {
                Some(single) => out.push(single),
                None => {
                    out.push(base);
                    out.push(c);
                }
            },
            Some(base) => {
                out.push(base);
                out.push(c);
            }
            None => out.push(c),
        }
    }
    out
}

fn is_mark(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

fn join(base: char, mark: char) -> Option<char> {
    Some(match (base, mark) {
        ('a', '\u{0301}') => 'á',
        ('e', '\u{0301}') => 'é',
        ('i', '\u{0301}') => 'í',
        ('o', '\u{0301}') => 'ó',
        ('u', '\u{0301}') => 'ú',
        ('n', '\u{0303}') => 'ñ',
        ('u', '\u{0308}') => 'ü',
        ('c', '\u{0327}') => 'ç',
        _ => return None,
    })
}

#[derive(Default)]
pub struct Scanned {
    pub found: Option<Found>,
    /// «entregar mañana antes del viernes» carries a date and a deadline.
    pub also: Option<Found>,
    /// Kept only when nothing was taken: a second date is noise, not a reading.
    pub offer: Option<Found>,
}

enum Reading {
    Taken(Found),
    Offered(Found),
}

/// Right to left: a phrase that means the date sits at the end, not mid-sentence.
pub fn scan(tokens: &[Token], v: &Vocabulary) -> Scanned {
    let mut offer = None;

    for end in (0..=tokens.len()).rev() {
        match at(tokens, end, v) {
            Some(Reading::Taken(found)) => {
                let also = other_role(tokens, &found, v);
                return Scanned {
                    found: Some(found),
                    also,
                    offer: None,
                };
            }
            Some(Reading::Offered(found)) if offer.is_none() => offer = Some(found),
            _ => {}
        }
    }
    Scanned {
        found: None,
        also: None,
        offer,
    }
}

fn other_role(tokens: &[Token], taken: &Found, v: &Vocabulary) -> Option<Found> {
    let limit = taken.spans.iter().map(|(from, _)| *from).min()?;
    (0..=limit).rev().find_map(|end| match at(tokens, end, v) {
        Some(Reading::Taken(found)) if found.role != taken.role => Some(found),
        _ => None,
    })
}

fn at(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<Reading> {
    if end == 0 {
        return None;
    }
    let trailing = end == tokens.len();

    let mut cursor = end;
    let mut time = None;
    let mut assumed_hour = false;

    if let Some((t, from, guessed)) = match_time(tokens, cursor, v) {
        time = Some(t);
        cursor = from;
        assumed_hour = guessed;
    }

    let anchor = match_anchor(tokens, cursor, v).map(|(a, from)| {
        cursor = from;
        a
    });

    if anchor.is_none() && time.is_none() {
        return None;
    }

    // «por la mañana» names a part of the day, not the day after; the article is what tells them apart.
    if anchor.is_some() && names_part_of_day(tokens, cursor, end, v) {
        return None;
    }

    let role = role_before(tokens, cursor, v);
    let strong = trailing || time.is_some() || prefixed(tokens, cursor, v);

    // «mañana de verano» carries its own complement, so the word is a noun with nothing to offer.
    if !strong && takes_complement(tokens, end, v) {
        return None;
    }

    // The phrase may be naming the noun beside it — «el informe del lunes» — so it is offered, not taken.
    let describes = qualifies_next(tokens, end, v);
    let ambiguous = (anchor.is_some() && time.is_none() && is_descriptive(tokens, cursor, v, role))
        || (!trailing && describes);

    // Unless a number follows: «el lunes 15» is the fifteenth, which this parser cannot read yet.
    if ambiguous
        && tokens
            .get(end)
            .is_some_and(|next| next.word.parse::<u16>().is_ok())
    {
        return None;
    }

    // Mid-sentence without a clock or temporal preposition, the reading is only assumed, not sure.
    let certainty = if strong && !assumed_hour {
        Certainty::Sure
    } else {
        Certainty::Assumed
    };

    // «the monday report»: the article belongs to the noun, so it must not be swallowed.
    let from = if describes {
        cursor
    } else {
        skip_particles(tokens, cursor, v, role)
    };
    let mut spans = vec![(from, end)];
    let mut anchor = anchor;

    // A clock is signal enough to go looking for the day it belongs to.
    if anchor.is_none()
        && time.is_some()
        && let Some((found, from, to)) = anchor_before(tokens, cursor, v)
    {
        anchor = Some(found);
        spans.insert(0, (from, to));
    }

    let found = Found {
        anchor,
        time,
        role,
        spans,
        certainty,
    };
    Some(if ambiguous {
        Reading::Offered(found)
    } else {
        Reading::Taken(found)
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

fn takes_complement(tokens: &[Token], end: usize, v: &Vocabulary) -> bool {
    tokens
        .get(end)
        .is_some_and(|next| v.genitive.contains(&next.word.as_str()))
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

/// The third field says the afternoon was assumed, not written.
fn match_time(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<(Time, usize, bool)> {
    if end == 0 {
        return None;
    }
    let last = &tokens[end - 1].word;

    if v.noon.contains(&last.as_str()) {
        return Some((
            Time::constant(12, 0, 0, 0),
            skip_time_preps(tokens, end - 1, v),
            false,
        ));
    }

    // «10 am» arrives as two words; alone, «am» is not a clock.
    if matches!(last.as_str(), "am" | "pm") && end >= 2 {
        let joined = format!("{}{last}", tokens[end - 2].word);
        let preceded = end >= 3 && v.clock_prep.contains(&tokens[end - 3].word.as_str());
        if let Some((t, guessed)) = parse_clock(&joined, preceded) {
            return Some((t, skip_time_preps(tokens, end - 2, v), guessed));
        }
    }

    let preceded = end >= 2 && v.clock_prep.contains(&tokens[end - 2].word.as_str());
    let (t, guessed) = parse_clock(last, preceded)?;
    Some((t, skip_time_preps(tokens, end - 1, v), guessed))
}

fn skip_time_preps(tokens: &[Token], mut from: usize, v: &Vocabulary) -> usize {
    while from > 0 && v.time_prep.contains(&tokens[from - 1].word.as_str()) {
        from -= 1;
    }
    from
}

/// A bare integer is a clock only behind its preposition, or version numbers get eaten.
fn parse_clock(word: &str, preceded: bool) -> Option<(Time, bool)> {
    let (digits, suffix) = split_suffix(word);

    if let Some((h, m)) = digits.split_once(':') {
        let hour: i8 = h.parse().ok()?;
        let m: i8 = m.parse().ok()?;
        let (hour, guessed) = afternoon(hour, suffix, h.starts_with('0'));
        return Time::new(hour, m, 0, 0).ok().map(|t| (t, guessed));
    }

    if suffix.is_some() || preceded {
        let hour: i8 = digits.parse().ok()?;
        let (hour, guessed) = afternoon(hour, suffix, digits.starts_with('0'));
        return Time::new(hour, 0, 0, 0).ok().map(|t| (t, guessed));
    }
    None
}

/// Only 1–6 read as PM (the night reading is absurd there); 7+ is ambiguous and taken as written. A leading zero is explicit 24-hour.
fn afternoon(hour: i8, suffix: Option<&str>, padded: bool) -> (i8, bool) {
    match apply_suffix(hour, suffix) {
        Some(h) if suffix.is_none() && !padded && (1..=6).contains(&h) => (h + 12, true),
        Some(h) => (h, false),
        None => (hour, false),
    }
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

    let phrased = [
        (v.weekend, Anchor::Weekend),
        (v.end_of_month, Anchor::EndOfMonth),
        (v.next_week, Anchor::EndOfNextWeek),
        (v.next_month, Anchor::EndOfNextMonth),
        (v.this_week, Anchor::EndOfWeek),
    ];
    for (phrases, anchor) in phrased {
        if let Some(from) = match_phrase(tokens, end, phrases) {
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

    // «por 30 días» is a duration and «hace 3 días» points backwards — neither is a date to guess at.
    if from > 0 {
        let before = tokens[from - 1].word.as_str();
        if v.past_prep.contains(&before) || (!asked && v.spans_prep.contains(&before)) {
            return None;
        }
    }
    // English puts the marker behind the unit: «3 days ago».
    if tokens
        .get(end)
        .is_some_and(|next| v.past_prep.contains(&next.word.as_str()))
    {
        return None;
    }
    Some((anchor, from))
}

fn as_day(word: &str, v: &Vocabulary) -> Option<u8> {
    if v.first.contains(&word) {
        return Some(1);
    }
    let bare = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|suffix| word.strip_suffix(suffix))
        .unwrap_or(word);
    bare.parse().ok().filter(|day| (1..=31).contains(day))
}

fn match_explicit_date(tokens: &[Token], end: usize, v: &Vocabulary) -> Option<(Anchor, usize)> {
    let last = tokens[end - 1].word.as_str();
    if v.idioms.contains(&last) {
        return None;
    }

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

    let slashed: Vec<&str> = last.split('/').collect();
    if let [d, m] | [d, m, _] = slashed.as_slice()
        && let (Ok(d), Ok(m)) = (d.parse::<u8>(), m.parse::<u8>())
        && (1..=31).contains(&d)
        && (1..=12).contains(&m)
    {
        let year = match slashed.as_slice() {
            [_, _, y] => Some(y.parse::<i16>().ok()?),
            _ => None,
        };
        return Some((Anchor::OnDate(d, Some(m), year), end - 1));
    }

    // "15 de agosto" and "august 15" are the same date in either order.
    if let Some(month) = v.month_index(last) {
        for back in 2..=3 {
            if end >= back
                && let Some(day) = as_day(&tokens[end - back].word, v)
            {
                return Some((Anchor::OnDate(day, Some(month), None), end - back));
            }
        }
        return None;
    }

    if let Some(day) = as_day(last, v) {
        for back in 2..=3 {
            if end >= back
                && let Some(month) = v.month_index(tokens[end - back].word.as_str())
            {
                return Some((Anchor::OnDate(day, Some(month), None), end - back));
            }
        }
        // «el lunes 15»: the weekday names it, the number dates it.
        if end >= 2 && v.weekday_index(tokens[end - 2].word.as_str()).is_some() {
            return Some((Anchor::OnDate(day, None, None), end - 2));
        }
    }
    None
}
