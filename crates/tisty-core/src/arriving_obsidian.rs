use super::spans;

pub(super) const CALLOUTS: &str = "callouts written the Obsidian way";
pub(super) const COMMENTS: &str = "comments written the Obsidian way";
pub(super) const ALIASES: &str = "links to a note under another name";
pub(super) const EMBEDS: &str = "files embedded the Obsidian way";

pub(super) fn obsidian(body: &str, said: &mut Vec<&'static str>) -> String {
    let mut note = |what: &'static str| {
        if !said.contains(&what) {
            said.push(what);
        }
    };
    let mut fenced = tisty_markdown::fences::fencing();
    let mut aside = false;
    let mut out = String::with_capacity(body.len());
    for line in body.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n');
        if !aside && fenced(bare) {
            out.push_str(line);
            continue;
        }
        let bare = match called(bare) {
            Some(said) => {
                note(CALLOUTS);
                said
            }
            None => bare.to_string(),
        };
        for (coded, part) in spans(&bare) {
            if coded {
                out.push_str(part);
                continue;
            }
            out.push_str(&worded(part, &mut aside, &mut note));
        }
        if line.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

// Obsidian has a dozen kinds and a fold mark; the language has five kinds, each the nearest in spirit.
fn called(line: &str) -> Option<String> {
    let lead = line.len() - line.trim_start().len();
    let rest = line[lead..].strip_prefix('>')?;
    let gap = rest.len() - rest.trim_start().len();
    let marker = rest[gap..].strip_prefix("[!")?;
    let (word, after) = marker.split_once(']')?;
    let folded = after.starts_with(['-', '+']);
    let after = after.strip_prefix(['-', '+']).unwrap_or(after);
    let kind = match word.to_ascii_lowercase().as_str() {
        "note" | "info" | "todo" | "abstract" | "summary" | "tldr" | "example" | "quote"
        | "cite" => "NOTE",
        "tip" | "hint" | "success" | "check" | "done" | "question" | "help" | "faq" => "TIP",
        "important" => "IMPORTANT",
        "warning" | "attention" => "WARNING",
        "caution" | "danger" | "error" | "failure" | "fail" | "missing" | "bug" => "CAUTION",
        _ => return None,
    };
    let known = ["note", "tip", "important", "warning", "caution"]
        .contains(&word.to_ascii_lowercase().as_str());
    if known && !folded {
        return None;
    }
    Some(format!(
        "{}>{}[!{kind}]{after}",
        &line[..lead],
        &rest[..gap]
    ))
}

fn worded(part: &str, aside: &mut bool, note: &mut impl FnMut(&'static str)) -> String {
    let mut out = String::with_capacity(part.len());
    let mut rest = part;
    loop {
        let comment = rest.find("%%");
        let link = if *aside { None } else { rest.find("[[") };
        match (comment, link) {
            (Some(at), Some(open)) if open < at => rest = linked(rest, open, &mut out, note),
            (Some(at), _) => {
                out.push_str(&rest[..at]);
                out.push_str(if *aside { "-->" } else { "<!--" });
                *aside = !*aside;
                note(COMMENTS);
                rest = &rest[at + 2..];
            }
            (None, Some(open)) => rest = linked(rest, open, &mut out, note),
            (None, None) => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

fn linked<'a>(
    rest: &'a str,
    open: usize,
    out: &mut String,
    note: &mut impl FnMut(&'static str),
) -> &'a str {
    let embed = open > 0 && rest[..open].ends_with('!');
    let start = if embed { open - 1 } else { open };
    let Some(shut) = rest[open + 2..].find("]]").map(|at| open + 2 + at) else {
        out.push_str(rest);
        return "";
    };
    let inner = &rest[open + 2..shut];
    let (name, alias) = match inner.split_once('|') {
        Some((name, alias)) => (name.trim(), Some(alias)),
        None => (inner.trim(), None),
    };
    out.push_str(&rest[..start]);
    match (embed, alias) {
        _ if name.is_empty() || name.contains(['[', '\n']) => out.push_str(&rest[start..shut + 2]),
        (true, _) => {
            note(EMBEDS);
            out.push_str(&format!("![{name}](<{name}>)"));
        }
        (false, Some(_)) => {
            note(ALIASES);
            out.push_str(&format!("[[{name}]]"));
        }
        (false, None) => out.push_str(&rest[start..shut + 2]),
    }
    &rest[shut + 2..]
}
