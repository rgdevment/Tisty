use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

use crate::inline::Told;
use crate::page::{self, Drawing};
use crate::shared::{CELLS_AT_MOST, shared};
use crate::{Block, Column, Kind, values};

const WORD_CELLS_AT_MOST: usize = 4_000_000;
const LOOKS_AHEAD: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Compared {
    pub html: String,
    pub gone: usize,
    pub came: usize,
}

/// One page for two bodies: blocks say `data-change` gone or came, words that differ sit in `del` and `ins`; None when too long.
pub fn compared(was: &str, now: &str) -> Option<Compared> {
    spending(was, now, CELLS_AT_MOST)
}

fn spending(was: &str, now: &str, cells: usize) -> Option<Compared> {
    let (was, now) = (crate::read(&even(was)), crate::read(&even(now)));
    let cells = Cell::new(cells);
    let bodies = Bodies {
        was: was.text(),
        now: now.text(),
        cells: &cells,
    };
    let steps = told(was.blocks(), now.blocks(), bodies)?;
    let (unnamed, named) = (HashMap::new(), page::anchors(&now));
    let side = |text, anchors, title, named| Side {
        text,
        drawing: Drawing {
            anchors,
            aligned: false,
            placed: false,
            title,
            named,
            told: Told::default(),
        },
    };
    let sides = Sides {
        was: side(bodies.was, &unnamed, page::titled(was.blocks()), false),
        now: side(bodies.now, &named, page::titled(now.blocks()), true),
    };
    let mut out = Compared::default();
    drawn(&steps, &sides, &mut out);
    Some(out)
}

fn even(body: &str) -> Cow<'_, str> {
    match body.contains('\r') {
        true => Cow::Owned(body.replace("\r\n", "\n")),
        false => Cow::Borrowed(body),
    }
}

#[derive(Clone, Copy)]
struct Bodies<'a> {
    was: &'a str,
    now: &'a str,
    cells: &'a Cell<usize>,
}

impl Bodies<'_> {
    // Blocks nested or words many, one page spends the same cells: none is made to cost its square again and again.
    fn shared<T: Hash + Eq>(&self, a: &[T], b: &[T], most: usize) -> Option<Vec<(usize, usize)>> {
        let found = shared(a, b, most.min(self.cells.get()))?;
        self.cells.set(self.cells.get() - found.cells);
        Some(found.pairs)
    }
}

type Marks = Vec<Range<usize>>;

enum Step<'a> {
    Same(&'a Block),
    Gone(&'a Block, Marks),
    Came(&'a Block, Marks),
    Within(&'a Block, &'a Block, Vec<Step<'a>>),
}

#[derive(PartialEq, Eq, Hash)]
enum Key<'a> {
    Said(&'a str),
    Cells(Vec<&'a str>),
}

fn key<'a>(block: &Block, text: &'a str) -> Key<'a> {
    let said = text[block.at.clone()].trim_end();
    match block.kind {
        Kind::Item { .. } => Key::Said(unmarked(said)),
        Kind::Row { .. } => Key::Cells(
            block
                .inner
                .iter()
                .map(|cell| text[cell.at.clone()].trim())
                .collect(),
        ),
        _ => Key::Said(said),
    }
}

fn unmarked(item: &str) -> &str {
    let said = item.trim_start();
    let digits = said.bytes().take_while(u8::is_ascii_digit).count();
    let rest = match digits {
        0 => said.strip_prefix(['-', '*', '+']),
        _ => said[digits..].strip_prefix(['.', ')']),
    };
    rest.map_or(said, str::trim_start)
}

fn told<'a>(was: &'a [Block], now: &'a [Block], bodies: Bodies<'a>) -> Option<Vec<Step<'a>>> {
    let keys = |blocks: &[Block], text| blocks.iter().map(|one| key(one, text)).collect();
    let (old, new): (Vec<Key>, Vec<Key>) = (keys(was, bodies.was), keys(now, bodies.now));
    let pairs = bodies.shared(&old, &new, CELLS_AT_MOST)?;
    let mut out = Vec::new();
    let (mut a, mut b) = (0, 0);
    for (at, to) in pairs.into_iter().chain([(was.len(), now.len())]) {
        between(&was[a..at], &now[b..to], bodies, &mut out);
        out.extend(now.get(to).map(Step::Same));
        (a, b) = (at + 1, to + 1);
    }
    Some(out)
}

struct Said<'a> {
    kind: &'a Kind,
    words: Vec<Range<usize>>,
    bag: Vec<&'a str>,
}

impl<'a> Said<'a> {
    // A bar, a bullet or a comma is in every block of its kind, so words weigh by their letters and digits alone.
    fn of(block: &'a Block, text: &'a str) -> Self {
        let words = words(text, block.at.clone());
        let mut bag: Vec<&str> = words
            .iter()
            .map(|one| text[one.clone()].trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .collect();
        bag.sort_unstable();
        Self {
            kind: &block.kind,
            words,
            bag,
        }
    }

    fn alike(&self, other: &Self) -> bool {
        let kin = std::mem::discriminant(self.kind) == std::mem::discriminant(other.kind);
        kin && 4 * common(&self.bag, &other.bag) >= (self.bag.len() + other.bag.len()).max(1)
    }
}

fn words(text: &str, at: Range<usize>) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut from = None;
    for (index, one) in text[at.clone()].char_indices() {
        match (one.is_whitespace(), from) {
            (true, Some(start)) => {
                out.push(at.start + start..at.start + index);
                from = None;
            }
            (false, None) => from = Some(index),
            _ => {}
        }
    }
    out.extend(from.map(|start| at.start + start..at.end));
    out
}

fn common(a: &[&str], b: &[&str]) -> usize {
    let (mut i, mut j, mut both) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                both += 1;
                i += 1;
                j += 1;
            }
        }
    }
    both
}

fn between<'a>(gone: &'a [Block], came: &'a [Block], bodies: Bodies<'a>, out: &mut Vec<Step<'a>>) {
    let went_alone = |one| Step::Gone(one, Marks::new());
    let came_alone = |one| Step::Came(one, Marks::new());
    let (mut from, mut waiting) = (0, 0);
    if !gone.is_empty() && !came.is_empty() {
        let went: Vec<Said> = gone.iter().map(|one| Said::of(one, bodies.was)).collect();
        for (to, new) in came.iter().enumerate() {
            if from == gone.len() {
                break;
            }
            let here = Said::of(new, bodies.now);
            let ahead = went[from..].iter().take(LOOKS_AHEAD).enumerate();
            let Some((by, there)) = ahead.into_iter().find(|(_, there)| there.alike(&here)) else {
                continue;
            };
            out.extend(gone[from..from + by].iter().map(went_alone));
            out.extend(came[waiting..to].iter().map(came_alone));
            pair((&gone[from + by], new), (there, &here), bodies, out);
            (from, waiting) = (from + by + 1, to + 1);
        }
    }
    out.extend(gone[from..].iter().map(went_alone));
    out.extend(came[waiting..].iter().map(came_alone));
}

fn pair<'a>(
    (gone, came): (&'a Block, &'a Block),
    (went, come): (&Said, &Said),
    bodies: Bodies<'a>,
    out: &mut Vec<Step<'a>>,
) {
    if let Some(steps) = within(gone, came, bodies) {
        out.push(Step::Within(gone, came, steps));
        return;
    }
    let (old, new): (Vec<&str>, Vec<&str>) = (
        went.words
            .iter()
            .map(|one| &bodies.was[one.clone()])
            .collect(),
        come.words
            .iter()
            .map(|one| &bodies.now[one.clone()])
            .collect(),
    );
    let (lost, won) = match bodies.shared(&old, &new, WORD_CELLS_AT_MOST) {
        Some(kept) => (
            marks(&went.words, kept.iter().map(|one| one.0)),
            marks(&come.words, kept.iter().map(|one| one.1)),
        ),
        None => (Marks::new(), Marks::new()),
    };
    out.push(Step::Gone(gone, lost));
    out.push(Step::Came(came, won));
}

fn marks(words: &[Range<usize>], kept: impl Iterator<Item = usize>) -> Marks {
    let mut out = Marks::new();
    let mut next = 0;
    for same in kept.chain([words.len()]) {
        if next < same {
            out.push(words[next].start..words[same - 1].end);
        }
        next = same + 1;
    }
    out
}

// A list turned numbered or a table with another head is a change of its own, so only one that opens as it did is told piece by piece.
fn within<'a>(gone: &'a Block, came: &'a Block, bodies: Bodies<'a>) -> Option<Vec<Step<'a>>> {
    let opens_alike = match (&gone.kind, &came.kind) {
        (Kind::Table { columns: old }, Kind::Table { columns: new }) => {
            let heads = (gone.inner.first()?, came.inner.first()?);
            columns_alike(old, new) && key(heads.0, bodies.was) == key(heads.1, bodies.now)
        }
        (Kind::Item { done: old }, Kind::Item { done: new }) => {
            old == new && own(gone, bodies.was) == own(came, bodies.now)
        }
        (Kind::Callout(old), Kind::Callout(new)) => {
            old == new && first_line(gone, bodies.was) == first_line(came, bodies.now)
        }
        (
            Kind::Quote
            | Kind::List { .. }
            | Kind::Details { .. }
            | Kind::Aligned { .. }
            | Kind::Note { .. },
            _,
        ) => gone.kind == came.kind,
        _ => false,
    };
    if !opens_alike {
        return None;
    }
    let rows = usize::from(matches!(came.kind, Kind::Table { .. }));
    told(gone.inner.get(rows..)?, came.inner.get(rows..)?, bodies)
}

fn columns_alike(old: &[Column], new: &[Column]) -> bool {
    let said = |one: &Column| {
        (
            one.lean,
            one.wide,
            one.holds.clone(),
            one.total,
            one.typed.is_some(),
        )
    };
    old.len() == new.len() && old.iter().zip(new).all(|(a, b)| said(a) == said(b))
}

fn own<'a>(item: &Block, text: &'a str) -> &'a str {
    let ends = item.inner.first().map_or(item.at.end, |one| one.at.start);
    unmarked(text[item.at.start..ends.max(item.at.start)].trim_end())
}

fn first_line<'a>(block: &Block, text: &'a str) -> Option<&'a str> {
    text[block.at.clone()].lines().next()
}

#[derive(Clone, Copy)]
struct Side<'a> {
    text: &'a str,
    drawing: Drawing<'a>,
}

impl<'a> Side<'a> {
    fn telling(&self, tag: &'static str, at: &'a [Range<usize>]) -> Drawing<'a> {
        Drawing {
            told: Told { tag, at },
            ..self.drawing
        }
    }
}

struct Sides<'a> {
    was: Side<'a>,
    now: Side<'a>,
}

fn drawn(steps: &[Step], sides: &Sides, out: &mut Compared) {
    let (was, now) = (&sides.was, &sides.now);
    for step in steps {
        match step {
            Step::Same(block) => page::draw(block, now.text, &now.drawing, &mut out.html),
            Step::Gone(block, marks) => {
                out.gone += 1;
                let drawing = was.telling("del", marks);
                page::draw_as(block, was.text, &drawing, Some("gone"), &mut out.html);
            }
            Step::Came(block, marks) => {
                out.came += 1;
                let drawing = now.telling("ins", marks);
                page::draw_as(block, now.text, &drawing, Some("came"), &mut out.html);
            }
            Step::Within(gone, came, inner) => within_drawn((gone, came), inner, sides, out),
        }
    }
}

fn within_drawn((gone, came): (&Block, &Block), inner: &[Step], sides: &Sides, out: &mut Compared) {
    let now = &sides.now;
    let at = page::placed_at(came, None);
    match &came.kind {
        Kind::Table { .. } => rows_drawn((gone, came), inner, sides, out),
        Kind::Item { done } => {
            page::item_open(*done, &at, came, now.text, &now.drawing, &mut out.html);
            drawn(inner, sides, out);
            out.html.push_str("</li>");
        }
        _ => {
            let Some((open, shut)) = page::shell(came, now.text, &now.drawing, &at) else {
                return;
            };
            out.html.push_str(&open);
            drawn(inner, sides, out);
            out.html.push_str(shut);
        }
    }
}

// Each row is read by the table it was written in, so a value that went shows as it showed then.
fn rows_drawn((gone, came): (&Block, &Block), rows: &[Step], sides: &Sides, out: &mut Compared) {
    let (Kind::Table { columns: old }, Kind::Table { columns: new }) = (&gone.kind, &came.kind)
    else {
        return;
    };
    let (was, now) = (&sides.was, &sides.now);
    let then = (old.as_slice(), &values::read(old, gone, was.text));
    let today = (new.as_slice(), &values::read(new, came, now.text));
    let place = |table: &Block, row: &Block| {
        table
            .inner
            .partition_point(|one| one.at.start < row.at.start)
    };
    let mut body = false;
    page::table_open(new, &page::placed_at(came, None), &mut out.html);
    page::row(
        today,
        (came, 0),
        now.text,
        &now.drawing,
        None,
        &mut body,
        &mut out.html,
    );
    for step in rows {
        match step {
            Step::Same(row) => {
                let at = (came, place(came, row));
                page::row(
                    today,
                    at,
                    now.text,
                    &now.drawing,
                    None,
                    &mut body,
                    &mut out.html,
                );
            }
            Step::Gone(row, marks) => {
                out.gone += 1;
                let (at, drawing) = ((gone, place(gone, row)), was.telling("del", marks));
                page::row(
                    then,
                    at,
                    was.text,
                    &drawing,
                    Some("gone"),
                    &mut body,
                    &mut out.html,
                );
            }
            Step::Came(row, marks) => {
                out.came += 1;
                let (at, drawing) = ((came, place(came, row)), now.telling("ins", marks));
                page::row(
                    today,
                    at,
                    now.text,
                    &drawing,
                    Some("came"),
                    &mut body,
                    &mut out.html,
                );
            }
            Step::Within(..) => {}
        }
    }
    page::table_shut(new, today.1, body, &mut out.html);
}

#[cfg(test)]
#[path = "compared_test.rs"]
mod tests;
