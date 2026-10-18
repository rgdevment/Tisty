use crate::event::{Op, PageChange, Paged};
use crate::model::TaskId;

struct Section {
    name: String,
    words: String,
}

// Past three spaces a line is code, or the words of a list item, never a heading.
fn heading(line: &str) -> Option<String> {
    let lead = line.len() - line.trim_start_matches(' ').len();
    let bare = line.trim_start_matches(' ');
    let marks = bare.chars().take_while(|one| *one == '#').count();
    let rest = bare.get(marks..)?;
    if lead > 3 || !(1..=6).contains(&marks) || !rest.starts_with([' ', '\t']) {
        return None;
    }
    let words = rest.trim();
    // A closing run of marks follows a space; one stuck to a word is the word's, as in «C#».
    let closed = words.trim_end_matches('#');
    let name = match closed.is_empty() || closed.ends_with([' ', '\t']) {
        true => closed.trim(),
        false => words,
    };
    Some(name.to_string()).filter(|name| !name.is_empty())
}

// A box is a step with a trail of its own, and a hint is the template's, never the person's words.
fn own_words(line: &str) -> bool {
    let bare = line.trim_start_matches(['>', ' ', '\t']);
    let item = bare
        .strip_prefix(['-', '*', '+'])
        .or_else(|| {
            let digits = bare.chars().take_while(char::is_ascii_digit).count();
            (digits > 0)
                .then(|| bare[digits..].strip_prefix(['.', ')']))
                .flatten()
        })
        .map(str::trim_start);
    let boxed = item.is_some_and(|rest| {
        ["[ ]", "[x]", "[X]"]
            .iter()
            .any(|one| rest.starts_with(one))
    });
    !boxed && !bare.starts_with("<!--")
}

fn sections(text: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    let mut fenced = tisty_markdown::fences::fencing();
    for line in text.lines() {
        let code = fenced(line);
        if !code && let Some(name) = heading(line) {
            out.push(Section {
                name,
                words: String::new(),
            });
            continue;
        }
        if let Some(last) = out.last_mut()
            && (code || own_words(line))
            && !line.trim().is_empty()
        {
            last.words.push_str(line.trim());
            last.words.push('\n');
        }
    }
    out
}

/// A section keeps its name wherever it moves; only the names nobody kept were renamed, added or taken off.
pub fn paged(task: TaskId, before: &str, now: &str) -> Vec<Op> {
    let (was, is) = (sections(before), sections(now));
    let mut ops = Vec::new();
    let mut said = |section: &str, change: PageChange| {
        ops.push(Op::TaskPaged {
            id: task,
            d: Paged {
                section: section.to_string(),
                change,
            },
        });
    };
    let mut taken = vec![false; was.len()];
    let mut loose = Vec::new();
    for one in &is {
        match (0..was.len()).find(|at| !taken[*at] && was[*at].name == one.name) {
            Some(at) => {
                taken[at] = true;
                if was[at].words != one.words {
                    said(&one.name, PageChange::Written);
                }
            }
            None => loose.push(one),
        }
    }
    let mut gone = was
        .iter()
        .zip(&taken)
        .filter(|(_, kept)| !**kept)
        .map(|(one, _)| one);
    for one in loose {
        match gone.next() {
            Some(old) => {
                said(
                    &one.name,
                    PageChange::Renamed {
                        from: old.name.clone(),
                    },
                );
                if old.words != one.words {
                    said(&one.name, PageChange::Written);
                }
            }
            None => said(&one.name, PageChange::Added),
        }
    }
    for old in gone {
        said(&old.name, PageChange::Removed);
    }
    ops
}

/// Writing in a section for a while is one chapter of the trail, so it is one event in the log too.
pub const WRITING_FOR: i64 = 30 * 60;

/// A section already told as written stays untold while the writing goes on; each pause keeps it going.
pub fn quieted(
    ops: Vec<Op>,
    told: &mut std::collections::HashMap<(TaskId, String), i64>,
    now: i64,
) -> Vec<Op> {
    ops.into_iter()
        .filter(|op| match op {
            Op::TaskPaged { id, d } if matches!(d.change, PageChange::Written) => {
                let key = (*id, d.section.clone());
                let going = told.get(&key).is_some_and(|at| now - at < WRITING_FOR);
                told.insert(key, now);
                !going
            }
            _ => true,
        })
        .collect()
}

#[cfg(test)]
#[path = "paged_test.rs"]
mod tests;
