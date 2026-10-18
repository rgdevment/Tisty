use serde::Serialize;
use tisty_markdown::Kind;

use crate::arriving::tidied;

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

/// Code keeps its own words, so a widget's HTML is never told as outside the language.
pub fn mends(body: &str) -> Vec<Mend> {
    let even = body.replace("\r\n", "\n");
    if tidied(&even).body.trim_end() == even.trim_end() {
        return Vec::new();
    }
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
    let Some(kept) = crate::lcs::table(&a, &b) else {
        return vec![Piece {
            said: said.to_string(),
            fixed: fixed.to_string(),
        }];
    };
    let mut out: Vec<Changed> = Vec::new();
    let mut now: Option<Changed> = None;
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
        let (mut x, mut y, before) = match now.take() {
            Some((mut x, mut y, before)) if blank => {
                x.extend(&between);
                y.extend(&between);
                (x, y, before)
            }
            Some(done) => {
                out.push(done);
                (Vec::new(), Vec::new(), last_word(&between))
            }
            None => (Vec::new(), Vec::new(), last_word(&between)),
        };
        between.clear();
        if j == b.len() || (i < a.len() && kept.at(i + 1, j) >= kept.at(i, j + 1)) {
            x.push(a[i]);
            i += 1;
        } else {
            y.push(b[j]);
            j += 1;
        }
        now = Some((x, y, before));
    }
    out.extend(now);
    out.into_iter()
        .map(|(x, y, before)| {
            // Words only added name nothing written, so the word before them shows where they go.
            let lead = match x.concat().trim().is_empty() {
                true => before.concat(),
                false => String::new(),
            };
            Piece {
                said: format!("{lead}{}", x.concat()).trim().to_string(),
                fixed: format!("{lead}{}", y.concat()).trim().to_string(),
            }
        })
        .filter(|one| !one.said.is_empty() || !one.fixed.is_empty())
        .collect()
}

type Changed<'a> = (Vec<&'a str>, Vec<&'a str>, Vec<&'a str>);

fn last_word<'a>(between: &[&'a str]) -> Vec<&'a str> {
    match between.iter().rposition(|one| !one.trim().is_empty()) {
        Some(at) => between[at..].to_vec(),
        None => Vec::new(),
    }
}

#[cfg(test)]
#[path = "mending_test.rs"]
mod tests;
