use tisty_carrier::Hitch;

const DAV: &str = "DAV:";

/// What PROPFIND asks for: only what tells a file apart and the room left, never who owns it or when it was made.
pub(super) const ASKED: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/><d:getcontentlength/><d:getetag/><d:getlastmodified/><d:quota-available-bytes/></d:prop></d:propfind>"#;

/// One thing a server answered for, its properties read only from the part it answered 200.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Found {
    pub(super) href: String,
    pub(super) collection: bool,
    pub(super) bytes: Option<u64>,
    pub(super) etag: Option<String>,
    pub(super) modified: Option<String>,
    pub(super) room: Option<i64>,
}

impl Found {
    /// Its ETag as given, or, from a server that keeps none, its size and date: a revision either way.
    pub(super) fn revision(&self) -> String {
        match &self.etag {
            Some(etag) => etag.clone(),
            None => format!(
                "modified:{}:{}",
                self.modified.as_deref().unwrap_or_default(),
                self.bytes.unwrap_or(0)
            ),
        }
    }
}

pub(super) fn read(body: &str) -> Result<Vec<Found>, Hitch> {
    let shape = || Hitch::Broke("the WebDAV server answered in a shape Tisty does not read".into());
    let document = roxmltree::Document::parse(body).map_err(|_| shape())?;
    let root = document.root_element();
    if !is(root, "multistatus") {
        return Err(shape());
    }
    Ok(root
        .children()
        .filter(|one| is(*one, "response"))
        .filter_map(found)
        .collect())
}

fn found(response: roxmltree::Node) -> Option<Found> {
    let href = child(response, "href")?.text()?.trim().to_string();
    let mut found = Found {
        href,
        ..Found::default()
    };
    for propstat in response.children().filter(|one| is(*one, "propstat")) {
        let fine = child(propstat, "status")
            .and_then(|one| one.text())
            .is_some_and(|status| status.split_whitespace().nth(1) == Some("200"));
        let Some(prop) = child(propstat, "prop").filter(|_| fine) else {
            continue;
        };
        for one in prop.children().filter(roxmltree::Node::is_element) {
            let text = || one.text().map(str::trim).filter(|said| !said.is_empty());
            match one.tag_name().name() {
                "resourcetype" => found.collection = child(one, "collection").is_some(),
                "getcontentlength" => found.bytes = text().and_then(|n| n.parse().ok()),
                "getetag" => found.etag = text().map(str::to_string),
                "getlastmodified" => found.modified = text().map(str::to_string),
                "quota-available-bytes" => found.room = text().and_then(|n| n.parse().ok()),
                _ => {}
            }
        }
    }
    Some(found)
}

fn is(node: roxmltree::Node, name: &str) -> bool {
    node.is_element() && node.tag_name().name() == name && node.tag_name().namespace() == Some(DAV)
}

fn child<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    name: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    node.children().find(|one| is(*one, name))
}

#[cfg(test)]
#[path = "multistatus_test.rs"]
mod tests;
