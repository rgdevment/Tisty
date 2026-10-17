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

pub fn tidied(body: &str) -> Tidied {
    let windows = body.contains("\r\n");
    let even = match windows {
        true => body.replace("\r\n", "\n"),
        false => body.to_string(),
    };
    let plain = crate::speaking::turned(&even);
    let mut opened = match windows {
        true => vec!["Windows line endings"],
        false => Vec::new(),
    };
    if plain != even {
        opened.push("what an older Tisty wrote, in its own words now");
    }
    if crate::docs::survives(&plain).is_ok() {
        return Tidied {
            body: plain,
            changed: opened,
        };
    }
    let body = plain.as_str();
    let whole_again = body;
    let mut said: Vec<&'static str> = opened;
    let mut note = |what: &'static str, changed: &mut Vec<&'static str>| {
        if !changed.contains(&what) {
            changed.push(what);
        }
    };

    let body = body.trim_start_matches('\u{feff}');
    let (head, body) = body.split_at(tisty_markdown::front::span(body).map_or(0, |at| at.end));

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
        if let Some(open) = opens(line) {
            fence = Some(open);
            out.push(line.to_string());
            continue;
        }
        let named = mended(line);
        if named.is_some() {
            note("a colour or an icon Tisty does not have", &mut said);
        }
        out.push(named.unwrap_or_else(|| line.to_string()));
    }

    let whole = out.join("\n");

    let mut walked = String::with_capacity(whole.len());
    let mut fenced: Option<String> = None;
    let mut aside = false;
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
        for (kept, part) in crate::docs::commented(bare, &mut aside) {
            match kept {
                true => walked.push_str(part),
                false => walked.push_str(&plainer(part, &mut said, &mut note)),
            }
        }
        if line.ends_with('\n') {
            walked.push('\n');
        }
    }

    match said.is_empty() {
        true => Tidied {
            body: whole_again.to_string(),
            changed: said,
        },
        false => Tidied {
            body: format!("{head}{}", spaced(&walked)),
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

// Outside code, a stray pen colour or icon becomes what the language can say instead.
fn mended(line: &str) -> Option<String> {
    let mut changed = false;
    let said: String = spans(line)
        .into_iter()
        .map(|(coded, part)| {
            if coded {
                return part.to_string();
            }
            let mut out = part.to_string();
            for (at, instead, _) in crate::docs::strays(part).into_iter().rev() {
                out.replace_range(at, &instead);
                changed = true;
            }
            out
        })
        .collect();
    changed.then_some(said)
}

pub fn spans(line: &str) -> Vec<(bool, &str)> {
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
                .char_indices()
                .rev()
                .find(|(_, one)| one.is_whitespace())
                .map_or(0, |(at, one)| at + one.len_utf8());
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
