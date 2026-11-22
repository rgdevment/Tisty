use std::io::{BufRead, Write};

use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
struct Check {
    bytes: usize,
    hash: [u32; 2],
}

#[derive(Deserialize)]
struct Edit {
    was: Check,
    from: usize,
    to: usize,
    insert: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Sent {
    Body(String),
    Edit(Edit),
}

#[derive(Deserialize)]
#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]
enum Asked {
    #[serde(rename = "doc_placed")]
    Placed { body: String, check: Check },
    #[serde(rename = "doc_drawn")]
    Drawn { body: String },
    #[serde(rename = "doc_placed_near")]
    PlacedNear {
        sent: Sent,
        now: Check,
        from: usize,
        to: usize,
    },
}

struct Held {
    doc: tisty_markdown::Doc,
    check: Check,
}

fn near(
    doc: &tisty_markdown::Doc,
    from: usize,
    to: usize,
    shown: Option<[u32; 2]>,
    reached: bool,
) -> Value {
    let near = tisty_markdown::placed_near(doc, from, to, shown, reached);
    json!({
        "count": near.count,
        "sum": near.sum,
        "from": near.drawn.start,
        "to": near.drawn.end,
        "html": near.html,
        "heads": near.heads,
        "whole": near.whole,
    })
}

fn reread(held: &mut Option<Held>, body: &str, now: Check, from: usize, to: usize) -> Value {
    let (doc, shown) = match held.take() {
        Some(was) => {
            let shown = was.doc.labels();
            (tisty_markdown::reread(was.doc, body), Some(shown))
        }
        None => (tisty_markdown::read(body), None),
    };
    let said = near(&doc, from, to, shown, true);
    *held = Some(Held { doc, check: now });
    said
}

fn edited(held: &mut Option<Held>, edit: &Edit, now: Check, from: usize, to: usize) -> Value {
    let was = edit.was;
    let agreed = held
        .as_ref()
        .is_some_and(|one| one.check == was && one.doc.text().len() == was.bytes);
    let adds_up = edit.from <= edit.to
        && edit.to <= was.bytes
        && was.bytes - edit.to + edit.from + edit.insert.len() == now.bytes;
    if !agreed || !adds_up {
        return Value::Null;
    }
    let Some(one) = held.take() else {
        return Value::Null;
    };
    let shown = one.doc.labels();
    let reached = tisty_markdown::reaches_heading(&one.doc, edit.from..edit.to);
    match tisty_markdown::reread_edit(one.doc, edit.from..edit.to, &edit.insert) {
        Ok(doc) => {
            let said = near(&doc, from, to, Some(shown), reached);
            *held = Some(Held { doc, check: now });
            said
        }
        Err(doc) => {
            *held = Some(Held { doc, check: was });
            Value::Null
        }
    }
}

fn answer(held: &mut Option<Held>, asked: Asked) -> Value {
    match asked {
        Asked::Placed { body, check } => {
            let doc = tisty_markdown::read(&body);
            let html = tisty_markdown::html_placed(&doc);
            *held = Some(Held { doc, check });
            json!(html)
        }
        Asked::Drawn { body } => json!(tisty_markdown::html(&tisty_markdown::read(&body))),
        Asked::PlacedNear {
            sent,
            now,
            from,
            to,
        } => match sent {
            Sent::Body(body) => reread(held, &body, now, from, to),
            Sent::Edit(edit) => edited(held, &edit, now, from, to),
        },
    }
}

fn main() {
    let mut held = None;
    let mut out = std::io::stdout();
    for line in std::io::stdin().lock().lines() {
        let line = line.expect("a question from the page");
        let said = match serde_json::from_str::<Asked>(&line) {
            Ok(asked) => answer(&mut held, asked),
            Err(e) => json!({ "refused": e.to_string() }),
        };
        writeln!(out, "{said}").expect("an answer to the page");
        out.flush().expect("the answer handed over");
    }
}
