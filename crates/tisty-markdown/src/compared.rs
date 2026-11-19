use std::borrow::Cow;
use std::cell::{Cell, LazyCell, OnceCell};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
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
    drawn(&steps, &sides, None, &mut out);
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
    Said(u64, &'a str),
    Cells(Vec<&'a str>),
}

// The same line indented is code and no longer a paragraph: what a block is made of weighs with what it says.
fn shape(block: &Block, into: &mut DefaultHasher) {
    let inner = &block.inner[usize::from(loose(block))..];
    std::mem::discriminant(&block.kind).hash(into);
    inner.len().hash(into);
    inner.iter().for_each(|one| shape(one, into));
}

// One blank line in a list moves every item's words into a paragraph of its own, and none of them changed.
fn loose(block: &Block) -> bool {
    let wrapped = matches!(block.inner.first(), Some(first) if first.kind == Kind::Paragraph);
    matches!(block.kind, Kind::Item { .. }) && block.spans.is_empty() && wrapped
}

fn key<'a>(block: &Block, text: &'a str) -> Key<'a> {
    if let Kind::Row { .. } = block.kind {
        let cells = block.inner.iter();
        return Key::Cells(cells.map(|cell| text[cell.at.clone()].trim()).collect());
    }
    let said = text[block.at.clone()].trim_end();
    let mut made = DefaultHasher::new();
    shape(block, &mut made);
    match block.kind {
        Kind::Item { .. } => Key::Said(made.finish(), unmarked(said)),
        _ => Key::Said(made.finish(), said),
    }
}

fn marker(item: &str) -> usize {
    let lead = item.len() - item.trim_start().len();
    let mark = crate::fences::opener(&item[lead..]).unwrap_or(0);
    (lead + mark).min(item.len())
}

fn unmarked(item: &str) -> &str {
    &item[marker(item)..]
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
        let bars = matches!(block.kind, Kind::Row { .. });
        let words = words(text, block.at.clone(), bars);
        let said = match block.kind {
            Kind::Item { .. } => block.at.start + marker(&text[block.at.clone()]),
            _ => block.at.start,
        };
        let mut bag: Vec<&str> = words
            .iter()
            .filter(|one| one.start >= said)
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

fn words(text: &str, at: Range<usize>, bars: bool) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut from = None;
    for (index, one) in text[at.clone()].char_indices() {
        let bar = bars && one == '|';
        match (one.is_whitespace() || bar, from) {
            (true, Some(start)) => {
                out.push(at.start + start..at.start + index);
                from = None;
            }
            (false, None) => from = Some(index),
            _ => {}
        }
        if bar {
            out.push(at.start + index..at.start + index + 1);
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
    let went: Vec<OnceCell<Said>> = gone.iter().map(|_| OnceCell::new()).collect();
    let said = |at: usize| went[at].get_or_init(|| Said::of(&gone[at], bodies.was));
    for (to, new) in came.iter().enumerate() {
        if from == gone.len() {
            break;
        }
        let here = Said::of(new, bodies.now);
        let mut ahead = (from..gone.len()).take(LOOKS_AHEAD);
        let Some(at) = ahead.find(|at| said(*at).alike(&here)) else {
            continue;
        };
        out.extend(gone[from..at].iter().map(went_alone));
        out.extend(came[waiting..to].iter().map(came_alone));
        pair((&gone[at], new), (said(at), &here), bodies, out);
        (from, waiting) = (at + 1, to + 1);
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
            let alike =
                loose(gone) == loose(came) && own(gone, bodies.was).eq(own(came, bodies.now));
            old == new && alike
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
    let said = |one: &'_ Column| (one.lean, one.wide, one.total, one.typed.is_some());
    let alike = |a: &Column, b: &Column| said(a) == said(b) && a.holds == b.holds;
    old.len() == new.len() && old.iter().zip(new).all(|(a, b)| alike(a, b))
}

fn own<'a>(item: &'a Block, text: &'a str) -> impl Iterator<Item = &'a str> {
    item.spans.iter().map(move |one| &text[one.at.clone()])
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
    fn telling(&self, change: &'static str, at: &'a [Range<usize>]) -> Drawing<'a> {
        Drawing {
            told: Told { change, at },
            ..self.drawing
        }
    }
}

struct Sides<'a> {
    was: Side<'a>,
    now: Side<'a>,
}

// An item that went and the one in its place would both count, so in a numbered list each says its own number.
fn standing(block: &Block, change: Option<&str>, number: Option<u64>) -> String {
    let at = page::placed_at(block, change);
    match number {
        Some(number) => format!("{at} value=\"{number}\""),
        None => at,
    }
}

fn next(number: &mut Option<u64>) -> Option<u64> {
    let said = *number;
    *number = said.map(|at| at + 1);
    said
}

fn drawn(steps: &[Step], sides: &Sides, first: Option<u64>, out: &mut Compared) {
    let (was, now) = (&sides.was, &sides.now);
    let (mut then, mut today) = (first, first);
    for step in steps {
        match step {
            Step::Same(block) => {
                let at = standing(block, None, next(&mut today));
                next(&mut then);
                page::draw_as(block, now.text, &now.drawing, &at, &mut out.html);
            }
            Step::Gone(block, marks) => {
                out.gone += 1;
                let at = standing(block, Some("gone"), next(&mut then));
                let drawing = was.telling("gone", marks);
                page::draw_as(block, was.text, &drawing, &at, &mut out.html);
            }
            Step::Came(block, marks) => {
                out.came += 1;
                let at = standing(block, Some("came"), next(&mut today));
                let drawing = now.telling("came", marks);
                page::draw_as(block, now.text, &drawing, &at, &mut out.html);
            }
            Step::Within(gone, came, inner) => {
                let at = standing(came, None, next(&mut today));
                next(&mut then);
                within_drawn((gone, came), inner, sides, &at, out);
            }
        }
    }
}

fn within_drawn(
    (gone, came): (&Block, &Block),
    inner: &[Step],
    sides: &Sides,
    at: &str,
    out: &mut Compared,
) {
    let now = &sides.now;
    match &came.kind {
        Kind::Table { .. } => rows_drawn((gone, came), inner, sides, at, out),
        Kind::Item { done } => {
            page::item_open(*done, at, came, now.text, &now.drawing, &mut out.html);
            drawn(inner, sides, None, out);
            out.html.push_str("</li>");
        }
        kind => {
            let Some((open, shut)) = page::shell(came, now.text, &now.drawing, at) else {
                return;
            };
            let first = match kind {
                Kind::List { start } => *start,
                _ => None,
            };
            out.html.push_str(&open);
            drawn(inner, sides, first, out);
            out.html.push_str(shut);
        }
    }
}

// Each row is read by the table it was written in, so a value that went shows as it showed then.
fn rows_drawn(
    (gone, came): (&Block, &Block),
    rows: &[Step],
    sides: &Sides,
    at: &str,
    out: &mut Compared,
) {
    let (Kind::Table { columns: old }, Kind::Table { columns: new }) = (&gone.kind, &came.kind)
    else {
        return;
    };
    let (was, now) = (&sides.was, &sides.now);
    let read_then = LazyCell::new(|| values::read(old, gone, was.text));
    let today = (new.as_slice(), &values::read(new, came, now.text));
    let place = |table: &Block, row: &Block| {
        table
            .inner
            .partition_point(|one| one.at.start < row.at.start)
    };
    let mut body = false;
    page::table_open(new, at, &mut out.html);
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
                let (at, drawing) = ((gone, place(gone, row)), was.telling("gone", marks));
                page::row(
                    (old.as_slice(), &*read_then),
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
                let (at, drawing) = ((came, place(came, row)), now.telling("came", marks));
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
