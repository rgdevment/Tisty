use std::path::Path;

use crate::Result;
use crate::model::Kept;
use crate::state::State;

// A card's title stops at this many characters, so a longer line only has to begin with it.
const TITLE_KEPT: usize = 120;

/// The book whose line for `page` should change, read only when the page's heading did.
pub fn book_to_relabel<'a>(state: &'a State, page: &str, title: &str) -> Option<&'a Kept> {
    let kept = state.docs.values().find(|one| one.file == page)?;
    if kept.title.as_deref() == Some(title) {
        return None;
    }
    let up = state.docs.get(&kept.page_of?)?;
    // A compaction waiting for the person keeps its one undo; its line is mended once they decide.
    up.compacted.is_none().then_some(up)
}

/// No step back is kept: the line follows the page's heading, and one would cost an earlier undo.
pub fn relabel_page(root: &Path, parent: &str, page: &str, title: &str) -> Result<Option<String>> {
    super::alone(root, || {
        let was = super::read(root, parent)?;
        let Some(now) = relabeled(&was, page, title) else {
            return Ok(None);
        };
        super::written(root, parent, &now)?;
        Ok(Some(super::settled(&now)))
    })
}

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
    let mut fence = super::Fencing::default();
    let mut changed = false;
    let lines: Vec<String> = whole
        .split_inclusive('\n')
        .map(|line| {
            let flat = line.trim_end_matches(['\n', '\r']);
            let bare = flat.trim();
            // A card in code is an example, and a line holding more than this card keeps the rest.
            let alone = bare.starts_with("![")
                && bare.ends_with(&target)
                && bare.matches("](").count() == 1;
            if fence.inside(flat) || !alone {
                return line.to_string();
            }
            changed = true;
            let indent = &flat[..flat.len() - flat.trim_start().len()];
            format!("{indent}{card}{}", &line[flat.len()..])
        })
        .collect();
    let now = lines.concat();
    (changed && super::titled(&now) == super::titled(whole)).then_some(now)
}

pub fn another_name(said: &str, named: &str, title: &str) -> bool {
    let cut_short = title.chars().count() >= TITLE_KEPT && said.starts_with(named);
    said != named && !cut_short
}

#[cfg(test)]
#[path = "relabel_test.rs"]
mod tests;
