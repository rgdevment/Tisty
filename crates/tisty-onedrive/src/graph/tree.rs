use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use tisty_carrier::Seen;

pub(super) const ITEM_FIELDS: &str = "id,name,eTag,cTag,size,file,folder,deleted,parentReference";
// A loop in what the feed says is never followed forever: no path Tisty writes goes this deep.
const DEEPEST: usize = 64;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct DriveItem {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "eTag")]
    pub etag: Option<String>,
    #[serde(default, rename = "cTag")]
    pub ctag: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub file: Option<FileFacet>,
    #[serde(default)]
    pub folder: Option<serde_json::Value>,
    #[serde(default)]
    pub deleted: Option<serde_json::Value>,
    #[serde(default, rename = "parentReference")]
    pub parent: Option<ParentReference>,
    #[serde(default, rename = "@microsoft.graph.downloadUrl")]
    pub download: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct FileFacet {
    #[serde(default)]
    pub hashes: Option<Hashes>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct Hashes {
    #[serde(default, rename = "quickXorHash")]
    pub quick_xor: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct ParentReference {
    #[serde(default)]
    pub id: Option<String>,
}

impl DriveItem {
    pub fn seen(&self, name: &str) -> Seen {
        Seen {
            name: name.to_string(),
            bytes: self.size.unwrap_or(0),
            hash: self.hash(),
            revision: self.revision(),
        }
    }

    // The content's own tag stays put through a rename or OneDrive's own touch-ups, which change the eTag.
    pub fn revision(&self) -> String {
        self.ctag
            .clone()
            .or_else(|| self.etag.clone())
            .unwrap_or_default()
    }

    // Printed as `hash_of` prints a file here, so the two are compared as they are.
    fn hash(&self) -> String {
        self.file
            .as_ref()
            .and_then(|file| file.hashes.as_ref())
            .and_then(|hashes| hashes.quick_xor.as_deref())
            .map(|one| format!("quickxor:{one}"))
            .unwrap_or_default()
    }

    pub fn is_file(&self) -> bool {
        self.file.is_some() && self.folder.is_none()
    }
}

#[derive(Debug, Clone)]
struct Kept {
    name: String,
    parent: Option<String>,
    item: Option<DriveItem>,
}

/// Everything under the app's folder, by id: the feed of changes never says a path.
#[derive(Debug, Clone, Default)]
pub(super) struct Tree {
    root: String,
    items: HashMap<String, Kept>,
}

impl Tree {
    pub fn under(root: &str) -> Self {
        Self {
            root: root.to_string(),
            items: HashMap::new(),
        }
    }

    // The feed shows each item as it stands now, and the last time it names one is the one that counts.
    pub fn applied(&mut self, items: Vec<DriveItem>) {
        for one in items {
            if one.id == self.root {
                continue;
            }
            if one.deleted.is_some() {
                self.items.remove(&one.id);
                continue;
            }
            let Some(name) = one.name.clone() else {
                continue;
            };
            let parent = one.parent.as_ref().and_then(|parent| parent.id.clone());
            let item = one.is_file().then_some(one.clone());
            self.items.insert(one.id, Kept { name, parent, item });
        }
    }

    fn path_of(&self, id: &str) -> Option<String> {
        let mut parts = Vec::new();
        let mut at = id;
        for _ in 0..DEEPEST {
            if at == self.root {
                parts.reverse();
                return Some(parts.join("/"));
            }
            let kept = self.items.get(at)?;
            parts.push(kept.name.as_str());
            at = kept.parent.as_deref()?;
        }
        None
    }

    /// Only files whose every folder up to the app's own is known; one cut off from it is not there.
    pub fn files(&self) -> BTreeMap<String, Seen> {
        self.items
            .iter()
            .filter_map(|(id, kept)| {
                let item = kept.item.as_ref()?;
                let path = self.path_of(id)?;
                Some((path.clone(), item.seen(&path)))
            })
            .collect()
    }
}

pub(super) fn within(path: &str, under: &str) -> bool {
    under.is_empty()
        || path
            .strip_prefix(under)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

pub(super) fn moved(
    before: &BTreeMap<String, Seen>,
    after: &BTreeMap<String, Seen>,
) -> (Vec<Seen>, Vec<String>) {
    let changed = after
        .iter()
        .filter(|(path, seen)| before.get(*path) != Some(seen))
        .map(|(_, seen)| seen.clone())
        .collect();
    let gone = before
        .keys()
        .filter(|path| !after.contains_key(*path))
        .cloned()
        .collect();
    (changed, gone)
}

#[cfg(test)]
#[path = "tree_test.rs"]
mod tests;
