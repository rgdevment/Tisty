use unicode_normalization::UnicodeNormalization;

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

pub fn composed(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    text.nfc().collect()
}

#[cfg(test)]
#[path = "text_test.rs"]
mod tests;
