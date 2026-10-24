use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tisty_core::witness::{self, Fact, channel};
use tisty_sync::{MARKER, NAMED, STORE};

use super::index::{Index, Mirrored, digest_of, on_shelf, path_of, stamp_of};
use super::pace::Pace;
use crate::{Changes, Expect, Hitch, Remote, Seen};

const PAPERS: &str = "docs";
const RACY_NANOS: u128 = 2_000_000_000;
const MIRRORED_AT_MOST: u64 = 64 * 1024 * 1024;
const LITTER_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

static TURN: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pushed {
    pub put: usize,
    pub left: usize,
    /// The file whose broken write stopped the push, so the person learns which one holds the rest back.
    pub stuck: Option<String>,
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
    homing: bool,
    along: &mut dyn FnMut(crate::Reached),
) -> Result<usize, Hitch> {
    heal(tree, index);
    if let Some(trouble) = still_barred(remote, tree, index)? {
        return Err(Hitch::Barred(trouble));
    }
    let first = index.cursor.is_none();
    let told = match remote.changes(index.cursor.as_deref()) {
        Err(Hitch::Broke(_) | Hitch::Missing(_) | Hitch::Changed(_)) if index.cursor.is_some() => {
            witness::warn(
                channel::CLOUD,
                "the cloud no longer knew where this machine had read up to, so it was read whole",
                &[],
            );
            index.cursor = None;
            remote.changes(None)?
        }
        told => told?,
    };
    heeded(remote, tree, index, data, &told)?;
    let emptied = |listed_nothing: bool, gone: &BTreeSet<&str>| {
        index.tree.contains_key(tisty_sync::NAMED)
            && (listed_nothing || index.tree.keys().all(|name| gone.contains(name.as_str())))
    };
    let emptied = match &told {
        Changes::Whole { seen, .. } => emptied(seen.is_empty(), &BTreeSet::new()),
        Changes::Since { changed, gone, .. } => {
            changed.is_empty() && emptied(false, &gone.iter().map(String::as_str).collect())
        }
    };
    // Moving home reads an emptied store through, so the move learns nothing is left up there and goes on.
    if emptied && !homing {
        return Err(Hitch::Emptied);
    }
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
            index.owed.retain(|name, _| listed.contains(name.as_str()));
            (seen, gone, cursor)
        }
        Changes::Since {
            changed,
            gone,
            cursor,
        } => (changed, gone, cursor),
    };
    let mut due = std::mem::take(&mut index.owed);
    for name in gone {
        due.remove(&name);
        forget(tree, index, &name);
    }
    for seen in changed {
        due.insert(seen.name.clone(), seen);
    }
    let whole = due.len();
    let mut due = due.into_values().enumerate();
    // Only a first read is long enough to need counting: a later one brings a handful.
    let mut told = |done: usize| {
        if first && whole > 0 {
            along(crate::Reached::Along {
                stage: crate::Stage::Log,
                done,
                whole,
            });
        }
    };
    told(0);
    while let Some((done, seen)) = due.next() {
        told(done + 1);
        match learn(remote, tree, index, data, seen.clone()) {
            Ok(false) => {}
            Ok(true) => {
                index.owed.insert(seen.name.clone(), seen);
            }
            Err(hitch) => {
                index.owed.insert(seen.name.clone(), seen);
                index
                    .owed
                    .extend(due.map(|(_, one)| (one.name.clone(), one)));
                return Err(hitch);
            }
        }
    }
    index.cursor = Some(cursor);
    match emptied {
        true => Err(Hitch::Emptied),
        false => Ok(index.owed.len()),
    }
}

fn still_barred(
    remote: &dyn Remote,
    tree: &Path,
    index: &Index,
) -> Result<Option<tisty_sync::Trouble>, Hitch> {
    match tisty_sync::later(tree) {
        Some(trouble) if !word_moved(remote, index)? => Ok(Some(trouble)),
        _ => Ok(None),
    }
}

pub fn word_moved(remote: &dyn Remote, index: &Index) -> Result<bool, Hitch> {
    let kept = index
        .tree
        .get(NAMED)
        .map(|kept| kept.seen.revision.as_str());
    let up_there = remote.about(NAMED)?.map(|seen| seen.revision);
    Ok(up_there.as_deref() != kept)
}

fn word_listed(told: &Changes) -> Option<&[Seen]> {
    let named = |seen: &[Seen]| seen.iter().any(|one| one.name == NAMED);
    match told {
        Changes::Whole { seen, .. } => named(seen).then_some(seen.as_slice()),
        Changes::Since { changed, gone, .. } => {
            (named(changed) || !gone.iter().any(|name| name == NAMED)).then_some(changed.as_slice())
        }
    }
}

fn heeded(
    remote: &dyn Remote,
    tree: &Path,
    index: &mut Index,
    data: Option<&Path>,
    told: &Changes,
) -> Result<(), Hitch> {
    let Some(listed) = word_listed(told) else {
        return Ok(());
    };
    let spoiled = matches!(
        tisty_sync::refuses(tree),
        Err(tisty_sync::Trouble::Unshaped(_))
    );
    if spoiled {
        index.tree.remove(NAMED);
    }
    let word = match listed
        .iter()
        .find(|one| one.name == NAMED)
        .or_else(|| index.owed.get(NAMED))
        .cloned()
    {
        None if spoiled => {
            let up_there = remote.about(NAMED)?;
            if up_there.is_none() {
                let _ = std::fs::remove_file(tree.join(NAMED));
            }
            up_there
        }
        word => word,
    };
    if let Some(word) = word {
        match learn(remote, tree, index, data, word.clone()) {
            Ok(false) => {
                index.owed.remove(NAMED);
            }
            Ok(true) => {
                index.owed.insert(NAMED.to_string(), word);
                return Err(Hitch::Troubled(format!("{NAMED} could not be brought")));
            }
            Err(hitch) => {
                index.owed.insert(NAMED.to_string(), word);
                return Err(hitch);
            }
        }
    }
    match tisty_sync::later(tree) {
        Some(trouble) => Err(Hitch::Barred(trouble)),
        None => Ok(()),
    }
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
    if index.shelf.remove(name).is_some() {
        index.vanished.insert(name.to_string());
    }
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
            index.shelved(&seen.name.clone(), seen);
        }
        return Ok(false);
    }
    if !mirrored(&seen.name) {
        return Ok(false);
    }
    let Some(at) = path_of(tree, &seen.name) else {
        witness::warn(
            channel::CLOUD,
            "the cloud holds a name that cannot be a path here, so it was left alone",
            &[("at", Fact::Id(seen.name))],
        );
        return Ok(false);
    };
    if seen.bytes > MIRRORED_AT_MOST {
        witness::warn(
            channel::CLOUD,
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
        match ours_still(remote, index, &seen, &at) {
            Ok(true) => {
                if let Some(kept) = index.tree.get_mut(&seen.name) {
                    kept.seen = seen;
                }
                return Ok(false);
            }
            Ok(false) | Err(Hitch::Missing(_)) => forget_base(data, &seen.name),
            Err(Hitch::Broke(_)) => return Ok(true),
            Err(other) => return Err(other),
        }
    }
    match download(remote, &at, &seen.name) {
        Ok(got) => note(index, got, &at),
        Err(Hitch::Missing(_)) => {}
        Err(Hitch::Broke(why)) => {
            witness::warn(
                channel::CLOUD,
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

// Drive moves a file past a write of ours on its own, so a revision we never wrote can still hold the bytes we last sent.
fn ours_still(remote: &dyn Remote, index: &Index, seen: &Seen, at: &Path) -> Result<bool, Hitch> {
    let Some(kept) = index.tree.get(&seen.name) else {
        return Ok(false);
    };
    if !seen.hash.is_empty() && !kept.seen.hash.is_empty() {
        return Ok(seen.hash == kept.seen.hash);
    }
    if seen.bytes != kept.len || kept.digest.is_empty() {
        return Ok(false);
    }
    let part = tisty_core::parting::beside(at, TURN.fetch_add(1, Ordering::Relaxed));
    let fetched = fetch_whole(remote, &seen.name, &part).map(|_| digest_of(&part));
    let _ = std::fs::remove_file(&part);
    Ok(fetched?.is_some_and(|digest| digest == kept.digest))
}

fn forget_base(data: &Path, name: &str) {
    let id = name
        .strip_prefix(&format!("{PAPERS}/"))
        .and_then(|leaf| leaf.strip_suffix(".md"));
    let Some(id) = id else {
        return;
    };
    witness::warn(
        channel::CLOUD,
        "a document changed in the cloud before a change of ours went up, so the person decides",
        &[("at", Fact::Id(id.to_string()))],
    );
    let mut said = tisty_core::docs::Carried::read(data);
    said.forget(id);
    let _ = said.save(data);
    tisty_core::docs::forget_carried(data, id);
}

pub(super) fn resembles(remote: &dyn Remote, at: &Path, seen: &Seen) -> bool {
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
        return Err(Hitch::Broke(format!("{name} came short")));
    }
    if !got.hash.is_empty() && remote.hash_of(part)? != got.hash {
        return Err(Hitch::Broke(format!(
            "{name} is not what the provider holds"
        )));
    }
    Ok(got)
}

pub fn push(
    remote: &dyn Remote,
    tree: &Path,
    index: &mut Index,
    pace: &mut Pace,
) -> (Pushed, Option<Hitch>) {
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
    // A stamp already up there only changes to raise the place, and older builds must read that first.
    let (raised, stamp): (Vec<&Found>, Vec<&Found>) = stamp
        .into_iter()
        .partition(|one| index.tree.contains_key(&one.name));
    rest.sort_by_key(|one| (history_first(&one.name), one.name.clone()));
    rest.splice(0..0, raised);
    pace.owe(
        rest.len() + stamp.len(),
        rest.iter().chain(&stamp).map(|one| one.len).sum(),
    );

    for (at, one) in rest.iter().enumerate() {
        if let Err(hitch) = upload(remote, index, one, &mut done, pace) {
            done.left += rest.len() - at + gone.len() + stamp.len();
            done.stuck = matches!(hitch, Hitch::Broke(_)).then(|| one.name.clone());
            return (done, Some(hitch));
        }
    }
    for (at, (name, revision)) in gone.iter().enumerate() {
        match remote.delete(name, Some(revision)) {
            Ok(()) | Err(Hitch::Missing(_)) => {
                index.tree.remove(name);
                pace.carried(index);
            }
            Err(Hitch::Changed(_)) => {
                done.left += 1;
                read_again(remote, index, name);
            }
            Err(hitch) => {
                done.left += gone.len() - at + stamp.len();
                return (done, Some(hitch));
            }
        }
    }
    if done.left > 0 {
        done.left += stamp.len();
        for one in stamp {
            pace.went(one.len);
        }
        return (done, None);
    }
    for one in stamp {
        if let Err(hitch) = upload(remote, index, one, &mut done, pace) {
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
    pace: &mut Pace,
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
            pace.landed(index, one.len);
            Ok(())
        }
        Err(Hitch::Changed(_) | Hitch::Missing(_)) => {
            done.left += 1;
            pace.went(one.len);
            read_again(remote, index, &one.name);
            Ok(())
        }
        Err(other) => Err(other),
    }
}

// What refused a write is read again, since a change the provider never tells would leave the write refused every round.
fn read_again(remote: &dyn Remote, index: &mut Index, name: &str) {
    let learned = match remote.about(name) {
        Ok(Some(seen)) => index.owed.insert(name.to_string(), seen.clone()) != Some(seen),
        Ok(None) => index.tree.remove(name).is_some(),
        Err(_) => false,
    };
    // A refusal that leaves what changed written down is progress, so a push of many refused files is no loop.
    if learned {
        remote.moved_on();
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
    tisty_core::store::is_segment(leaf)
        || tisty_core::store::beside_a_segment(leaf)
            .is_some_and(|(stem, _)| tisty_core::store::is_segment(&format!("{stem}.tisty")))
}

pub(super) fn mirrored(name: &str) -> bool {
    name == NAMED || name == tisty_core::moved::NAMED || is_history(name) || is_doc(name)
}

#[cfg(test)]
#[path = "mirror_test.rs"]
mod tests;
