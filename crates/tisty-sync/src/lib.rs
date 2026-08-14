use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_core::witness::{self, Fact, channel};

pub use tisty_core::store::MARKER;

const STORE: &str = "store";
const HELD: &str = "attachments";
const PAPERS: &str = "docs";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    NotThere(String),
    OtherStore { theirs: String },
    Unreadable(String),
    Refused(String),
    Broke(String),
    WouldReset { theirs: String },
    NotAllowed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    Both,
    Push,
    Pull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undecided {
    pub id: String,
    pub theirs: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Moved {
    pub sent: usize,
    pub brought: usize,
    pub undecided: Vec<Undecided>,
}

pub fn carry(
    data: &Path,
    device: &str,
    dest: &Path,
    way: Way,
    alive: &[String],
) -> Result<Moved, Trouble> {
    if !dest.is_dir() {
        return Err(Trouble::NotThere(dest.display().to_string()));
    }
    let store = data.join(STORE);
    let ours = settled(&store, dest)?;

    let mut moved = Moved::default();
    if matches!(way, Way::Both | Way::Pull) {
        moved.brought = bring(&store, device, dest)?;
        moved.brought += copy_held(&dest.join(HELD), &data.join(HELD))?;
    }
    if matches!(way, Way::Both | Way::Push) {
        let who = tisty_core::event::DeviceId(device.to_string());
        let said =
            tisty_core::store::ledger(&store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
        if !said.may_write(&who) {
            return Err(Trouble::NotAllowed(device.to_string()));
        }
        write(&dest.join(STORE).join(MARKER), ours.as_bytes())?;
        moved.sent = copy_segments(&store.join(device), &dest.join(STORE).join(device))?;
        moved.sent += copy_held(&data.join(HELD), &dest.join(HELD))?;
    }
    if matches!(way, Way::Both) && !alive.is_empty() {
        let papers = carry_papers(data, dest, alive)?;
        moved.sent += papers.sent;
        moved.brought += papers.brought;
        moved.undecided = papers.undecided;
    }
    Ok(moved)
}

fn settled(store: &Path, dest: &Path) -> Result<String, Trouble> {
    let ours = tisty_core::store::peek_identity(store);
    let theirs = theirs(dest);
    let we_are_new = ours.is_none() && !tisty_core::store::inhabited(store);
    let they_are_new = theirs.is_none() && !tisty_core::store::inhabited(dest.join(STORE));

    if let (Some(ours), Some(theirs)) = (&ours, &theirs) {
        claims(theirs, ours)?;
        return Ok(ours.clone());
    }
    match (&ours, &theirs) {
        (None, Some(theirs)) if we_are_new => {
            write(&store.join(MARKER), theirs.as_bytes())?;
            return Ok(theirs.clone());
        }
        (Some(ours), None) if they_are_new => return Ok(ours.clone()),
        (None, None) if they_are_new => {
            return tisty_core::store::identity(store)
                .map_err(|e| Trouble::Unreadable(e.to_string()));
        }
        _ => {}
    }

    Err(Trouble::WouldReset {
        theirs: theirs.unwrap_or_else(|| dest.display().to_string()),
    })
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

pub fn theirs(dest: &Path) -> Option<String> {
    tisty_core::store::peek_identity(dest.join(STORE))
}

fn bring(store: &Path, device: &str, dest: &Path) -> Result<usize, Trouble> {
    let mut brought = 0;
    let at = dest.join(STORE);
    let entries = match std::fs::read_dir(&at) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::SYNC,
                    "folder unreadable",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
            }
            return Ok(0);
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let named = entry.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        if named.eq_ignore_ascii_case(device) || !entry.path().is_dir() {
            continue;
        }
        let mine = store.join(named);
        if !settled_already(&entry.path(), &mine) {
            tisty_core::store::check_device(&entry.path())
                .map_err(|e| Trouble::Unreadable(e.to_string()))?;
        }
        brought += copy_segments(&entry.path(), &mine)?;
    }

    if brought > 0 {
        tisty_core::store::read_all(store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    }
    Ok(brought)
}

fn settled_already(theirs: &Path, mine: &Path) -> bool {
    let Ok(offered) = tisty_core::store::segments_in(theirs) else {
        return false;
    };
    !offered.is_empty()
        && offered.iter().all(|at| {
            at.file_name()
                .is_some_and(|named| same(at, &mine.join(named)))
        })
}

fn copy_segments(from: &Path, into: &Path) -> Result<usize, Trouble> {
    let mut carried = match tisty_core::store::segments_in(from) {
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
    carried.sort();

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
            if !same(&counter, &target) {
                copy_onto(&counter, &target)?;
            }
        }

        let target = into.join(named);
        if same(&at, &target) {
            continue;
        }
        copy_onto(&at, &target)?;
        done += 1;
    }
    Ok(done)
}

fn sweep(dir: &Path) {
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

fn copy_held(from: &Path, into: &Path) -> Result<usize, Trouble> {
    let mut done = 0;
    let shelves = match std::fs::read_dir(from) {
        Ok(shelves) => shelves,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::SYNC,
                    "attachments unreadable",
                    &[
                        ("at", Fact::Path(from.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return Ok(0);
        }
    };
    for shelf in shelves.filter_map(|e| e.ok()) {
        if !shelf.path().is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        sweep(&into.join(shelf.file_name()));
        for file in files.filter_map(|e| e.ok()) {
            let at = file.path();
            if !at.is_file() {
                continue;
            }
            let Some(rest) = at.strip_prefix(from).ok() else {
                continue;
            };
            let target = into.join(rest);
            if std::fs::metadata(&at).map(|m| m.len()).ok()
                == std::fs::metadata(&target).map(|m| m.len()).ok()
            {
                continue;
            }
            copy_onto(&at, &target)?;
            done += 1;
        }
    }
    Ok(done)
}

fn same(from: &Path, to: &Path) -> bool {
    let (Ok(a), Ok(b)) = (std::fs::metadata(from), std::fs::metadata(to)) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    match (a.modified(), b.modified()) {
        (Ok(a), Ok(b)) => a
            .duration_since(b)
            .or_else(|_| b.duration_since(a))
            .is_ok_and(|apart| apart <= std::time::Duration::from_secs(2)),
        _ => false,
    }
}

static ROUND: AtomicU64 = AtomicU64::new(0);

fn write(at: &Path, body: &[u8]) -> Result<(), Trouble> {
    written(at, body, None)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keep {
    Mine,
    Theirs,
    Both,
}

pub fn settle(data: &Path, dest: &Path, id: &str, keep: Keep) -> Result<Option<String>, Trouble> {
    use tisty_core::docs::{Carried, print_of};

    let mine = tisty_core::docs::resolve(&data.join(PAPERS), id)
        .map_err(|_| Trouble::Refused(id.to_string()))?;
    let theirs = tisty_core::docs::resolve(&dest.join(PAPERS), id)
        .map_err(|_| Trouble::Refused(id.to_string()))?;
    let mut said = Carried::read(data);

    if keep == Keep::Both {
        return Ok(Some(std::fs::read_to_string(&theirs).map_err(io)?));
    }

    match keep {
        Keep::Theirs => {
            std::fs::create_dir_all(data.join(PAPERS)).map_err(io)?;
            copy_onto(&theirs, &mine)?;
        }
        _ => {
            std::fs::create_dir_all(dest.join(PAPERS)).map_err(io)?;
            copy_onto(&mine, &theirs)?;
        }
    }

    match print_of(&mine) {
        Ok(Some(print)) => said.keep(id, &print),
        Ok(None) => said.forget(id),
        Err(e) => return Err(io(e)),
    }
    said.save(data)
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    Ok(None)
}

pub fn carry_papers(data: &Path, dest: &Path, alive: &[String]) -> Result<Moved, Trouble> {
    use tisty_core::docs::{Carried, Move, moved, print_of};

    let here = data.join(PAPERS);
    let there = dest.join(PAPERS);
    let mut said = Carried::read(data);
    let mut done = Moved::default();

    let outcome = (|| -> Result<(), Trouble> {
        for id in alive {
            let (Ok(mine), Ok(theirs)) = (
                tisty_core::docs::resolve(&here, id),
                tisty_core::docs::resolve(&there, id),
            ) else {
                witness::warn(
                    channel::SYNC,
                    "a document was named in a way no document can be named",
                    &[("at", Fact::Id(id.clone()))],
                );
                continue;
            };
            let (ours, yours) = match (print_of(&mine), print_of(&theirs)) {
                (Ok(ours), Ok(yours)) => (ours, yours),
                (here, there) => {
                    witness::warn(
                        channel::SYNC,
                        "a document could not be read, so this turn leaves it alone",
                        &[("at", Fact::Id(id.clone()))],
                    );
                    let _ = (here, there);
                    continue;
                }
            };

            match moved(said.of(id), ours.as_deref(), yours.as_deref()) {
                Move::Nothing => {
                    if let Some(print) = ours.or(yours) {
                        said.keep(id, &print);
                    }
                }
                Move::Send => {
                    std::fs::create_dir_all(&there).map_err(io)?;
                    copy_onto(&mine, &theirs)?;
                    done.sent += 1;
                    if let Some(print) = ours {
                        said.keep(id, &print);
                    }
                }
                Move::Bring => {
                    std::fs::create_dir_all(&here).map_err(io)?;
                    copy_onto(&theirs, &mine)?;
                    done.brought += 1;
                    if let Some(print) = yours {
                        said.keep(id, &print);
                    }
                }
                Move::TheyDecide => done.undecided.push(Undecided {
                    id: id.clone(),
                    theirs: yours.unwrap_or_default(),
                }),
            }
        }
        Ok(())
    })();

    said.save(data)
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    outcome?;
    Ok(done)
}

fn copy_onto(from: &Path, at: &Path) -> Result<(), Trouble> {
    let body = std::fs::read(from).map_err(io)?;
    let when = std::fs::metadata(from).and_then(|m| m.modified()).ok();
    written(at, &body, when)
}

fn written(at: &Path, body: &[u8], when: Option<std::time::SystemTime>) -> Result<(), Trouble> {
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent).map_err(io)?;
    }
    let mine = ROUND.fetch_add(1, Ordering::Relaxed);
    let tmp = at.with_extension(format!("{}.{mine}.part", std::process::id()));

    let done = (|| {
        let file = std::fs::File::create(&tmp)?;
        std::io::Write::write_all(&mut &file, body)?;
        file.sync_all()?;
        if let Some(when) = when {
            file.set_modified(when)?;
        }
        std::fs::rename(&tmp, at)
    })();

    if let Err(e) = done {
        let _ = std::fs::remove_file(&tmp);
        witness::warn(
            channel::SYNC,
            "file not carried",
            &[
                ("at", Fact::Path(at.to_path_buf())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        return Err(io(e));
    }
    Ok(())
}

fn io(e: std::io::Error) -> Trouble {
    match e.kind() {
        std::io::ErrorKind::NotFound => Trouble::NotThere(e.to_string()),
        std::io::ErrorKind::PermissionDenied => Trouble::Refused(e.to_string()),
        _ => Trouble::Broke(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tisty_core::event::{DeviceId, TaskAdd};
    use tisty_core::{Op, Store};
    use ulid::Ulid;

    struct Machine {
        _dir: tempfile::TempDir,
        data: PathBuf,
        store: PathBuf,
        device: String,
    }

    fn blank(named: &str) -> Machine {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data");
        let store = data.join("store");
        Machine {
            _dir: dir,
            data,
            store,
            device: named.into(),
        }
    }

    fn machine(named: &str) -> Machine {
        let one = blank(named);
        wrote(&one, format!("lo de {named}"));
        one
    }

    fn wrote(who: &Machine, title: String) {
        let mut held = Store::open(&who.store, DeviceId(who.device.clone())).unwrap();
        held.append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(title, "a0"),
        })
        .unwrap();
    }

    fn joined(who: &Machine, shared: &Path) {
        carry(&who.data, &who.device, shared, Way::Pull, &[]).unwrap();
    }

    fn titles(store: &Path) -> Vec<String> {
        tisty_core::State::replay(&tisty_core::store::read_all(store).unwrap())
            .tasks
            .values()
            .map(|task| task.title.clone())
            .collect()
    }

    fn says(who: &Machine, op: Op) {
        let mut held = Store::open(&who.store, DeviceId(who.device.clone())).unwrap();
        held.append(op).unwrap();
    }

    #[test]
    fn a_store_with_no_list_yet_lets_everyone_write() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(moved.sent > 0, "an older store must not be locked out");
    }

    #[test]
    fn a_machine_on_the_list_writes_as_it_always_did() {
        let one = machine("dev_a");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_a".into()),
            },
        );
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(moved.sent > 0);
    }

    fn paper(who: &Machine, id: &str, body: &str) {
        let at = who.data.join("docs");
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join(format!("{id}.md")), body).unwrap();
    }

    fn theirs(shared: &Path, id: &str, body: &str) {
        let at = shared.join("docs");
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join(format!("{id}.md")), body).unwrap();
    }

    fn body(at: &Path, id: &str) -> String {
        std::fs::read_to_string(at.join("docs").join(format!("{id}.md"))).unwrap()
    }

    fn at_odds(one: &Machine, shared: &Path) -> Vec<String> {
        let alive = vec!["dev_a-0001".to_string()];
        paper(one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared, &alive).unwrap();
        paper(one, "dev_a-0001", "# Minuta\n\nlo mio");
        theirs(shared, "dev_a-0001", "# Minuta\n\nlo suyo");
        alive
    }

    #[test]
    fn keeping_mine_leaves_the_folder_holding_mine_and_asks_no_more() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());

        let brought = settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        assert_eq!(brought, None);
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo mio");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert!(
            done.undecided.is_empty(),
            "it asked again about a settled one"
        );
    }

    #[test]
    fn what_is_written_after_settling_travels_instead_of_being_asked_about_again() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio, y algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling it did not become the new common ground"
        );
        assert_eq!(done.sent, 1);
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nlo mio, y algo mas"
        );
    }

    #[test]
    fn keeping_theirs_takes_it_home_and_asks_no_more() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());

        settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs).unwrap();

        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo suyo");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert!(
            done.undecided.is_empty(),
            "it asked again about a settled one"
        );
    }

    #[test]
    fn keeping_both_hands_back_the_other_body_before_anything_is_overwritten() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());

        let brought = settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();

        assert_eq!(brought.as_deref(), Some("# Minuta\n\nlo suyo"));
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nlo suyo",
            "it overwrote the other version before it was anywhere safe"
        );
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo mio");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(done.undecided.len(), 1, "it settled before being told to");
    }

    #[test]
    fn keeping_both_settles_once_the_other_version_is_safe() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();

        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo mio");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert!(done.undecided.is_empty());
    }

    #[test]
    fn a_name_no_document_could_have_never_reaches_the_disk() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let loot = shared.path().join("loot.md");
        std::fs::write(&loot, "no es tuyo").unwrap();

        let done = carry_papers(
            &one.data,
            shared.path(),
            &["../loot".to_string(), "../../loot".to_string()],
        )
        .unwrap();

        assert_eq!(done.sent + done.brought, 0, "it walked out of the store");
        assert_eq!(std::fs::read_to_string(&loot).unwrap(), "no es tuyo");
        let said = std::fs::read_to_string(one.data.join("carried.json")).unwrap_or_default();
        assert!(
            !said.contains("loot"),
            "the ledger learned a name it must not know"
        );
    }

    #[test]
    fn settling_a_name_no_document_could_have_is_refused() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let why = settle(&one.data, shared.path(), "../../loot", Keep::Theirs);

        assert!(why.is_err(), "it settled a document that cannot exist");
    }

    #[test]
    fn a_document_written_here_lands_in_the_folder() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");

        let done = carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap();

        assert_eq!(done.sent, 1);
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo que dije");
    }

    #[test]
    fn a_document_only_the_folder_has_comes_home() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        theirs(shared.path(), "dev_b-0001", "# Suya");

        let done = carry_papers(&one.data, shared.path(), &["dev_b-0001".into()]).unwrap();

        assert_eq!(done.brought, 1);
        assert_eq!(body(&one.data, "dev_b-0001"), "# Suya");
    }

    #[test]
    fn only_the_side_that_changed_travels_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        theirs(shared.path(), "dev_a-0001", "# Minuta\n\ny algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(done.brought, 1, "what changed there did not come home");
        assert_eq!(done.sent, 0, "it pushed over what it had just been given");
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\ny algo mas");
    }

    #[test]
    fn when_both_sides_moved_it_asks_instead_of_choosing() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio");
        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo suyo");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(done.undecided.len(), 1);
        assert_eq!(done.undecided[0].id, "dev_a-0001");
        assert_eq!(done.sent + done.brought, 0, "it moved something anyway");
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo mio");
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo suyo");
    }

    #[test]
    fn a_document_the_log_never_mentions_is_left_where_it_is() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        theirs(shared.path(), "dev_b-0009", "# Nadie la nombra");

        let done = carry_papers(&one.data, shared.path(), &[]).unwrap();

        assert_eq!(done.brought, 0);
        assert!(!one.data.join("docs").join("dev_b-0009.md").exists());
    }

    #[test]
    fn asking_twice_asks_twice_instead_of_deciding_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared.path(), &alive).unwrap();
        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio");
        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo suyo");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(done.undecided.len(), 1, "it settled it on its own");
    }

    #[test]
    fn settling_with_theirs_also_lets_the_next_edit_travel_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo suyo, y algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling with theirs did not become the new common ground"
        );
        assert_eq!(done.sent, 1);
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nlo suyo, y algo mas"
        );
    }

    #[test]
    fn settling_with_both_lets_a_later_local_edit_travel_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio, y algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling both did not become the new common ground"
        );
        assert_eq!(done.sent, 1);
    }

    #[test]
    fn settling_with_both_lets_a_later_remote_edit_travel_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        theirs(
            shared.path(),
            "dev_a-0001",
            "# Minuta\n\nlo mio, visto desde el otro lado",
        );
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling both did not become the new common ground on the other side"
        );
        assert_eq!(done.brought, 1);
    }

    #[test]
    fn four_alternating_synchronizations_converge_without_losing_a_single_edit() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];

        paper(&one, "dev_a-0001", "# Minuta\n\nversion uno");
        let first = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(first.sent, 1, "the first version never left");

        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nversion dos");
        let second = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(second.brought, 1, "the second version did not come home");
        assert!(second.undecided.is_empty());
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nversion dos");

        paper(&one, "dev_a-0001", "# Minuta\n\nversion tres");
        let third = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(third.sent, 1, "the third version never left");
        assert!(third.undecided.is_empty());
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nversion tres"
        );

        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nversion cuatro");
        let fourth = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(fourth.brought, 1, "the fourth version did not come home");
        assert!(fourth.undecided.is_empty());

        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nversion cuatro");
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nversion cuatro"
        );
    }

    #[test]
    fn a_body_missing_from_the_shared_folder_is_sent_back_not_left_absent() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        std::fs::remove_file(shared.path().join("docs").join("dev_a-0001.md")).unwrap();
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.sent, 1,
            "a body only the folder lost was not sent back"
        );
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo que dije");
    }

    #[test]
    fn a_body_deleted_here_by_accident_comes_back_from_what_the_folder_still_has() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        std::fs::remove_file(one.data.join("docs").join("dev_a-0001.md")).unwrap();
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.brought, 1,
            "a body only lost here was not brought back"
        );
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo que dije");
    }

    #[test]
    fn a_body_gone_from_both_sides_is_not_forgotten_and_comes_back_clean_once_one_side_writes_again()
     {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta\n\nversion uno");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        std::fs::remove_file(one.data.join("docs").join("dev_a-0001.md")).unwrap();
        std::fs::remove_file(shared.path().join("docs").join("dev_a-0001.md")).unwrap();
        let vanished = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(vanished.sent + vanished.brought, 0);
        assert!(vanished.undecided.is_empty());

        paper(&one, "dev_a-0001", "# Minuta\n\nvuelve distinta");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.sent, 1,
            "a body that came back was held against its old self"
        );
        assert!(done.undecided.is_empty());
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nvuelve distinta"
        );
    }

    #[test]
    fn two_sides_that_reach_the_same_words_on_their_own_settle_without_being_asked() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        let conflicted = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(conflicted.undecided.len(), 1);

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mismo al fin");
        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo mismo al fin");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "it kept asking after both sides said the same thing"
        );
        assert_eq!(
            done.sent + done.brought,
            0,
            "it moved something nobody had changed anywhere"
        );
    }

    #[test]
    fn content_no_editor_would_be_proud_of_still_crosses_byte_for_byte() {
        let long = "y".repeat(600 * 1024);
        let cases: [(&str, &str); 6] = [
            ("empty", ""),
            ("blank", "   \n\t  \n  "),
            ("long, well past what the editor would ever open", &long),
            ("accented", "café ñandú 日本語 🎉 texto con acentós"),
            ("windows line endings", "una linea\r\notra linea\r\n"),
            ("a byte order mark up front", "\u{FEFF}# Titulo\n\ncuerpo"),
        ];

        for (label, first) in cases {
            let one = blank("dev_a");
            let shared = tempfile::tempdir().unwrap();
            let alive = ["dev_a-0001".to_string()];
            paper(&one, "dev_a-0001", first);

            let sent = carry_papers(&one.data, shared.path(), &alive).unwrap();
            assert_eq!(sent.sent, 1, "did not travel on: {label}");
            assert_eq!(
                body(shared.path(), "dev_a-0001"),
                first,
                "bytes changed in transit on: {label}"
            );

            let still = carry_papers(&one.data, shared.path(), &alive).unwrap();
            assert_eq!(
                still.sent + still.brought,
                0,
                "moved again with nothing changed on: {label}"
            );

            let edited = format!("{first}\nmas");
            theirs(shared.path(), "dev_a-0001", &edited);
            let round = carry_papers(&one.data, shared.path(), &alive).unwrap();
            assert_eq!(
                round.brought, 1,
                "the edit on top did not come home on: {label}"
            );
            assert_eq!(
                body(&one.data, "dev_a-0001"),
                edited,
                "bytes changed coming home on: {label}"
            );
        }
    }

    #[test]
    fn a_machine_nobody_ever_named_is_not_locked_out_by_someone_elses_list() {
        let one = machine("dev_a");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(
            moved.sent > 0,
            "the first machine to join shut the door on the rest"
        );
    }

    #[test]
    fn being_named_once_and_dropped_is_not_the_same_as_never_being_named() {
        let said = tisty_core::store::Ledger {
            allowed: [DeviceId("dev_b".into())].into(),
            named: [DeviceId("dev_a".into()), DeviceId("dev_b".into())].into(),
        };

        assert!(!said.may_write(&DeviceId("dev_a".into())));
        assert!(said.may_write(&DeviceId("dev_b".into())));
        assert!(said.may_write(&DeviceId("dev_c".into())));
        assert!(said.was_removed(&DeviceId("dev_a".into())));
        assert!(!said.was_removed(&DeviceId("dev_c".into())));
    }

    #[test]
    fn two_machines_removing_each_other_at_once_do_not_brick_the_store() {
        let said = tisty_core::store::Ledger {
            allowed: Default::default(),
            named: [DeviceId("dev_a".into()), DeviceId("dev_b".into())].into(),
        };

        assert!(
            said.may_write(&DeviceId("dev_a".into())),
            "nobody could ever write here again"
        );
        assert!(said.may_write(&DeviceId("dev_b".into())));
    }

    #[test]
    fn a_machine_that_was_removed_writes_nothing_at_all() {
        let one = machine("dev_a");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_a".into()),
            },
        );
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId("dev_a".into()),
            },
        );
        let shared = tempfile::tempdir().unwrap();

        let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

        assert!(
            matches!(why, Trouble::NotAllowed(_)),
            "it went through: {why:?}"
        );
        assert!(
            !shared.path().join(STORE).join("dev_a").exists(),
            "it wrote before refusing"
        );
    }

    #[test]
    fn a_machine_that_was_removed_still_brings_what_is_there() {
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let one = blank("dev_a");
        joined(&one, shared.path());
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId("dev_a".into()),
            },
        );

        wrote(&other, "lo dicho despues".into());
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
        let moved = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        assert!(moved.brought > 0, "being removed is not being cut off");
        assert!(
            titles(&one.store).contains(&"lo dicho despues".to_string()),
            "{:?}",
            titles(&one.store)
        );
    }

    #[test]
    fn the_word_that_removes_it_is_read_before_it_writes() {
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let one = blank("dev_a");
        joined(&one, shared.path());

        says(
            &other,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        says(
            &other,
            Op::DeviceRemove {
                d: DeviceId("dev_a".into()),
            },
        );
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

        assert!(
            matches!(why, Trouble::NotAllowed(_)),
            "it pushed on stale news: {why:?}"
        );
    }

    #[test]
    fn what_one_machine_leaves_the_other_takes_home() {
        let one = machine("dev_a");
        let other = blank("dev_b");
        let shared = tempfile::tempdir().unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        joined(&other, shared.path());
        wrote(&other, "lo de dev_b".into());
        carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let mine = titles(&one.store);
        assert!(mine.contains(&"lo de dev_a".to_string()), "{mine:?}");
        assert!(mine.contains(&"lo de dev_b".to_string()), "{mine:?}");
        assert_eq!(titles(&other.store).len(), 2);
    }

    #[test]
    fn nobody_ever_writes_over_their_own_directory() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        std::fs::write(shared.path().join("store/dev_a/active.tisty"), b"").unwrap();

        let _ = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
        assert_eq!(titles(&one.store).len(), 1, "the emptied copy came home");
    }

    #[test]
    fn a_directory_that_differs_only_in_case_is_still_our_own() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let theirs = shared.path().join("store/DEV_A");
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(theirs.join("active.tisty"), b"").unwrap();

        let _ = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
        assert_eq!(titles(&one.store).len(), 1, "our own log was overwritten");
    }

    #[test]
    fn what_is_left_behind_is_never_removed() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let stranger = shared.path().join("store/dev_z");
        std::fs::create_dir_all(&stranger).unwrap();
        std::fs::write(stranger.join("keep.txt"), b"not ours").unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
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
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
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
    fn two_histories_are_never_joined_at_all() {
        let one = machine("dev_a");
        std::fs::remove_file(one.store.join(MARKER)).ok();
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let Err(Trouble::WouldReset { .. }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("two histories were joined");
        };

        assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
        assert!(
            !shared.path().join("store/dev_a").exists(),
            "something moved"
        );

        let again = carry(&one.data, &one.device, shared.path(), Way::Both, &[]);
        assert!(
            matches!(again, Err(Trouble::WouldReset { .. })),
            "asking twice is not consent: {again:?}"
        );
        assert_eq!(titles(&one.store).len(), 1, "it joined them anyway");
    }

    #[test]
    fn a_store_with_history_and_no_marker_is_not_adopted() {
        let one = machine("dev_a");
        std::fs::remove_file(one.store.join(MARKER)).ok();
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let Err(Trouble::WouldReset { .. }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("an unmarked store merged into a stranger's history");
        };
        assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
    }

    #[test]
    fn a_folder_full_of_history_with_no_marker_is_refused() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
        std::fs::remove_file(shared.path().join("store").join(MARKER)).unwrap();

        let Err(Trouble::WouldReset { .. }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("a folder with history and no marker was treated as empty");
        };
        assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
    }

    #[test]
    fn a_meeting_place_that_is_not_there_says_so() {
        let one = machine("dev_a");
        let gone = one.store.join("unplugged");

        assert!(matches!(
            carry(&one.data, &one.device, &gone, Way::Both, &[]),
            Err(Trouble::NotThere(_))
        ));
    }

    #[test]
    fn one_direction_only_does_one_direction() {
        let one = blank("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        assert!(
            titles(&one.store).is_empty(),
            "a push brought something back"
        );

        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
        assert_eq!(titles(&one.store).len(), 1);
    }

    #[test]
    fn a_machine_meeting_the_folder_for_the_first_time_adopts_its_name() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        std::fs::remove_file(other.store.join(MARKER)).ok();
        let shared = tempfile::tempdir().unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        std::fs::remove_dir_all(other.store.join(&other.device)).unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();

        assert_eq!(
            tisty_core::store::peek_identity(&other.store),
            tisty_core::store::peek_identity(&one.store)
        );
    }

    #[test]
    fn syncing_twice_over_carries_nothing_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let first = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        let again = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        assert!(first.sent > 0);
        assert_eq!(again.sent, 0, "it copied what was already identical");
    }

    #[test]
    fn a_gap_in_what_is_offered_is_refused_without_importing_it() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let theirs = shared.path().join("store/dev_b");
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(theirs.join("000002.tisty"), b"").unwrap();
        std::fs::write(shared.path().join("store").join(MARKER), b"01M0THEIRSTORE").unwrap();

        let Err(Trouble::Unreadable(_)) =
            carry(&one.data, &one.device, shared.path(), Way::Pull, &[])
        else {
            panic!("a segment with no predecessor was imported");
        };
        assert!(!one.store.join("dev_b").exists(), "it landed anyway");
        assert!(titles(&one.store).is_empty(), "our own store still reads");
    }

    #[test]
    fn a_conflict_copy_is_not_a_segment() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let mine = shared.path().join("store/dev_a");
        let held = std::fs::read(mine.join("active.tisty")).unwrap();
        std::fs::write(mine.join("active (conflicted copy).tisty"), &held).unwrap();

        let other = blank("dev_b");
        carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            titles(&other.store).len(),
            1,
            "the conflict copy was read as history"
        );
    }

    #[test]
    fn a_pull_with_nothing_to_bring_does_not_reread_everything() {
        let one = blank("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        std::fs::write(
            shared.path().join("store/dev_b/000001.tisty"),
            b"not json at all",
        )
        .unwrap();
        std::fs::write(shared.path().join("store/dev_b/000001.count"), b"1").unwrap();

        let again = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
        assert!(again.is_err(), "a changed segment went unchecked");
        assert_eq!(titles(&one.store).len(), 1, "our store still reads");
    }

    #[test]
    fn attachments_travel_with_the_tasks_that_name_them() {
        let one = machine("dev_a");
        let shelf = one.data.join("attachments").join("ab");
        std::fs::create_dir_all(&shelf).unwrap();
        std::fs::write(shelf.join("cd.png"), b"a picture").unwrap();

        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let other = blank("dev_b");
        carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            std::fs::read(other.data.join("attachments/ab/cd.png")).unwrap(),
            b"a picture"
        );
    }
}
