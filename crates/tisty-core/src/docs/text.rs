pub(super) use tisty_markdown::fences::{Fencing, quoteless};
pub use tisty_markdown::fences::{ends_fenced, fencing};

const NAMED_AT_MOST: usize = 200;

pub fn titled(body: &str) -> String {
    let body = body.trim_start_matches('\u{feff}');
    if let Some(named) = tisty_markdown::front::title(body) {
        let named: String = named.trim().chars().take(NAMED_AT_MOST).collect();
        return crate::text::plainly(&named);
    }
    let past = tisty_markdown::front::span(body).map_or(0, |at| at.end);
    let said: Vec<&str> = body[past..].lines().collect();
    let mut fence = Fencing::default();
    let first = said
        .iter()
        .find_map(|one| {
            if fence.inside(one) {
                return None;
            }
            let flat = one.trim();
            if flat.is_empty() {
                return None;
            }
            let opened = quoteless(one).trim_start_matches('#').trim_start();
            let said = match flat.starts_with('>') {
                true => crate::refs::alerted(opened).unwrap_or(opened),
                false => opened,
            };
            (!wordless(said)).then_some(said)
        })
        .unwrap_or_default();
    crate::text::plainly(unspanned(first).trim())
}

pub fn marked(body: &str, said: &str) -> String {
    let bare = body.trim_start_matches('\u{feff}');
    let past = tisty_markdown::front::span(bare).map_or(0, |at| at.end);
    let (front, rest) = bare.split_at(past);
    match retitled(front, said) {
        Some(front) => format!("{front}{rest}"),
        None => format!("{front}{}", headed(rest, said)),
    }
}

// The mark goes where the title comes from: a `title:` names the document before any heading does.
fn retitled(front: &str, said: &str) -> Option<String> {
    tisty_markdown::front::title(front)?;
    let mut out = String::with_capacity(front.len() + said.len() + 3);
    let mut done = false;
    for line in front.split_inclusive('\n') {
        let flat = line.trim_end();
        let keyed = flat
            .split_once(':')
            .filter(|(key, _)| !done && key.eq_ignore_ascii_case("title"));
        let Some((key, value)) = keyed else {
            out.push_str(line);
            continue;
        };
        let value = value.trim();
        let quoted = ['"', '\'']
            .into_iter()
            .find(|one| value.len() > 1 && value.starts_with(*one) && value.ends_with(*one));
        let value = match quoted {
            Some(one) => format!("{one}{} ({said}){one}", &value[1..value.len() - 1]),
            None => format!("{value} ({said})"),
        };
        out.push_str(&format!("{key}: {value}{}", &line[flat.len()..]));
        done = true;
    }
    done.then_some(out)
}

fn headed(bare: &str, said: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut done = false;
    let mut fence = Fencing::default();

    for line in bare.lines() {
        let trimmed = line.trim();
        if done || trimmed.is_empty() {
            out.push(line.to_string());
            continue;
        }
        if fence.inside(line) || wordless(trimmed) {
            out.push(line.to_string());
            continue;
        }
        let after = quoteless(line);
        // A row of a table, and a marker with nothing after it, are lines a name would break.
        let bare_line = trimmed.starts_with('|')
            || matches!(crate::refs::alerted(after), Some(rest) if rest.is_empty());
        match bare_line {
            false => {
                out.push(format!("{} ({said})", line.trim_end()));
                done = true;
            }
            true => out.push(line.to_string()),
        }
    }

    if !done {
        return format!("# {said}\n\n{bare}");
    }
    let mut whole = out.join("\n");
    if bare.ends_with('\n') {
        whole.push('\n');
    }
    whole
}

pub fn settled(body: &str) -> String {
    if body.is_empty() || body.ends_with('\n') {
        return body.to_string();
    }
    format!("{body}\n")
}

/// A heading inside a fence is code, not a title: skipping the fences is what keeps a shell
/// prompt from becoming a section.
pub fn headings(body: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (n, line) in body.lines().enumerate() {
        let bare = line.trim_start();
        if bare.starts_with("```") || bare.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let deep = bare.chars().take_while(|one| *one == '#').count();
        if deep == 0 || deep > 3 || !bare[deep..].starts_with(' ') {
            continue;
        }
        out.push((n + 1, deep, bare[deep + 1..].trim().to_string()));
    }
    out
}

/// Where a section ends: the next heading no deeper than its own, or the end of the document.
/// The blank lines before the next heading separate the two, so they belong to neither.
pub fn section_lines(body: &str, at: usize) -> Option<(usize, usize)> {
    let all = headings(body);
    let lines: Vec<&str> = body.lines().collect();
    section_ends(&all, &lines, at)
}

fn section_ends(
    all: &[(usize, usize, String)],
    lines: &[&str],
    at: usize,
) -> Option<(usize, usize)> {
    let (line, deep, _) = all.get(at)?;
    let mut last = all
        .iter()
        .skip(at + 1)
        .find(|(_, other, _)| other <= deep)
        .map(|(next, _, _)| next - 1)
        .unwrap_or(lines.len());
    while last > *line && lines.get(last - 1).is_some_and(|one| one.trim().is_empty()) {
        last -= 1;
    }
    Some((*line, last))
}

pub fn outlined(body: &str) -> Vec<Heading> {
    let all = headings(body);
    let lines: Vec<&str> = body.lines().collect();
    all.iter()
        .enumerate()
        .map(|(at, (line, level, title))| {
            let (_, to) = section_ends(&all, &lines, at).unwrap_or((*line, *line));
            let chars = lines[line - 1..to]
                .iter()
                .map(|one| one.chars().count() + 1)
                .sum();
            Heading {
                at,
                line: *line,
                level: *level,
                title: title.clone(),
                to,
                chars,
            }
        })
        .collect()
}

/// Cut where the lines really end, so a body that ended in a newline still does. A run that ends
/// before it begins, or begins past the last line, is nothing at all: callers splice a head and a
/// tail around an edit, and a tail that answered with the whole body would duplicate it.
pub fn lines_between(body: &str, from: usize, to: usize) -> String {
    if to < from {
        return String::new();
    }
    let mut start = None;
    let mut end = body.len();
    let mut at = 0usize;
    for (n, line) in body.split_inclusive('\n').enumerate() {
        if n + 1 == from {
            start = Some(at);
        }
        at += line.len();
        if n + 1 == to {
            end = at;
            break;
        }
    }
    let Some(start) = start else {
        return String::new();
    };
    body.get(start..end).unwrap_or_default().to_string()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Heading {
    pub at: usize,
    pub line: usize,
    pub level: usize,
    pub title: String,
    #[serde(default)]
    pub to: usize,
    #[serde(default)]
    pub chars: usize,
}

/// A picture is drawn where a link is followed, and an agent choosing a document wants to know
/// which of the two it is walking into.
pub(super) fn pointed_at(body: &str) -> (usize, usize) {
    let (mut drawn, mut followed) = (0, 0);
    let bytes = body.as_bytes();
    for (at, _) in body.match_indices('[') {
        let Some(shut) = crate::refs::shuts(&body[at + 1..]) else {
            continue;
        };
        if bytes.get(at + 1 + shut + 1) != Some(&b'(') {
            continue;
        }
        match at > 0 && bytes[at - 1] == b'!' {
            true => drawn += 1,
            false => followed += 1,
        }
    }
    (drawn, followed)
}

/// The tags a body carries first, since somebody meant those; then the words it leans on, which
/// nobody meant but which say what it is about all the same.
pub(super) fn standing_out(body: &str) -> Vec<String> {
    let mut out: Vec<String> = crate::tagging::tags_in(body)
        .iter()
        .map(|one| one.as_str().to_string())
        .collect();

    let mut times: std::collections::HashMap<String, usize> = Default::default();
    for word in crate::text::terms(body) {
        if word.chars().count() < A_WORD_AT_LEAST {
            continue;
        }
        *times.entry(word).or_default() += 1;
    }
    let mut said: Vec<(String, usize)> = times.into_iter().filter(|(_, n)| *n > 1).collect();
    said.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (word, _) in said {
        if out.len() >= KEYWORDS_AT_MOST {
            break;
        }
        if !out.contains(&word) {
            out.push(word);
        }
    }
    out.truncate(KEYWORDS_AT_MOST);
    out
}

/// A document imported from Windows keeps its line endings, and nobody types those.
pub(super) fn as_written(was: &str, old: &str, new: &str) -> (String, String) {
    if was.contains(old) || !was.contains("\r\n") {
        return (old.to_string(), new.to_string());
    }
    let crlf = |said: &str| said.replace("\r\n", "\n").replace('\n', "\r\n");
    (crlf(old), crlf(new))
}

pub(super) fn unpictured(body: &str, at: &str) -> String {
    let shut = format!("](<{at}>)");
    let mut said = String::with_capacity(body.len());
    let mut from = 0;
    while let Some(found) = body[from..].find(&shut).map(|n| from + n) {
        let opened = began(&body[..found]);
        let cut = match opened {
            Some(open) if body[..open].ends_with('!') => open - 1,
            _ => found,
        };
        said.push_str(&body[from..cut]);
        if cut != found {
            said.push_str(&body[cut + 1..found]);
        }
        said.push_str(&shut);
        from = found + shut.len();
    }
    said.push_str(&body[from..]);
    said
}

fn began(before: &str) -> Option<usize> {
    let mut escaped = false;
    let mut open = None;
    for (at, c) in before.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '[' => open = Some(at),
            _ => {}
        }
    }
    open
}

pub fn spelled(said: &str) -> String {
    let flat: String = said
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let flat = flat.trim().replace(' ', "-");
    let flat: String = flat.chars().take(60).collect();
    let flat = flat.trim_matches('-').to_string();
    if flat.is_empty() || crate::attach::reserved(&flat) {
        "documento".into()
    } else {
        flat
    }
}

const KEYWORDS_AT_MOST: usize = 12;

const A_WORD_AT_LEAST: usize = 4;

pub(super) fn unspanned(line: &str) -> String {
    if !line.contains("<span data-ico=") {
        return line.to_string();
    }
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find("<span data-ico=") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        let Some(shut) = after.find('>') else {
            return out + after;
        };
        let inner = &after[shut + 1..];
        match inner.find("</span>") {
            Some(ends) => {
                out.push_str(&inner[..ends]);
                rest = &inner[ends + "</span>".len()..];
            }
            None => return out + inner,
        }
    }
    out.push_str(rest);
    out
}

pub(super) fn wordless(said: &str) -> bool {
    said.is_empty()
        || !said
            .chars()
            .any(|c| c.is_alphanumeric() || matches!(c, '¿' | '?' | '¡' | '!'))
}
