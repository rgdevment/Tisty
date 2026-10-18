#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub value: String,
}

impl Line {
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }
}

pub fn unfolded(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        match (raw.strip_prefix([' ', '\t']), out.last_mut()) {
            (Some(more), Some(last)) => last.push_str(more),
            _ if raw.trim().is_empty() => {}
            _ => out.push(raw.to_string()),
        }
    }
    out
}

const DATED: [&str; 5] = ["DTSTART", "DTEND", "DUE", "EXDATE", "RECURRENCE-ID"];

// A date holds no colon, so its last one starts it: Outlook leaves «(UTC-05:00) Eastern» unquoted.
pub fn line(raw: &str) -> Option<Line> {
    let named = raw.split([';', ':']).next().unwrap_or("").trim();
    let dated = DATED.iter().any(|one| one.eq_ignore_ascii_case(named));
    let mut quoted = false;
    let colons = raw.char_indices().filter(|(_, one)| {
        if *one == '"' {
            quoted = !quoted;
        }
        *one == ':' && !quoted
    });
    let split = if dated {
        colons.last()?
    } else {
        colons.into_iter().next()?
    };
    let (head, value) = (&raw[..split.0], &raw[split.0 + 1..]);
    let mut parts = head.split(';');
    let name = parts.next()?.trim().to_ascii_uppercase();
    if name.is_empty() {
        return None;
    }
    let params = parts
        .filter_map(|one| one.split_once('='))
        .map(|(key, value)| {
            (
                key.trim().to_ascii_uppercase(),
                value.trim().trim_matches('"').to_string(),
            )
        })
        .collect();
    Some(Line {
        name,
        params,
        value: value.to_string(),
    })
}

pub fn unescaped(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(one) = chars.next() {
        if one != '\\' {
            out.push(one);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}
