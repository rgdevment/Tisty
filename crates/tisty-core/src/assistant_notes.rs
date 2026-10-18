pub const ASSISTANT_NOTES: [&str; 2] = ["## Assistant notes", "## Notas del asistente"];

// An assistant's words go under their own heading, so they never mix with what the person wrote.
pub fn noted(body: &str, note: &str, spanish: bool) -> String {
    let line = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let note = note.trim();
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

#[cfg(test)]
#[path = "assistant_notes_test.rs"]
mod tests;
