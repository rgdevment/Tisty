use std::ops::Range;
use std::path::Path;

#[derive(Debug, Default)]
pub(crate) struct Dictionary {
    said: String,
    entries: Vec<(Range<usize>, Range<usize>)>,
}

impl Dictionary {
    pub(crate) fn read(at: &Path) -> Result<Self, String> {
        let said = std::fs::read_to_string(at).map_err(|e| e.to_string())?;
        Self::parsed(said)
    }

    // Kept as written and searched in order, so the table costs about its own size in memory.
    pub(crate) fn parsed(said: String) -> Result<Self, String> {
        let mut entries: Vec<(Range<usize>, Range<usize>)> = Vec::new();
        let mut at = 0;
        for (n, line) in said.split_inclusive('\n').enumerate() {
            let text = line.strip_suffix('\n').unwrap_or(line);
            let Some((spanish, english)) = text
                .split_once('\t')
                .filter(|(a, b)| !a.is_empty() && !b.is_empty() && !b.contains('\t'))
            else {
                return Err(format!("line {} is not a word, a tab and a word", n + 1));
            };
            let spanish_at = at..at + spanish.len();
            let english_at = spanish_at.end + 1..spanish_at.end + 1 + english.len();
            if entries
                .last()
                .is_some_and(|(last, _)| said[last.clone()] >= *spanish)
            {
                return Err(format!("line {} is out of order", n + 1));
            }
            entries.push((spanish_at, english_at));
            at += line.len();
        }
        Ok(Self { said, entries })
    }

    pub(crate) fn english_of(&self, spanish: &str) -> Option<&str> {
        let at = self
            .entries
            .binary_search_by(|(word, _)| self.said[word.clone()].cmp(spanish))
            .ok()?;
        Some(&self.said[self.entries[at].1.clone()])
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
#[path = "dictionary_test.rs"]
mod tests;
