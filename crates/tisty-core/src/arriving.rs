#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Tidied {
    pub body: String,
    pub changed: Vec<&'static str>,
}

const TAGS: [(&str, &str, &str); 8] = [
    ("b", "**", "**"),
    ("strong", "**", "**"),
    ("i", "*", "*"),
    ("em", "*", "*"),
    ("code", "`", "`"),
    ("s", "~~", "~~"),
    ("del", "~~", "~~"),
    ("strike", "~~", "~~"),
];

const ENTITIES: [(&str, &str); 8] = [
    ("&nbsp;", " "),
    ("&amp;", "&"),
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&#39;", "'"),
    ("&apos;", "'"),
    ("&hellip;", "…"),
];

pub fn tidied(body: &str) -> Tidied {
    if crate::docs::survives(body).is_ok() {
        return Tidied {
            body: body.to_string(),
            changed: Vec::new(),
        };
    }
    let whole_again = body;
    let mut said: Vec<&'static str> = Vec::new();
    let mut note = |what: &'static str, changed: &mut Vec<&'static str>| {
        if !changed.contains(&what) {
            changed.push(what);
        }
    };

    let body = body.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let (body, cut) = unfronted(&body);
    if cut {
        note("front matter", &mut said);
    }
    let (body, less) = uncommented(&body);
    if less {
        note("HTML comments", &mut said);
    }

    let mut out: Vec<String> = Vec::new();
    let mut fence: Option<String> = None;
    let mut refs: Vec<(String, String)> = Vec::new();

    for line in body.lines() {
        if let Some(open) = &fence {
            if line.trim_start().starts_with(open.as_str()) {
                let shut = line.trim_start().to_string();
                if shut != line {
                    note("a fence written in from the margin", &mut said);
                }
                fence = None;
                out.push(shut);
                continue;
            }
            out.push(line.to_string());
            continue;
        }
        if let Some(open) = opens(line) {
            let (kept, told) = languaged(line, &open);
            if told {
                note("what a fence said after its language", &mut said);
            }
            let bare = kept.trim_start().to_string();
            if bare != kept {
                note("a fence written in from the margin", &mut said);
            }
            fence = Some(open);
            out.push(bare);
            continue;
        }
        if let Some(escaped) = unblocked(line) {
            note("a list item that opened on a block", &mut said);
            out.push(escaped);
            continue;
        }
        if let Some((name, at)) = defined(line) {
            refs.push((name, at));
            note("links written by reference", &mut said);
            continue;
        }
        out.push(line.to_string());
    }

    let mut whole = out.join("\n");
    for (name, at) in &refs {
        whole = whole
            .split_inclusive('\n')
            .map(|line| {
                let bare = line.trim_end_matches('\n');
                let ends = &line[bare.len()..];
                let said: String = spans(bare)
                    .into_iter()
                    .map(|(coded, part)| match coded {
                        true => part.to_string(),
                        false => inlined(part, name, at),
                    })
                    .collect();
                format!("{said}{ends}")
            })
            .collect();
    }

    let mut walked = String::with_capacity(whole.len());
    let mut fenced: Option<String> = None;
    for line in whole.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n');
        if let Some(open) = &fenced {
            if bare.trim_start().starts_with(open.as_str()) {
                fenced = None;
            }
            walked.push_str(line);
            continue;
        }
        if let Some(open) = opens(bare) {
            fenced = Some(open);
            walked.push_str(line);
            continue;
        }
        walked.push_str(&plainer(bare, &mut said, &mut note));
        if line.ends_with('\n') {
            walked.push('\n');
        }
    }

    let (walked, moved) = mathed(&walked);
    if moved {
        note("maths written between dollars", &mut said);
    }

    match said.is_empty() {
        true => Tidied {
            body: whole_again.to_string(),
            changed: said,
        },
        false => Tidied {
            body: spaced(&walked),
            changed: said,
        },
    }
}

fn plainer(
    line: &str,
    said: &mut Vec<&'static str>,
    note: &mut impl FnMut(&'static str, &mut Vec<&'static str>),
) -> String {
    spans(line)
        .into_iter()
        .map(|(coded, part)| match coded {
            true => part.to_string(),
            false => bared(part, said, note),
        })
        .collect()
}

fn spans(line: &str) -> Vec<(bool, &str)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'`' {
            at += 1;
            continue;
        }
        let run = bytes[at..].iter().take_while(|one| **one == b'`').count();
        let mut walk = at + run;
        let shut = loop {
            match line[walk..].find(&"`".repeat(run)) {
                None => break None,
                Some(found) => {
                    let start = walk + found;
                    let wide = bytes[start..]
                        .iter()
                        .take_while(|one| **one == b'`')
                        .count();
                    if wide == run {
                        break Some(start + run);
                    }
                    walk = start + wide;
                }
            }
        };
        match shut {
            None => at += run,
            Some(ends) => {
                if from < at {
                    out.push((false, &line[from..at]));
                }
                out.push((true, &line[at..ends]));
                from = ends;
                at = ends;
            }
        }
    }
    if from < line.len() {
        out.push((false, &line[from..]));
    }
    out
}

fn bared(
    line: &str,
    said: &mut Vec<&'static str>,
    note: &mut impl FnMut(&'static str, &mut Vec<&'static str>),
) -> String {
    let mut out = line.to_string();
    if out.contains("<br") {
        out = drop_of(&out, "<br");
        note("line breaks written as tags", said);
    }
    for (tag, open, close) in TAGS {
        let (found, changed) = paired(&out, tag, open, close);
        if changed {
            note("HTML that markdown can say", said);
        }
        out = found;
    }
    if out.contains("<!--") {
        out = commentless(&out);
        note("HTML comments", said);
    }
    if ENTITIES.iter().any(|(one, _)| out.contains(one)) {
        note("HTML entities", said);
        for _ in 0..8 {
            let was = out.clone();
            for (one, plain) in ENTITIES {
                out = out.replace(one, plain);
            }
            if out == was {
                break;
            }
        }
    }
    if tagged(&out) {
        out = tagless(&out);
        note("HTML with nothing markdown can say", said);
    }
    if tagged(&out) && out.matches('`').count() % 2 == 1 {
        out.push('`');
        note("a run of code left open", said);
    }
    out
}

fn unblocked(line: &str) -> Option<String> {
    let said = line.trim_start();
    let wide = line.len() - said.len();
    if wide >= 4 {
        return None;
    }
    let after = bulleted(said)?;
    let rest = &said[after..];
    let mark = rest.chars().next()?;
    if !matches!(mark, '#' | '>' | '-' | '*' | '+') {
        return None;
    }
    if matches!(mark, '-' | '*' | '+') && bulleted(rest).is_none() {
        return None;
    }
    if mark == '#' {
        let hashes = rest.chars().take_while(|one| *one == '#').count();
        let next = rest[hashes..].chars().next();
        if !(1..=6).contains(&hashes) || !matches!(next, None | Some(' ') | Some('\t')) {
            return None;
        }
    }
    Some(format!(
        "{}{}\\{}",
        &line[..wide],
        &said[..after],
        &said[after..]
    ))
}

fn bulleted(said: &str) -> Option<usize> {
    let bytes = said.as_bytes();
    let mut at = 0;
    if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
        at = 1;
    } else {
        while at < bytes.len() && bytes[at].is_ascii_digit() {
            at += 1;
        }
        if at == 0 || at > 9 || !matches!(bytes.get(at), Some(b'.' | b')')) {
            return None;
        }
        at += 1;
    }
    let gap = said[at..].len() - said[at..].trim_start_matches([' ', '\t']).len();
    match gap {
        0 => None,
        _ => Some(at + gap),
    }
}

fn opens(line: &str) -> Option<String> {
    let said = line.trim_start();
    for mark in ['`', '~'] {
        let run = said.chars().take_while(|one| *one == mark).count();
        if run >= 3 {
            return Some(mark.to_string().repeat(run));
        }
    }
    None
}

fn languaged(line: &str, open: &str) -> (String, bool) {
    let said = line.trim_start();
    let rest = said[open.len()..].trim();
    let mut words = rest.split_whitespace();
    let first = words.next().unwrap_or_default();
    let told = words.next().is_some();
    let lead = &line[..line.len() - said.len()];
    (format!("{lead}{open}{first}"), told)
}

fn unfronted(body: &str) -> (String, bool) {
    let mut lines = body.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (body.to_string(), false);
    }
    let mut inside: Vec<&str> = Vec::new();
    let mut kept: Vec<&str> = Vec::new();
    let mut shut = false;
    for line in lines {
        if !shut {
            if line.trim().is_empty() {
                return (body.to_string(), false);
            }
            if matches!(line.trim(), "---" | "...") {
                shut = true;
                continue;
            }
            inside.push(line);
            continue;
        }
        kept.push(line);
    }
    let keyed = inside
        .iter()
        .any(|one| keyword(one) && !one.starts_with(' ') && !one.starts_with('-'));
    match shut && keyed {
        true => (kept.join("\n").trim_start().to_string(), true),
        false => (body.to_string(), false),
    }
}

fn keyword(line: &str) -> bool {
    let Some((name, _)) = line.split_once(':') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|one| one.is_alphanumeric() || matches!(one, '_' | '-' | '.'))
}

fn defined(line: &str) -> Option<(String, String)> {
    let said = line.trim();
    let rest = said.strip_prefix('[')?;
    let shut = rest.find("]:")?;
    let name = &rest[..shut];
    if name.is_empty() || name.starts_with('^') {
        return None;
    }
    let at = rest[shut + 2..].trim();
    let at = at.split_whitespace().next().unwrap_or_default();
    match at.is_empty() {
        true => None,
        false => Some((name.to_lowercase(), at.trim_matches(['<', '>']).to_string())),
    }
}

fn inlined(body: &str, name: &str, at: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    loop {
        let Some(found) = rest.find("][") else {
            out.push_str(rest);
            return out;
        };
        let Some(shut) = rest[found + 2..].find(']') else {
            out.push_str(rest);
            return out;
        };
        let said = &rest[found + 2..found + 2 + shut];
        let inner = match said.is_empty() {
            true => rest[..found].rsplit('[').next().unwrap_or_default(),
            false => said,
        };
        if inner.to_lowercase() == name {
            out.push_str(&rest[..found + 1]);
            out.push_str(&format!("({at})"));
            rest = &rest[found + 3 + shut..];
        } else {
            out.push_str(&rest[..found + 2 + shut]);
            rest = &rest[found + 2 + shut..];
        }
    }
}

fn drop_of(line: &str, opens: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(found) = rest.find(opens) {
        out.push_str(&rest[..found]);
        match rest[found..].find('>') {
            Some(shut) => rest = &rest[found + shut + 1..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn paired(line: &str, tag: &str, open: &str, close: &str) -> (String, bool) {
    let shut = format!("</{tag}>");
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    let mut changed = false;
    loop {
        let Some(found) = opened(rest, tag) else {
            out.push_str(rest);
            return (out, changed);
        };
        let Some(ends) = rest[found.1..].find(&shut) else {
            out.push_str(rest);
            return (out, changed);
        };
        out.push_str(&rest[..found.0]);
        out.push_str(open);
        out.push_str(&rest[found.1..found.1 + ends]);
        out.push_str(close);
        rest = &rest[found.1 + ends + shut.len()..];
        changed = true;
    }
}

fn opened(line: &str, tag: &str) -> Option<(usize, usize)> {
    let mut at = 0;
    while let Some(found) = line[at..].find('<').map(|one| at + one) {
        let rest = &line[found + 1..];
        let shut = rest.find('>')?;
        let inner = &rest[..shut];
        let name = inner.split_whitespace().next().unwrap_or(inner);
        if name.eq_ignore_ascii_case(tag) {
            return Some((found, found + 1 + shut + 1));
        }
        at = match inner.rfind('<') {
            Some(last) => found + 1 + last,
            None => found + 1 + shut + 1,
        };
    }
    None
}

fn coded_at(rest: &str, found: usize) -> bool {
    let from = rest[..found].rfind('\n').map(|one| one + 1).unwrap_or(0);
    let line = &rest[from..];
    let ends = line.find('\n').unwrap_or(line.len());
    let at = found - from;
    let mut walk = 0;
    for (coded, part) in spans(&line[..ends]) {
        let wide = part.len();
        if at >= walk && at < walk + wide {
            return coded;
        }
        walk += wide;
    }
    false
}

fn uncommented(body: &str) -> (String, bool) {
    if !body.contains("<!--") {
        return (body.to_string(), false);
    }
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    let mut cut = false;
    while let Some(found) = rest.find("<!--") {
        if coded_at(rest, found) {
            let step = found + 4;
            out.push_str(&rest[..step]);
            rest = &rest[step..];
            continue;
        }
        out.push_str(&rest[..found]);
        cut = true;
        match rest[found..].find("-->") {
            Some(ends) => rest = &rest[found + ends + 3..],
            None => return (out, true),
        }
    }
    out.push_str(rest);
    (out, cut)
}

fn commentless(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(found) = rest.find("<!--") {
        out.push_str(&rest[..found]);
        match rest[found..].find("-->") {
            Some(shut) => rest = &rest[found + shut + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn tagged(line: &str) -> bool {
    crate::docs::markup(line).is_some()
}

fn tagless(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(found) = rest.find('<') {
        let after = &rest[found + 1..];
        let Some(shut) = after.find('>') else {
            out.push_str(rest);
            return out;
        };
        let inner = &after[..shut];
        let name = inner
            .trim_start_matches('/')
            .split(|one: char| one.is_whitespace() || one == '/' || one == '>')
            .next()
            .unwrap_or_default();
        let letters = !name.is_empty()
            && name
                .chars()
                .next()
                .is_some_and(|one| one.is_ascii_alphabetic())
            && name.chars().all(|one| one.is_ascii_alphanumeric())
            && crate::docs::markup(&format!("<{inner}>")).is_some();
        let shut_too = line.contains(&format!("</{name}>"));
        if letters && !crate::docs::kept(inner) && shut_too {
            out.push_str(&rest[..found]);
            rest = &rest[found + 1 + shut + 1..];
        } else if letters && !crate::docs::kept(inner) {
            let word = rest[..found]
                .rfind(char::is_whitespace)
                .map(|one| one + 1)
                .unwrap_or(0);
            out.push_str(&rest[..word]);
            out.push_str(&format!("`{}`", &rest[word..found + 1 + shut + 1]));
            rest = &rest[found + 1 + shut + 1..];
        } else {
            out.push_str(&rest[..found + 1 + shut + 1]);
            rest = &rest[found + 1 + shut + 1..];
        }
    }
    out.push_str(rest);
    out
}

fn mathed(body: &str) -> (String, bool) {
    if !body.contains("$$") {
        return (body.to_string(), false);
    }
    let mut out: Vec<String> = Vec::new();
    let mut held: Option<Vec<String>> = None;
    let mut fence: Option<String> = None;
    let mut moved = false;

    for line in body.lines() {
        if let Some(open) = &fence {
            if line.trim_start().starts_with(open.as_str()) {
                fence = None;
            }
            out.push(line.to_string());
            continue;
        }
        if held.is_none()
            && let Some(open) = opens(line)
        {
            fence = Some(open);
            out.push(line.to_string());
            continue;
        }
        let said = line.trim();
        match &mut held {
            Some(inner) => {
                if said.is_empty() {
                    out.push("```math".into());
                    out.append(inner);
                    out.push("```".into());
                    out.push(String::new());
                    held = None;
                    continue;
                }
                if said.ends_with("$$") {
                    let last = said.trim_end_matches("$$").trim_end();
                    if !last.is_empty() {
                        inner.push(last.to_string());
                    }
                    out.push("```math".into());
                    out.append(inner);
                    out.push("```".into());
                    held = None;
                } else {
                    inner.push(line.to_string());
                }
            }
            None => match lone(said) {
                Some(inner) => {
                    moved = true;
                    out.push("```math".into());
                    if !inner.is_empty() {
                        out.push(inner);
                    }
                    out.push("```".into());
                }
                None if said.starts_with("$$") => {
                    moved = true;
                    let rest = said.trim_start_matches("$$").trim();
                    held = Some(match rest.is_empty() {
                        true => Vec::new(),
                        false => vec![rest.to_string()],
                    });
                }
                None if said.contains("$$") => {
                    moved = true;
                    out.push(coded(line));
                }
                None => out.push(line.to_string()),
            },
        }
    }
    if let Some(inner) = held {
        out.push("```math".into());
        out.extend(inner);
        out.push("```".into());
    }
    (out.join("\n"), moved)
}

fn coded(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(found) = rest.find("$$") {
        let Some(shut) = rest[found + 2..].find("$$") else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..found]);
        let inner = &rest[found + 2..found + 2 + shut];
        match inner.contains('`') {
            true => out.push_str(inner.trim()),
            false => out.push_str(&format!("`{}`", inner.trim())),
        }
        rest = &rest[found + 4 + shut..];
    }
    out.push_str(rest);
    out
}

fn lone(said: &str) -> Option<String> {
    let rest = said.strip_prefix("$$")?;
    let inner = rest.strip_suffix("$$")?;
    Some(inner.trim().to_string())
}

fn spaced(body: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut fence: Option<String> = None;
    for line in body.lines() {
        if let Some(open) = &fence {
            if line.trim_start().starts_with(open.as_str()) {
                fence = None;
            }
            out.push(line.to_string());
            continue;
        }
        if line.trim_start().starts_with("```math")
            && out.last().is_some_and(|one| !one.trim().is_empty())
        {
            out.push(String::new());
        }
        if let Some(open) = opens(line) {
            fence = Some(open);
        }
        out.push(line.to_string());
    }
    let mut whole = out.join("\n");
    if !whole.ends_with('\n') {
        whole.push('\n');
    }
    whole
}
