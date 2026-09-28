use crate::model::Tag;

/// A hash pinned to a letter or a digit. `# Heading` carries a space and stays a heading, a colour
/// or a fragment sits behind something that is not a separator, and a fenced block is skipped
/// whole — the same fencing the title reader steps over.
/// Past this a body is not being tagged, it is carrying something else — a stylesheet pasted
/// outside a fence reads every colour as a tag. The log is append-only: a line of thousands of
/// them is written once and kept forever, on every machine.
pub const AT_MOST: usize = 64;

pub fn worth_keeping(tags: &[Tag]) -> Vec<Tag> {
    tags.iter()
        .filter(|one| one.worth_reading())
        .cloned()
        .collect()
}

pub fn tags_in(body: &str) -> Vec<Tag> {
    let mut found: Vec<Tag> = Vec::new();
    let mut seen: std::collections::HashSet<Tag> = std::collections::HashSet::new();
    let mut fenced = crate::docs::fencing();
    for line in body.lines() {
        if fenced(line) {
            continue;
        }
        // Composed first: text pasted from macOS carries «ñ» as two code points, and a reader
        // that stops at the first mark would file #diseño under «disen».
        for tag in tags_on(&crate::text::composed(line)) {
            if seen.insert(tag.clone()) {
                found.push(tag);
            }
            if found.len() == AT_MOST {
                return found;
            }
        }
    }
    found
}

fn tags_on(line: &str) -> Vec<Tag> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    // Only a backtick that closes marks code. One on its own is prose — a stray accent in a
    // sentence must not swallow every tag written after it.
    let mut code = false;
    let mut at = 0;
    while at < line.len() {
        if bytes[at] == b'`' {
            code = !code && line[at + 1..].contains('`');
            at += 1;
            continue;
        }
        if bytes[at] != b'#' || code {
            at += 1;
            continue;
        }
        let before = line[..at].chars().next_back();
        // Anything but a separator before the hash means it belongs to what came first: a colour
        // in `bg-#fff`, the fragment of an address, a word someone hyphenated.
        if before.is_some_and(|one| one.is_alphanumeric() || "#/-_.:".contains(one)) {
            at += 1;
            continue;
        }
        let rest = &line[at + 1..];
        // A letter or a digit against the hash, or it is not a tag: `#-` and `#_` are how code
        // and web addresses are written, not how anybody labels their own work.
        if !rest.chars().next().is_some_and(char::is_alphanumeric) {
            at += 1;
            continue;
        }
        let word: String = rest
            .chars()
            .take_while(|one| one.is_alphanumeric() || *one == '-' || *one == '_')
            .collect();
        if let Ok(tag) = Tag::new(&word)
            && tag.worth_reading()
        {
            found.push(tag);
        }
        at += 1 + word.len();
    }
    found
}

#[cfg(test)]
#[path = "tagging_test.rs"]
mod tests;
