use serde_json::{Map, Value};

const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub struct Member {
    pub whole: (usize, usize),
    pub value: (usize, usize),
}

fn blank(text: &[u8], mut i: usize) -> usize {
    loop {
        while text.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if text[i..].starts_with(b"//") {
            while i < text.len() && text[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if text[i..].starts_with(b"/*") {
            i += 2;
            while i < text.len() && !text[i..].starts_with(b"*/") {
                i += 1;
            }
            i = (i + 2).min(text.len());
            continue;
        }
        return i;
    }
}

fn quoted(text: &[u8], at: usize) -> Option<usize> {
    let mut i = at + 1;
    while i < text.len() {
        match text[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

fn ends(text: &[u8], at: usize) -> Option<usize> {
    let at = blank(text, at);
    match *text.get(at)? {
        b'"' => quoted(text, at),
        open @ (b'{' | b'[') => {
            let shut = if open == b'{' { b'}' } else { b']' };
            let mut deep = 0usize;
            let mut i = at;
            while i < text.len() {
                match text[i] {
                    b'"' => i = quoted(text, i)?,
                    b'/' => {
                        let past = blank(text, i);
                        i = if past > i { past } else { i + 1 };
                    }
                    c if c == open => {
                        deep += 1;
                        i += 1;
                    }
                    c if c == shut => {
                        deep -= 1;
                        i += 1;
                        if deep == 0 {
                            return Some(i);
                        }
                    }
                    _ => i += 1,
                }
            }
            None
        }
        _ => {
            let mut i = at;
            while text
                .get(i)
                .is_some_and(|c| !matches!(c, b',' | b'}' | b']') && !c.is_ascii_whitespace())
            {
                i += 1;
            }
            (i > at).then_some(i)
        }
    }
}

pub fn opens(text: &[u8]) -> Option<usize> {
    let from = if text.starts_with(BOM) { BOM.len() } else { 0 };
    let at = blank(text, from);
    (text.get(at) == Some(&b'{')).then_some(at)
}

pub fn member(text: &[u8], obj: usize, name: &str) -> Option<Member> {
    let mut i = blank(text, obj + 1);
    while text.get(i) == Some(&b'"') {
        let shut = quoted(text, i)?;
        let key = std::str::from_utf8(&text[i + 1..shut - 1]).ok()?;
        let colon = blank(text, shut);
        if text.get(colon) != Some(&b':') {
            return None;
        }
        let from = blank(text, colon + 1);
        let to = ends(text, from)?;
        if key == name {
            return Some(Member {
                whole: (i, to),
                value: (from, to),
            });
        }
        let past = blank(text, to);
        if text.get(past) != Some(&b',') {
            return None;
        }
        i = blank(text, past + 1);
    }
    None
}

/// The value as written, so a caller can read a string back out of a file serde would refuse for
/// its comments.
pub fn said<T: serde::de::DeserializeOwned>(text: &str, span: (usize, usize)) -> Option<T> {
    serde_json::from_str(&text[span.0..span.1]).ok()
}

/// The command as written, whether a string or, as OpenCode wants it, the first of a list.
pub fn reads(text: &str, key: &str, name: &str) -> Option<String> {
    let raw = text.as_bytes();
    let under = member(raw, opens(raw)?, key)?;
    if raw.get(under.value.0) != Some(&b'{') {
        return None;
    }
    let it = member(raw, under.value.0, name)?;
    let command = member(raw, it.value.0, "command")?;
    match said::<serde_json::Value>(text, command.value)? {
        serde_json::Value::String(one) => Some(one),
        serde_json::Value::Array(all) => all.first()?.as_str().map(str::to_string),
        _ => None,
    }
}

pub fn holds(text: &str, key: &str, name: &str) -> bool {
    let raw = text.as_bytes();
    let under = opens(raw).and_then(|root| member(raw, root, key));
    under.is_some_and(|it| {
        raw.get(it.value.0) == Some(&b'{') && member(raw, it.value.0, name).is_some()
    })
}

pub fn repoint(text: &str, key: &str, name: &str, command: &str) -> Option<String> {
    let raw = text.as_bytes();
    let under = member(raw, opens(raw)?, key)?;
    let entry = member(raw, under.value.0, name)?;
    let said = member(raw, entry.value.0, "command")?.value;
    let program = match raw.get(said.0) {
        Some(b'[') => *elements(raw, said.0)?.first()?,
        _ => said,
    };
    let quoted = serde_json::to_string(command).ok()?;
    let now = format!("{}{quoted}{}", &text[..program.0], &text[program.1..]);
    (reads(&now, key, name).as_deref() == Some(command)).then_some(now)
}

pub fn same_but(was: &str, now: &str, left_out: impl Fn(&mut Map<String, Value>)) -> bool {
    let whole = |text| match str::trim(text).is_empty() {
        true => "{}",
        false => text,
    };
    let (was, now) = (whole(was), whole(now));
    if !valid(was) || !valid(now) {
        return false;
    }
    let read = |text: &str| match value(text) {
        Some(Value::Object(mut doc)) => {
            left_out(&mut doc);
            Some(doc)
        }
        _ => None,
    };
    match (read(was), read(now)) {
        (Some(was), Some(now)) => was == now,
        _ => true,
    }
}

// Agents keep half an emoji or a number past what Rust holds, so being JSON and being read are two questions.
pub fn valid(text: &str) -> bool {
    serde_json::from_str::<serde::de::IgnoredAny>(&plain(text)).is_ok()
}

pub fn value(text: &str) -> Option<Value> {
    serde_json::from_str(&plain(text)).ok()
}

fn plain(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let raw = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < raw.len() {
        let to = match raw[i] {
            b'"' => quoted(raw, i).unwrap_or(raw.len()),
            b'/' => {
                let past = past_comment(raw, i);
                if past > i {
                    out.push(' ');
                    i = past;
                    continue;
                }
                i + 1
            }
            b',' if matches!(raw.get(blank(raw, i + 1)), Some(b']' | b'}')) => {
                i += 1;
                continue;
            }
            _ => (i + 1..=raw.len())
                .find(|&to| text.is_char_boundary(to))
                .unwrap_or(raw.len()),
        };
        out.push_str(&text[i..to]);
        i = to;
    }
    out
}

pub fn commented(text: &str) -> bool {
    let raw = text.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        i = match raw[i] {
            b'"' => quoted(raw, i).unwrap_or(raw.len()),
            b'/' if past_comment(raw, i) > i => return true,
            _ => i + 1,
        };
    }
    false
}

fn past_comment(raw: &[u8], i: usize) -> usize {
    match raw.get(i + 1) {
        Some(b'/' | b'*') => blank(raw, i),
        _ => i,
    }
}

/// Ours set field by field, so what the person added to the entry stays; `true` overwrites.
pub fn set_fields(
    text: &str,
    key: &str,
    name: &str,
    fields: &[(&str, String, bool)],
) -> Option<String> {
    let whole: Vec<String> = fields
        .iter()
        .map(|(field, value, _)| format!("\"{field}\": {value}"))
        .collect();
    let whole = format!("{{ {} }}", whole.join(", "));
    if text.trim().is_empty() {
        return set(text, key, name, &whole);
    }
    let found = |text: &str| -> Option<Option<usize>> {
        let raw = text.as_bytes();
        let Some(under) = member(raw, opens(raw)?, key) else {
            return Some(None);
        };
        Some(member(raw, under.value.0, name).map(|it| it.value.0))
    };
    let Some(mut at) = found(text)? else {
        return set(text, key, name, &whole);
    };
    let mut now = text.to_string();
    for (field, value, overwrite) in fields {
        if now.as_bytes().get(at) != Some(&b'{') {
            return None;
        }
        now = match member(now.as_bytes(), at, field) {
            Some(there) if *overwrite => {
                format!("{}{value}{}", &now[..there.value.0], &now[there.value.1..])
            }
            Some(_) => continue,
            None => put(&now, at, &format!("\"{field}\": {value}")),
        };
        at = found(&now)??;
    }
    Some(now)
}

pub fn set(text: &str, key: &str, name: &str, entry: &str) -> Option<String> {
    if text.trim().is_empty() {
        return Some(format!(
            "{{\n  \"{key}\": {{\n    \"{name}\": {entry}\n  }}\n}}\n"
        ));
    }
    let raw = text.as_bytes();
    let root = opens(raw)?;
    let Some(under) = member(raw, root, key) else {
        let pad = deeper(text, root);
        let piece = format!("\"{key}\": {{\n{pad}\"{name}\": {entry}\n}}");
        return Some(put(text, root, &piece));
    };
    if raw.get(under.value.0) != Some(&b'{') {
        return None;
    }
    match member(raw, under.value.0, name) {
        Some(it) => Some(format!(
            "{}{entry}{}",
            &text[..it.value.0],
            &text[it.value.1..]
        )),
        None => Some(put(text, under.value.0, &format!("\"{name}\": {entry}"))),
    }
}

pub fn unset(text: &str, key: &str, name: &str) -> Option<String> {
    let raw = text.as_bytes();
    let root = opens(raw)?;
    let under = member(raw, root, key)?;
    if raw.get(under.value.0) != Some(&b'{') {
        return None;
    }
    unset_in(text, under.value.0, name)
}

pub fn unset_top(text: &str, name: &str) -> Option<String> {
    unset_in(text, opens(text.as_bytes())?, name)
}

fn unset_in(text: &str, object: usize, name: &str) -> Option<String> {
    let raw = text.as_bytes();
    let it = member(raw, object, name)?;
    let past = blank(raw, it.whole.1);
    if raw.get(past) != Some(&b',') {
        let before = members(raw, object)?
            .iter()
            .rev()
            .map(|one| one.whole.1)
            .find(|&end| end <= it.whole.0);
        return Some(last_taken(text, before, it.whole));
    }
    if text[it.whole.1..past].trim().is_empty() {
        return Some(line_taken(text, it.whole.0, past + 1));
    }
    let kept = format!("{}{}", &text[..past], &text[past + 1..]);
    Some(line_taken(&kept, it.whole.0, it.whole.1))
}

fn line_taken(text: &str, mut from: usize, mut to: usize) -> String {
    let raw = text.as_bytes();
    while from > 0 && matches!(raw[from - 1], b' ' | b'\t') {
        from -= 1;
    }
    if from > 0 && raw[from - 1] == b'\n' {
        while matches!(raw.get(to), Some(b' ' | b'\t' | b'\r')) {
            to += 1;
        }
        if raw.get(to) == Some(&b'\n') {
            to += 1;
        }
    }
    format!("{}{}", &text[..from], &text[to..])
}

// The comma left behind by the last one goes with it, wherever a comment pushed it.
fn last_taken(text: &str, before: Option<usize>, span: (usize, usize)) -> String {
    let raw = text.as_bytes();
    let comma = before
        .map(|end| blank(raw, end))
        .filter(|&at| raw.get(at) == Some(&b','));
    match comma {
        Some(at) if text[at + 1..span.0].trim().is_empty() => line_taken(text, at, span.1),
        Some(at) => {
            let kept = format!("{}{}", &text[..at], &text[at + 1..]);
            line_taken(&kept, span.0 - 1, span.1 - 1)
        }
        None => line_taken(text, span.0, span.1),
    }
}

fn members(raw: &[u8], obj: usize) -> Option<Vec<Member>> {
    let mut all = Vec::new();
    let mut i = blank(raw, obj + 1);
    while raw.get(i) == Some(&b'"') {
        let colon = blank(raw, quoted(raw, i)?);
        let from = blank(raw, colon + 1);
        let to = ends(raw, from)?;
        all.push(Member {
            whole: (i, to),
            value: (from, to),
        });
        let past = blank(raw, to);
        if raw.get(past) != Some(&b',') {
            break;
        }
        i = blank(raw, past + 1);
    }
    Some(all)
}

pub fn elements(text: &[u8], list: usize) -> Option<Vec<(usize, usize)>> {
    if text.get(list) != Some(&b'[') {
        return None;
    }
    let mut all = Vec::new();
    let mut i = blank(text, list + 1);
    while text.get(i) != Some(&b']') {
        let to = ends(text, i)?;
        all.push((i, to));
        let past = blank(text, to);
        match text.get(past) {
            Some(b',') => i = blank(text, past + 1),
            Some(b']') => i = past,
            _ => return None,
        }
    }
    Some(all)
}

pub fn shut(text: &[u8], at: usize) -> Option<usize> {
    ends(text, at)
}

pub fn push(text: &str, list: usize, piece: &str) -> Option<String> {
    let all = elements(text.as_bytes(), list)?;
    let pad = deeper(text, list);
    let laid: Vec<String> = piece.lines().map(|line| format!("{pad}{line}")).collect();
    let laid = laid.join("\n");
    Some(match all.last() {
        Some(&(_, end)) => format!("{},\n{laid}{}", &text[..end], &text[end..]),
        None => {
            let shut = filled_from(text, list);
            format!(
                "{}\n{laid}\n{}{}",
                &text[..list + 1],
                nudge(text, list),
                &text[shut..]
            )
        }
    })
}

pub fn cut(text: &str, list: usize, element: (usize, usize)) -> Option<String> {
    let raw = text.as_bytes();
    let past = blank(raw, element.1);
    if raw.get(past) != Some(&b',') {
        let before = elements(raw, list)?
            .iter()
            .rev()
            .map(|one| one.1)
            .find(|&end| end <= element.0);
        return Some(last_taken(text, before, element));
    }
    let next = blank(raw, past + 1);
    let apart = |from: usize, to: usize| text[from..to].trim().is_empty();
    if apart(element.1, past) && apart(past + 1, next) {
        return Some(format!("{}{}", &text[..element.0], &text[next..]));
    }
    let kept = format!("{}{}", &text[..past], &text[past + 1..]);
    Some(line_taken(&kept, element.0, element.1))
}

fn put(text: &str, obj: usize, piece: &str) -> String {
    let raw = text.as_bytes();
    let bare = raw.get(blank(raw, obj + 1)) == Some(&b'}');
    let first = filled_from(text, obj);
    let pad = deeper(text, obj);
    let laid: Vec<String> = piece.lines().map(|line| format!("{pad}{line}")).collect();
    let laid = laid.join("\n");

    let mut out = String::with_capacity(text.len() + laid.len() + 4);
    out.push_str(&text[..obj + 1]);
    out.push('\n');
    out.push_str(&laid);
    if bare {
        out.push('\n');
        out.push_str(&nudge(text, obj));
        out.push_str(&text[first..]);
    } else {
        out.push(',');
        out.push_str(&text[obj + 1..]);
    }
    out
}

fn filled_from(text: &str, open: usize) -> usize {
    let shut = blank(text.as_bytes(), open + 1);
    match text[open + 1..shut].trim().is_empty() {
        true => shut,
        false => open + 1,
    }
}

fn nudge(text: &str, at: usize) -> String {
    let line = text[..at].rfind('\n').map_or(0, |n| n + 1);
    text[line..at]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

fn deeper(text: &str, at: usize) -> String {
    let here = nudge(text, at);
    let step = if here.contains('\t') || text.contains("\n\t") {
        "\t"
    } else {
        "  "
    };
    format!("{here}{step}")
}

#[cfg(test)]
#[path = "json_test.rs"]
mod tests;
