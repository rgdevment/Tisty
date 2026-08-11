//! Up goes ours, down comes everyone else's. One writer per directory, so
//! there is nothing to merge and nothing to choose.

use std::path::{Path, PathBuf};

pub use tisty_core::store::MARKER;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    NotThere(String),
    /// Merging two histories into an append-only log cannot be undone.
    OtherStore {
        theirs: String,
    },
    Unreadable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    Both,
    Push,
    Pull,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Moved {
    pub sent: usize,
    pub brought: usize,
}

/// A machine that has never met the folder adopts its name; one that has met a
/// different one is refused. Two stores in a folder is the one undoable mistake.
pub fn carry(store: &Path, device: &str, dest: &Path, way: Way) -> Result<Moved, Trouble> {
    if !dest.is_dir() {
        return Err(Trouble::NotThere(dest.display().to_string()));
    }

    let theirs = theirs(dest);
    let ours = match (tisty_core::store::peek_identity(store), theirs.is_empty()) {
        (Some(ours), false) => {
            claims(&theirs, &ours)?;
            ours
        }
        (Some(ours), true) => ours,
        (None, false) => {
            write(&store.join(MARKER), theirs.as_bytes())?;
            theirs
        }
        (None, true) => {
            tisty_core::store::identity(store).map_err(|e| Trouble::Unreadable(e.to_string()))?
        }
    };

    let mut moved = Moved::default();
    if matches!(way, Way::Both | Way::Push) {
        write(&dest.join("store").join(MARKER), ours.as_bytes())?;
        moved.sent = copy_segments(&store.join(device), &dest.join("store").join(device))?;
    }
    if matches!(way, Way::Both | Way::Pull) {
        moved.brought = bring(store, device, dest)?;
    }
    Ok(moved)
}

pub fn claims(theirs: &str, ours: &str) -> Result<(), Trouble> {
    let theirs = theirs.trim();
    if theirs.is_empty() || theirs == ours.trim() {
        Ok(())
    } else {
        Err(Trouble::OtherStore {
            theirs: theirs.to_string(),
        })
    }
}

pub fn theirs(dest: &Path) -> String {
    std::fs::read_to_string(dest.join("store").join(MARKER))
        .map(|held| held.trim().to_string())
        .unwrap_or_default()
}

fn bring(store: &Path, device: &str, dest: &Path) -> Result<usize, Trouble> {
    let mut brought = 0;
    let Ok(entries) = std::fs::read_dir(dest.join("store")) else {
        return Ok(0);
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let named = entry.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        if named == device || !entry.path().is_dir() {
            continue;
        }
        brought += copy_segments(&entry.path(), &store.join(named))?;
    }

    tisty_core::store::read_all(store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    Ok(brought)
}

/// In order: a `000002` without its `000001` stops the whole store, not one device.
fn copy_segments(from: &Path, into: &Path) -> Result<usize, Trouble> {
    let Ok(entries) = std::fs::read_dir(from) else {
        return Ok(0);
    };
    let mut carried: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|at| at.is_file() && at.extension().is_some_and(|e| e == "tisty"))
        .collect();
    carried.sort();

    std::fs::create_dir_all(into).map_err(io)?;
    let mut done = 0;
    for at in carried {
        let Some(named) = at.file_name() else {
            continue;
        };
        let target = into.join(named);
        if same(&at, &target) {
            continue;
        }
        let counter = at.with_extension("count");
        if let Some(named) = counter.file_name().filter(|_| counter.is_file()) {
            let body = std::fs::read(&counter).map_err(io)?;
            write(&into.join(named), &body)?;
        }
        let body = std::fs::read(&at).map_err(io)?;
        write(&target, &body)?;
        done += 1;
    }
    Ok(done)
}

fn same(from: &Path, to: &Path) -> bool {
    match (std::fs::metadata(from), std::fs::metadata(to)) {
        (Ok(a), Ok(b)) => a.len() == b.len(),
        _ => false,
    }
}

fn write(at: &Path, body: &[u8]) -> Result<(), Trouble> {
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent).map_err(io)?;
    }
    let tmp = at.with_extension(format!("{}.part", std::process::id()));
    std::fs::write(&tmp, body).map_err(io)?;
    std::fs::rename(&tmp, at).map_err(io)
}

fn io(e: std::io::Error) -> Trouble {
    Trouble::NotThere(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tisty_core::event::{DeviceId, TaskAdd};
    use tisty_core::{Op, Store};
    use ulid::Ulid;

    struct Machine {
        _dir: tempfile::TempDir,
        store: PathBuf,
        device: String,
    }

    fn machine(named: &str) -> Machine {
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("store");
        let mut held = Store::open(&store, DeviceId(named.into())).unwrap();
        held.append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(format!("lo de {named}"), "a0"),
        })
        .unwrap();
        Machine {
            _dir: dir,
            store,
            device: named.into(),
        }
    }

    fn titles(store: &Path) -> Vec<String> {
        tisty_core::State::replay(&tisty_core::store::read_all(store).unwrap())
            .tasks
            .values()
            .map(|task| task.title.clone())
            .collect()
    }

    #[test]
    fn what_one_machine_leaves_the_other_takes_home() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();

        carry(&one.store, &one.device, shared.path(), Way::Both).unwrap();
        carry(&other.store, &other.device, shared.path(), Way::Both).unwrap();
        carry(&one.store, &one.device, shared.path(), Way::Both).unwrap();

        let mine = titles(&one.store);
        assert!(mine.contains(&"lo de dev_a".to_string()), "{mine:?}");
        assert!(mine.contains(&"lo de dev_b".to_string()), "{mine:?}");
        assert_eq!(titles(&other.store).len(), 2);
    }

    #[test]
    fn nobody_ever_writes_over_their_own_directory() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.store, &one.device, shared.path(), Way::Both).unwrap();

        std::fs::write(shared.path().join("store/dev_a/active.tisty"), b"").unwrap();

        carry(&one.store, &one.device, shared.path(), Way::Pull).unwrap();
        assert_eq!(titles(&one.store).len(), 1, "the emptied copy came home");
    }

    #[test]
    fn what_is_left_behind_is_never_removed() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let stranger = shared.path().join("store/dev_z");
        std::fs::create_dir_all(&stranger).unwrap();
        std::fs::write(stranger.join("keep.txt"), b"not ours").unwrap();

        carry(&one.store, &one.device, shared.path(), Way::Both).unwrap();
        assert!(stranger.join("keep.txt").exists());
    }

    #[test]
    fn a_folder_of_another_store_is_refused_before_anything_moves() {
        let one = machine("dev_a");
        std::fs::write(one.store.join(MARKER), b"01OURS").unwrap();
        let shared = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(shared.path().join("store")).unwrap();
        std::fs::write(shared.path().join("store").join(MARKER), b"01THEIRS").unwrap();

        let Err(Trouble::OtherStore { theirs }) =
            carry(&one.store, &one.device, shared.path(), Way::Both)
        else {
            panic!("two histories were about to be merged");
        };
        assert_eq!(theirs, "01THEIRS");
        assert!(
            !shared.path().join("store/dev_a").exists(),
            "something moved"
        );
    }

    #[test]
    fn a_meeting_place_that_is_not_there_says_so() {
        let one = machine("dev_a");
        let gone = one.store.join("unplugged");

        assert!(matches!(
            carry(&one.store, &one.device, &gone, Way::Both),
            Err(Trouble::NotThere(_))
        ));
    }

    #[test]
    fn one_direction_only_does_one_direction() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.store, &other.device, shared.path(), Way::Push).unwrap();

        carry(&one.store, &one.device, shared.path(), Way::Push).unwrap();
        assert_eq!(titles(&one.store).len(), 1, "a push brought something back");

        carry(&one.store, &one.device, shared.path(), Way::Pull).unwrap();
        assert_eq!(titles(&one.store).len(), 2);
    }

    /// A second machine of the same person is not a second store: it takes the
    /// name the folder already has instead of insisting on its own.
    #[test]
    fn a_machine_meeting_the_folder_for_the_first_time_adopts_its_name() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();

        carry(&one.store, &one.device, shared.path(), Way::Both).unwrap();
        carry(&other.store, &other.device, shared.path(), Way::Both).unwrap();

        assert_eq!(
            tisty_core::store::peek_identity(&other.store),
            tisty_core::store::peek_identity(&one.store)
        );
    }

    #[test]
    fn syncing_twice_over_carries_nothing_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let first = carry(&one.store, &one.device, shared.path(), Way::Push).unwrap();
        let again = carry(&one.store, &one.device, shared.path(), Way::Push).unwrap();

        assert!(first.sent > 0);
        assert_eq!(again.sent, 0, "it copied what was already identical");
    }
}
