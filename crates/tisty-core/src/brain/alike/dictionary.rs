use std::collections::HashMap;
use std::path::Path;

// A Spanish word, folded, and the English word it stands for: one line each, a tab between.
#[derive(Debug, Default)]
pub struct Dictionary {
    english: HashMap<String, String>,
}

impl Dictionary {
    pub fn read(at: &Path) -> Result<Self, String> {
        let said = std::fs::read_to_string(at).map_err(|e| e.to_string())?;
        Self::parsed(&said)
    }

    pub(crate) fn parsed(said: &str) -> Result<Self, String> {
        let mut english = HashMap::new();
        for (n, line) in said.lines().enumerate() {
            match line.split_once('\t') {
                Some((spanish, word)) if !spanish.is_empty() && !word.is_empty() => {
                    english.insert(spanish.to_string(), word.to_string());
                }
                _ => return Err(format!("line {} is not a word, a tab and a word", n + 1)),
            }
        }
        Ok(Self { english })
    }

    pub(crate) fn english_of(&self, spanish: &str) -> Option<&str> {
        self.english.get(spanish).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.english.len()
    }

    pub fn is_empty(&self) -> bool {
        self.english.is_empty()
    }
}

#[cfg(test)]
#[path = "dictionary_test.rs"]
mod tests;
