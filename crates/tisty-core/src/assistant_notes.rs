pub const ASSISTANT_NOTES: [&str; 2] = ["## Assistant notes", "## Notas del asistente"];

// An assistant's words go under their own heading, so they never mix with what the person wrote.
pub fn noted(body: &str, note: &str, spanish: bool) -> String {
    let line = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let note = kept_inside(note.trim());
    let note = note.as_str();
    let Some((_, words)) = crate::closing::held(body, &ASSISTANT_NOTES) else {
        let kept = body.trim_end_matches(['\n', '\r']);
        let heading = ASSISTANT_NOTES[usize::from(spanish)];
        let gap = if kept.is_empty() {
            String::new()
        } else {
            format!("{line}{line}")
        };
        return format!("{kept}{gap}{heading}{line}{line}{note}{line}");
    };
    let head = body[..words.end].trim_end_matches(['\n', '\r']);
    let rest = &body[words.end..];
    let tail = if rest.is_empty() { "" } else { line };
    format!("{head}{line}{line}{note}{line}{tail}{rest}")
}

pub fn rewritten(body: &str, note: &str, spanish: bool) -> String {
    let Some((_, words)) = crate::closing::held(body, &ASSISTANT_NOTES) else {
        return noted(body, note, spanish);
    };
    let line = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let note = kept_inside(note.trim()).replace('\n', line);
    let head = body[..words.start].trim_end_matches(['\n', '\r']);
    let rest = &body[words.end..];
    let tail = if rest.is_empty() { "" } else { line };
    format!("{head}{line}{line}{note}{line}{tail}{rest}")
}

pub fn section(body: &str) -> Option<&str> {
    crate::closing::held(body, &ASSISTANT_NOTES).map(|(_, words)| &body[words])
}

pub fn put_back(body: &str, words: &str) -> Option<String> {
    let (_, now) = crate::closing::held(body, &ASSISTANT_NOTES)?;
    Some(format!("{}{words}{}", &body[..now.start], &body[now.end..]))
}

pub fn signed(words: &str) -> (Vec<String>, bool) {
    let mut who = Vec::new();
    let mut before = false;
    for line in words.lines() {
        match signature(line) {
            Some(by) => who.push(by.to_string()),
            None if who.is_empty() && !line.trim().is_empty() => before = true,
            None => {}
        }
    }
    (who, before)
}

fn signature(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("**")?;
    let (day, by) = rest.split_once("** · ")?;
    let stamped = day.len() == 16
        && day.as_bytes()[4] == b'-'
        && day.as_bytes()[10] == b' '
        && day.as_bytes()[13] == b':';
    (stamped && !by.trim().is_empty()).then_some(by.trim())
}

pub fn outside(body: &str) -> String {
    match crate::closing::held(body, &ASSISTANT_NOTES) {
        Some((head, words)) => format!("{}{}", &body[..head.start], &body[words.end..]),
        None => body.to_string(),
    }
}

// Its headings sit under the section and a fence left open is closed, so nothing it says leaves it.
fn kept_inside(note: &str) -> String {
    let deeper = crate::task_docs::deeper(note);
    let fences: Vec<&str> = deeper
        .lines()
        .map(str::trim_start)
        .filter(|one| one.starts_with("```") || one.starts_with("~~~"))
        .collect();
    match (fences.len() % 2, fences.last()) {
        (1, Some(open)) => format!("{deeper}\n{}", &open[..3]),
        _ => deeper,
    }
}

#[cfg(test)]
#[path = "assistant_notes_test.rs"]
mod tests;
