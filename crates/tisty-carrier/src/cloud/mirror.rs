use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::index::{Index, Mirrored, on_shelf, path_of, stamp_of};
use crate::{Changes, Expect, Hitch, Remote, Seen};

const NAMED: &str = "tisty.toml";
const SHELF: &str = "attachments";

static TURN: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pushed {
    pub put: usize,
    pub removed: usize,
    pub left: usize,
}

struct Found {
    name: String,
    at: PathBuf,
    len: u64,
    stamp: u128,
}

pub fn pull(remote: &dyn Remote, tree: &Path, index: &mut Index) -> Result<(), Hitch> {
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
    for seen in changed {
        learn(remote, tree, index, seen)?;
    }
    index.cursor = Some(cursor);
    Ok(())
}

fn forget(tree: &Path, index: &mut Index, name: &str) {
    index.shelf.remove(name);
    if index.tree.remove(name).is_some() {
        let _ = std::fs::remove_file(path_of(tree, name));
    }
}

fn learn(remote: &dyn Remote, tree: &Path, index: &mut Index, seen: Seen) -> Result<(), Hitch> {
    if on_shelf(&seen.name) {
        index.shelf.insert(seen.name.clone(), seen);
        return Ok(());
    }
    if !mirrored(&seen.name) {
        return Ok(());
    }
    if index
        .tree
        .get(&seen.name)
        .is_some_and(|have| have.seen.revision == seen.revision)
    {
        return Ok(());
    }
    let at = path_of(tree, &seen.name);
    let seen = match resembles(remote, &at, &seen) {
        true => seen,
        false => download(remote, &at, &seen.name)?,
    };
    note(index, seen, &at);
    Ok(())
}

fn resembles(remote: &dyn Remote, at: &Path, seen: &Seen) -> bool {
    !seen.hash.is_empty()
        && std::fs::metadata(at).is_ok_and(|one| one.is_file() && one.len() == seen.bytes)
        && remote.hash_of(at).is_ok_and(|hash| hash == seen.hash)
}

fn note(index: &mut Index, seen: Seen, at: &Path) {
    if let Ok(meta) = std::fs::metadata(at) {
        let mirrored = Mirrored {
            len: meta.len(),
            stamp: stamp_of(&meta),
            seen,
        };
        index.tree.insert(mirrored.seen.name.clone(), mirrored);
    }
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
    walk(tree, tree, &mut here);
    for one in &here {
        if let Some(kept) = index.tree.get_mut(&one.name)
            && (kept.len, kept.stamp) != (one.len, one.stamp)
            && same_bytes(remote, one, &kept.seen)
        {
            (kept.len, kept.stamp) = (one.len, one.stamp);
        }
    }
    let present: BTreeSet<&str> = here.iter().map(|one| one.name.as_str()).collect();
    let gone: Vec<(String, String)> = index
        .tree
        .iter()
        .filter(|(name, _)| !present.contains(name.as_str()))
        .map(|(name, kept)| (name.clone(), kept.seen.revision.clone()))
        .collect();
    let (stamp, mut rest): (Vec<&Found>, Vec<&Found>) = here
        .iter()
        .filter(|one| {
            index
                .tree
                .get(&one.name)
                .is_none_or(|kept| (kept.len, kept.stamp) != (one.len, one.stamp))
        })
        .partition(|one| one.name == NAMED);
    rest.sort_by(|one, other| one.name.cmp(&other.name));

    for (at, one) in rest.iter().enumerate() {
        if let Err(hitch) = upload(remote, index, one, &mut done) {
            done.left += rest.len() - at;
            return (done, Some(hitch));
        }
    }
    for (at, (name, revision)) in gone.iter().enumerate() {
        match remote.delete(name, Some(revision)) {
            Ok(()) | Err(Hitch::Missing(_)) => {
                index.tree.remove(name);
                done.removed += 1;
            }
            Err(Hitch::Changed(_)) => done.left += 1,
            Err(hitch) => {
                done.left += gone.len() - at;
                return (done, Some(hitch));
            }
        }
    }
    if done.left == 0 {
        for one in stamp {
            if let Err(hitch) = upload(remote, index, one, &mut done) {
                done.left += 1;
                return (done, Some(hitch));
            }
        }
    }
    (done, None)
}

fn same_bytes(remote: &dyn Remote, one: &Found, up: &Seen) -> bool {
    !up.hash.is_empty()
        && up.bytes == one.len
        && remote.hash_of(&one.at).is_ok_and(|hash| hash == up.hash)
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
            index.tree.insert(
                one.name.clone(),
                Mirrored {
                    seen,
                    len: one.len,
                    stamp: one.stamp,
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

fn walk(root: &Path, dir: &Path, into: &mut Vec<Found>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|one| one.ok()) {
        let at = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            walk(root, &at, into);
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

fn mirrored(name: &str) -> bool {
    let leaf = name.rsplit('/').next().unwrap_or(name);
    let litter = leaf.ends_with(".part") || leaf.ends_with(".tmp") || leaf == ".lock";
    !name.starts_with(&format!("{SHELF}/")) && !litter
}
