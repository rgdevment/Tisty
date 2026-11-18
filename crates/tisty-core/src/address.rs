use crate::model::Kept;
use crate::state::State;

const DOC: &str = "tisty://doc";
const NAME_AT_MOST: usize = 48;

pub fn of_doc(state: &State, kept: &Kept) -> String {
    let book = kept.page_of.and_then(|up| state.docs.get(&up));
    let shelf = book.unwrap_or(kept).folder;
    let folders = shelf.map(|at| state.trail(at)).unwrap_or_default();
    let titles = [book, Some(kept)]
        .into_iter()
        .flatten()
        .filter_map(|one| one.title.as_deref());
    let mut out = String::from(DOC);
    for name in folders.into_iter().chain(titles) {
        let said = worded(name);
        if !said.is_empty() {
            out.push('/');
            out.push_str(&said);
        }
    }
    out.push_str("?id=");
    out.push_str(&kept.file);
    out
}

// Any page can fire an address, so only its id is read: the names before it are words for a reader.
pub fn doc_asked(said: &str) -> Option<&str> {
    let said = said.trim();
    let head = said.get(..DOC.len())?;
    if !head.eq_ignore_ascii_case(DOC) {
        return None;
    }
    let (words, id) = said[DOC.len()..].split_once("?id=")?;
    let plain = words.is_empty() || (words.starts_with('/') && !words.contains(['?', '#']));
    (plain && crate::docs::well_formed(id)).then_some(id)
}

fn worded(name: &str) -> String {
    let mut out = String::new();
    for one in crate::text::folded(name).chars() {
        if out.len() >= NAME_AT_MOST {
            break;
        }
        if one.is_ascii_alphanumeric() {
            out.push(one);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.truncate(out.trim_end_matches('-').len());
    out
}

#[cfg(test)]
#[path = "address_test.rs"]
mod tests;
