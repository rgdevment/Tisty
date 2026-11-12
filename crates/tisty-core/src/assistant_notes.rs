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

// What came after the digest stays where it is, so ticks and notes since are never undone with it.
pub fn swapped(body: &str, heading: &str, put: &str) -> Option<String> {
    let (_, span) = crate::closing::held(body, &ASSISTANT_NOTES)?;
    let words = &body[span.clone()];
    let opened = words.trim_start();
    let lead = &words[..words.len() - opened.len()];
    let heading = Some(heading.trim()).filter(|one| !one.is_empty())?;
    let rest = opened.strip_prefix(heading)?;
    Some(format!(
        "{}{lead}{}{rest}{}",
        &body[..span.start],
        put.trim(),
        &body[span.end..]
    ))
}

pub fn signed(words: &str) -> Vec<String> {
    words
        .lines()
        .filter_map(signature)
        .map(str::to_string)
        .collect()
}

/// Lines no assistant wrote: new beside what an assistant last `left`, unless under an entry it signed since.
pub fn foreign(words: &str, left: Option<&str>) -> Vec<String> {
    let known: std::collections::BTreeSet<&str> = left
        .map(|was| was.lines().map(str::trim).collect())
        .unwrap_or_default();
    let mut theirs = Vec::new();
    let mut signed_yet = false;
    let mut new_entry = false;
    for line in words.lines() {
        let said = line.trim();
        if signature(line).is_some() {
            signed_yet = true;
            new_entry = !known.contains(said);
            continue;
        }
        let unknown = match left {
            Some(_) => !new_entry && !known.contains(said),
            None => !signed_yet,
        };
        if unknown && !said.is_empty() {
            theirs.push(said.to_string());
        }
    }
    theirs
}

// What the person wrote before this write stays theirs, so a later digest still finds it.
pub fn left_by_assistants(was: &str, now: &str, left: Option<&str>) -> Option<String> {
    let theirs = section(was)
        .map(|words| foreign(words, left))
        .unwrap_or_default();
    let words = section(now)?;
    Some(
        words
            .lines()
            .filter(|line| !theirs.iter().any(|one| one == line.trim()))
            .collect::<Vec<_>>()
            .join("\n"),
    )
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
