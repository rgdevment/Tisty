use std::collections::HashMap;

pub(crate) const KEPT_AT_MOST: usize = 1024;
const SHORTEST: usize = 3;
const LONGEST: usize = 32;
// A long run of letters and digits is an id or a hash: two notes naming one say nothing alike.
const ID_LIKE_FROM: usize = 8;

const STOP: &[&str] = &[
    "and", "are", "but", "can", "for", "from", "has", "have", "how", "into", "its", "not", "one",
    "only", "our", "out", "that", "the", "their", "them", "then", "there", "these", "they", "this",
    "those", "was", "were", "what", "when", "where", "which", "who", "why", "will", "with", "you",
    "your", "all", "any", "also", "been", "being", "does", "each", "more", "most", "other", "same",
    "some", "such", "than", "too", "very", "just", "about", "after", "before", "over", "under",
    "again", "would", "could", "should", "may", "might", "must", "here", "both", "own", "por",
    "con", "del", "para", "los", "las", "una", "uno", "unos", "unas", "que", "sin", "sus", "como",
    "este", "esta", "estos", "estas", "ese", "esa", "esos", "esas", "entre", "pero", "mas", "muy",
    "hay", "son", "ser", "fue", "era", "han", "hace", "sobre", "cada", "todo", "toda", "todos",
    "todas", "otro", "otra", "otros", "otras", "solo", "tambien", "donde", "cuando", "quien",
    "cual", "cuales", "porque", "desde", "hasta", "tiene", "tienen", "puede", "pueden", "nada",
    "algo", "aqui", "alli", "asi", "les", "nos", "mis", "tus",
];

pub(crate) fn counted(texts: &[&str]) -> Vec<(String, u32)> {
    let mut seen: HashMap<String, (String, u32)> = HashMap::new();
    for text in texts {
        for word in text.split(|one: char| !one.is_alphanumeric()) {
            let Some(key) = weighed(word) else {
                continue;
            };
            seen.entry(key)
                .or_insert_with(|| (word.to_lowercase(), 0))
                .1 += 1;
        }
    }
    let mut all: Vec<(String, u32)> = seen.into_values().collect();
    all.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    all.truncate(KEPT_AT_MOST);
    all
}

pub(crate) fn key(word: &str) -> String {
    crate::text::folded(word)
}

fn weighed(word: &str) -> Option<String> {
    let length = word.chars().count();
    if !(SHORTEST..=LONGEST).contains(&length) {
        return None;
    }
    let digits = word.chars().filter(char::is_ascii_digit).count();
    if digits == length || (digits > 0 && length >= ID_LIKE_FROM) {
        return None;
    }
    let key = key(word);
    (!STOP.contains(&key.as_str())).then_some(key)
}

pub(crate) fn said(words: &[(String, u32)]) -> String {
    words
        .iter()
        .map(|(word, count)| format!("{word}:{count}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn read(said: &str) -> Vec<(String, u32)> {
    said.split(' ')
        .filter_map(|pair| {
            let (word, count) = pair.rsplit_once(':')?;
            Some((word.to_string(), count.parse().ok()?))
        })
        .collect()
}

#[cfg(test)]
#[path = "words_test.rs"]
mod tests;
