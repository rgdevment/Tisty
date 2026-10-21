use std::path::Path;

use crate::Result;

// A card's title stops at this many characters, so a longer line only has to begin with it.
const TITLE_KEPT: usize = 120;

/// A page given a new heading shows it where its document lists it; the new body when it changed.
pub fn relabel_page(
    root: &Path,
    data: &Path,
    parent: &str,
    page: &str,
    title: &str,
) -> Result<Option<String>> {
    super::alone(root, || {
        let was = super::read(root, parent)?;
        let Some(now) = relabeled(&was, page, title) else {
            return Ok(None);
        };
        super::kept_before(data, parent, &was, &now)?;
        super::written(root, parent, &now)?;
        Ok(Some(now))
    })
}

/// The body with every card of `page` showing `title`, when the words it shows are another name.
pub fn relabeled(whole: &str, page: &str, title: &str) -> Option<String> {
    let named = crate::text::worded(title);
    let stale = crate::refs::paper_labels(whole)
        .into_iter()
        .any(|(file, said)| {
            file == page && another_name(&crate::text::worded(&said), &named, title)
        });
    if named.is_empty() || !stale {
        return None;
    }
    let target = format!("]({}{page})", crate::refs::DOC);
    let card = crate::refs::card(page, title);
    let mut changed = false;
    let lines: Vec<String> = whole
        .split_inclusive('\n')
        .map(|line| {
            let flat = line.trim_end_matches(['\n', '\r']);
            let bare = flat.trim();
            if !(bare.starts_with("![") && bare.ends_with(&target)) {
                return line.to_string();
            }
            changed = true;
            let indent = &flat[..flat.len() - flat.trim_start().len()];
            format!("{indent}{card}{}", &line[flat.len()..])
        })
        .collect();
    changed.then(|| lines.concat())
}

fn another_name(said: &str, named: &str, title: &str) -> bool {
    let cut_short = title.chars().count() >= TITLE_KEPT && said.starts_with(named);
    said != named && !cut_short
}

#[cfg(test)]
#[path = "relabel_test.rs"]
mod tests;
