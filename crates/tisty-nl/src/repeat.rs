use jiff::Zoned;
use tisty_core::DateSpec;
use tisty_core::model::{Cadence, Repeat, Unit};

use crate::vocab::Vocabulary;

pub struct Took {
    pub text: String,
    pub repeat: Option<Repeat>,
    pub from: usize,
    pub to: usize,
    /// Where the right-hand side starts in the cut text, and how far it moved:
    /// a reading at or past `head` sat at `head + shift` in what was typed.
    pub head: usize,
    pub shift: usize,
    /// The first occurrence, when the phrase named a weekday. A fixed repeat
    /// with no date never fires, and a weekday at the start of a sentence is
    /// not read as one — so it is settled here rather than left to luck.
    pub first: Option<DateSpec>,
}

/// Naming a day makes it fixed; naming only an interval makes it relative.
///
/// «cada martes» is the bin, and it goes out on Tuesday whether or not you took
/// it out last week. «cada 3 días» is a habit, and three days start counting
/// when you actually did it. The distinction G1 asked for falls out of how the
/// sentence is said, so neither form needs a syntax of its own.
pub fn take(text: &str, now: &Zoned, v: &Vocabulary) -> Took {
    let words: Vec<(usize, &str)> = spots(text);

    for (i, (at, word)) in words.iter().enumerate() {
        // «diariamente», «weekly»: a whole cadence in one word, opening nothing.
        if let Some(unit) = v
            .cadences
            .iter()
            .find(|(said, _)| said.iter().any(|one| one.eq_ignore_ascii_case(word)))
            .map(|(_, unit)| *unit)
        {
            let to = at + word.len();
            let repeat = Repeat::Done(Cadence { every: 1, unit });
            return cut(text, *at, to, Some(repeat), *at, to);
        }

        let Some(opened) = opens(&words[i..], v) else {
            continue;
        };
        let rest = &words[i + opened..];

        if let Some((repeat, day)) = weekly(rest, v) {
            let (day_at, day_word) = rest[0];
            let to = day_at + day_word.len();
            let mut took = cut(text, *at, to, Some(repeat), *at, to);
            took.first = next_weekday(now, day);
            return took;
        }
        if let Some((repeat, taken)) = interval(rest, v) {
            let last = rest[taken - 1];
            let to = last.0 + last.1.len();
            return cut(text, *at, to, Some(repeat), *at, to);
        }
    }

    Took {
        text: text.to_string(),
        repeat: None,
        from: 0,
        to: 0,
        head: 0,
        shift: 0,
        first: None,
    }
}

/// How many words the opening took: «cada» is one, «todos los» is two.
fn opens(from: &[(usize, &str)], v: &Vocabulary) -> Option<usize> {
    v.every
        .iter()
        .find(|said| {
            said.iter().enumerate().all(|(k, one)| {
                from.get(k)
                    .is_some_and(|(_, word)| one.eq_ignore_ascii_case(word))
            })
        })
        .map(|said| said.len())
}

fn weekly(rest: &[(usize, &str)], v: &Vocabulary) -> Option<(Repeat, usize)> {
    let (_, word) = rest.first()?;
    let which = v
        .weekdays
        .iter()
        .position(|day| day.iter().any(|one| one.eq_ignore_ascii_case(word)))?;

    // «cada lunes y jueves» is two days a week. Reading one of them and
    // dropping the other silently is worse than reading nothing.
    if let Some((_, next)) = rest.get(1)
        && v.linker.iter().any(|one| one.eq_ignore_ascii_case(next))
        && rest.get(2).is_some_and(|(_, after)| {
            v.weekdays
                .iter()
                .any(|day| day.iter().any(|one| one.eq_ignore_ascii_case(after)))
        })
    {
        return None;
    }

    Some((
        Repeat::Due(Cadence {
            every: 1,
            unit: Unit::Week,
        }),
        which,
    ))
}

fn next_weekday(now: &Zoned, which: usize) -> Option<DateSpec> {
    let today = now.date();
    let from = today.weekday().to_monday_zero_offset() as usize;
    let ahead = (which + 7 - from) % 7;
    let at = today
        .checked_add(jiff::ToSpan::days(if ahead == 0 { 7 } else { ahead } as i64))
        .ok()?;
    Some(DateSpec::all_day(
        at,
        now.time_zone().iana_name().unwrap_or("UTC"),
    ))
}

fn interval(rest: &[(usize, &str)], v: &Vocabulary) -> Option<(Repeat, usize)> {
    let mut taken = 0;
    let mut every: u16 = 1;

    if let Some((_, word)) = rest.first()
        && let Ok(n) = word.parse::<u16>()
    {
        every = n;
        taken += 1;
    } else if let Some((_, word)) = rest.first()
        && v.one.iter().any(|one| one.eq_ignore_ascii_case(word))
    {
        taken += 1;
    }

    let (_, word) = rest.get(taken)?;
    let unit = unit_of(word, v)?;
    taken += 1;

    // «cada 0 días» would store a repeat that never fires, and nobody writes a
    // cadence in the thousands: both are likelier to be a typo than a wish.
    if every == 0 || every > 999 {
        return None;
    }
    Some((Repeat::Done(Cadence { every, unit }), taken))
}

fn unit_of(word: &str, v: &Vocabulary) -> Option<Unit> {
    let has = |set: &[&str]| set.iter().any(|one| one.eq_ignore_ascii_case(word));
    if has(v.days_unit) {
        Some(Unit::Day)
    } else if has(v.weeks_unit) {
        Some(Unit::Week)
    } else if has(v.months_unit) {
        Some(Unit::Month)
    } else if has(v.years_unit) {
        Some(Unit::Year)
    } else {
        None
    }
}

fn cut(
    text: &str,
    from: usize,
    to: usize,
    repeat: Option<Repeat>,
    span_from: usize,
    span_to: usize,
) -> Took {
    let trimmed = text[..from].trim_end();
    let right_at = to + (text[to..].len() - text[to..].trim_start().len());
    let right = &text[right_at..];

    let left = if !trimmed.is_empty() && !right.is_empty() {
        format!("{trimmed} ")
    } else {
        trimmed.to_string()
    };
    // Measured, not guessed from the total: the whole string is trimmed at the
    // end too, and counting those bytes as part of the cut pushes later
    // readings into the middle of a character.
    let head = left.len();
    Took {
        head,
        shift: right_at.saturating_sub(head),
        text: format!("{left}{right}"),
        repeat,
        from: span_from,
        to: span_to,
        first: None,
    }
}

fn spots(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut at = 0;
    for part in text.split_whitespace() {
        let start = text[at..].find(part).map(|k| at + k).unwrap_or(at);
        out.push((start, part));
        at = start + part.len();
    }
    out
}
