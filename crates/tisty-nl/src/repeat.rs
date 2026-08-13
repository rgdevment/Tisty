use jiff::Zoned;
use tisty_core::DateSpec;
use tisty_core::model::{Cadence, Repeat, Unit};

use crate::vocab::Vocabulary;

pub struct Took {
    pub text: String,
    pub repeat: Option<Repeat>,
    pub from: usize,
    pub to: usize,
    pub head: usize,
    pub shift: usize,
    pub first: Option<DateSpec>,
}

pub fn take(text: &str, masked: &str, now: &Zoned, v: &Vocabulary) -> Took {
    let words = crate::words(masked);

    for (i, (at, word)) in words.iter().enumerate() {
        if words.get(i + 1).is_none()
            && let Some(unit) = v
                .cadences
                .iter()
                .find(|(said, _)| said.iter().any(|one| one.eq_ignore_ascii_case(word)))
                .map(|(_, unit)| *unit)
        {
            let to = at + word.len();
            let repeat = Repeat::done(Cadence { every: 1, unit });
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
    let bare = word.strip_suffix('s').unwrap_or(word);
    let which = v.weekdays.iter().position(|day| {
        day.iter()
            .any(|one| one.eq_ignore_ascii_case(word) || one.eq_ignore_ascii_case(bare))
    })?;

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
        Repeat::due(Cadence {
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

    if every == 0 || every > 999 {
        return None;
    }
    Some((Repeat::done(Cadence { every, unit }), taken))
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
