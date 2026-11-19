pub(crate) const SPANISH: &str = "es";
pub(crate) const ENGLISH: &str = "en";

const SPANISH_TELLS: &[&str] = &[
    "el", "la", "los", "las", "de", "del", "que", "y", "en", "un", "una", "para", "con", "por",
    "al", "mi", "se", "lo", "es", "su", "sus", "como", "pero", "mas", "este", "esta",
];
const ENGLISH_TELLS: &[&str] = &[
    "the", "a", "an", "of", "to", "and", "in", "is", "for", "on", "my", "your", "with", "it", "be",
    "this", "that", "at", "from", "by", "out", "up", "are", "can", "you",
];
const SPANISH_MARKS: &str = "ñáéíóú¿¡";

// The words that say nothing tell the language, so the round reads them before they are dropped.
pub(crate) fn guessed(texts: &[&str]) -> Option<&'static str> {
    let (mut spanish, mut english) = (0usize, 0usize);
    for text in texts {
        for word in text.split(|one: char| !one.is_alphanumeric()) {
            let tells = |all: &[&str]| all.iter().any(|one| one.eq_ignore_ascii_case(word));
            spanish += usize::from(tells(SPANISH_TELLS));
            english += usize::from(tells(ENGLISH_TELLS));
        }
    }
    match spanish.cmp(&english) {
        std::cmp::Ordering::Greater => Some(SPANISH),
        std::cmp::Ordering::Less => Some(ENGLISH),
        std::cmp::Ordering::Equal => texts
            .iter()
            .any(|text| text.chars().any(|one| SPANISH_MARKS.contains(one)))
            .then_some(SPANISH),
    }
}

#[cfg(test)]
#[path = "spoken_test.rs"]
mod tests;
