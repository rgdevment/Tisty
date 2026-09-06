use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    Error, Result,
    event::{DeviceId, DocAdd, FolderAdd, Op, Said},
    model::{DEEPEST, DocId, FolderId, Kept},
    state::State,
};

pub const EXTENSION: &str = "tistydoc";
const KIND: &str = "tisty-docs";
const VERSION: u32 = 1;
const MANIFEST: &str = "tisty-docs.json";
const CARRIED: [&str; 2] = ["docs", "attachments"];
const AT_MOST: u64 = 8 * 1024 * 1024 * 1024;
const AT_MOST_FILES: usize = 200_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub kind: String,
    pub version: u32,
    pub from: String,
    pub folders: Vec<Shelf>,
    pub docs: Vec<Paper>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shelf {
    pub id: String,
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paper {
    pub file: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrote: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub locked: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub done: usize,
    pub whole: usize,
    pub bytes: u64,
}

#[derive(Default)]
pub struct Along<'a> {
    pub also: Option<&'a Path>,
    pub say: Option<&'a dyn Fn(Step)>,
}

impl Along<'_> {
    fn at(&self, done: usize, whole: usize, bytes: u64) {
        if let Some(say) = self.say {
            say(Step { done, whole, bytes });
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Sent {
    pub docs: usize,
    pub pages: usize,
    pub folders: usize,
    pub files: usize,
    pub bytes: u64,
    pub missed: usize,
    pub left: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Landed {
    pub docs: usize,
    pub pages: usize,
    pub folders: usize,
    pub joined: usize,
    pub files: usize,
    pub missed: usize,
}

pub fn write(
    data: &Path,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
    if into.starts_with(data) || data.starts_with(into) {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }
    let papers = chosen(state, which);
    if papers.is_empty() {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }

    let made = filled(data, state, &papers, into, along);
    if made.is_err() {
        let _ = std::fs::remove_file(into);
    }
    made
}

fn filled(
    data: &Path,
    state: &State,
    papers: &[&Kept],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
    let root = data.join("docs");
    let mut sent = Sent::default();
    let mut bodies: Vec<(&Kept, String)> = Vec::new();
    for one in papers {
        let Ok(body) = crate::docs::read(&root, &one.file) else {
            continue;
        };
        bodies.push((one, body));
    }

    let mut beside: BTreeMap<String, PathBuf> = BTreeMap::new();
    for (_, body) in &bodies {
        for one in crate::refs::extract(body).into_iter().map(|one| one.target) {
            if !crate::attach::names_an_attachment(&one) || beside.contains_key(&one) {
                continue;
            }
            match crate::attach::found(&one, data, along.also) {
                Ok(from) if from.is_file() => {
                    beside.insert(one, from);
                }
                _ => {
                    if !sent.left.contains(&one) {
                        sent.left.push(one);
                    }
                }
            }
        }
    }

    let held: BTreeSet<&str> = bodies.iter().map(|(one, _)| one.file.as_str()).collect();
    let manifest = Manifest {
        kind: KIND.into(),
        version: VERSION,
        from: crate::store::peek_identity(data.join("store")).unwrap_or_default(),
        folders: shelves(state, &bodies),
        docs: bodies
            .iter()
            .map(|(one, body)| Paper {
                file: one.file.clone(),
                order: one.order.clone(),
                title: Some(crate::docs::titled(body)),
                folder: one.folder.map(|at| at.to_string()),
                page_of: one
                    .page_of
                    .and_then(|up| state.docs.get(&up))
                    .map(|up| up.file.clone())
                    .filter(|up| held.contains(up.as_str())),
                wrote: one.wrote,
                archived: one.archived,
                locked: one.locked,
            })
            .collect(),
    };

    let file = std::fs::File::create(into)?;
    let _ = crate::paths::ours_alone(into);
    let mut zip = zip::ZipWriter::new(file);
    let plain = zip::write::SimpleFileOptions::default();
    let kept = plain.compression_method(zip::CompressionMethod::Stored);

    zip.start_file(MANIFEST, plain).map_err(zipped)?;
    std::io::Write::write_all(
        &mut zip,
        serde_json::to_string_pretty(&manifest)?.as_bytes(),
    )?;

    let whole = bodies.len() + beside.len();
    for (one, body) in &bodies {
        zip.start_file(format!("docs/{}.md", one.file), plain)
            .map_err(zipped)?;
        std::io::Write::write_all(&mut zip, body.as_bytes())?;
        sent.bytes = sent.bytes.saturating_add(body.len() as u64);
        match one.page_of.is_some() {
            true => sent.pages += 1,
            false => sent.docs += 1,
        }
        along.at(sent.docs + sent.pages, whole, sent.bytes);
    }

    for (named, from) in &beside {
        let weighs = std::fs::metadata(from)?.len();
        sent.files += 1;
        sent.bytes = sent.bytes.saturating_add(weighs);
        if sent.bytes > AT_MOST || sent.files + bodies.len() > AT_MOST_FILES {
            return Err(Error::TooBig);
        }
        zip.start_file(named, kept).map_err(zipped)?;
        let mut file = std::fs::File::open(from)?;
        std::io::copy(&mut file, &mut zip)?;
        along.at(bodies.len() + sent.files, whole, sent.bytes);
    }

    sent.folders = manifest.folders.len();
    zip.finish().map_err(zipped)?;
    Ok(sent)
}

pub fn plainly(
    data: &Path,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
    if into.starts_with(data) || data.starts_with(into) {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }
    let papers = chosen(state, which);
    if papers.is_empty() {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }

    let mut sent = Sent::default();
    let mut shelves: BTreeSet<PathBuf> = BTreeSet::new();
    let whole = papers.iter().filter(|one| one.page_of.is_none()).count();
    for one in papers.iter().filter(|one| one.page_of.is_none()) {
        let under = trail(state, one.folder, into);
        std::fs::create_dir_all(&under)?;
        for at in under.ancestors().take_while(|at| *at != into) {
            shelves.insert(at.to_path_buf());
        }

        let pages: Vec<String> = state
            .pages_of(one.id)
            .iter()
            .map(|page| page.file.clone())
            .collect();
        let named = free(
            &under,
            &crate::docs::spelled(match one.title.as_deref() {
                Some(said) if !said.is_empty() => said,
                _ => one.file.as_str(),
            }),
        );
        let Ok(took) =
            crate::docs::laid_out_as(data, &one.file, &pages, &under, Some(&named), along.also)
        else {
            sent.missed += 1;
            continue;
        };
        sent.docs += 1;
        sent.missed += took.missed;
        sent.pages += pages.len() - took.missed;
        sent.files += took.files;
        along.at(sent.docs, whole, sent.bytes);
        for gone in took.left {
            left_behind(&mut sent.left, gone);
        }
    }
    sent.folders = shelves.len();
    Ok(sent)
}

fn trail(state: &State, folder: Option<FolderId>, into: &Path) -> PathBuf {
    let mut names: Vec<String> = Vec::new();
    let mut at = folder;
    while let Some(id) = at {
        let Some(one) = state.folders.get(&id) else {
            break;
        };
        names.push(crate::docs::spelled(&one.name));
        at = one.parent;
        if names.len() > DEEPEST {
            break;
        }
    }
    names
        .iter()
        .rev()
        .fold(into.to_path_buf(), |at, one| at.join(one))
}

fn free(under: &Path, named: &str) -> String {
    if !under.join(named).exists() {
        return named.to_string();
    }
    for n in 2..100 {
        let tried = format!("{named} {n}");
        if !under.join(&tried).exists() {
            return tried;
        }
    }
    named.to_string()
}

fn left_behind(left: &mut Vec<String>, one: String) {
    if !left.contains(&one) {
        left.push(one);
    }
}

fn chosen<'a>(state: &'a State, which: &[String]) -> Vec<&'a Kept> {
    let asked: BTreeSet<&str> = which.iter().map(String::as_str).collect();
    let mut found: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| {
            asked.is_empty()
                || asked.contains(one.file.as_str())
                || one.page_of.is_some_and(|up| {
                    state
                        .docs
                        .get(&up)
                        .is_some_and(|up| asked.contains(up.file.as_str()))
                })
        })
        .collect();
    found.sort_by(|a, b| {
        a.page_of
            .is_some()
            .cmp(&b.page_of.is_some())
            .then(a.order.cmp(&b.order))
            .then(a.id.cmp(&b.id))
    });
    found
}

fn shelves(state: &State, bodies: &[(&Kept, String)]) -> Vec<Shelf> {
    let mut wanted: BTreeSet<FolderId> = BTreeSet::new();
    for (one, _) in bodies {
        let mut at = one.folder;
        while let Some(id) = at {
            if !wanted.insert(id) {
                break;
            }
            at = state.folders.get(&id).and_then(|one| one.parent);
        }
    }

    let mut found: Vec<Shelf> = wanted
        .iter()
        .filter_map(|id| state.folders.get(id))
        .map(|one| Shelf {
            id: one.id.to_string(),
            name: one.name.clone(),
            order: one.order.clone(),
            parent: one
                .parent
                .filter(|up| wanted.contains(up))
                .map(|up| up.to_string()),
            icon: one.icon.clone(),
            color: one.color.clone(),
        })
        .collect();
    found.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    found
}

pub fn read(
    data: &Path,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
) -> Result<(Landed, Vec<Op>)> {
    let file = std::fs::File::open(from)?;
    let mut zip = zip::ZipArchive::new(file).map_err(zipped)?;
    let manifest = manifest_in(&mut zip, from)?;

    let staged = data.join(format!(".landing-{}", std::process::id()));
    swept(data);
    let whole = zip.len() + manifest.docs.len();
    let done = unpack(&mut zip, &staged, along, whole)
        .and_then(|_| taken_in(data, state, device, &manifest, &staged, along, whole));
    let _ = std::fs::remove_dir_all(&staged);
    done
}

fn swept(data: &Path) {
    let Ok(entries) = std::fs::read_dir(data) else {
        return;
    };
    for at in entries.filter_map(|one| one.ok()).map(|one| one.path()) {
        let stale = at.is_dir()
            && at
                .file_name()
                .and_then(|one| one.to_str())
                .is_some_and(|one| one.starts_with(".landing-"));
        if stale && std::fs::remove_dir_all(&at).is_err() {
            crate::witness::warn(
                crate::witness::channel::BACKUP,
                "what an interrupted landing left behind could not be swept",
                &[("at", crate::witness::Fact::Path(at.clone()))],
            );
        }
    }
}

fn manifest_in<R: Read + Seek>(zip: &mut zip::ZipArchive<R>, from: &Path) -> Result<Manifest> {
    let mut held = zip
        .by_name(MANIFEST)
        .map_err(|_| Error::NotForAnAgent(from.display().to_string()))?;
    let mut said = String::new();
    held.read_to_string(&mut said)?;
    let manifest: Manifest = serde_json::from_str(&said)
        .map_err(|_| Error::NotForAnAgent(from.display().to_string()))?;
    if manifest.kind != KIND {
        return Err(Error::NotForAnAgent(from.display().to_string()));
    }
    if manifest.version > VERSION {
        return Err(Error::UnsupportedVersion(manifest.version));
    }
    Ok(manifest)
}

fn taken_in(
    data: &Path,
    state: &State,
    device: &DeviceId,
    manifest: &Manifest,
    staged: &Path,
    along: &Along,
    whole: usize,
) -> Result<(Landed, Vec<Op>)> {
    let mut landed = Landed::default();
    let mut ops: Vec<Op> = Vec::new();

    let filed = shelved(state, manifest, &mut landed, &mut ops);

    let root = data.join("docs");
    let mut carried: BTreeMap<String, String> = BTreeMap::new();
    let mut named: BTreeMap<String, (String, DocId)> = BTreeMap::new();
    let mut ordered: Vec<(Option<FolderId>, String)> = Vec::new();
    let mut written: Vec<(String, String)> = Vec::new();

    for paper in ordering(manifest) {
        let Ok(body) =
            std::fs::read_to_string(staged.join("docs").join(format!("{}.md", paper.file)))
        else {
            landed.missed += 1;
            continue;
        };
        let body = brought(data, staged, &body, &mut carried, &mut landed);

        let up = paper
            .page_of
            .as_ref()
            .and_then(|file| named.get(file))
            .map(|(_, id)| *id);
        if paper.page_of.is_some() && up.is_none() {
            landed.missed += 1;
            continue;
        }
        let folder = match up {
            Some(_) => None,
            None => paper.folder.as_ref().and_then(|at| filed.get(at)).copied(),
        };

        let made = crate::docs::create(&root, device, &body)?;
        let id = Ulid::generate();
        named.insert(paper.file.clone(), (made.id.clone(), id));

        let order = crate::order::last_of(
            state
                .docs
                .values()
                .filter(|one| one.folder == folder && one.page_of.is_none())
                .map(|one| one.order.as_str())
                .chain(
                    ordered
                        .iter()
                        .filter(|(at, _)| *at == folder)
                        .map(|(_, key)| key.as_str()),
                ),
        );
        ordered.push((folder, order.clone()));

        ops.push(Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id.clone(),
                order,
                said: Some(Said {
                    title: made.title.clone(),
                    bytes: Some(body.len() as u64),
                    tags: Some(crate::tagging::tags_in(&body)),
                }),
                folder,
                page_of: up,
            },
        });
        if paper.archived {
            ops.push(Op::DocArchive { id });
        }
        if paper.locked {
            ops.push(Op::DocLock { id });
        }
        match up.is_some() {
            true => landed.pages += 1,
            false => landed.docs += 1,
        }
        along.at(
            whole - manifest.docs.len() + landed.docs + landed.pages,
            whole,
            0,
        );
        written.push((made.id, body));
    }

    for (file, body) in written {
        let told = pointed(&body, &named);
        if told != body {
            crate::docs::write(&root, &file, &told)?;
        }
    }

    Ok((landed, ops))
}

fn ordering(manifest: &Manifest) -> Vec<&Paper> {
    let mut found: Vec<&Paper> = manifest
        .docs
        .iter()
        .filter(|one| one.page_of.is_none())
        .collect();
    for one in &manifest.docs {
        if one.page_of.is_some() {
            found.push(one);
        }
    }
    found
}

fn shelved(
    state: &State,
    manifest: &Manifest,
    landed: &mut Landed,
    ops: &mut Vec<Op>,
) -> BTreeMap<String, FolderId> {
    let mut filed: BTreeMap<String, FolderId> = BTreeMap::new();
    let mut deep: BTreeMap<FolderId, usize> = BTreeMap::new();
    let mut fresh: Vec<(FolderId, Option<FolderId>, String, String)> = Vec::new();

    for shelf in downwards(manifest) {
        let parent = shelf.parent.as_ref().and_then(|up| filed.get(up)).copied();
        let under = match parent {
            Some(at) => deep
                .get(&at)
                .copied()
                .unwrap_or_else(|| state.depth(Some(at))),
            None => 0,
        };
        if under >= DEEPEST {
            if let Some(at) = parent {
                filed.insert(shelf.id.clone(), at);
            }
            continue;
        }

        let standing = state
            .folders
            .values()
            .find(|one| one.parent == parent && alike(&one.name, &shelf.name))
            .map(|one| one.id)
            .or_else(|| {
                fresh
                    .iter()
                    .find(|(_, up, name, _)| *up == parent && alike(name, &shelf.name))
                    .map(|(id, ..)| *id)
            });
        if let Some(id) = standing {
            landed.joined += 1;
            deep.insert(id, under + 1);
            filed.insert(shelf.id.clone(), id);
            continue;
        }

        let mut keys: Vec<String> = state
            .under(parent)
            .iter()
            .map(|one| one.order.clone())
            .collect();
        keys.extend(
            fresh
                .iter()
                .filter(|(_, up, _, _)| *up == parent)
                .map(|(_, _, _, key)| key.clone()),
        );
        let order = crate::order::last_of(keys.iter().map(String::as_str));

        let id = Ulid::generate();
        ops.push(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: shelf.name.clone(),
                order: order.clone(),
                parent,
                icon: shelf
                    .icon
                    .clone()
                    .filter(|one| crate::model::icon::known(one)),
                color: shelf
                    .color
                    .as_ref()
                    .and_then(|one| crate::model::hue::kept(one))
                    .map(str::to_string),
            },
        });
        fresh.push((id, parent, shelf.name.clone(), order));
        deep.insert(id, under + 1);
        filed.insert(shelf.id.clone(), id);
        landed.folders += 1;
    }
    filed
}

fn downwards(manifest: &Manifest) -> Vec<&Shelf> {
    let mut found: Vec<&Shelf> = Vec::new();
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut left: Vec<&Shelf> = manifest.folders.iter().collect();
    while !left.is_empty() {
        let (ready, waiting): (Vec<&Shelf>, Vec<&Shelf>) = left.into_iter().partition(|one| {
            one.parent
                .as_ref()
                .is_none_or(|up| done.contains(up.as_str()))
        });
        if ready.is_empty() {
            found.extend(waiting);
            break;
        }
        for one in &ready {
            done.insert(one.id.as_str());
        }
        found.extend(ready);
        left = waiting;
    }
    found
}

fn alike(one: &str, other: &str) -> bool {
    crate::text::composed(one.trim()).to_lowercase()
        == crate::text::composed(other.trim()).to_lowercase()
}

fn brought(
    data: &Path,
    staged: &Path,
    body: &str,
    carried: &mut BTreeMap<String, String>,
    landed: &mut Landed,
) -> String {
    let mut told = body.to_string();
    for one in crate::refs::extract(body).into_iter().map(|one| one.target) {
        if !crate::attach::names_an_attachment(&one) {
            continue;
        }
        if !carried.contains_key(&one) {
            let Some(at) = safe(&one) else {
                continue;
            };
            let from = staged.join(at);
            if !from.is_file() {
                landed.missed += 1;
                continue;
            }
            let Ok(kept) = crate::attach::keep(&from, data, crate::attach::COPIED_IN_DOC) else {
                landed.missed += 1;
                continue;
            };
            landed.files += 1;
            carried.insert(one.clone(), kept.at);
        }
        if let Some(now) = carried.get(&one)
            && *now != one
        {
            told = told.replace(&one, now);
        }
    }
    told
}

fn pointed(body: &str, named: &BTreeMap<String, (String, DocId)>) -> String {
    let mut told = body.to_string();
    for file in crate::refs::papers(body) {
        if let Some((now, _)) = named.get(&file) {
            told = told.replace(
                &format!("{}{file}", crate::refs::DOC),
                &format!("{}{now}", crate::refs::DOC),
            );
        }
    }
    told
}

fn unpack<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    into: &Path,
    along: &Along,
    whole: usize,
) -> Result<usize> {
    if zip.len() > AT_MOST_FILES {
        return Err(Error::TooBig);
    }
    let mut files = 0;
    let mut bytes = 0u64;

    for i in 0..zip.len() {
        let mut held = zip.by_index(i).map_err(zipped)?;
        if held.is_dir() {
            continue;
        }
        let Some(rest) = safe(held.name()) else {
            continue;
        };
        if held.size() > AT_MOST.saturating_sub(bytes) {
            return Err(Error::TooBig);
        }

        let at = into.join(&rest);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(&at)?;
        let _ = crate::paths::ours_alone(&at);
        let room = AT_MOST.saturating_sub(bytes).saturating_add(1);
        let written = std::io::copy(&mut held.by_ref().take(room), &mut file)?;
        if written >= room {
            return Err(Error::TooBig);
        }
        bytes = bytes.saturating_add(written);
        files += 1;
        along.at(i + 1, whole, bytes);
    }
    Ok(files)
}

fn safe(named: &str) -> Option<PathBuf> {
    let at = Path::new(named);
    if !at
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let head = at.components().next()?.as_os_str().to_str()?;
    CARRIED.contains(&head).then(|| at.to_path_buf())
}

fn zipped(e: zip::result::ZipError) -> Error {
    Error::Io(std::io::Error::other(e.to_string()))
}
