use serde::Serialize;
use tisty_markdown::Kind;

use crate::arriving::tidied;

/// Past this many words a block is told as a whole: comparing them word by word costs their square.
const WORDS_AT_MOST: usize = 2_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mend {
    pub from: usize,
    pub to: usize,
    pub fixed: String,
    pub pieces: Vec<Piece>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Piece {
    pub said: String,
    pub fixed: String,
}

/// The blocks a person wrote outside the Tisty language, each with what the translator would put
/// in its place. Code keeps its own words, so a widget's HTML is never one of them.
pub fn mends(body: &str) -> Vec<Mend> {
    let doc = tisty_markdown::read(body);
    doc.blocks()
        .iter()
        .filter(|block| !matches!(block.kind, Kind::Front | Kind::Code { .. }))
        .filter_map(|block| {
            let said = body.get(block.at.clone())?.trim_end();
            let windows = said.contains("\r\n");
            let even = said.replace("\r\n", "\n");
            let fixed = tidied(&even).body;
            let fixed = fixed.trim_end();
            if fixed == even {
                return None;
            }
            let fixed = match windows {
                true => fixed.replace('\n', "\r\n"),
                false => fixed.to_string(),
            };
            Some(Mend {
                from: block.at.start,
                to: block.at.start + said.len(),
                pieces: pieces(said, &fixed),
                fixed,
            })
        })
        .collect()
}

fn words(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut blank = None;
    for (at, one) in text.char_indices() {
        let now = one.is_whitespace();
        if blank.is_some_and(|was| was != now) {
            out.push(&text[start..at]);
            start = at;
        }
        blank = Some(now);
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

/// What changed, word by word; changes apart only by spaces are one piece, as `<b>two words</b>` is.
fn pieces(said: &str, fixed: &str) -> Vec<Piece> {
    let (a, b) = (words(said), words(fixed));
    if a.len() > WORDS_AT_MOST || b.len() > WORDS_AT_MOST {
        return vec![Piece {
            said: said.to_string(),
            fixed: fixed.to_string(),
        }];
    }
    let mut kept = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            kept[i][j] = match a[i] == b[j] {
                true => kept[i + 1][j + 1] + 1,
                false => kept[i + 1][j].max(kept[i][j + 1]),
            };
        }
    }
    let mut out: Vec<(Vec<&str>, Vec<&str>)> = Vec::new();
    let mut now: Option<(Vec<&str>, Vec<&str>)> = None;
    let mut between: Vec<&str> = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            between.push(a[i]);
            i += 1;
            j += 1;
            continue;
        }
        let blank = between.iter().all(|one| one.trim().is_empty());
        let (mut x, mut y) = match now.take() {
            Some((mut x, mut y)) if blank => {
                x.extend(&between);
                y.extend(&between);
                (x, y)
            }
            Some(done) => {
                out.push(done);
                (Vec::new(), Vec::new())
            }
            None => (Vec::new(), Vec::new()),
        };
        between.clear();
        if j == b.len() || (i < a.len() && kept[i + 1][j] >= kept[i][j + 1]) {
            x.push(a[i]);
            i += 1;
        } else {
            y.push(b[j]);
            j += 1;
        }
        now = Some((x, y));
    }
    out.extend(now);
    out.into_iter()
        .map(|(x, y)| Piece {
            said: x.concat().trim().to_string(),
            fixed: y.concat().trim().to_string(),
        })
        .filter(|one| !one.said.is_empty())
        .collect()
}

#[cfg(test)]
#[path = "mending_test.rs"]
mod tests;
