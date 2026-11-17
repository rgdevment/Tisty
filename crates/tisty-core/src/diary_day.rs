use std::collections::{HashMap, HashSet};
use std::ops::Range;

use tisty_markdown::{Callout, Kind};

use crate::diary::{blank_free, day_named, trailed};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayAt {
    pub day: i8,
    pub nth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cut {
    pub day: i8,
    pub heading: usize,
    pub end: usize,
}

// A heading that names no day stays inside the day above it, so nothing the person wrote is lost.
pub fn cuts(body: &str) -> Vec<Cut> {
    let starts: Vec<(usize, i8)> = crate::docs::headings(body)
        .into_iter()
        .filter(|(_, level, _)| *level == 2)
        .filter_map(|(line, _, title)| Some((line - 1, day_named(&title)?)))
        .collect();
    let total = body.lines().count();
    starts
        .iter()
        .enumerate()
        .map(|(at, (heading, day))| Cut {
            day: *day,
            heading: *heading,
            end: starts.get(at + 1).map_or(total, |(next, _)| *next),
        })
        .collect()
}

pub fn places(days: impl IntoIterator<Item = i8>) -> Vec<DayAt> {
    let mut seen: HashMap<i8, usize> = HashMap::new();
    days.into_iter()
        .map(|day| {
            let nth = seen.entry(day).or_default();
            let at = DayAt { day, nth: *nth };
            *nth += 1;
            at
        })
        .collect()
}

fn found(body: &str, at: DayAt) -> Option<Cut> {
    cuts(body)
        .into_iter()
        .filter(|one| one.day == at.day)
        .nth(at.nth)
}

// Only the notes the translator draws at the day's own level count, so the window numbers them alike.
fn noted(lines: &[&str]) -> Vec<Range<usize>> {
    let mut starts = Vec::with_capacity(lines.len());
    let mut at = 0;
    for line in lines {
        starts.push(at);
        at += line.len() + 1;
    }
    let line_of = |byte: usize| starts.partition_point(|one| *one <= byte).saturating_sub(1);
    tisty_markdown::read(&lines.join("\n"))
        .blocks()
        .iter()
        .filter(|block| matches!(block.kind, Kind::Callout(Callout::Idea)))
        .map(|block| {
            let last = block.at.end.saturating_sub(1).max(block.at.start);
            line_of(block.at.start)..line_of(last) + 1
        })
        .collect()
}

fn unblank<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    blank_free(trailed(lines))
}

fn words_between(lines: &[&str], notes: &[Range<usize>]) -> String {
    let ends = notes
        .iter()
        .map(|one| (one.start, one.end))
        .chain(std::iter::once((lines.len(), lines.len())));
    let mut from = 0;
    let mut pieces = Vec::new();
    for (start, end) in ends {
        let piece = unblank(&lines[from..start]);
        if !piece.is_empty() {
            pieces.push(piece.join("\n"));
        }
        from = end;
    }
    pieces.join("\n\n")
}

pub fn parted(day: &str) -> (String, Vec<String>) {
    let lines: Vec<&str> = day.lines().collect();
    let notes = noted(&lines);
    let said = notes.iter().map(|one| lines[one.clone()].join("\n"));
    (words_between(&lines, &notes), said.collect())
}

pub fn words(body: &str, at: DayAt) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let cut = found(body, at)?;
    let day = &lines[cut.heading + 1..cut.end];
    Some(words_between(day, &noted(day)))
}

pub fn rewritten(body: &str, at: DayAt, was: &str, said: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let cut = found(body, at)?;
    let day = &lines[cut.heading + 1..cut.end];
    let notes = noted(day);
    if words_between(day, &notes) != was {
        return None;
    }
    let said = kept_in_the_day(said.trim());
    let mut kept: Vec<&str> = said.lines().collect();
    for note in notes {
        if !kept.is_empty() {
            kept.push("");
        }
        kept.extend(&day[note]);
    }
    Some(spliced(body, &lines, cut, &kept))
}

// Only a title or a heading that would start a day of its own goes under the day; the rest stay as written.
fn kept_in_the_day(said: &str) -> String {
    let cutting: HashSet<usize> = crate::docs::headings(said)
        .into_iter()
        .filter(|(_, level, title)| *level == 1 || (*level == 2 && day_named(title).is_some()))
        .map(|(line, _, _)| line - 1)
        .collect();
    said.lines()
        .enumerate()
        .map(|(at, line)| {
            let lead = line.len() - line.trim_start().len();
            let marks = line[lead..].chars().take_while(|one| *one == '#').count();
            match cutting.contains(&at) {
                true => format!("{}###{}", &line[..lead], &line[lead + marks..]),
                false => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn unstuck(body: &str, at: DayAt, shown: &str, note: usize) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let cut = found(body, at)?;
    let day = &lines[cut.heading + 1..cut.end];
    let gone = noted(day).get(note)?.clone();
    let seen: Vec<&str> = shown.lines().collect();
    let seen_cut = found(shown, at)?;
    let seen_day = &seen[seen_cut.heading + 1..seen_cut.end];
    let seen_note = noted(seen_day).get(note)?.clone();
    if day[gone.clone()] != seen_day[seen_note] {
        return None;
    }
    let before = trailed(&day[..gone.start]);
    let after = blank_free(&day[gone.end..]);
    let mut kept = before.to_vec();
    if !before.is_empty() && !after.is_empty() {
        kept.push("");
    }
    kept.extend(after);
    Some(spliced(body, &lines, cut, &kept))
}

fn spliced(body: &str, lines: &[&str], cut: Cut, day: &[&str]) -> String {
    let ending = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let day = unblank(day);
    let rest = blank_free(&lines[cut.end..]);
    let mut out: Vec<&str> = Vec::new();
    if day.is_empty() {
        let head = trailed(&lines[..cut.heading]);
        out.extend(head);
        if !head.is_empty() && !rest.is_empty() {
            out.push("");
        }
    } else {
        out.extend(&lines[..=cut.heading]);
        out.push("");
        out.extend(day);
        if !rest.is_empty() {
            out.push("");
        }
    }
    out.extend(rest);
    format!("{}{ending}", out.join(ending))
}

#[cfg(test)]
#[path = "diary_day_test.rs"]
mod tests;
