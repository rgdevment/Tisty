use std::collections::BTreeSet;
use std::path::Path;

use jiff::civil::Date;

use crate::event::{DeviceId, DocAdd, Said};
use crate::model::{DocId, Kept, Task};
use crate::{Op, Result, State};

/// The first release that keeps the diary from agents; an older one hands it to them like any page.
pub const SINCE: &str = "1.48.0";

pub struct Words {
    pub diary: &'static str,
    pub months: [&'static str; 12],
    pub weekdays: [&'static str; 7],
}

pub static ENGLISH: Words = Words {
    diary: "Diary",
    months: [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    weekdays: [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ],
};

pub static SPANISH: Words = Words {
    diary: "Diario",
    months: [
        "Enero",
        "Febrero",
        "Marzo",
        "Abril",
        "Mayo",
        "Junio",
        "Julio",
        "Agosto",
        "Septiembre",
        "Octubre",
        "Noviembre",
        "Diciembre",
    ],
    weekdays: [
        "Lunes",
        "Martes",
        "Miércoles",
        "Jueves",
        "Viernes",
        "Sábado",
        "Domingo",
    ],
};

static SPOKEN: [&Words; 2] = [&ENGLISH, &SPANISH];

pub fn words_for(locale: &str) -> &'static Words {
    match locale.trim().to_lowercase().starts_with("es") {
        true => &SPANISH,
        false => &ENGLISH,
    }
}

pub fn book(year: i16, words: &Words) -> String {
    format!("---\ndiary: on\n---\n\n# {} {year}\n", words.diary)
}

pub fn page(month: i8, words: &Words) -> String {
    format!("# {}\n", words.months[month_index(month)])
}

fn month_index(month: i8) -> usize {
    usize::from(month.clamp(1, 12).unsigned_abs()) - 1
}

pub fn day_title(date: Date, words: &Words) -> String {
    let weekday = usize::from(date.weekday().to_monday_zero_offset().unsigned_abs());
    format!("{} {}", words.weekdays[weekday], date.day())
}

// Titles are read in every language Tisty speaks, so a switch of language never starts a second month.
pub fn year_named(title: &str) -> Option<i16> {
    title
        .split(|one: char| !one.is_ascii_digit())
        .find(|one| one.len() == 4)
        .and_then(|one| one.parse().ok())
}

pub fn month_named(title: &str) -> Option<i8> {
    let lower = title.to_lowercase();
    let said: Vec<&str> = lower.split(|one: char| !one.is_alphabetic()).collect();
    SPOKEN.iter().find_map(|words| {
        let at = words
            .months
            .iter()
            .position(|month| said.contains(&month.to_lowercase().as_str()))?;
        i8::try_from(at + 1).ok()
    })
}

pub fn day_named(heading: &str) -> Option<i8> {
    heading
        .split(|one: char| !one.is_ascii_digit())
        .find(|one| !one.is_empty())
        .filter(|one| one.len() <= 2)
        .and_then(|one| one.parse().ok())
        .filter(|day| (1..=31).contains(day))
}

fn says_so(root: &Path, kept: &Kept) -> bool {
    crate::docs::read(root, &kept.file).is_ok_and(|body| tisty_markdown::front::diary(&body))
}

// Only a book read whole that no longer says so stops being one: a file not there yet keeps it shut.
fn still_one(root: &Path, kept: &Kept) -> bool {
    kept.diary
        && kept.page_of.is_none()
        && crate::docs::read(root, &kept.file)
            .map_or(true, |body| tisty_markdown::front::diary(&body))
}

pub fn books(root: &Path, state: &State) -> BTreeSet<DocId> {
    state
        .docs
        .values()
        .filter(|one| still_one(root, one))
        .map(|one| one.id)
        .collect()
}

pub fn within(books: &BTreeSet<DocId>, kept: &Kept) -> bool {
    books.contains(&kept.id) || kept.page_of.is_some_and(|up| books.contains(&up))
}

pub fn holds(root: &Path, state: &State, kept: &Kept) -> bool {
    let book = match kept.page_of {
        Some(up) => state.docs.get(&up),
        None => Some(kept),
    };
    book.is_some_and(|one| still_one(root, one))
}

pub fn out_of_sight(root: &Path, state: &mut State) {
    if state.diary_open {
        return;
    }
    let books = books(root, state);
    let hidden: Vec<DocId> = state
        .docs
        .values()
        .filter(|one| within(&books, one))
        .map(|one| one.id)
        .collect();
    for id in hidden {
        state.docs.remove(&id);
    }
}

fn year_of(kept: &Kept) -> Option<i16> {
    kept.title
        .as_deref()
        .and_then(year_named)
        .or_else(|| Some(kept.made?.to_zoned(jiff::tz::TimeZone::UTC).year()))
}

// The oldest wins, as with Brainstorm, so two machines that each began the year write in one book.
pub fn book_of<'a>(root: &Path, state: &'a State, year: i16) -> Option<&'a Kept> {
    let mut books: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| {
            one.diary
                && one.page_of.is_none()
                && one.task_of.is_none()
                && !state.held_away(one)
                && year_of(one) == Some(year)
        })
        .collect();
    books.sort_by_key(|one| one.id);
    books.into_iter().find(|one| says_so(root, one))
}

pub fn page_of(state: &State, book: DocId, month: i8) -> Option<&Kept> {
    let mut pages: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| {
            one.page_of == Some(book)
                && !state.held_away(one)
                && one.title.as_deref().and_then(month_named) == Some(month)
        })
        .collect();
    pages.sort_by_key(|one| one.id);
    pages.first().copied()
}

pub fn newest_page(state: &State, book: DocId) -> Option<&Kept> {
    state
        .docs
        .values()
        .filter(|one| one.page_of == Some(book) && !state.held_away(one))
        .filter_map(|one| Some((one.title.as_deref().and_then(month_named)?, one)))
        .max_by_key(|(month, one)| (*month, std::cmp::Reverse(one.id)))
        .map(|(_, one)| one)
}

#[derive(Debug)]
pub struct Opening {
    pub ops: Vec<Op>,
    pub book: String,
    pub page: String,
    pub page_made: bool,
}

#[derive(Debug)]
pub enum Opened {
    Ready(Opening),
    Locked { book: String },
    Waiting { behind: Vec<DeviceId> },
}

pub fn begun(state: &State) -> bool {
    state.docs.values().any(|one| one.diary)
}

// Once it has begun, the place each round raises stops an older machine, so nothing waits on it here.
pub fn held_by(state: &State, here: &DeviceId) -> Vec<DeviceId> {
    if begun(state) {
        return Vec::new();
    }
    state
        .not_yet_on(SINCE)
        .into_iter()
        .filter(|device| device != here)
        .collect()
}

pub struct Maker<'a> {
    pub root: &'a Path,
    pub device: &'a DeviceId,
    pub by: Option<String>,
    pub writer: &'a crate::history::Writer<'a>,
}

pub fn opened(maker: &Maker, state: &State, date: Date, words: &Words) -> Result<Opened> {
    let behind = held_by(state, maker.device);
    if !behind.is_empty() {
        return Ok(Opened::Waiting { behind });
    }
    let mut ops = Vec::new();
    let found = book_of(maker.root, state, date.year());
    let (book_id, book) = match found {
        Some(one) => (one.id, one.file.clone()),
        None => {
            let order = crate::order::last_of(
                state
                    .docs
                    .values()
                    .filter(|one| one.page_of.is_none() && one.folder.is_none())
                    .map(|one| one.order.as_str()),
            );
            let (id, file, op) = made(maker, &book(date.year(), words), order, None)?;
            ops.push(op);
            (id, file)
        }
    };
    let month = date.month();
    let (page, page_made) = match found.and_then(|_| page_of(state, book_id, month)) {
        Some(one) => (one.file.clone(), false),
        None if state.shut(book_id) => return Ok(Opened::Locked { book }),
        None => {
            let order = crate::order::last_of(
                state
                    .docs
                    .values()
                    .filter(|one| one.page_of == Some(book_id))
                    .map(|one| one.order.as_str()),
            );
            let (_, file, op) = made(maker, &page(month, words), order, Some(book_id))?;
            ops.push(op);
            (file, true)
        }
    };
    Ok(Opened::Ready(Opening {
        ops,
        book,
        page,
        page_made,
    }))
}

fn made(
    maker: &Maker,
    body: &str,
    order: String,
    page_of: Option<DocId>,
) -> Result<(DocId, String, Op)> {
    let file = crate::docs::create_by(maker.root, maker.writer, maker.device, body)?.id;
    let id = ulid::Ulid::generate();
    let op = Op::DocAdd {
        id,
        d: DocAdd {
            wrote: None,
            guest: false,
            sown: false,
            made: None,
            by: maker.by.clone(),
            file: file.clone(),
            order,
            said: Some(Said::of(body)),
            folder: None,
            page_of,
            task_of: None,
            edited: None,
        },
    };
    Ok((id, file, op))
}

pub fn named(
    root: &Path,
    writer: &crate::history::Writer,
    opening: &Opening,
) -> Result<Option<String>> {
    if !opening.page_made {
        return Ok(None);
    }
    let naming = crate::docs::name_at_end(root, writer, &opening.book, &[opening.page.as_str()])?;
    Ok(match naming {
        crate::docs::Naming::Wrote { whole, .. } => Some(whole),
        _ => None,
    })
}

pub fn written_on(body: &str, date: Date, words: &Words, text: &str) -> String {
    let ending = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let said = under_the_day(text.trim());
    let said: Vec<&str> = said.lines().collect();
    let lines: Vec<&str> = body.lines().collect();
    let days = crate::diary_day::cuts(body);
    let today = date.day();
    let heading = format!("## {}", day_title(date, words));
    let mut out: Vec<&str> = Vec::new();
    match days.iter().find(|one| one.day == today) {
        Some(cut) => {
            let last = cut.heading + 1 + trailed(&lines[cut.heading + 1..cut.end]).len();
            out.extend(&lines[..last]);
            out.push("");
            out.extend(&said);
            let rest = blank_free(&lines[last..]);
            if !rest.is_empty() {
                out.push("");
                out.extend(rest);
            }
        }
        None => {
            let older = days.iter().find(|one| one.day < today);
            let line = older.map_or(lines.len() + 1, |one| one.heading + 1);
            let head = trailed(&lines[..line - 1]);
            out.extend(head);
            if !head.is_empty() {
                out.push("");
            }
            out.push(&heading);
            out.push("");
            out.extend(&said);
            let rest = &lines[line - 1..];
            if !rest.is_empty() {
                out.push("");
                out.extend(rest);
            }
        }
    }
    format!("{}{ending}", out.join(ending))
}

pub(crate) fn trailed<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    let keep = lines
        .iter()
        .rposition(|one| !one.trim().is_empty())
        .map_or(0, |at| at + 1);
    &lines[..keep]
}

pub(crate) fn blank_free<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    let from = lines
        .iter()
        .position(|one| !one.trim().is_empty())
        .unwrap_or(lines.len());
    &lines[from..]
}

// A heading in what is written sits under the day, so it never starts a day of its own.
fn under_the_day(said: &str) -> String {
    let mut fenced = tisty_markdown::fences::fencing();
    said.lines()
        .map(|line| {
            if fenced(line) {
                return line.to_string();
            }
            let marks = line.chars().take_while(|one| *one == '#').count();
            match (1..=2).contains(&marks) && line[marks..].starts_with([' ', '\t']) {
                true => format!("###{}", &line[marks..]),
                false => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn month_of_page(root: &Path, state: &State, kept: &Kept) -> Option<(i16, i8)> {
    let book = state.docs.get(&kept.page_of?)?;
    if !book.diary || !says_so(root, book) {
        return None;
    }
    Some((year_of(book)?, kept.title.as_deref().and_then(month_named)?))
}

pub fn closed_on<'a>(state: &'a State, date: Date, zone: &jiff::tz::TimeZone) -> Vec<&'a Task> {
    let mut closed: Vec<&Task> = state
        .tasks
        .values()
        .filter(|task| {
            task.status == crate::model::Status::Done
                && !task.hidden
                && task.counted_on(zone) == Some(date)
        })
        .collect();
    closed.sort_by_key(|task| (task.completed_at, task.id));
    closed
}

#[cfg(test)]
#[path = "diary_test.rs"]
mod tests;
