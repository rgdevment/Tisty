use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Deserialize;
use tisty_carrier::Seen;

pub(super) const FOLDER: &str = "application/vnd.google-apps.folder";
pub(super) const FILE_FIELDS: &str =
    "id,name,parents,mimeType,size,sha256Checksum,md5Checksum,headRevisionId,version,trashed";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct File {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub mime_type: String,
    pub size: Option<String>,
    pub sha256_checksum: Option<String>,
    pub md5_checksum: Option<String>,
    pub head_revision_id: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub trashed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Page {
    #[serde(default)]
    pub files: Vec<File>,
    pub next_page_token: Option<String>,
}

#[derive(Default, Clone)]
pub(super) struct Tree {
    pub folders: HashMap<String, String>,
    // Every folder by its id, a racing machine's second `docs` too, so what sits in either is found.
    pub folder_paths: HashMap<String, String>,
    pub files: BTreeMap<String, Vec<File>>,
}

pub(super) fn seen(name: &str, file: &File) -> Seen {
    Seen {
        name: name.to_string(),
        bytes: file
            .size
            .as_deref()
            .and_then(|one| one.parse().ok())
            .unwrap_or(0),
        hash: file
            .sha256_checksum
            .as_deref()
            .map(|one| format!("sha256:{one}"))
            .unwrap_or_default(),
        revision: format!("{}.{}", file.id, file.version.as_deref().unwrap_or("0")),
    }
}

pub(super) fn within(name: &str, under: &str) -> bool {
    let under = under.trim_end_matches('/');
    under.is_empty() || name.starts_with(&format!("{under}/"))
}

pub(super) fn split(name: &str) -> (Option<&str>, &str) {
    match name.rsplit_once('/') {
        Some((folders, leaf)) => (Some(folders), leaf),
        None => (None, name),
    }
}

// A root is a Tisty folder not inside another folder Tisty made, wherever the person moved it.
pub(super) fn roots_in(name: &str, all: &[File]) -> Vec<String> {
    let ours: std::collections::HashSet<&str> = all
        .iter()
        .filter(|one| one.mime_type == FOLDER)
        .map(|one| one.id.as_str())
        .collect();
    all.iter()
        .filter(|one| one.mime_type == FOLDER && one.name == name)
        .filter(|one| {
            one.parents
                .first()
                .is_none_or(|parent| !ours.contains(parent.as_str()))
        })
        .map(|one| one.id.clone())
        .collect()
}

// Every root Tisty folder counts, so what a racing first machine wrote in its own is still found.
pub(super) fn planted(roots: &[String], all: Vec<File>) -> Tree {
    let folder_of: HashMap<&str, (&str, Option<&str>)> = all
        .iter()
        .filter(|one| one.mime_type == FOLDER)
        .map(|one| {
            (
                one.id.as_str(),
                (one.name.as_str(), one.parents.first().map(String::as_str)),
            )
        })
        .collect();
    let mut tree = Tree::default();
    for one in &all {
        let Some(path) = path_of(
            &folder_of,
            roots,
            one.parents.first().map(String::as_str),
            &one.name,
        ) else {
            continue;
        };
        match one.mime_type == FOLDER {
            true => {
                tree.folder_paths.insert(one.id.clone(), path.clone());
                tree.folders.entry(path).or_insert_with(|| one.id.clone());
            }
            false => tree.files.entry(path).or_default().push(one.clone()),
        }
    }
    tree
}

fn path_of<'a>(
    folder_of: &HashMap<&'a str, (&'a str, Option<&'a str>)>,
    roots: &[String],
    mut parent: Option<&'a str>,
    leaf: &str,
) -> Option<String> {
    let mut parts = vec![leaf.to_string()];
    for _ in 0..64 {
        let id = parent?;
        if roots.iter().any(|root| root == id) {
            parts.reverse();
            return Some(parts.join("/"));
        }
        let (name, up) = folder_of.get(id)?;
        parts.push((*name).to_string());
        parent = *up;
    }
    None
}

#[derive(Clone, Deserialize)]
pub(super) struct Changed {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(default)]
    pub removed: bool,
    pub file: Option<File>,
}

impl Changed {
    pub fn standing(&self) -> Option<&File> {
        self.file
            .as_ref()
            .filter(|one| !self.removed && !one.trashed)
    }
}

pub(super) struct Applied {
    pub changed: Vec<String>,
    pub gone: Vec<String>,
}

// A folder renamed, moved or binned reshapes every path below it, which only a whole listing says.
pub(super) fn applied(
    tree: &Tree,
    roots: &[String],
    changes: &[Changed],
) -> Option<(Tree, Applied)> {
    let mut next = tree.clone();
    let (folders, files): (Vec<&Changed>, Vec<&Changed>) = changes.iter().partition(|one| {
        one.file
            .as_ref()
            .is_some_and(|file| file.mime_type == FOLDER)
            || next.folder_paths.contains_key(&one.file_id)
            || roots.contains(&one.file_id)
    });
    let mut waiting: Vec<&Changed> = Vec::new();
    for one in folders {
        let known = next.folder_paths.get(&one.file_id).cloned();
        let root = roots.contains(&one.file_id);
        match (known, one.standing()) {
            (Some(was), Some(file)) if path_in(&next, roots, file).as_ref() == Some(&was) => {}
            (None, Some(_)) if !root => waiting.push(one),
            (None, None) if !root => {}
            _ => return None,
        }
    }
    placed(&mut next, roots, waiting);
    let mut where_is: HashMap<String, String> = next
        .files
        .iter()
        .flat_map(|(path, copies)| {
            copies
                .iter()
                .map(move |copy| (copy.id.clone(), path.clone()))
        })
        .collect();
    let mut touched = BTreeSet::new();
    for one in files {
        let mut kept_at = None;
        if let Some(was) = where_is.remove(&one.file_id)
            && let Some(copies) = next.files.get_mut(&was)
            && let Some(at) = copies.iter().position(|copy| copy.id == one.file_id)
        {
            copies.remove(at);
            touched.insert(was.clone());
            kept_at = Some((was, at));
        }
        let Some(file) = one.standing() else {
            continue;
        };
        let Some(path) = path_in(&next, roots, file) else {
            continue;
        };
        let copies = next.files.entry(path.clone()).or_default();
        match kept_at {
            Some((was, at)) if was == path => copies.insert(at.min(copies.len()), file.clone()),
            _ => copies.push(file.clone()),
        }
        where_is.insert(file.id.clone(), path.clone());
        touched.insert(path);
    }
    let (changed, gone): (Vec<String>, Vec<String>) = touched.into_iter().partition(|path| {
        next.files
            .get(path)
            .is_some_and(|copies| !copies.is_empty())
    });
    for path in &gone {
        next.files.remove(path);
    }
    Some((next, Applied { changed, gone }))
}

// A folder can be listed before the folder it sits in, so they are placed until none is left to place.
fn placed(tree: &mut Tree, roots: &[String], mut waiting: Vec<&Changed>) {
    while !waiting.is_empty() {
        let before = waiting.len();
        waiting.retain(|one| {
            let Some(file) = one.standing() else {
                return false;
            };
            match path_in(tree, roots, file) {
                Some(path) => {
                    tree.folder_paths.insert(file.id.clone(), path.clone());
                    tree.folders.entry(path).or_insert_with(|| file.id.clone());
                    false
                }
                None => true,
            }
        });
        if waiting.len() == before {
            break;
        }
    }
}

fn path_in(tree: &Tree, roots: &[String], file: &File) -> Option<String> {
    let parent = file.parents.first()?;
    if roots.contains(parent) {
        return Some(file.name.clone());
    }
    tree.folder_paths
        .get(parent)
        .map(|path| format!("{path}/{}", file.name))
}
