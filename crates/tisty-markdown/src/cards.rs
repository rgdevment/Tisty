use std::fmt::Write as _;

use crate::inline::escaped;

pub struct Words {
    pub kind_doc: &'static str,
    pub untitled: &'static str,
    pub itself: &'static str,
    pub gone_doc: &'static str,
    pub here: &'static str,
    pub away: &'static str,
    pub gone: &'static str,
    pub page_of: &'static str,
    pub more: &'static str,
    pub kind_task: &'static str,
    pub task_away: &'static str,
    pub finish: &'static str,
    pub parts: &'static str,
    pub steps: &'static str,
    pub waiting: &'static str,
    pub done: &'static str,
    pub dropped: &'static str,
    pub due: &'static str,
    pub closed_on: &'static str,
}

pub static ENGLISH: Words = Words {
    kind_doc: "Document",
    untitled: "Untitled",
    itself: "This very document",
    gone_doc: "That document is not on this machine",
    here: "You are here",
    away: "Archived",
    gone: "Gone",
    page_of: "Page {name} of {other}",
    more: "More on this card",
    kind_task: "Task",
    task_away: "That task is not on this machine",
    finish: "Mark it done",
    parts: "{name} of {other} parts",
    steps: "{name} of {other} steps",
    waiting: "{name} to confirm",
    done: "Done",
    dropped: "Not doing",
    due: "due {name}",
    closed_on: "closed on {name}",
};

pub static SPANISH: Words = Words {
    kind_doc: "Documento",
    untitled: "Sin título",
    itself: "Este mismo documento",
    gone_doc: "Ese documento no está en esta máquina",
    here: "Estás aquí",
    away: "Archivado",
    gone: "Ya no está",
    page_of: "Página {name} de {other}",
    more: "Más sobre esta tarjeta",
    kind_task: "Tarea",
    task_away: "Esa tarea no está en esta máquina",
    finish: "Marcarla hecha",
    parts: "{name} de {other} partes",
    steps: "{name} de {other} pasos",
    waiting: "{name} por confirmar",
    done: "Hecha",
    dropped: "No se hará",
    due: "vence {name}",
    closed_on: "cerrada el {name}",
};

pub fn words(spoken: &str) -> &'static Words {
    if spoken.to_lowercase().starts_with("es") {
        &SPANISH
    } else {
        &ENGLISH
    }
}

const PAGE: &str = r#"<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" /> <path d="M14 2v5a1 1 0 0 0 1 1h5" /> <path d="M10 9H8" /> <path d="M16 13H8" /> <path d="M16 17H8" />"#;
const BOOK: &str = r#"<path d="M12 5v16" /> <path d="M20.001 19A2 2 0 0022 17V5a2 2 0 00-1.999-2L16 3.002A5 5 0 0012 5a5 5 0 00-4-2H4a2 2 0 00-2 2v12a2 2 0 001.999 2H8a5 5 0 014 2 5 5 0 014-2z" />"#;
const ARCHIVE: &str = r#"<rect width="20" height="5" x="2" y="3" rx="1" /> <path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8" /> <path d="M10 12h4" />"#;
const TODO: &str = r#"<circle cx="12" cy="12" r="10" />"#;
const DONE: &str = r#"<circle cx="12" cy="12" r="10" /> <path d="m9 12 2 2 4-4" />"#;

fn glyph(paths: &str) -> String {
    format!(r#"<svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">{paths}</svg>"#)
}

fn filled(said: &str, name: &str, other: &str) -> String {
    said.replace("{name}", name).replace("{other}", other)
}

fn piece(kind: &str, said: &str) -> String {
    format!(r#"<span class="{kind}">{said}</span>"#)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paper {
    Doc { title: String },
    Page { title: String, at: usize, of: usize },
    Away { title: String },
    Itself { title: String },
    Gone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCard {
    pub file: String,
    pub words: String,
    pub paper: Paper,
    pub blurb: String,
    pub more: bool,
}

impl DocCard {
    // An example names a document no store holds, so it is drawn by the words it was written with.
    pub fn example(file: &str, words: &str) -> Self {
        Self {
            file: file.to_string(),
            words: words.to_string(),
            paper: Paper::Doc {
                title: words.to_string(),
            },
            blurb: String::new(),
            more: false,
        }
    }
}

const BLURB_UNITS: usize = 160;

fn spanless(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find("::") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 2..];
        let name = after
            .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .unwrap_or(after.len());
        let style = after[name..]
            .strip_prefix(' ')
            .map(|more| {
                more.find(|c: char| !c.is_ascii_lowercase())
                    .unwrap_or(more.len())
            })
            .filter(|letters| *letters > 0)
            .map_or(0, |letters| letters + 1);
        if name > 0 && after[name + style..].starts_with("::") {
            out.push(':');
            out.push_str(&after[..name]);
            out.push(':');
            rest = &after[name + style + 2..];
        } else {
            out.push(':');
            rest = &rest[at + 1..];
        }
    }
    out.push_str(rest);
    out
}

pub fn blurb(body: &str) -> String {
    let joined = body
        .split('\n')
        .skip(1)
        .map(|line| {
            spanless(line)
                .trim_start_matches(|c: char| "#>*+-".contains(c) || c.is_whitespace())
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let mut units = 0;
    joined
        .chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= BLURB_UNITS
        })
        .collect()
}

pub fn doc_card(card: &DocCard, words: &Words) -> String {
    let titled = |title: &str| {
        let title = title.trim();
        escaped(if title.is_empty() {
            words.untitled
        } else {
            title
        })
    };
    let (kind, paths, name, chip) = match &card.paper {
        Paper::Doc { title } => ("doc", PAGE, titled(title), piece("kind", words.kind_doc)),
        Paper::Page { title, at, of } => {
            let said = filled(words.page_of, &at.to_string(), &of.to_string());
            ("page", BOOK, titled(title), piece("chip", &said))
        }
        Paper::Away { title } => (
            "away",
            ARCHIVE,
            titled(title),
            piece("chip away", words.away),
        ),
        Paper::Itself { title } => ("self", PAGE, titled(title), piece("chip here", words.here)),
        Paper::Gone => (
            "gone",
            PAGE,
            titled(&card.words),
            piece("chip gone", words.gone),
        ),
    };
    let open = !matches!(card.paper, Paper::Itself { .. } | Paper::Gone);
    let file = escaped(&card.file);
    let blurb = match card.paper {
        Paper::Itself { .. } => words.itself.to_string(),
        Paper::Gone => words.gone_doc.to_string(),
        _ => escaped(&card.blurb),
    };
    let mut out = format!(r#"<span class="doc-card is-{kind}" contenteditable="false""#);
    if open {
        let _ = write!(out, r#" tabindex="0" role="link" data-doc="{file}""#);
    }
    let _ = write!(
        out,
        r#"><span class="card-tile">{}</span>{}{}<span class="card-meta">{chip}"#,
        glyph(paths),
        piece("card-name", &name),
        piece("card-blurb", &blurb),
    );
    if card.more {
        let _ = write!(
            out,
            r#"<button type="button" class="card-more" data-asked="more" data-target="{}{file}" aria-label="{more}" title="{more}">⋯</button>"#,
            crate::refs::DOC,
            more = words.more,
        );
    }
    if open {
        out.push_str(&piece("card-go", "›"));
    }
    out.push_str("</span></span>");
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    Open,
    Done,
    Dropped,
}

// The window words a day in its own calendar, so a card's dates read as every other date it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct When {
    pub at: String,
    pub timed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub id: String,
    pub text: String,
    pub done: bool,
    /// An agent's word on an open task, waiting for the person to confirm it.
    pub said: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub title: String,
    pub standing: Standing,
    pub list: Option<String>,
    pub date: Option<When>,
    pub deadline: Option<When>,
    pub closed: Option<String>,
    pub steps: Vec<Step>,
    pub parts: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCard {
    pub id: String,
    pub words: String,
    pub task: Option<Task>,
    pub acting: bool,
}

pub fn step_html(text: &str) -> String {
    let html = crate::html(&crate::read(text));
    let html = html.trim();
    let inline = html
        .strip_prefix("<p")
        .and_then(|rest| rest.split_once('>'))
        .and_then(|(_, inner)| inner.strip_suffix("</p>"))
        .filter(|inner| !inner.contains("<p"));
    inline.unwrap_or(html).to_string()
}

fn timed(when: &When, says: &str) -> String {
    let clock = if when.timed { " data-timed" } else { "" };
    format!(
        r#"<time datetime="{}" data-say="{says}"{clock}></time>"#,
        escaped(&when.at)
    )
}

fn about(task: &Task, words: &Words) -> String {
    let mut said: Vec<String> = task.list.as_deref().map(escaped).into_iter().collect();
    match task.standing {
        Standing::Open => {
            said.extend(task.date.as_ref().map(|one| timed(one, "when")));
            said.extend(
                task.deadline
                    .as_ref()
                    .map(|one| filled(words.due, &timed(one, "when"), "")),
            );
        }
        Standing::Done | Standing::Dropped => said.extend(task.closed.as_ref().map(|at| {
            let day = timed(
                &When {
                    at: at.clone(),
                    timed: false,
                },
                "day",
            );
            filled(words.closed_on, &day, "")
        })),
    }
    said.join(" · ")
}

fn asked(kind: &str, id: &str, label: &str) -> String {
    format!(
        r#"<button type="button" data-asked="{kind}" data-task="{id}" aria-label="{}""#,
        escaped(label)
    )
}

fn chips(task: &Task, id: &str, words: &Words) -> String {
    let mut out = String::new();
    if let Some((closed, all)) = task.parts {
        out.push_str(&piece(
            "chip",
            &filled(words.parts, &closed.to_string(), &all.to_string()),
        ));
    }
    if !task.steps.is_empty() {
        let done = task.steps.iter().filter(|one| one.done).count();
        let waiting = task
            .steps
            .iter()
            .filter(|one| one.said && !one.done)
            .count();
        let mut said = filled(
            words.steps,
            &done.to_string(),
            &task.steps.len().to_string(),
        );
        if waiting > 0 {
            said.push_str(" · ");
            said.push_str(&filled(words.waiting, &waiting.to_string(), ""));
        }
        let _ = write!(
            out,
            r#"{} class="chip card-unfold" aria-expanded="false">{said}</button>"#,
            asked("taskSteps", id, &said)
        );
    }
    match task.standing {
        Standing::Open => {}
        Standing::Done => out.push_str(&piece("chip here", words.done)),
        Standing::Dropped => out.push_str(&piece("chip here", words.dropped)),
    }
    if out.is_empty() {
        out = piece("kind", words.kind_task);
    }
    out
}

fn steps(task: &Task, id: &str, ticking: bool) -> String {
    let mut out = String::from(r#"<span class="card-steps">"#);
    for step in &task.steps {
        let row = if step.done {
            "card-step is-done"
        } else {
            "card-step"
        };
        let ticked = if ticking {
            format!(
                r#"{} class="card-step-box" data-step="{}" role="checkbox" aria-checked="{}"></button>"#,
                asked("taskStep", id, &crate::reading::plain(&step.text)),
                escaped(&step.id),
                step.done
            )
        } else {
            piece("card-step-box", "")
        };
        let _ = write!(
            out,
            r#"<span class="{row}">{ticked}<span class="card-step-words">{}</span></span>"#,
            step_html(&step.text)
        );
    }
    out.push_str("</span>");
    out
}

pub fn task_card(card: &TaskCard, words: &Words) -> String {
    let id = escaped(&card.id);
    let Some(task) = &card.task else {
        return format!(
            r#"<span class="doc-card is-task is-gone" contenteditable="false" data-task="{id}"><span class="card-tile">{}</span>{}{}<span class="card-meta"></span></span>"#,
            glyph(TODO),
            piece("card-name", &escaped(&card.words)),
            piece("card-blurb", words.task_away),
        );
    };
    let open = matches!(task.standing, Standing::Open);
    let ticking = open && card.acting;
    let tile = if ticking {
        format!(
            r#"{} class="card-check" title="{finish}">{}</button>"#,
            asked("taskDone", &id, words.finish),
            glyph(TODO),
            finish = words.finish,
        )
    } else {
        glyph(if open { TODO } else { DONE })
    };
    let mut out = format!(
        r#"<span class="doc-card is-task is-{}" contenteditable="false" tabindex="0" role="link" data-task="{id}"><span class="card-tile">{tile}</span>{}{}<span class="card-meta">{}{}</span>"#,
        if open { "open" } else { "closed" },
        piece("card-name", &escaped(&task.title)),
        piece("card-blurb", &about(task, words)),
        chips(task, &id, words),
        piece("card-go", "›"),
    );
    if !task.steps.is_empty() {
        out.push_str(&steps(task, &id, ticking));
    }
    out.push_str("</span>");
    out
}

#[cfg(test)]
#[path = "cards_test.rs"]
mod tests;
