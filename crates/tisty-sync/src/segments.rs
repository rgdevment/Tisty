use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{STORE, Trouble, copy_onto, io, plainly};

pub(crate) type Named = std::collections::BTreeSet<std::ffi::OsString>;

#[derive(Clone, Copy)]
pub(crate) enum Toward {
    Folder,
    Home,
}

#[derive(Default)]
pub(crate) struct Alike(std::collections::BTreeMap<String, Named>);

impl Alike {
    pub(crate) fn of(&mut self, who: &str, theirs: &Path, mine: &Path) -> &Named {
        self.0
            .entry(who.to_string())
            .or_insert_with(|| alike_in(theirs, mine))
    }

    pub(crate) fn settled(&mut self, who: &str, theirs: &Path, mine: &Path, way: Toward) -> bool {
        let from = from_to(theirs, mine, way).0.to_path_buf();
        all_of(&from, self.of(who, theirs, mine))
    }

    pub(crate) fn carried(
        &mut self,
        who: &str,
        theirs: &Path,
        mine: &Path,
        way: Toward,
        again: bool,
    ) -> Result<usize, Trouble> {
        let (from, into) = from_to(theirs, mine, way);
        let (from, into) = (from.to_path_buf(), into.to_path_buf());
        let done = copy_segments(&from, &into, again, self.of(who, theirs, mine))?;
        if done > 0 {
            self.0.remove(who);
        }
        Ok(done)
    }
}

fn from_to<'a>(theirs: &'a Path, mine: &'a Path, way: Toward) -> (&'a Path, &'a Path) {
    match way {
        Toward::Folder => (mine, theirs),
        Toward::Home => (theirs, mine),
    }
}

fn alike_in(theirs: &Path, mine: &Path) -> Named {
    let Ok(offered) = tisty_core::store::segments_in(theirs) else {
        return Named::default();
    };
    offered
        .iter()
        .filter(|at| {
            at.file_name()
                .is_some_and(|named| same(at, &mine.join(named)))
        })
        .filter_map(|at| at.file_name().map(std::ffi::OsStr::to_os_string))
        .collect()
}

fn all_of(dir: &Path, known: &Named) -> bool {
    tisty_core::store::segments_in(dir)
        .map(|held| {
            !held.is_empty()
                && held
                    .iter()
                    .filter_map(|at| at.file_name())
                    .all(|named| known.contains(named))
        })
        .unwrap_or(false)
}

pub(crate) fn copy_segments(
    from: &Path,
    into: &Path,
    again: bool,
    known: &Named,
) -> Result<usize, Trouble> {
    let carried = match tisty_core::store::segments_in(from) {
        Ok(carried) => carried,
        Err(e) => {
            if !matches!(&e, tisty_core::Error::Io(io) if io.kind() == std::io::ErrorKind::NotFound)
            {
                witness::warn(
                    channel::SYNC,
                    "segments unlistable",
                    &[
                        ("at", Fact::Path(from.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return Ok(0);
        }
    };
    if carried.is_empty() {
        return Ok(0);
    }
    std::fs::create_dir_all(into).map_err(io)?;
    sweep(into);
    let mut done = 0;
    for at in carried {
        let Some(named) = at.file_name() else {
            continue;
        };
        let counter = at.with_extension("count");
        if let Some(tally) = counter.file_name().filter(|_| counter.is_file()) {
            let target = into.join(tally);
            if again || !same(&counter, &target) {
                copy_onto(&counter, &target)?;
            }
        }

        let target = into.join(named);
        let sealed = named.to_str().is_some_and(tisty_core::store::is_sealed);
        if !again && ((sealed && known.contains(named)) || same(&at, &target)) {
            continue;
        }
        copy_onto(&at, &target)?;
        done += 1;
    }
    Ok(done)
}

pub(crate) fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mine = format!(".{}.", std::process::id());
    for at in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
        let ours = at
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".part") && n.contains(&mine));
        if ours && let Err(e) = std::fs::remove_file(&at) {
            witness::warn(
                channel::SYNC,
                "leftover not removed",
                &[
                    ("at", Fact::Path(at.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }
}

pub(crate) fn same(from: &Path, to: &Path) -> bool {
    use std::io::Read;

    let (Ok(a), Ok(b)) = (std::fs::metadata(from), std::fs::metadata(to)) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    if a.len() == 0 {
        return true;
    }
    let (Ok(here), Ok(there)) = (std::fs::File::open(from), std::fs::File::open(to)) else {
        return false;
    };
    tisty_core::counting::opened();
    tisty_core::counting::opened();
    let mut here = std::io::BufReader::new(here);
    let mut there = std::io::BufReader::new(there);
    let mut one = [0u8; 16 * 1024];
    let mut two = [0u8; 16 * 1024];
    loop {
        let read = match here.read(&mut one) {
            Ok(0) => return true,
            Ok(read) => read,
            Err(_) => return false,
        };
        if there.read_exact(&mut two[..read]).is_err() || one[..read] != two[..read] {
            return false;
        }
    }
}

pub(crate) enum Grew {
    Yes,
    No,
    Cannot,
    Unread,
}

enum Whole {
    Said(Vec<u8>),
    Empty,
    Unread,
}

fn whole_of(device_dir: &Path) -> Whole {
    let Ok(segments) = tisty_core::store::segments_in(device_dir) else {
        return Whole::Unread;
    };
    let mut said = Vec::new();
    for at in segments {
        let Ok(more) = std::fs::read(at) else {
            return Whole::Unread;
        };
        said.extend(more);
    }
    if said.is_empty() {
        Whole::Empty
    } else {
        Whole::Said(said)
    }
}

pub(crate) fn one_grew_from_the_other(here: &Path, there: &Path) -> Grew {
    let (ours, theirs) = match (whole_of(here), whole_of(there)) {
        (Whole::Said(ours), Whole::Said(theirs)) => (ours, theirs),
        (Whole::Unread, _) | (_, Whole::Unread) => return Grew::Unread,
        _ => return Grew::Cannot,
    };
    let grew = if ours.len() <= theirs.len() {
        theirs.starts_with(&ours)
    } else {
        ours.starts_with(&theirs)
    };
    if grew { Grew::Yes } else { Grew::No }
}

pub(crate) fn hand_on(
    store: &Path,
    device: &str,
    dest: &Path,
    again: bool,
    alike: &mut Alike,
) -> Result<usize, Trouble> {
    let there = dest.join(STORE);
    let Ok(entries) = std::fs::read_dir(store) else {
        return Ok(0);
    };
    let mut sent = 0;
    for entry in entries.filter_map(|e| e.ok()) {
        let named = entry.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        if named.eq_ignore_ascii_case(device) || !entry.path().is_dir() {
            continue;
        }
        let theirs = there.join(named);
        if alike.settled(named, &theirs, &entry.path(), Toward::Folder)
            || !ours_reaches_further(&entry.path(), &theirs)
        {
            continue;
        }
        plainly(&theirs)?;
        let done = alike.carried(named, &theirs, &entry.path(), Toward::Folder, again)?;
        if done > 0 {
            witness::note(
                channel::SYNC,
                "a history this machine was holding for another was handed on",
                &[
                    ("at", Fact::Id(named.to_string())),
                    ("sent", Fact::Count(done)),
                ],
            );
        }
        sent += done;
    }
    Ok(sent)
}

pub(crate) fn ours_went_missing(mine: &Path, theirs: &Path) -> bool {
    let held = match tisty_core::store::distinct_in(mine) {
        Ok(held) => held,
        Err(tisty_core::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(_) => return false,
    };
    if tisty_core::store::check_device(theirs).is_err() {
        return false;
    }
    let Ok(coming) = tisty_core::store::distinct_in(theirs) else {
        return false;
    };
    coming > held && (held == 0 || matches!(one_grew_from_the_other(mine, theirs), Grew::Yes))
}

pub(crate) fn ours_reaches_further(mine: &Path, theirs: &Path) -> bool {
    let Ok(ours) = tisty_core::store::distinct_in(mine) else {
        return false;
    };
    match tisty_core::store::check_device(theirs) {
        Ok(_) => {}
        Err(tisty_core::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            return ours > 0;
        }
        Err(_) => return false,
    }
    let Ok(held) = tisty_core::store::distinct_in(theirs) else {
        return false;
    };
    ours > held && (held == 0 || matches!(one_grew_from_the_other(mine, theirs), Grew::Yes))
}
