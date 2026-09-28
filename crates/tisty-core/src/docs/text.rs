/// Steps over fenced blocks line by line, for anything that reads a body and must not read code.
pub fn fencing() -> impl FnMut(&str) -> bool {
    let mut fence = Fencing::default();
    move |line| fence.inside(line)
}

pub fn ends_fenced(text: &str) -> bool {
    let mut fence = Fencing::default();
    for line in text.lines() {
        fence.inside(line);
    }
    fence.open.is_some()
}

pub(crate) fn fenced_spans(text: &str) -> Vec<(usize, usize)> {
    let mut fence = Fencing::default();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        if fence.inside(line.trim_end_matches(['\n', '\r'])) {
            match out.last_mut() {
                Some(last) if last.1 == at => last.1 = end,
                _ => out.push((at, end)),
            }
        }
        at = end;
    }
    out
}

/// A fence opens on three or more of one marker and closes on the same, at least as long.
/// The quote prefix comes off first, or a fence written inside a quote is never seen.
#[derive(Default)]
pub(super) struct Fencing {
    pub(super) open: Option<(char, usize, usize, usize)>,
    pub(super) base: usize,
    pub(super) told: bool,
}

pub(super) fn bullet(said: &str) -> Option<usize> {
    let bytes = said.as_bytes();
    let mut at = 0;
    if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
        at = 1;
    } else {
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        if at == 0 || at > 9 || !matches!(bytes.get(at), Some(b'.' | b')')) {
            return None;
        }
        at += 1;
    }
    let gap = said[at..]
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .count();
    (gap > 0).then_some(at + gap)
}

pub(super) fn listed(base: usize, wide: usize, said: &str) -> usize {
    if said.is_empty() {
        return base;
    }
    match bullet(said) {
        Some(after) if wide <= base => wide + after,
        _ if wide < base => 0,
        _ => base,
    }
}

pub(crate) fn nameless(said: &str) -> String {
    let mut from = 0;
    while let Some(found) = said[from..].find("title=\"") {
        let at = from + found;
        from = at + 7;
        if said[..at]
            .chars()
            .next_back()
            .is_some_and(|one| one.is_alphanumeric() || one == '_')
        {
            continue;
        }
        let rest = &said[from..];
        let bytes = rest.as_bytes();
        let mut over = 0;
        let end = loop {
            match bytes.get(over) {
                None => return said.to_string(),
                Some(b'\\') => over += 2,
                Some(b'"') => break over,
                _ => over += 1,
            }
        };
        return format!("{} {}", &said[..at], &rest[end + 1..]);
    }
    said.to_string()
}

fn spacing(said: &str) -> usize {
    said.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

pub(super) fn quoted(line: &str) -> (usize, usize, &str) {
    let mut said = line;
    let mut deep = 0;
    let mut wide = spacing(said);

    while wide < 4 {
        let Some(rest) = said.trim_start().strip_prefix('>') else {
            break;
        };
        said = rest.strip_prefix(' ').unwrap_or(rest);
        deep += 1;
        wide = spacing(said);
    }
    (deep, wide, said.trim())
}

pub(super) fn quoteless(line: &str) -> &str {
    quoted(line).2
}

pub fn titled(body: &str) -> String {
    let body = body.trim_start_matches('\u{feff}');
    let mut said: Vec<&str> = body.lines().collect();
    if let Some(start) = said.iter().position(|one| !one.trim().is_empty())
        && said[start].trim() == "---"
        && let Some(shuts) = said
            .iter()
            .skip(start + 1)
            .position(|one| one.trim() == "---")
    {
        said.drain(..start + shuts + 2);
    }
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
    let mut seen = 0;
    let mut out: Vec<String> = Vec::new();
    let mut done = false;
    let mut fence = Fencing::default();

    for line in bare.lines() {
        let trimmed = line.trim();
        if done || trimmed.is_empty() {
            out.push(line.to_string());
            continue;
        }
        if trimmed == "---"
            && seen == 0
            && bare.lines().filter(|one| one.trim() == "---").count() > 1
        {
            seen = 1;
            out.push(line.to_string());
            continue;
        }
        if seen == 1 {
            if trimmed == "---" {
                seen = 2;
            }
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

impl Fencing {
    pub(super) fn inside(&mut self, line: &str) -> bool {
        let (deep, wide, said) = quoted(line);
        if self.open.is_none() {
            self.base = listed(self.base, wide, said);
        }
        let (wide, said) = match bullet(said) {
            Some(after) => (wide + after, &said[after..]),
            None => (wide, said),
        };
        self.told = false;
        let mut marker = None;
        for mark in ['`', '~'] {
            let many = said.chars().take_while(|c| *c == mark).count();
            if many < 3 || wide >= self.base + 4 {
                continue;
            }
            let after = &said[many.min(said.len())..];
            if mark == '`' && after.contains('`') {
                self.told = self.open.is_none();
                continue;
            }
            marker = Some((mark, many));
            break;
        }

        if let Some((open, was, held, room)) = self.open {
            if said.is_empty() {
                return true;
            }
            if deep >= held && wide >= room {
                if let Some((mark, many)) = marker
                    && mark == open
                    && many >= was
                    && deep == held
                {
                    self.open = None;
                }
                return true;
            }
            self.open = None;
        }

        match marker {
            Some((mark, many)) => {
                self.told = nameless(&said[many..]).split_whitespace().count() > 1;
                self.open = Some((mark, many, deep, wide));
                true
            }
            None => false,
        }
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
