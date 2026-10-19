const HIDDEN: &str = "…";
const GIVEN_AWAY_BY: [&str; 8] = [
    "ya29.", "1//0", "4/0", "GOCSPX-", "sl.", "EwA", "EwB", "M.C",
];
const SECRET_NAMES: [&str; 3] = ["token", "secret", "password"];

pub(crate) fn masked(text: &str) -> String {
    let unmailed = without_mail(text);
    let text = unmailed.as_str();
    let mut out = String::with_capacity(text.len());
    let mut named_secret = false;
    let mut bearing = false;
    let mut between = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(at) = rest.find(|one: char| part_of_a_word(one)) {
            let (gap, after) = rest.split_at(at);
            between.push_str(gap);
            out.push_str(gap);
            let end = after
                .find(|one: char| !part_of_a_word(one))
                .unwrap_or(after.len());
            let (word, after) = after.split_at(end);
            let (word, after) = match after.starts_with("://") {
                true => {
                    let end = after.find(char::is_whitespace).unwrap_or(after.len());
                    let (address, after) = after.split_at(end);
                    out.push_str(word);
                    out.push_str(&without_query(address));
                    rest = after;
                    named_secret = false;
                    bearing = false;
                    between.clear();
                    continue;
                }
                false => (word, after),
            };
            let told = bearing || named_secret && (between.contains('=') || between.contains(':'));
            match told || a_token(word) {
                true => out.push_str(HIDDEN),
                false => out.push_str(word),
            }
            let lower = word.to_lowercase();
            named_secret = SECRET_NAMES.iter().any(|one| lower.contains(one));
            bearing = lower == "bearer" || lower == "basic";
            between.clear();
            rest = after;
        } else {
            out.push_str(rest);
            break;
        }
    }
    out
}

const FILE_KINDS: [&str; 22] = [
    "png", "jpg", "jpeg", "gif", "svg", "webp", "ico", "bmp", "pdf", "md", "txt", "json", "toml",
    "log", "zip", "html", "css", "js", "ts", "tsx", "rs", "jsonl",
];

fn without_mail(text: &str) -> String {
    let local = |one: char| one.is_alphanumeric() || "._%+-".contains(one);
    let host = |one: char| one.is_alphanumeric() || ".-".contains(one);
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        let before = &rest[..at];
        let after = &rest[at + 1..];
        let start = before
            .char_indices()
            .rev()
            .take_while(|(_, one)| local(*one))
            .last()
            .map_or(before.len(), |(at, _)| at);
        let end = after.find(|one: char| !host(one)).unwrap_or(after.len());
        let domain = after[..end].trim_end_matches('.');
        let a_mail = start < before.len()
            && domain.rsplit_once('.').is_some_and(|(name, top)| {
                !name.is_empty()
                    && (2..=24).contains(&top.len())
                    && top.chars().all(|one| one.is_ascii_alphabetic())
                    && !FILE_KINDS.contains(&top.to_ascii_lowercase().as_str())
            });
        match a_mail {
            true => {
                out.push_str(&before[..start]);
                out.push_str(HIDDEN);
                out.push('@');
                out.push_str(HIDDEN);
                rest = &after[domain.len()..];
            }
            false => {
                out.push_str(before);
                out.push('@');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn part_of_a_word(one: char) -> bool {
    one.is_ascii_alphanumeric() || "-_.~+/!*$".contains(one)
}

fn without_query(address: &str) -> String {
    match address.find(['?', '#']) {
        Some(at) => format!("{}?{HIDDEN}", &address[..at]),
        None => address.to_string(),
    }
}

fn a_token(word: &str) -> bool {
    let word = word.trim_end_matches('.');
    if word.len() >= 16 && GIVEN_AWAY_BY.iter().any(|one| word.starts_with(one)) {
        return true;
    }
    if word.starts_with("eyJ") && word.contains('.') && word.len() >= 20 {
        return true;
    }
    word.len() >= 32
        && !word.contains('/')
        && word.chars().any(|one| one.is_ascii_digit())
        && word.chars().any(|one| one.is_ascii_uppercase())
        && word.chars().any(|one| one.is_ascii_lowercase())
}

#[cfg(test)]
#[path = "masked_test.rs"]
mod tests;
