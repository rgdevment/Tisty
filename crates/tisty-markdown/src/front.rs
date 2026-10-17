use std::ops::Range;

/// One property of the front matter: its key and what it holds, a list already split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: String,
    pub values: Vec<String>,
}

/// The bytes of the front matter at the top of a document, both fences included, the way
/// GitHub and Obsidian find it: `---` on the first line, words right under it, `---` or `...` to close.
pub fn span(text: &str) -> Option<Range<usize>> {
    let skip = text.len() - text.trim_start_matches('\u{feff}').len();
    let mut at = skip;
    let mut lines = text[skip..].split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end() != "---" {
        return None;
    }
    at += first.len();
    let mut said = false;
    for line in lines {
        let flat = line.trim_end();
        if !said && flat.trim().is_empty() {
            return None;
        }
        at += line.len();
        if said && (flat == "---" || flat == "...") {
            return Some(skip..at);
        }
        said = true;
    }
    None
}

/// What the front matter says, read the plain way most notes write it: `key: value`, a list
/// in brackets, or a list of `- item` lines under the key. Anything deeper is kept as its lines.
pub fn fields(text: &str) -> Vec<Field> {
    let Some(at) = span(text) else {
        return Vec::new();
    };
    let inner: Vec<&str> = text[at].lines().collect();
    let mut out: Vec<Field> = Vec::new();
    let mut listing = false;
    for line in &inner[1..inner.len().saturating_sub(1)] {
        let flat = line.trim();
        if flat.is_empty() || flat.starts_with('#') {
            continue;
        }
        let nested = line.starts_with([' ', '\t']);
        if let Some(last) = out.last_mut() {
            match flat.strip_prefix("- ") {
                Some(item) if listing => {
                    last.values.push(unquoted(item));
                    continue;
                }
                _ if nested => {
                    last.values.push(flat.to_string());
                    continue;
                }
                _ => {}
            }
        }
        let Some((key, value)) = flat.split_once(':') else {
            continue;
        };
        let values = listed(value.trim());
        listing = values.is_empty();
        out.push(Field {
            key: key.trim().to_string(),
            values,
        });
    }
    out
}

/// The title the front matter gives, when it gives one.
pub fn title(text: &str) -> Option<String> {
    fields(text)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case("title"))
        .and_then(|one| one.values.into_iter().next())
        .filter(|one| !one.trim().is_empty())
}

/// The tags the front matter files the document under, written with or without their `#`.
pub fn tags(text: &str) -> Vec<String> {
    fields(text)
        .into_iter()
        .filter(|one| one.key.eq_ignore_ascii_case("tags") || one.key.eq_ignore_ascii_case("tag"))
        .flat_map(|one| words(&one.values))
        .collect()
}

/// Tags written in one value, `a, b` or `#a #b`, one per word and without their `#`.
pub fn words(values: &[String]) -> Vec<String> {
    values
        .iter()
        .flat_map(|one| one.split([',', ' ']))
        .map(|word| word.trim_start_matches('#').to_string())
        .filter(|one| !one.is_empty())
        .collect()
}

fn listed(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    match value
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        Some(inner) => inner
            .split(',')
            .map(unquoted)
            .filter(|one| !one.is_empty())
            .collect(),
        None => vec![unquoted(value)],
    }
}

fn unquoted(said: &str) -> String {
    let flat = said.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = flat
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner.to_string();
        }
    }
    flat.to_string()
}
