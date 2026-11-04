use serde_json::{Value, json};
use tisty_contracts::check;

const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';
const ZERO_WIDTH_JOINER: char = '\u{200D}';
const LINE_AT_MOST: usize = 8192;
const NAME_AT_MOST: usize = 80;

fn holds(contract: &str, value: &Value) -> bool {
    check(contract, value).is_empty()
}

fn is_a_line(said: &str) -> bool {
    holds("cited", &json!({ "document": "x", "title": said }))
}

fn on_one_line(said: &str) -> String {
    said.split_whitespace().collect::<Vec<_>>().join(" ")
}

// The contract's own check decides what is unseen, so this can never keep what sealing would refuse.
fn seen_only(said: &str) -> String {
    let points: Vec<char> = said.chars().collect();
    let mut out = String::with_capacity(said.len());
    for (at, one) in points.iter().enumerate() {
        let stays = if matches!(*one, ZERO_WIDTH_NON_JOINER | ZERO_WIDTH_JOINER) {
            let before = at.checked_sub(1).and_then(|was| points.get(was));
            let among: String = before
                .into_iter()
                .chain([one])
                .chain(points.get(at + 1))
                .collect();
            is_a_line(&among)
        } else {
            is_a_line(one.encode_utf8(&mut [0; 4]))
        };
        if stays {
            out.push(*one);
        }
    }
    out
}

pub fn line(said: &str) -> Option<String> {
    let flat: String = on_one_line(said).chars().take(LINE_AT_MOST).collect();
    let mut seen = if is_a_line(&flat) {
        flat
    } else {
        on_one_line(&seen_only(&flat))
    };
    if !is_a_line(&seen) {
        seen.retain(|one| !matches!(one, ZERO_WIDTH_NON_JOINER | ZERO_WIDTH_JOINER));
    }
    let kept = seen.trim();
    (!kept.is_empty()).then(|| kept.to_owned())
}

pub fn file_name(said: &str) -> Option<String> {
    let last = said.rsplit(['/', '\\']).next().unwrap_or_default();
    line(last).filter(|one| one.chars().any(|point| point != '.'))
}

pub fn name(said: &str) -> Option<String> {
    let kept: String = line(said)?.chars().take(NAME_AT_MOST).collect();
    let kept = kept.trim_end().to_owned();
    let document = json!({ "id": "x", "html": "", "body": "", "signed_as": kept });
    holds("document", &document).then_some(kept)
}

pub fn is_a_tag(said: &str) -> bool {
    holds("tag", &json!({ "name": said }))
}

#[cfg(test)]
#[path = "clean_test.rs"]
mod tests;
