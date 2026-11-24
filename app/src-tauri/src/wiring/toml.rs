use toml_edit::{Document, DocumentMut, Item, TableLike, Value};

const BOM: char = '\u{feff}';

pub fn reads(text: &str, key: &str, name: &str) -> Option<String> {
    parsed(text)?
        .get(key)?
        .get(name)?
        .get("command")?
        .as_str()
        .map(str::to_string)
}

pub fn holds(text: &str, key: &str, name: &str) -> bool {
    parsed(text).is_some_and(|doc| doc.get(key).is_some_and(|it| it.get(name).is_some()))
}

pub fn set(text: &str, key: &str, name: &str, command: &str) -> Option<String> {
    if !text.trim().is_empty() {
        parsed(text)?;
    }
    let now = match holds(text, key, name) {
        true => {
            let now = field_set(text, key, name, "command", Value::from(command))?;
            match args(&now, key, name) == Some(vec!["mcp".to_string()]) {
                true => now,
                false => field_set(&now, key, name, "args", Value::from_iter(["mcp"]))?,
            }
        }
        false if parsed(text).is_some_and(|doc| doc.get(key).is_some_and(|it| !plain(it))) => {
            return None;
        }
        false => appended(text, key, name, command),
    };
    let sound = reads(&now, key, name).as_deref() == Some(command);
    (sound && rest(&now, key, name) == rest(text, key, name)).then_some(now)
}

pub fn repoint(text: &str, key: &str, name: &str, command: &str) -> Option<String> {
    reads(text, key, name)?;
    let now = spliced(text, key, name, "command", &Value::from(command))?;
    let sound = reads(&now, key, name).as_deref() == Some(command);
    (sound && rest(&now, key, name) == rest(text, key, name)).then_some(now)
}

pub fn unset(text: &str, key: &str, name: &str) -> Option<String> {
    if !holds(text, key, name) {
        return None;
    }
    let gone =
        |now: &String| !holds(now, key, name) && others(now, key, name) == others(text, key, name);
    let now = lines_cut(text, key, name).filter(gone).or_else(|| {
        table_removed(text, key, name)
            .filter(gone)
            .filter(|now| notes(now) == notes(text))
    })?;
    Some(match now.trim().is_empty() {
        true => String::new(),
        false => now,
    })
}

fn table_removed(text: &str, key: &str, name: &str) -> Option<String> {
    faithful(text, |doc| {
        let Some(under) = doc.get_mut(key).and_then(Item::as_table_like_mut) else {
            return;
        };
        under.remove(name);
        if under.is_empty() {
            doc.remove(key);
        }
    })
}

// Line by line the person's comments around the tables stay, which go with them in the library.
fn lines_cut(text: &str, key: &str, name: &str) -> Option<String> {
    let mut kept = String::with_capacity(text.len());
    let mut ours = false;
    let mut found = false;
    let mut notes = String::new();
    for line in text.split_inclusive('\n') {
        let said = line.trim_start();
        if said.starts_with('[') {
            kept.push_str(&std::mem::take(&mut notes));
            ours = heads(line, key, name);
            found |= ours;
        }
        match ours {
            false => kept.push_str(line),
            true if said.starts_with('#') || (!notes.is_empty() && said.is_empty()) => {
                notes.push_str(line);
            }
            true => notes.clear(),
        }
    }
    kept.push_str(&notes);
    found.then_some(kept)
}

fn heads(line: &str, key: &str, name: &str) -> bool {
    let head = line.split('#').next().unwrap_or_default().trim();
    head.strip_prefix('[')
        .and_then(|it| it.strip_suffix(']'))
        .is_some_and(|inner| {
            let mut parts = inner
                .split('.')
                .map(|part| part.trim().trim_matches(['"', '\'']));
            parts.next() == Some(key) && parts.next() == Some(name)
        })
}

fn parsed(text: &str) -> Option<::toml::Table> {
    text.strip_prefix(BOM).unwrap_or(text).parse().ok()
}

fn args(text: &str, key: &str, name: &str) -> Option<Vec<String>> {
    let doc = parsed(text)?;
    let all = doc.get(key)?.get(name)?.get("args")?.as_array()?.clone();
    all.into_iter()
        .map(|one| one.as_str().map(str::to_string))
        .collect()
}

fn notes(text: &str) -> Vec<&str> {
    let lines = text.lines().map(str::trim);
    lines.filter(|line| line.starts_with('#')).collect()
}

fn field_set(text: &str, key: &str, name: &str, field: &str, value: Value) -> Option<String> {
    spliced(text, key, name, field, &value).or_else(|| {
        edited(text, key, name, |entry| {
            entry.insert(field, Item::Value(value));
        })
    })
}

// Only the bytes of the value change, so however the rest is written it stays.
fn spliced(text: &str, key: &str, name: &str, field: &str, value: &Value) -> Option<String> {
    let body = text.strip_prefix(BOM).unwrap_or(text);
    let mark = text.len() - body.len();
    let doc = Document::parse(body).ok()?;
    let there = doc
        .get(key)
        .and_then(Item::as_table_like)?
        .get(name)
        .and_then(Item::as_table_like)?
        .get(field)?;
    let span = there.span()?;
    Some(format!(
        "{}{}{}",
        &text[..mark + span.start],
        value.to_string().trim(),
        &text[mark + span.end..]
    ))
}

fn plain(item: &::toml::Value) -> bool {
    item.is_table()
}

fn others(text: &str, key: &str, name: &str) -> Option<::toml::Table> {
    let mut doc = parsed(text)?;
    let under = doc.get_mut(key).and_then(::toml::Value::as_table_mut);
    if under.is_some_and(|it| {
        it.remove(name);
        it.is_empty()
    }) {
        doc.remove(key);
    }
    Some(doc)
}

fn rest(text: &str, key: &str, name: &str) -> Option<::toml::Table> {
    let mut doc = parsed(text)?;
    if let Some(under) = doc.get_mut(key).and_then(::toml::Value::as_table_mut) {
        if let Some(entry) = under.get_mut(name).and_then(::toml::Value::as_table_mut) {
            entry.remove("command");
            entry.remove("args");
        }
        let emptied = |it: &::toml::Value| it.as_table().is_some_and(::toml::Table::is_empty);
        if under.get(name).is_some_and(emptied) {
            under.remove(name);
        }
        if under.is_empty() {
            doc.remove(key);
        }
    }
    Some(doc)
}

fn appended(text: &str, key: &str, name: &str, command: &str) -> String {
    let line = match text.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push_str(line);
    }
    if !out.trim().is_empty() {
        out.push_str(line);
    }
    let said = ::toml::Value::from(command);
    out.push_str(&format!(
        "[{key}.{name}]{line}command = {said}{line}args = [\"mcp\"]{line}"
    ));
    out
}

fn edited(
    text: &str,
    key: &str,
    name: &str,
    change: impl FnOnce(&mut dyn TableLike),
) -> Option<String> {
    faithful(text, |doc| {
        let entry = doc
            .get_mut(key)
            .and_then(Item::as_table_like_mut)
            .and_then(|under| under.get_mut(name))
            .and_then(Item::as_table_like_mut);
        if let Some(entry) = entry {
            change(entry);
        }
    })
}

// A file this library would not give back byte for byte is not one to write through it.
fn faithful(text: &str, change: impl FnOnce(&mut DocumentMut)) -> Option<String> {
    let body = text.strip_prefix(BOM).unwrap_or(text);
    let windows = body.contains("\r\n");
    let mut plain = match windows {
        true => body.replace("\r\n", "\n"),
        false => body.to_string(),
    };
    if windows && plain.replace('\n', "\r\n") != body {
        return None;
    }
    let ended = plain.ends_with('\n');
    if !ended {
        plain.push('\n');
    }
    let mut doc: DocumentMut = plain.parse().ok()?;
    if doc.to_string() != plain {
        return None;
    }
    change(&mut doc);
    let mut now = doc.to_string();
    if !ended && now.ends_with('\n') {
        now.pop();
    }
    if windows {
        now = now.replace('\n', "\r\n");
    }
    Some(format!("{}{now}", &text[..text.len() - body.len()]))
}

#[cfg(test)]
#[path = "toml_test.rs"]
mod tests;
