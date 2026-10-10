use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tisty_core::witness::{self, Fact, channel};
use tisty_sync::{MARKER, NAMED, STORE};

use super::index::{Index, Mirrored, digest_of, on_shelf, path_of, stamp_of};
use crate::{Changes, Expect, Hitch, Remote, Seen};

const PAPERS: &str = "docs";
const RACY_NANOS: u128 = 2_000_000_000;
const MIRRORED_AT_MOST: u64 = 64 * 1024 * 1024;
const LITTER_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

static TURN: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pushed {
    pub put: usize,
    pub left: usize,
}

struct Found {
    name: String,
    at: PathBuf,
    len: u64,
    stamp: u128,
}

pub fn pull(
    remote: &dyn Remote,
    tree: &Path,
    index: &mut Index,
    data: Option<&Path>,
) -> Result<(), Hitch> {
    heal(tree, index);
    let told = remote.changes(index.cursor.as_deref())?;
    let (changed, gone, cursor) = match told {
        Changes::Whole { seen, cursor } => {
            let listed: BTreeSet<&str> = seen.iter().map(|one| one.name.as_str()).collect();
            let gone = index
                .tree
                .keys()
                .chain(index.shelf.keys())
                .filter(|name| !listed.contains(name.as_str()))
                .cloned()
                .collect();
            if index.cursor.is_none() {
                clear_unlisted(tree, &listed);
            }
            (seen, gone, cursor)
        }
        Changes::Since {
            changed,
            gone,
            cursor,
        } => (changed, gone, cursor),
    };
    for name in gone {
        forget(tree, index, &name);
    }
    let mut held_back = false;
    for seen in changed {
        held_back |= learn(remote, tree, index, data, seen)?;
    }
    if !held_back {
        index.cursor = Some(cursor);
    }
    Ok(())
}

fn heal(tree: &Path, index: &mut Index) {
    let docs_stand = tree.join(PAPERS).is_dir();
    let missing: Vec<String> = index
        .tree
        .keys()
        .filter(|name| !is_doc(name) || !docs_stand)
        .filter(|name| !path_of(tree, name).is_some_and(|at| at.exists()))
        .cloned()
        .collect();
    if missing.is_empty() {
        return;
    }
    for name in missing {
        index.tree.remove(&name);
    }
    index.cursor = None;
}

fn clear_unlisted(tree: &Path, listed: &BTreeSet<&str>) {
    let mut here = Vec::new();
    walk(tree, tree, &mut here);
    for one in here
        .into_iter()
        .filter(|one| !listed.contains(one.name.as_str()))
    {
        let _ = std::fs::remove_file(&one.at);
    }
}

fn forget(tree: &Path, index: &mut Index, name: &str) {
    index.shelf.remove(name);
    if index.tree.remove(name).is_some()
        && let Some(at) = path_of(tree, name)
    {
        let _ = std::fs::remove_file(at);
    }
}

fn learn(
    remote: &dyn Remote,
    tree: &Path,
    index: &mut Index,
    data: Option<&Path>,
    seen: Seen,
) -> Result<bool, Hitch> {
    if on_shelf(&seen.name) {
        if crate::named_well(&seen.name).is_ok() {
            index.shelf.insert(seen.name.clone(), seen);
        }
        return Ok(false);
    }
    if !mirrored(&seen.name) {
        return Ok(false);
    }
    let Some(at) = path_of(tree, &seen.name) else {
        witness::warn(
            channel::SYNC,
            "the cloud holds a name that cannot be a path here, so it was left alone",
            &[("at", Fact::Id(seen.name))],
        );
        return Ok(false);
    };
    if seen.bytes > MIRRORED_AT_MOST {
        witness::warn(
            channel::SYNC,
            "the cloud holds a file too big to be history or a document, so it was left alone",
            &[("at", Fact::Id(seen.name))],
        );
        return Ok(false);
    }
    if index
        .tree
        .get(&seen.name)
        .is_some_and(|have| have.seen.revision == seen.revision)
    {
        return Ok(false);
    }
    if resembles(remote, &at, &seen) {
        note(index, seen, &at);
        return Ok(false);
    }
    if is_doc(&seen.name) && pending(index, &seen.name, &at) {
        let Some(data) = data else {
            return Ok(true);
        };
        forget_base(data, &seen.name);
    }
    match download(remote, &at, &seen.name) {
        Ok(got) => note(index, got, &at),
        Err(Hitch::Missing(_)) => {}
        Err(Hitch::Broke(why)) => {
            witness::warn(
                channel::SYNC,
                "a file in the cloud could not be brought, so it was left for another round",
                &[("at", Fact::Id(seen.name)), ("why", Fact::Why(why))],
            );
            return Ok(true);
        }
        Err(other) => return Err(other),
    }
    Ok(false)
}

fn pending(index: &Index, name: &str, at: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(at) else {
        return false;
    };
    match index.tree.get(name) {
        Some(kept) => {
            (kept.len, kept.stamp) != (meta.len(), stamp_of(&meta))
                && !(kept.len == meta.len() && digest_of(at).as_ref() == Some(&kept.digest))
        }
        None => meta.is_file(),
    }
}

fn forget_base(data: &Path, name: &str) {
    let id = name
        .strip_prefix(&format!("{PAPERS}/"))
        .and_then(|leaf| leaf.strip_suffix(".md"));
    let Some(id) = id else {
        return;
    };
    witness::warn(
        channel::SYNC,
        "a document changed in the cloud before a change of ours went up, so the person decides",
        &[("at", Fact::Id(id.to_string()))],
    );
    let mut said = tisty_core::docs::Carried::read(data);
    said.forget(id);
    let _ = said.save(data);
    tisty_core::docs::forget_carried(data, id);
}

fn resembles(remote: &dyn Remote, at: &Path, seen: &Seen) -> bool {
    !seen.hash.is_empty()
        && std::fs::metadata(at).is_ok_and(|one| one.is_file() && one.len() == seen.bytes)
        && remote.hash_of(at).is_ok_and(|hash| hash == seen.hash)
}

fn racy(stamp: u128) -> u128 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    match now.saturating_sub(stamp) < RACY_NANOS {
        true => 0,
        false => stamp,
    }
}

fn note(index: &mut Index, seen: Seen, at: &Path) {
    let (Ok(meta), Some(digest)) = (std::fs::metadata(at), digest_of(at)) else {
        return;
    };
    let mirrored = Mirrored {
        len: meta.len(),
        stamp: racy(stamp_of(&meta)),
        digest,
        seen,
    };
    index.confirmed |= mirrored.seen.name == NAMED;
    index.tree.insert(mirrored.seen.name.clone(), mirrored);
}

fn download(remote: &dyn Remote, at: &Path, name: &str) -> Result<Seen, Hitch> {
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let part = tisty_core::parting::beside(at, TURN.fetch_add(1, Ordering::Relaxed));
    let fetched = fetch_whole(remote, name, &part).and_then(|got| {
        std::fs::rename(&part, at)?;
        Ok(got)
    });
    if fetched.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    fetched
}

fn fetch_whole(remote: &dyn Remote, name: &str, part: &Path) -> Result<Seen, Hitch> {
    let mut file = std::fs::File::create(part)?;
    let got = remote.fetch(name, 0, &mut file)?;
    file.sync_all()?;
    drop(file);
    if std::fs::metadata(part)?.len() != got.bytes {
        return Err(Hitch::Unreachable(format!("{name} came short")));
    }
    if !got.hash.is_empty() && remote.hash_of(part)? != got.hash {
        return Err(Hitch::Broke(format!(
            "{name} is not what the provider holds"
        )));
    }
    Ok(got)
}

pub fn push(remote: &dyn Remote, tree: &Path, index: &mut Index) -> (Pushed, Option<Hitch>) {
    let mut done = Pushed::default();
    let mut here = Vec::new();
    let sound = walk(tree, tree, &mut here);
    let dirty: Vec<&Found> = here.iter().filter(|one| is_dirty(index, one)).collect();
    let present: BTreeSet<&str> = here.iter().map(|one| one.name.as_str()).collect();
    let gone: Vec<(String, String)> = match sound && tree.join(PAPERS).is_dir() {
        true => index
            .tree
            .iter()
            .filter(|(name, _)| is_doc(name) && !present.contains(name.as_str()))
            .map(|(name, kept)| (name.clone(), kept.seen.revision.clone()))
            .collect(),
        false => Vec::new(),
    };
    let (stamp, mut rest): (Vec<&Found>, Vec<&Found>) =
        dirty.into_iter().partition(|one| one.name == NAMED);
    rest.sort_by_key(|one| (history_first(&one.name), one.name.clone()));

    for (at, one) in rest.iter().enumerate() {
        if let Err(hitch) = upload(remote, index, one, &mut done) {
            done.left += rest.len() - at + gone.len() + stamp.len();
            return (done, Some(hitch));
        }
    }
    for (at, (name, revision)) in gone.iter().enumerate() {
        match remote.delete(name, Some(revision)) {
            Ok(()) | Err(Hitch::Missing(_)) => {
                index.tree.remove(name);
            }
            Err(Hitch::Changed(_)) => done.left += 1,
            Err(hitch) => {
                done.left += gone.len() - at + stamp.len();
                return (done, Some(hitch));
            }
        }
    }
    if done.left > 0 {
        done.left += stamp.len();
        return (done, None);
    }
    for one in stamp {
        if let Err(hitch) = upload(remote, index, one, &mut done) {
            done.left += 1;
            return (done, Some(hitch));
        }
    }
    (done, None)
}

fn is_dirty(index: &mut Index, one: &Found) -> bool {
    match index.tree.get_mut(&one.name) {
        None => true,
        Some(kept) if (kept.len, kept.stamp) == (one.len, one.stamp) => false,
        Some(kept) if kept.len == one.len && digest_of(&one.at).as_ref() == Some(&kept.digest) => {
            kept.stamp = racy(one.stamp);
            false
        }
        Some(_) => true,
    }
}

fn history_first(name: &str) -> u8 {
    match name
        .strip_prefix(STORE)
        .is_some_and(|rest| rest.starts_with('/'))
    {
        true => 0,
        false => 1,
    }
}

fn upload(
    remote: &dyn Remote,
    index: &mut Index,
    one: &Found,
    done: &mut Pushed,
) -> Result<(), Hitch> {
    let expect = match index.tree.get(&one.name) {
        Some(kept) => Expect::Revision(kept.seen.revision.clone()),
        None => Expect::Absent,
    };
    match remote.put(&one.name, &one.at, expect) {
        Ok(seen) => {
            landed(remote, one, &seen)?;
            index.confirmed |= one.name == NAMED;
            index.tree.insert(
                one.name.clone(),
                Mirrored {
                    seen,
                    len: one.len,
                    stamp: racy(one.stamp),
                    digest: digest_of(&one.at).unwrap_or_default(),
                },
            );
            done.put += 1;
            Ok(())
        }
        Err(Hitch::Changed(_) | Hitch::Missing(_)) => {
            done.left += 1;
            Ok(())
        }
        Err(other) => Err(other),
    }
}

fn landed(remote: &dyn Remote, one: &Found, seen: &Seen) -> Result<(), Hitch> {
    if seen.bytes != one.len {
        return Err(Hitch::Broke(format!(
            "{} landed with other bytes",
            one.name
        )));
    }
    if !seen.hash.is_empty() && remote.hash_of(&one.at)? != seen.hash {
        return Err(Hitch::Broke(format!(
            "{} landed with another hash",
            one.name
        )));
    }
    Ok(())
}

pub fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|one| one.ok()) {
        let at = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            sweep(&at);
            continue;
        }
        let Some(leaf) = at.file_name().and_then(|named| named.to_str()) else {
            continue;
        };
        let spent = if leaf.ends_with(".part") {
            tisty_core::parting::spent(&at, leaf)
        } else if leaf.ends_with(".tmp") {
            meta.modified()
                .ok()
                .and_then(|when| when.elapsed().ok())
                .is_some_and(|age| age > LITTER_AFTER)
        } else {
            false
        };
        if spent {
            let _ = std::fs::remove_file(&at);
        }
    }
}

fn walk(root: &Path, dir: &Path, into: &mut Vec<Found>) -> bool {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(why) => return why.kind() == std::io::ErrorKind::NotFound,
    };
    let mut sound = true;
    for entry in entries {
        let Ok(entry) = entry else {
            sound = false;
            continue;
        };
        let at = entry.path();
        let Ok(meta) = entry.metadata() else {
            sound = false;
            continue;
        };
        if meta.is_dir() {
            sound &= walk(root, &at, into);
            continue;
        }
        let Some(name) = named(root, &at) else {
            continue;
        };
        if meta.is_file() && mirrored(&name) {
            into.push(Found {
                name,
                at,
                len: meta.len(),
                stamp: stamp_of(&meta),
            });
        }
    }
    sound
}

fn named(root: &Path, at: &Path) -> Option<String> {
    let parts: Option<Vec<&str>> = at
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|part| part.as_os_str().to_str())
        .collect();
    Some(parts?.join("/"))
}

fn under(name: &str, top: &str) -> bool {
    name.strip_prefix(top)
        .is_some_and(|rest| rest.starts_with('/'))
}

fn is_doc(name: &str) -> bool {
    under(name, PAPERS)
        && name[PAPERS.len() + 1..].split_once('/').is_none()
        && tisty_core::docs::a_body(&name[PAPERS.len() + 1..])
}

// Only what the round reads: a copy a desktop client left beside a segment would be fetched for nothing.
fn is_history(name: &str) -> bool {
    let Some(rest) = name
        .strip_prefix(STORE)
        .and_then(|rest| rest.strip_prefix('/'))
    else {
        return false;
    };
    if rest == MARKER {
        return true;
    }
    let Some((_, leaf)) = rest.split_once('/').filter(|(_, leaf)| !leaf.contains('/')) else {
        return false;
    };
    leaf.rsplit_once('.').is_some_and(|(stem, ext)| {
        matches!(ext, "tisty" | "sig" | "count")
            && tisty_core::store::is_segment(&format!("{stem}.tisty"))
    })
}

fn mirrored(name: &str) -> bool {
    name == NAMED || is_history(name) || is_doc(name)
}
