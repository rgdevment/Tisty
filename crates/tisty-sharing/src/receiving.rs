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
    let (languages, cited, editors) = (
        &document["languages"],
        &document["cited"],
        &document["editors"],
    );
    (!languages.is_null() || !cited.is_null() || !editors.is_null()).then(|| {
        serde_json::json!({
            "lang": document["lang"],
            "languages": languages,
            "cited": cited,
            "editors": editors,
        })
        .to_string()
    })
}

/// Another's version brings who edited it along, and only that: what this copy carries of languages and citing stays its own.
pub fn editors_brought(was: Option<&str>, arrived: Option<&str>) -> Option<String> {
    let editors = arrived
        .and_then(|said| serde_json::from_str::<Value>(said).ok())
        .map(|said| said["editors"].clone());
    with_editors(was, editors)
}

/// What a later version says of its languages and cited documents replaces what an editor held, or its next version would undo the owner's.
pub fn map_brought(was: Option<&str>, arrived: Option<&str>) -> Option<String> {
    let arrived = arrived.and_then(|said| serde_json::from_str::<Value>(said).ok());
    let mut kept = was
        .and_then(|said| serde_json::from_str::<Value>(said).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    let fields = kept.as_object_mut()?;
    for field in ["lang", "languages", "cited"] {
        match arrived
            .as_ref()
            .map(|one| one[field].clone())
            .filter(|one| !one.is_null())
        {
            Some(said) => fields.insert(field.to_owned(), said),
            None => fields.remove(field),
        };
    }
    (!fields.is_empty()).then(|| kept.to_string())
}

pub(crate) fn with_editors(was: Option<&str>, editors: Option<Value>) -> Option<String> {
    let editors = editors.filter(|said| !said.is_null());
    let mut kept = was
        .and_then(|said| serde_json::from_str::<Value>(said).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    match editors {
        Some(editors) => kept["editors"] = editors,
        None => return was.map(str::to_owned),
    }
    Some(kept.to_string())
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
