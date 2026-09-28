use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{Trouble, copy_onto, io};

pub(crate) type Named = std::collections::BTreeSet<std::ffi::OsString>;

pub(crate) fn matching(theirs: &Path, mine: &Path) -> (Named, bool) {
    let Ok(offered) = tisty_core::store::segments_in(theirs) else {
        return (Named::default(), false);
    };
    let known: Named = offered
        .iter()
        .filter(|at| {
            at.file_name()
                .is_some_and(|named| same(at, &mine.join(named)))
        })
        .filter_map(|at| at.file_name().map(std::ffi::OsStr::to_os_string))
        .collect();
    let settled = !offered.is_empty() && known.len() == offered.len();
    (known, settled)
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
        if !again && (known.contains(named) || same(&at, &target)) {
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
