pub fn reads(text: &str, key: &str, name: &str) -> Option<String> {
    let doc: ::toml::Table = text.parse().ok()?;
    doc.get(key)?
        .get(name)?
        .get("command")?
        .as_str()
        .map(str::to_string)
}

pub fn set(text: &str, key: &str, name: &str, entry: &str) -> Option<String> {
    if !text.trim().is_empty() && text.parse::<::toml::Table>().is_err() {
        return None;
    }
    let head = format!("[{key}.{name}]");
    if let Some((from, to)) = span(text, key, name) {
        return Some(format!("{}{head}\n{entry}\n{}", &text[..from], &text[to..]));
    }
    // Written some other way — inline, or a name we would not recognise — and rewriting it would
    // mean guessing at somebody else's formatting.
    if reads(text, key, name).is_some() {
        return None;
    }
    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.trim().is_empty() {
        out.push('\n');
    }
    out.push_str(&head);
    out.push('\n');
    out.push_str(entry);
    out.push('\n');
    Some(out)
}

pub fn unset(text: &str, key: &str, name: &str) -> Option<String> {
    let (from, to) = span(text, key, name)?;
    let kept = format!("{}{}", &text[..from], &text[to..]);
    Some(match kept.trim().is_empty() {
        true => String::new(),
        false => kept,
    })
}

fn span(text: &str, key: &str, name: &str) -> Option<(usize, usize)> {
    let want = [key, name];
    let mut ours: Option<usize> = None;
    let mut at = 0usize;
    for line in text.split_inclusive('\n') {
        let head = line.trim_start().starts_with('[');
        if head {
            if let Some(from) = ours {
                return Some((from, at));
            }
            if same(line, &want) {
                ours = Some(at);
            }
        }
        at += line.len();
    }
    ours.map(|from| (from, text.len()))
}

fn same(line: &str, want: &[&str]) -> bool {
    let Some(inner) = line
        .trim()
        .strip_prefix('[')
        .and_then(|it| it.strip_suffix(']'))
    else {
        return false;
    };
    if inner.starts_with('[') {
        return false;
    }
    let parts: Vec<&str> = inner
        .split('.')
        .map(|part| part.trim().trim_matches(['"', '\'']))
        .collect();
    parts == want
}

#[cfg(test)]
#[path = "toml_test.rs"]
mod tests;
