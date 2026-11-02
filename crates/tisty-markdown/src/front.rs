use std::ops::Range;

// Past these the block is kept as written but nothing in it is read.
pub const BLOCK_AT_MOST: usize = 4 * 1024;
pub const KEYS_AT_MOST: usize = 32;
pub const VALUE_AT_MOST: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: String,
    pub values: Vec<String>,
}

// The rule GitHub and Obsidian follow: `---` first, words right under it, `---` or `...` to close.
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

pub fn opens(text: &str) -> bool {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    lines.next().map(str::trim_end) == Some("---")
        && lines.next().is_some_and(|one| !one.trim().is_empty())
}

pub fn fields(text: &str) -> Vec<Field> {
    let Some(at) = span(text).filter(|at| at.len() <= BLOCK_AT_MOST) else {
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
    let past = out.len() > KEYS_AT_MOST
        || out
            .iter()
            .flat_map(|one| &one.values)
            .any(|one| one.chars().count() > VALUE_AT_MOST);
    match past {
        true => Vec::new(),
        false => out,
    }
}

pub fn title(text: &str) -> Option<String> {
    fields(text)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case("title"))
        .and_then(|one| one.values.into_iter().next())
        .filter(|one| one.chars().any(char::is_alphanumeric))
}

pub fn tagging(key: &str) -> bool {
    key.eq_ignore_ascii_case("tags") || key.eq_ignore_ascii_case("tag")
}

pub fn tags(text: &str) -> Vec<String> {
    fields(text)
        .into_iter()
        .filter(|one| tagging(&one.key))
        .flat_map(|one| words(&one.values))
        .collect()
}

/// The language the body says it is in, as a tag in its usual case: `es`, `pt-BR`, `zh-Hant`.
pub fn lang(text: &str) -> Option<String> {
    fields(text)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case("lang"))
        .and_then(|one| one.values.into_iter().next())
        .and_then(|one| tagged(&one))
}

// Only language, script and region: what a person picks a language by, and nothing a typo slips through.
pub fn tagged(said: &str) -> Option<String> {
    let mut parts = said.trim().split(['-', '_']);
    let language = parts.next().filter(|one| {
        (2..=3).contains(&one.len()) && one.chars().all(|c| c.is_ascii_alphabetic())
    })?;
    let mut out = language.to_ascii_lowercase();
    let mut script = false;
    let mut region = false;
    for part in parts {
        let letters = part.chars().all(|c| c.is_ascii_alphabetic());
        let digits = part.chars().all(|c| c.is_ascii_digit());
        if part.len() == 4 && letters && !script && !region {
            let (first, rest) = part.split_at(1);
            out.push_str(&format!(
                "-{}{}",
                first.to_ascii_uppercase(),
                rest.to_ascii_lowercase()
            ));
            script = true;
        } else if ((part.len() == 2 && letters) || (part.len() == 3 && digits)) && !region {
            out.push_str(&format!("-{}", part.to_ascii_uppercase()));
            region = true;
        } else {
            return None;
        }
    }
    Some(out)
}

// One value may hold several tags, `a, b` or `#a #b`.
pub fn words(values: &[String]) -> Vec<String> {
    values
        .iter()
        .flat_map(|one| one.split([',', ' ']))
        .map(|word| word.trim_start_matches('#').to_string())
        .filter(|one| !one.is_empty())
        .collect()
}

// Single quotes keep a backslash and a double quote literal, where double quotes read escapes.
pub fn quoted(said: &str) -> String {
    match said.contains(['"', '\\']) {
        true => format!("'{}'", said.replace('\'', "''")),
        false => format!("\"{said}\""),
    }
}

// The `title:` line rewritten whole, so a list or a quote around the old value never lingers.
pub fn titled_as(front: &str, name: &str) -> Option<String> {
    title(front)?;
    let name = quoted(name);
    let mut out = String::with_capacity(front.len() + name.len());
    let mut done = false;
    for line in front.split_inclusive('\n') {
        let keyed = !done
            && !line.starts_with([' ', '\t'])
            && line
                .split_once(':')
                .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case("title"));
        if !keyed {
            out.push_str(line);
            continue;
        }
        let ending = &line[line.trim_end().len()..];
        out.push_str(&format!("title: {name}{ending}"));
        done = true;
    }
    done.then_some(out)
}

/// The text with one property set where it stands, or added last, in a block made when there is none.
pub fn with_field(text: &str, key: &str, value: &str) -> String {
    let said = format!("{key}: {value}\n");
    let Some(at) = span(text) else {
        let skip = text.len() - text.trim_start_matches('\u{feff}').len();
        return format!("{}---\n{said}---\n{}", &text[..skip], &text[skip..]);
    };
    let lines: Vec<&str> = text[at.clone()].split_inclusive('\n').collect();
    let closing = lines.len() - 1;
    let mut block = String::with_capacity(at.len() + said.len());
    let mut done = false;
    for (n, line) in lines.iter().enumerate() {
        let keyed = n > 0
            && n < closing
            && !line.starts_with([' ', '\t'])
            && line
                .split_once(':')
                .is_some_and(|(named, _)| named.trim().eq_ignore_ascii_case(key));
        if keyed && !done {
            block.push_str(&said);
            done = true;
            continue;
        }
        if n == closing && !done {
            block.push_str(&said);
        }
        block.push_str(line);
    }
    format!("{}{block}{}", &text[..at.start], &text[at.end..])
}

/// The text with one property taken out, and the block with it when nothing else is left in it.
pub fn without_field(text: &str, key: &str) -> String {
    let Some(at) = span(text) else {
        return text.to_string();
    };
    let lines: Vec<&str> = text[at.clone()].split_inclusive('\n').collect();
    let closing = lines.len() - 1;
    let kept: Vec<&str> = lines[1..closing]
        .iter()
        .copied()
        .filter(|line| {
            line.starts_with([' ', '\t'])
                || !line
                    .split_once(':')
                    .is_some_and(|(named, _)| named.trim().eq_ignore_ascii_case(key))
        })
        .collect();
    let rest = &text[at.end..];
    if kept.iter().all(|line| line.trim().is_empty()) {
        return format!(
            "{}{}",
            &text[..at.start],
            rest.strip_prefix('\n').unwrap_or(rest)
        );
    }
    format!(
        "{}{}{}{}{rest}",
        &text[..at.start],
        lines[0],
        kept.concat(),
        lines[closing]
    )
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
            return match quote {
                '\'' => inner.replace("''", "'"),
                _ => inner.to_string(),
            };
        }
    }
    flat.to_string()
}
