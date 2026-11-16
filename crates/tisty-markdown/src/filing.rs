use std::collections::{HashMap, HashSet};
use std::ops::Range;

use crate::fences::Fencing;
use crate::rereading::Changed;
use crate::tags::{AT_MOST, fronted, lines_from, on_line, past};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Filing {
    pub(crate) filed: HashSet<String>,
    counts: HashMap<String, usize>,
    skipped: HashMap<String, usize>,
    fenced: Vec<Range<usize>>,
}

impl Filing {
    pub(crate) fn of(text: &str) -> Filing {
        let mut filing = Filing::default();
        if !cfg!(feature = "tags") {
            return filing;
        }
        filing.tally(fronted(text).collect(), false, true);
        let mut fence = Fencing::default();
        let mut fenced = Vec::new();
        for (taken, line) in lines_from(text, past(text)) {
            let skipped = fence.inside(line);
            filing.tally(on_line(line), skipped, true);
            if skipped {
                fenced.push(taken);
            }
        }
        filing.fenced = joined(fenced);
        filing.filed = filing.filed_in(text);
        filing
    }

    pub(crate) fn after(&self, old: &str, text: &str, changed: &Changed) -> (Filing, bool) {
        if !cfg!(feature = "tags") {
            return (Filing::default(), true);
        }
        let from = past(old);
        if from != past(text) || changed.start < from {
            return (Filing::of(text), false);
        }
        let first = old[..changed.start]
            .rfind('\n')
            .map_or(0, |at| at + 1)
            .max(from);
        let old_last = line_end(old, changed.old_end);
        let new_last = old_last - changed.old_end + changed.new_end;
        if let Some(next) = self.within_line(old, text, first, old_last, new_last) {
            return (next, true);
        }
        let mut was = Fencing::default();
        for (_, line) in lines_from(&old[..first], self.anchor(old, from, first)) {
            was.inside(line);
        }
        let mut now = was.clone();
        let mut next = self.clone();
        let mut touched = false;
        for (_, line) in lines_from(&old[..old_last], first) {
            let skipped = was.inside(line);
            touched |= next.tally(on_line(line), skipped, false);
        }
        let mut fenced = Vec::new();
        for (taken, line) in lines_from(&text[..new_last], first) {
            let skipped = now.inside(line);
            touched |= next.tally(on_line(line), skipped, true);
            if skipped {
                fenced.push(taken);
            }
        }
        let settled = was == now;
        let mut until = new_last;
        for (taken, line) in lines_from(text, new_last) {
            if was == now {
                break;
            }
            let (left, right) = (was.inside(line), now.inside(line));
            if left != right {
                touched |= next.tally(on_line(line), left, false);
                next.tally(on_line(line), right, true);
            }
            if right {
                fenced.push(taken.clone());
            }
            until = taken.end;
        }
        next.fenced = self.spliced(first, until - new_last + old_last, until, fenced);
        if touched {
            next.filed = next.filed_in(text);
        }
        (next, settled)
    }

    // One line that keeps its margin, quote and list mark and holds no fence moves the machine as it did.
    fn within_line(
        &self,
        old: &str,
        text: &str,
        first: usize,
        old_last: usize,
        new_last: usize,
    ) -> Option<Filing> {
        let (was, now) = (
            only_line(old, first, old_last)?,
            only_line(text, first, new_last)?,
        );
        let head = |one: &str| {
            let (deep, wide, said) = crate::fences::quoted(one);
            (deep, wide, said.is_empty(), crate::fences::opener(said))
        };
        let fenced = |one: &str| one.contains("```") || one.contains("~~~");
        if head(was) != head(now) || fenced(was) || fenced(now) {
            return None;
        }
        let skipped = self.skips(first);
        let mut next = self.clone();
        let touched =
            next.tally(on_line(was), skipped, false) | next.tally(on_line(now), skipped, true);
        let made = skipped.then_some(first..new_last);
        next.fenced = self.spliced(first, old_last, new_last, made);
        if touched {
            next.filed = next.filed_in(text);
        }
        Some(next)
    }

    pub(crate) fn far(&self) -> Vec<String> {
        let capped = self.counts.len() > AT_MOST;
        let mut far: Vec<String> = self
            .filed
            .iter()
            .filter(|name| capped || self.skipped.contains_key(*name))
            .map(|name| format!("#{name}"))
            .collect();
        far.sort_unstable();
        far
    }

    // A name an edit files or unfiles is written nowhere else, unless in code the fences skip or past the cap.
    pub(crate) fn alone(&self, was: &Filing, settled: bool) -> bool {
        if self.filed == was.filed {
            return true;
        }
        let capped = was.counts.len() > AT_MOST || self.counts.len() > AT_MOST;
        settled
            && !capped
            && self
                .filed
                .symmetric_difference(&was.filed)
                .all(|name| !was.skipped.contains_key(name) && !self.skipped.contains_key(name))
    }

    fn tally(&mut self, names: Vec<String>, skipped: bool, adding: bool) -> bool {
        let into = if skipped {
            &mut self.skipped
        } else {
            &mut self.counts
        };
        let any = !names.is_empty();
        for name in names {
            let count = into.get(&name).copied().unwrap_or(0);
            match if adding {
                count + 1
            } else {
                count.saturating_sub(1)
            } {
                0 => into.remove(&name),
                more => into.insert(name, more),
            };
        }
        any
    }

    fn filed_in(&self, text: &str) -> HashSet<String> {
        if self.counts.len() <= AT_MOST {
            self.counts.keys().cloned().collect()
        } else {
            crate::tags::tags_in(text).into_iter().collect()
        }
    }

    // A line at the margin sets how deep the fences read lists whatever came before, once no fence is open above it.
    fn anchor(&self, text: &str, from: usize, first: usize) -> usize {
        let mut end = first;
        while end > from {
            let start = text[..end - 1].rfind('\n').map_or(0, |at| at + 1).max(from);
            let line = lines_from(&text[..end], start)
                .next()
                .map_or("", |(_, line)| line);
            let (_, wide, said) = crate::fences::quoted(line);
            if wide == 0 && !said.is_empty() && (start == from || !self.skips(start - 1)) {
                return start;
            }
            end = start;
        }
        from
    }

    fn skips(&self, at: usize) -> bool {
        let next = self.fenced.partition_point(|one| one.end <= at);
        self.fenced.get(next).is_some_and(|one| one.start <= at)
    }

    fn spliced(
        &self,
        first: usize,
        old_until: usize,
        until: usize,
        made: impl IntoIterator<Item = Range<usize>>,
    ) -> Vec<Range<usize>> {
        let moved = |at: usize| at - old_until + until;
        let before = self
            .fenced
            .iter()
            .filter(|one| one.start < first)
            .map(|one| one.start..one.end.min(first));
        let after = self
            .fenced
            .iter()
            .filter(|one| one.end > old_until)
            .map(|one| moved(one.start.max(old_until))..moved(one.end));
        joined(before.chain(made).chain(after))
    }
}

fn only_line(said: &str, first: usize, last: usize) -> Option<&str> {
    let mut lines = lines_from(&said[..last], first);
    let (_, one) = lines.next()?;
    lines.next().is_none().then_some(one)
}

fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |end| at + end + 1)
}

fn joined(runs: impl IntoIterator<Item = Range<usize>>) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    for one in runs {
        match out.last_mut() {
            Some(last) if last.end == one.start => last.end = one.end,
            _ => out.push(one),
        }
    }
    out
}

#[cfg(test)]
#[path = "filing_test.rs"]
mod tests;
