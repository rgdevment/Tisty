use super::{Titled, Titles, spans};

pub(super) const CALLOUTS: &str = "callouts written the Obsidian way";
pub(super) const COMMENTS: &str = "comments written the Obsidian way";
pub(super) const ALIASES: &str = "links to a note under another name";
pub(super) const EMBEDS: &str = "files embedded the Obsidian way";
pub(super) const BREAKS: &str = "page breaks written the LaTeX way";
pub(super) const TITLED: &str = "notes named by title, now linked to the document that bears it";
pub(super) const TWINS: &str =
    "notes named by a title more than one document bears, left as references";
pub(super) const ABSENT: &str =
    "notes embedded by a title no document here bears, left as references";

pub(super) fn obsidian(body: &str, said: &mut Vec<&'static str>, titles: &Titles) -> String {
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
        if !aside && matches!(bare.trim(), r"\newpage" | r"\pagebreak" | r"\clearpage") {
            note(BREAKS);
            out.push_str("\n---\n");
            if line.ends_with('\n') {
                out.push('\n');
            }
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
            out.push_str(&worded(part, &mut aside, &mut note, titles));
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

fn worded(
    part: &str,
    aside: &mut bool,
    note: &mut impl FnMut(&'static str),
    titles: &Titles,
) -> String {
    let mut out = String::with_capacity(part.len());
    let mut rest = part;
    loop {
        let comment = rest.find("%%");
        let link = if *aside { None } else { rest.find("[[") };
        match (comment, link) {
            (Some(at), Some(open)) if open < at => {
                rest = linked(rest, open, &mut out, note, titles)
            }
            (Some(at), _) => {
                out.push_str(&rest[..at]);
                out.push_str(if *aside { "-->" } else { "<!--" });
                *aside = !*aside;
                note(COMMENTS);
                rest = &rest[at + 2..];
            }
            (None, Some(open)) => rest = linked(rest, open, &mut out, note, titles),
            (None, None) => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

// `folder/note#heading` names the note itself; an unfound name with an extension is a file.
fn titled(name: &str) -> &str {
    let bare = name.split('#').next().unwrap_or(name).trim();
    let bare = bare.rsplit('/').next().unwrap_or(bare);
    bare.strip_suffix(".md").unwrap_or(bare)
}

fn filed(title: &str) -> bool {
    title.rsplit_once('.').is_some_and(|(_, ext)| {
        (1..=5).contains(&ext.len()) && ext.chars().all(|c| c.is_ascii_alphanumeric())
    })
}

fn worded_as(label: &str) -> String {
    label
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

fn linked<'a>(
    rest: &'a str,
    open: usize,
    out: &mut String,
    note: &mut impl FnMut(&'static str),
    titles: &Titles,
) -> &'a str {
    let embed = open > 0 && rest[..open].ends_with('!');
    let start = if embed { open - 1 } else { open };
    let Some(shut) = rest[open + 2..].find("]]").map(|at| open + 2 + at) else {
        out.push_str(rest);
        return "";
    };
    let inner = &rest[open + 2..shut];
    let (name, alias) = match inner.split_once('|') {
        Some((name, alias)) => (name.trim(), Some(alias.trim())),
        None => (inner.trim(), None),
    };
    out.push_str(&rest[..start]);
    if name.is_empty() || name.contains(['[', '\n']) {
        out.push_str(&rest[start..shut + 2]);
        return &rest[shut + 2..];
    }
    let title = titled(name);
    let found = match title.is_empty() {
        true => Titled::None,
        false => titles.find(title),
    };
    match (embed, alias, found) {
        (_, _, Titled::One(id)) => {
            note(TITLED);
            let label = worded_as(alias.filter(|one| !one.is_empty()).unwrap_or(name));
            let bang = if embed { "!" } else { "" };
            out.push_str(&format!("{bang}[{label}](tisty:doc/{id})"));
        }
        (_, _, Titled::Many) => {
            note(TWINS);
            out.push_str(&format!("[[{name}]]"));
        }
        (true, _, Titled::None) if !title.is_empty() && !filed(title) => {
            note(ABSENT);
            out.push_str(&format!("[[{name}]]"));
        }
        (true, _, Titled::None) => {
            note(EMBEDS);
            out.push_str(&format!("![{name}](<{name}>)"));
        }
        (false, Some(_), Titled::None) => {
            note(ALIASES);
            out.push_str(&format!("[[{name}]]"));
        }
        (false, None, Titled::None) => out.push_str(&rest[start..shut + 2]),
    }
    &rest[shut + 2..]
}
