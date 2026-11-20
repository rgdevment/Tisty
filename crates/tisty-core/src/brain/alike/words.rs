use std::collections::HashMap;

pub(crate) const KEPT_AT_MOST: usize = 256;
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

// How a text is laid out, in Tisty's own words: a guide and its translation share these and nothing they are about.
const LAYOUT: &[&str] = &[
    "negrita",
    "negritas",
    "cursiva",
    "cursivas",
    "tachado",
    "subrayado",
    "titulo",
    "titulos",
    "subtitulo",
    "encabezado",
    "encabezados",
    "nivel",
    "niveles",
    "numero",
    "numeros",
    "numerada",
    "lista",
    "listas",
    "tabla",
    "tablas",
    "fila",
    "filas",
    "columna",
    "columnas",
    "celda",
    "celdas",
    "enlace",
    "enlaces",
    "bloque",
    "bloques",
    "casilla",
    "casillas",
    "vineta",
    "vinetas",
    "parrafo",
    "parrafos",
    "markdown",
    "bold",
    "italic",
    "italics",
    "strikethrough",
    "underline",
    "heading",
    "headings",
    "title",
    "titles",
    "subtitle",
    "level",
    "levels",
    "number",
    "numbers",
    "numbered",
    "list",
    "lists",
    "table",
    "tables",
    "row",
    "rows",
    "column",
    "columns",
    "cell",
    "cells",
    "link",
    "links",
    "block",
    "blocks",
    "checkbox",
    "checkboxes",
    "bullet",
    "bullets",
    "paragraph",
    "paragraphs",
];

pub(crate) type Word = (String, String, u32);

pub(crate) fn counted(texts: &[&str]) -> Vec<Word> {
    let mut seen: HashMap<String, (String, u32)> = HashMap::new();
    let spoken = texts
        .iter()
        .flat_map(|text| text.split_whitespace())
        .filter(|token| !addressed(token));
    for token in spoken {
        for word in token.split(|one: char| !one.is_alphanumeric()) {
            let Some(key) = weighed(word) else {
                continue;
            };
            seen.entry(key)
                .or_insert_with(|| (word.to_lowercase(), 0))
                .1 += 1;
        }
    }
    let mut all: Vec<Word> = seen
        .into_iter()
        .map(|(key, (written, count))| (key, written, count))
        .collect();
    // Among words said as often, a longer one is likelier to be rare, so it outlasts the cut.
    all.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.0.chars().count().cmp(&a.0.chars().count()))
            .then_with(|| a.0.cmp(&b.0))
    });
    all.truncate(KEPT_AT_MOST);
    all
}

const WRAPPING: &[char] = &[
    '(', ')', '[', ']', '"', '\'', '«', '»', '“', '”', '‘', '’', '¿', '?', '¡', '!', ',', ';',
];
const CODE_MARKS: &[char] = &['=', '<', '>', '{', '}', '`', '|', '$'];

// A link, a path, a file, a mail, a tag, an emoji, an entity or a name from code points somewhere, never alike.
fn addressed(token: &str) -> bool {
    let token = token.trim_matches(WRAPPING);
    let emoji = token.len() > 2 && token.starts_with(':') && token.ends_with(':');
    let token = token.trim_end_matches(['.', ':']);
    let joined = |mark: char| {
        token
            .match_indices(mark)
            .any(|(at, _)| around(token, at, mark.len_utf8()))
    };
    let separators = token.matches(['/', '\\']).count();
    emoji
        || token.contains("://")
        || token.starts_with("www.")
        || token.starts_with("--")
        || token.starts_with('&')
        || token.starts_with('#') && token[1..].starts_with(char::is_alphabetic)
        || token.contains(CODE_MARKS)
        || joined('.')
        || joined('_')
        || joined('@')
        || separators >= 2
        || separators == 1 && (token.contains('.') || token.starts_with(['/', '\\', '~']))
}

fn around(token: &str, at: usize, wide: usize) -> bool {
    let before = token[..at].chars().next_back();
    let after = token[at + wide..].chars().next();
    before.is_some_and(char::is_alphanumeric) && after.is_some_and(char::is_alphanumeric)
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
    let key = crate::text::folded(word);
    (!STOP.contains(&key.as_str()) && !LAYOUT.contains(&key.as_str())).then_some(key)
}

// The folded key is kept, so a pass never folds a word again; the written form only when it differs.
pub(crate) fn said(words: &[Word]) -> String {
    words
        .iter()
        .map(|(key, written, count)| match key == written {
            true => format!("{key}:{count}"),
            false => format!("{key}:{count}:{written}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn read(said: &str) -> Vec<Word> {
    said.split(' ')
        .filter_map(|one| {
            let mut parts = one.split(':');
            let key = parts.next()?;
            let count = parts.next()?.parse().ok()?;
            let written = parts.next().unwrap_or(key);
            Some((key.to_string(), written.to_string(), count))
        })
        .collect()
}

#[cfg(test)]
#[path = "words_test.rs"]
mod tests;
