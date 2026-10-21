pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn plainly(text: &str) -> String {
    composed(
        &text
            .chars()
            .filter(|c| {
                !c.is_control() && !matches!(*c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            })
            .collect::<String>(),
    )
    .trim()
    .chars()
    .take(120)
    .collect()
}

pub fn folded(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    text.to_lowercase()
        .nfd()
        .filter(|c| !matches!(*c as u32, 0x0300..=0x036F))
        .collect()
}

pub const TERMS_AT_MOST: usize = 12;

pub fn terms(query: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut word = String::new();
    let mut quoted = false;

    for c in folded(query).chars() {
        match c {
            '"' | '\u{201c}' | '\u{201d}' => {
                if !word.is_empty() {
                    found.push(std::mem::take(&mut word));
                }
                quoted = !quoted;
            }
            c if c.is_whitespace() && !quoted => {
                if !word.is_empty() {
                    found.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
        if found.len() >= TERMS_AT_MOST {
            return found;
        }
    }
    if !word.is_empty() {
        found.push(word);
    }
    found
}

pub use tisty_markdown::tags::composed;

pub fn subject(title: &str) -> String {
    let plain = |one: char| !one.is_alphanumeric();
    let bare = title.trim_matches(plain);
    let bare = without_ticket(bare).map_or(bare, |rest| rest.trim_matches(plain));
    one_line(&folded(bare))
}

fn without_ticket(title: &str) -> Option<&str> {
    let letters = title.chars().take_while(char::is_ascii_uppercase).count();
    let rest = title[letters..].strip_prefix('-')?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    let after = &rest[digits..];
    (letters >= 2
        && digits > 0
        && after
            .chars()
            .next()
            .is_none_or(|one| !one.is_alphanumeric()))
    .then_some(after)
}

#[cfg(test)]
#[path = "text_test.rs"]
mod tests;

// Words that say nothing of a subject would make every title and folder look alike.
const EMPTY: [&str; 20] = [
    "and", "for", "the", "with", "from", "into", "por", "con", "del", "para", "los", "las", "una",
    "que", "sin", "sus", "como", "este", "esta", "entre",
];

/// Every word, numbers and short ones too, so markup around them decides nothing.
pub fn worded(said: &str) -> String {
    folded(said)
        .split(|one: char| !one.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn telling(said: &str) -> Vec<String> {
    folded(said)
        .split(|one: char| !one.is_alphanumeric())
        .filter(|one| one.chars().count() >= 3)
        .filter(|one| !one.chars().all(|c| c.is_ascii_digit()))
        .filter(|one| !EMPTY.contains(one))
        .map(str::to_string)
        .collect()
}
