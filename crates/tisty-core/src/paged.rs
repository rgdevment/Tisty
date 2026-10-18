use crate::event::{Op, PageChange, Paged};
use crate::model::TaskId;

struct Section {
    name: String,
    words: String,
}

fn heading(line: &str) -> Option<String> {
    let bare = line.trim_start();
    let marks = bare.chars().take_while(|one| *one == '#').count();
    let rest = bare.get(marks..)?;
    ((1..=6).contains(&marks) && rest.starts_with([' ', '\t']))
        .then(|| rest.trim().trim_end_matches('#').trim().to_string())
        .filter(|name| !name.is_empty())
}

// A box is a step, and a step leaves its own trail; a hint is the template's, never the person's.
fn own_words(line: &str) -> bool {
    let bare = line.trim_start_matches(['>', ' ', '\t']);
    let boxed = ["- [", "* [", "+ ["]
        .iter()
        .any(|lead| bare.starts_with(lead))
        || bare.split_once(['.', ')']).is_some_and(|(digits, rest)| {
            !digits.is_empty()
                && digits.chars().all(|one| one.is_ascii_digit())
                && rest.trim_start().starts_with('[')
        });
    !boxed && !bare.starts_with("<!--")
}

fn sections(text: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    let mut fence: Option<&str> = None;
    for line in text.lines() {
        let mark = ["```", "~~~"]
            .into_iter()
            .find(|one| line.trim_start().starts_with(one));
        if let Some(mark) = mark {
            fence = match fence {
                None => Some(mark),
                Some(open) if open == mark => None,
                open => open,
            };
        } else if fence.is_none()
            && let Some(name) = heading(line)
        {
            out.push(Section {
                name,
                words: String::new(),
            });
            continue;
        }
        if let Some(last) = out.last_mut()
            && own_words(line)
            && !line.trim().is_empty()
        {
            last.words.push_str(line.trim());
            last.words.push('\n');
        }
    }
    out
}

/// What changed in a page's sections between two readings of it, for the task's trail.
pub fn paged(task: TaskId, before: &str, now: &str) -> Vec<Op> {
    let (was, is) = (sections(before), sections(now));
    let names = |all: &[Section]| all.iter().map(|one| one.name.clone()).collect::<Vec<_>>();
    let anchors = crate::lcs::pairs(&names(&was), &names(&is));
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
    let starts = std::iter::once((0, 0)).chain(anchors.iter().map(|(a, b)| (a + 1, b + 1)));
    let ends = anchors
        .iter()
        .copied()
        .chain(std::iter::once((was.len(), is.len())));
    // Between two sections that kept their names, one renamed reads as such, the rest as added or gone.
    for ((was_at, is_at), (was_end, is_end)) in starts.zip(ends) {
        let gone = &was[was_at..was_end];
        let came = &is[is_at..is_end];
        for (at, one) in came.iter().enumerate() {
            match gone.get(at) {
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
        for old in gone.iter().skip(came.len()) {
            said(&old.name, PageChange::Removed);
        }
    }
    for (old, one) in anchors {
        if was[old].words != is[one].words {
            said(&is[one].name, PageChange::Written);
        }
    }
    ops
}

#[cfg(test)]
#[path = "paged_test.rs"]
mod tests;
