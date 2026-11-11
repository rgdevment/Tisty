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
    let (languages, cited, editors, reviews) = (
        &document["languages"],
        &document["cited"],
        &document["editors"],
        &document["reviews"],
    );
    let any = [languages, cited, editors, reviews];
    any.iter().any(|one| !one.is_null()).then(|| {
        serde_json::json!({
            "lang": document["lang"],
            "languages": languages,
            "cited": cited,
            "editors": editors,
            "reviews": reviews,
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

/// What a later version says of its languages, cited documents and reviews replaces what an editor held, or its next version would undo the owner's.
pub fn map_brought(was: Option<&str>, arrived: Option<&str>) -> Option<String> {
    let arrived = arrived.and_then(|said| serde_json::from_str::<Value>(said).ok());
    let mut kept = was
        .and_then(|said| serde_json::from_str::<Value>(said).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    let fields = kept.as_object_mut()?;
    for field in ["lang", "languages", "cited", "reviews"] {
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

fn object_of(said: Option<&str>) -> Value {
    said.and_then(|said| serde_json::from_str::<Value>(said).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}))
}

fn written(kept: &Value) -> Option<String> {
    kept.as_object()
        .filter(|fields| !fields.is_empty())
        .map(|_| kept.to_string())
}

/// The owner keeps the map it holds and takes in by id the languages others added, never by lang, or its next version would drop them (166).
pub fn languages_joined(was: Option<&str>, arrived: Option<&str>) -> Option<String> {
    let mut kept = object_of(was);
    let theirs = object_of(arrived)["languages"].as_array().cloned();
    let Some(theirs) = theirs else {
        return written(&kept);
    };
    let mut joined = kept["languages"].as_array().cloned().unwrap_or_default();
    for one in theirs {
        if joined
            .iter()
            .all(|held| held["document"] != one["document"])
        {
            joined.push(one);
        }
    }
    kept["languages"] = Value::Array(joined);
    written(&kept)
}

pub fn with_reviews(was: Option<&str>, reviews: &[tisty_contracts::Review]) -> Option<String> {
    let mut kept = object_of(was);
    let fields = kept.as_object_mut()?;
    match serde_json::to_value(reviews) {
        Ok(said) if !reviews.is_empty() => fields.insert("reviews".to_owned(), said),
        _ => fields.remove("reviews"),
    };
    written(&kept)
}

pub fn reviews_carried(said: Option<&str>) -> Vec<tisty_contracts::Review> {
    serde_json::from_value(object_of(said)["reviews"].clone()).unwrap_or_default()
}

pub fn languages_gone(
    was: Option<&str>,
    gone: &std::collections::BTreeSet<String>,
    gone_of: &std::collections::BTreeSet<String>,
) -> Option<String> {
    let mut kept = object_of(was);
    let stays = |one: &Value, field: &str, out: &std::collections::BTreeSet<String>| {
        one[field].as_str().is_none_or(|said| !out.contains(said))
    };
    if let Some(languages) = kept["languages"].as_array_mut() {
        languages.retain(|one| stays(one, "document", gone));
    }
    if let Some(aside) = kept.get_mut(SET_ASIDE).and_then(Value::as_array_mut) {
        aside.retain(|one| stays(one, "document", gone));
    }
    if let Some(reviews) = kept["reviews"].as_array_mut() {
        reviews.retain(|one| stays(one, "of", gone_of));
    }
    written(&kept)
}

const SET_ASIDE: &str = "set_aside";

// What its owner left out leaves the map, and only the owner's log keeps how to reach it again; no version carries it (166).
pub fn set_aside(was: Option<&str>, document: &str) -> Option<String> {
    moved(was, document, "languages", SET_ASIDE)
}

pub fn brought_back(was: Option<&str>, document: &str) -> Option<String> {
    moved(was, document, SET_ASIDE, "languages")
}

pub fn languages_set_aside(said: Option<&str>) -> Vec<tisty_contracts::Translation> {
    each_alone(&object_of(said)[SET_ASIDE])
}

// One entry that does not read leaves only itself out, never what the others say.
fn each_alone<T: serde::de::DeserializeOwned>(said: &Value) -> Vec<T> {
    said.as_array()
        .into_iter()
        .flatten()
        .filter_map(|one| serde_json::from_value(one.clone()).ok())
        .collect()
}

pub struct Languages {
    pub map: Vec<tisty_contracts::Translation>,
    pub set_aside: Vec<tisty_contracts::Translation>,
    pub reviews: Vec<tisty_contracts::Review>,
}

pub fn languages_carried(said: Option<&str>) -> Languages {
    let said = object_of(said);
    Languages {
        map: serde_json::from_value(said["languages"].clone()).unwrap_or_default(),
        set_aside: each_alone(&said[SET_ASIDE]),
        reviews: serde_json::from_value(said["reviews"].clone()).unwrap_or_default(),
    }
}

fn moved(was: Option<&str>, document: &str, from: &str, to: &str) -> Option<String> {
    let mut kept = object_of(was);
    let fields = kept.as_object_mut()?;
    let names = |one: &Value| one["document"].as_str() == Some(document);
    let mut there: Vec<Value> = fields
        .remove(from)
        .and_then(|said| serde_json::from_value(said).ok())
        .unwrap_or_default();
    let taken = there.iter().position(names).map(|at| there.remove(at));
    if !there.is_empty() {
        fields.insert(from.to_owned(), Value::Array(there));
    }
    if let Some(one) = taken {
        let mut landing: Vec<Value> = fields
            .remove(to)
            .and_then(|said| serde_json::from_value(said).ok())
            .unwrap_or_default();
        landing.retain(|held| !names(held));
        landing.push(one);
        fields.insert(to.to_owned(), Value::Array(landing));
    }
    written(&kept)
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
