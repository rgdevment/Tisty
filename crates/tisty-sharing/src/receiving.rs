use serde_json::Value;

pub const CARRIED_CITED: &str = "folio-cited-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    pub title: Option<String>,
    pub body: String,
    pub pages: Vec<String>,
    pub owner_name: Option<String>,
    pub signed_as: Option<String>,
    pub carried: Option<String>,
}

// Nobody here holds what a document's other languages and cited documents are, only how readers reach them.
fn carried_from(document: &Value) -> Option<String> {
    let (languages, cited) = (&document["languages"], &document["cited"]);
    (!languages.is_null() || !cited.is_null()).then(|| {
        serde_json::json!({ "lang": document["lang"], "languages": languages, "cited": cited })
            .to_string()
    })
}

fn cited_places(document: &Value) -> Vec<(String, String)> {
    let many = document["cited"].as_array().map_or(0, Vec::len);
    (0..many)
        .rev()
        .map(|at| {
            (
                format!("tisty:cited/{at}"),
                format!("tisty:doc/{CARRIED_CITED}{at}"),
            )
        })
        .collect()
}

fn said(value: &Value, field: &str) -> Option<String> {
    value[field].as_str().map(str::to_owned)
}

fn listed(document: &Value, field: &str) -> Vec<(u64, String)> {
    let mut all: Vec<(u64, String)> = document[field]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|one| Some((one["at"].as_u64()?, one["name"].as_str()?.to_owned())))
        .collect();
    // The longest number goes first, so `tisty:picture/1` never eats the start of `tisty:picture/10`.
    all.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
    all
}

fn places_kept(
    document: &Value,
    field: &str,
    scheme: &str,
    parts: &[Vec<u8>],
    keep: &mut dyn FnMut(&str, &[u8]) -> Option<String>,
) -> Vec<(String, String)> {
    listed(document, field)
        .into_iter()
        .filter_map(|(at, name)| {
            let bytes = parts.get(usize::try_from(at).ok()?)?;
            Some((format!("{scheme}{at}"), keep(&name, bytes)?))
        })
        .collect()
}

fn local(body: &str, places: &[(String, String)]) -> String {
    places
        .iter()
        .fold(body.to_owned(), |text, (travelling, here)| {
            text.replace(travelling, here)
        })
}

pub fn received(
    document: &Value,
    pictures: &[Vec<u8>],
    files: &[Vec<u8>],
    keep: &mut dyn FnMut(&str, &[u8]) -> Option<String>,
) -> Received {
    let mut places = places_kept(document, "pictures", "tisty:picture/", pictures, keep);
    places.extend(places_kept(document, "files", "tisty:file/", files, keep));
    places.extend(cited_places(document));
    let body = document["body"].as_str().unwrap_or_default();
    let pages = document["pages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|page| page["body"].as_str())
        .map(|page| local(page, &places))
        .collect();
    Received {
        title: said(document, "title"),
        body: local(body, &places),
        pages,
        owner_name: said(document, "owner_name"),
        signed_as: said(document, "signed_as"),
        carried: carried_from(document),
    }
}

#[cfg(test)]
#[path = "receiving_test.rs"]
mod tests;
