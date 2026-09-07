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

pub const EXTENSION: &str = "tistyx";
const KIND: &str = "tisty-docs";
const VERSION: u32 = 1;
const MANIFEST: &str = "tisty-docs.json";
const CARRIED: [&str; 2] = ["docs", "attachments"];
const AT_MOST: u64 = 8 * 1024 * 1024 * 1024;
const AT_MOST_FILES: usize = 200_000;
const MANIFEST_AT_MOST: u64 = 16 * 1024 * 1024;
const PAPERS_AT_MOST: usize = 50_000;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub made: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
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
        return Err(Error::NothingToCarry);
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
            sent.missed += 1;
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
                    let held = along.also.unwrap_or(data).join("attachments");
                    let named = from
                        .strip_prefix(data.join("attachments"))
                        .or_else(|_| from.strip_prefix(&held))
                        .map(|rest| {
                            format!("attachments/{}", rest.display()).replace(char::from(92), "/")
                        })
                        .unwrap_or_else(|_| one.clone());
                    beside.insert(named, from);
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
                made: one.made,
                by: one.by.clone().or_else(|| state.signed.alias.clone()),
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
        let Ok(weighs) = std::fs::metadata(from).map(|one| one.len()) else {
            left_behind(&mut sent.left, named.clone());
            continue;
        };
        sent.files += 1;
        sent.bytes = sent.bytes.saturating_add(weighs);
        if sent.bytes > AT_MOST || sent.files + bodies.len() > AT_MOST_FILES {
            return Err(Error::TooBig);
        }
        let Ok(mut file) = std::fs::File::open(from) else {
            sent.files -= 1;
            sent.bytes = sent.bytes.saturating_sub(weighs);
            left_behind(&mut sent.left, named.clone());
            continue;
        };
        zip.start_file(named, kept).map_err(zipped)?;
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
        return Err(Error::NothingToCarry);
    }

    let mut sent = Sent::default();
    let mut shelves: BTreeSet<PathBuf> = BTreeSet::new();
    let fresh = !into.exists();
    let shelved = trails(state, into);
    let whole = papers.iter().filter(|one| one.page_of.is_none()).count();
    for one in papers.iter().filter(|one| one.page_of.is_none()) {
        let under = one
            .folder
            .and_then(|at| shelved.get(&at).cloned())
            .unwrap_or_else(|| into.to_path_buf());
        if let Err(e) = std::fs::create_dir_all(&under) {
            if fresh {
                let _ = std::fs::remove_dir_all(into);
            }
            return Err(Error::Io(e));
        }
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

fn trails(state: &State, into: &Path) -> BTreeMap<FolderId, PathBuf> {
    let mut found: BTreeMap<FolderId, PathBuf> = BTreeMap::new();
    let mut left: Vec<(Option<FolderId>, PathBuf)> = vec![(None, into.to_path_buf())];
    let mut deep = 0;
    while let Some((parent, at)) = left.pop() {
        deep += 1;
        if deep > DEEPEST * DEEPEST {
            break;
        }
        let mut taken: BTreeSet<String> = BTreeSet::new();
        for one in state.under(parent) {
            let mut named = crate::docs::spelled(&one.name);
            if !taken.insert(named.clone()) {
                for n in 2..100 {
                    let tried = format!("{named} {n}");
                    if taken.insert(tried.clone()) {
                        named = tried;
                        break;
                    }
                }
            }
            let mine = at.join(&named);
            found.insert(one.id, mine.clone());
            left.push((Some(one.id), mine));
        }
    }
    found
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

    let mine = crate::store::peek_identity(data.join("store"));
    let elsewhere = match (&mine, manifest.from.trim()) {
        (Some(mine), from) if !from.is_empty() => mine != from,
        _ => true,
    };
    let staged = data.join(format!(".landing-{}", std::process::id()));
    swept(data);
    let whole = zip.len() + manifest.docs.len();
    let done = unpack(&mut zip, &staged, along, whole).and_then(|_| {
        taken_in(
            data,
            state,
            device,
            &Landing {
                manifest: &manifest,
                staged: &staged,
                along,
                whole,
                elsewhere,
            },
        )
    });
    let _ = std::fs::remove_dir_all(&staged);
    done
}

pub fn swept(data: &Path) {
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
        .map_err(|_| Error::NotAParcel(from.display().to_string()))?;
    let mut said = String::new();
    let read = held
        .by_ref()
        .take(MANIFEST_AT_MOST + 1)
        .read_to_string(&mut said)? as u64;
    if read > MANIFEST_AT_MOST {
        return Err(Error::TooBig);
    }
    let manifest: Manifest =
        serde_json::from_str(&said).map_err(|_| Error::NotAParcel(from.display().to_string()))?;
    if manifest.kind != KIND {
        return Err(Error::NotAParcel(from.display().to_string()));
    }
    if manifest.version > VERSION {
        return Err(Error::ParcelNewer(manifest.version));
    }
    if manifest.docs.len() > PAPERS_AT_MOST || manifest.folders.len() > PAPERS_AT_MOST {
        return Err(Error::TooBig);
    }
    Ok(manifest)
}

struct Landing<'a> {
    manifest: &'a Manifest,
    staged: &'a Path,
    along: &'a Along<'a>,
    whole: usize,
    elsewhere: bool,
}

fn taken_in(
    data: &Path,
    state: &State,
    device: &DeviceId,
    landing: &Landing,
) -> Result<(Landed, Vec<Op>)> {
    let Landing {
        manifest,
        staged,
        along,
        whole,
        elsewhere,
    } = *landing;
    let mut landed = Landed::default();
    let mut ops: Vec<Op> = Vec::new();

    let filed = shelved(state, manifest, &mut landed, &mut ops);

    let root = data.join("docs");
    let mut carried: BTreeMap<String, String> = BTreeMap::new();
    let mut named: BTreeMap<String, (String, DocId)> = BTreeMap::new();
    let mut ordered: Vec<(Option<FolderId>, String)> = Vec::new();
    let mut written: Vec<(String, String)> = Vec::new();

    for paper in ordering(manifest) {
        let Ok(body) = crate::docs::read(&staged.join("docs"), &paper.file) else {
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

        let Ok(made) = crate::docs::create(&root, device, &body) else {
            landed.missed += 1;
            continue;
        };
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
                made: paper.made,
                by: paper
                    .by
                    .as_deref()
                    .map(signed_as)
                    .filter(|one| !one.is_empty()),
                guest: elsewhere,
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
        if told != body && crate::docs::write(&root, &file, &told).is_err() {
            landed.missed += 1;
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
                name: named(&shelf.name),
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

fn named(said: &str) -> String {
    let plain = crate::text::plainly(said);
    let mut cut: String = plain
        .chars()
        .take(crate::model::FOLDER_NAME_AT_MOST)
        .collect();
    if cut.trim().is_empty() {
        cut = "?".into();
    }
    cut.trim().to_string()
}

fn signed_as(said: &str) -> String {
    crate::text::plainly(said)
        .chars()
        .take(crate::event::ALIAS_AT_MOST)
        .collect::<String>()
        .trim()
        .to_string()
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

const MARK: char = '\u{0}';

fn pointed(body: &str, named: &BTreeMap<String, (String, DocId)>) -> String {
    let mut found: Vec<String> = crate::refs::papers(body);
    // The long name first: one id can be a prefix of another, and a plain replace would rewrite
    // the middle of the longer one.
    found.sort_by_key(|one| std::cmp::Reverse(one.len()));
    let mut told = body.replace(MARK, "");
    for file in found {
        if let Some((now, _)) = named.get(&file) {
            told = told.replace(
                &format!("{}{file}", crate::refs::DOC),
                &format!("{MARK}{now}"),
            );
        }
    }
    told.replace(MARK, crate::refs::DOC)
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
