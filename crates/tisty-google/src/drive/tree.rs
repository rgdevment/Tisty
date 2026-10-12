use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use tisty_carrier::Seen;

pub(super) const FOLDER: &str = "application/vnd.google-apps.folder";
pub(super) const FILE_FIELDS: &str = "id,name,parents,mimeType,size,sha256Checksum,version,trashed";

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Default)]
pub(super) struct Tree {
    pub folders: HashMap<String, String>,
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
