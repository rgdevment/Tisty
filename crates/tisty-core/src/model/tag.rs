use std::fmt;

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a tag needs at least one letter or digit")]
pub struct InvalidTag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Tag(String);

impl Tag {
    pub fn new(raw: &str) -> Result<Self, InvalidTag> {
        // Accents come off: somebody writes #camion on Monday and #camión on Friday, and one tag
        // is what they meant both times. Read back through serde, so tags already written fold
        // themselves the next time the log is read.
        let normalised: String = crate::text::composed(raw)
            .trim()
            .to_lowercase()
            .nfd()
            .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
            .map(|c| {
                if c.is_whitespace() || c == '_' {
                    '-'
                } else {
                    c
                }
            })
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();

        let trimmed = normalised.trim_matches('-');
        if !trimmed.chars().any(char::is_alphanumeric) {
            return Err(InvalidTag);
        }
        Ok(Self(collapse_dashes(trimmed)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn collapse_dashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut previous_dash = false;
    for c in s.chars() {
        if c == '-' {
            if !previous_dash {
                out.push(c);
            }
            previous_dash = true;
        } else {
            out.push(c);
            previous_dash = false;
        }
    }
    out
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Tag {
    type Error = InvalidTag;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Tag> for String {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl std::str::FromStr for Tag {
    type Err = InvalidTag;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(raw: &str) -> String {
        Tag::new(raw).unwrap().to_string()
    }

    #[test]
    fn casing_and_spacing_collapse_into_one_tag() {
        assert_eq!(tag("Work"), "work");
        assert_eq!(tag("  WORK  "), "work");
        assert_eq!(tag("mi etiqueta"), "mi-etiqueta");
        assert_eq!(tag("mi_etiqueta"), "mi-etiqueta");
        assert_eq!(tag("mi   etiqueta"), "mi-etiqueta");
    }

    #[test]
    fn an_accent_written_apart_from_its_letter_lands_on_the_same_tag() {
        assert_eq!(tag("disen\u{0303}o"), tag("diseño"));
        assert_eq!(tag("gestio\u{0301}n"), tag("gestión"));
    }

    #[test]
    fn punctuation_is_dropped_not_kept_as_separator() {
        assert_eq!(tag("b2b/b2c"), "b2bb2c");
        assert_eq!(tag("#bug!"), "bug");
    }

    /// One word, one tag: nobody writes the accent the same way twice, and a tag people cannot
    /// hit reliably is a tag that quietly splits their work in two.
    #[test]
    fn a_word_is_one_tag_however_its_accents_were_typed() {
        assert_eq!(tag("migración"), "migracion");
        assert_eq!(tag("camión"), tag("camion"));
        assert_eq!(tag("camión"), tag("CAMIÓN"));
        assert_eq!(tag("niño"), "nino");
    }

    #[test]
    fn an_accent_that_is_the_whole_letter_still_leaves_something_behind() {
        assert_eq!(tag("año"), "ano");
        assert_eq!(Tag::new("´"), Err(InvalidTag));
    }

    #[test]
    fn a_tag_without_letters_or_digits_is_rejected() {
        assert_eq!(Tag::new("---"), Err(InvalidTag));
        assert_eq!(Tag::new("  "), Err(InvalidTag));
        assert_eq!(Tag::new(""), Err(InvalidTag));
    }

    #[test]
    fn deserialisation_normalises() {
        let tag: Tag = serde_json::from_str(r#""  Work  ""#).unwrap();
        assert_eq!(tag.as_str(), "work");
    }

    #[test]
    fn deserialising_an_empty_tag_fails() {
        assert!(serde_json::from_str::<Tag>(r#""!!""#).is_err());
    }
}
