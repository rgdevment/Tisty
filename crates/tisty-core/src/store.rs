use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::{
    Error, Result,
    event::{DeviceId, Event, KNOWN_OPS, Op, SCHEMA_VERSION},
    witness::{self, Fact, channel},
};

mod identity;

pub use identity::{
    KEEP, MARKER, brought_home, displaced, identity, kept_at, kept_before_the_store_goes,
    peek_identity, secret, secret_kept,
};

const ACTIVE: &str = "active.tisty";
const LOCK: &str = ".lock";
const LOCK_WAIT_MS: u64 = 500;
const LOCK_POLL_MS: u64 = 5;
const SEGMENT_MAX_EVENTS: usize = 5_000;

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    dir: PathBuf,
    device: DeviceId,
    active_events: usize,
    head: jiff::Timestamp,
    seq: u64,
    seen: u64,
    overtaken: bool,
    lock: Option<File>,
    via: Option<String>,
}

impl Store {
    pub fn open(store_root: impl AsRef<Path>, device: DeviceId) -> Result<Self> {
        let root = store_root.as_ref().to_path_buf();
        let dir = root.join(&device.0);
        std::fs::create_dir_all(&dir)?;
        if let Err(e) = crate::paths::ours_alone(&dir) {
            witness::warn(
                channel::STORE,
                "store not made private",
                &[
                    ("at", Fact::Path(dir.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
        if let Some(parent) = dir.parent() {
            let _ = crate::paths::ours_alone(parent);
        }

        if let Err(e) = identity(&root) {
            witness::warn(
                channel::STORE,
                "the store could not be given a name of its own",
                &[("why", Fact::Why(e.to_string()))],
            );
        }

        mend(&dir);
        let (active_events, head, seq) = tail_of(&dir.join(ACTIVE))?;
        let seen = active_size(&dir.join(ACTIVE));

        Ok(Self {
            active_events,
            head,
            seq,
            seen,
            overtaken: false,
            root,
            dir,
            device,
            via: None,
            lock: None,
        })
    }

    fn acquire(&mut self) -> Result<()> {
        if self.lock.is_some() {
            return Ok(());
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(self.dir.join(LOCK))?;

        let mut waited = 0;
        while let Err(held) = file.try_lock() {
            if let std::fs::TryLockError::Error(why) = held {
                return Err(why.into());
            }
            if waited >= LOCK_WAIT_MS {
                return Err(Error::AlreadyRunning);
            }
            std::thread::sleep(std::time::Duration::from_millis(LOCK_POLL_MS));
            waited += LOCK_POLL_MS;
        }

        self.lock = Some(file);
        self.catch_up()
    }

    fn catch_up(&mut self) -> Result<()> {
        let active = self.dir.join(ACTIVE);
        let size = active_size(&active);
        if size == self.seen {
            return Ok(());
        }
        self.overtaken = true;

        let (events, head, seq) = tail_of(&active)?;
        self.active_events = events;
        if (head, seq) > (self.head, self.seq) {
            self.head = head;
            self.seq = seq;
        }
        self.seen = size;
        Ok(())
    }

    fn locked<T>(&mut self, write: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.acquire()?;
        let out = write(self);
        self.lock = None;
        out
    }

    pub fn device(&self) -> &DeviceId {
        &self.device
    }

    pub fn overtaken(&self) -> bool {
        self.overtaken
    }

    pub fn append(&mut self, op: Op) -> Result<Event> {
        self.locked(|s| {
            let event = s.minted(op);
            s.write(&event)?;
            Ok(event)
        })
    }

    /// The client this store writes on behalf of, sealed into every event from here on.
    pub fn speaking_through(mut self, via: Option<String>) -> Self {
        self.via = via;
        self
    }

    fn minted(&mut self, op: Op) -> Event {
        let (timestamp, seq) = self.stamp();
        let optional = op.is_optional();
        let mut event = Event::new(self.device.clone(), timestamp, op);
        event.seq = seq;
        event.optional = optional;
        event.via = self.via.clone();
        // Read per write, not once at open: the window keeps a Store alive for the whole session
        // and a laptop that travels would keep stamping the zone it started in.
        event.zone = jiff::tz::TimeZone::system()
            .iana_name()
            .map(ToString::to_string);
        event
    }

    fn stamp(&mut self) -> (jiff::Timestamp, u64) {
        let now = jiff::Timestamp::now();
        if now > self.head {
            self.head = now;
            self.seq = 0;
        } else {
            self.seq += 1;
        }
        (self.head, self.seq)
    }

    pub fn append_batch(&mut self, ops: Vec<Op>) -> Result<Vec<Event>> {
        self.append_batch_marked(ops, false)
    }

    pub fn append_batch_marked(&mut self, ops: Vec<Op>, undo: bool) -> Result<Vec<Event>> {
        self.append_batch_tagged(ops, undo, false)
    }

    pub fn append_batch_tagged(
        &mut self,
        ops: Vec<Op>,
        undo: bool,
        redo: bool,
    ) -> Result<Vec<Event>> {
        let batch = (ops.len() > 1).then(ulid::Ulid::generate);

        self.locked(|s| {
            let mut written = Vec::with_capacity(ops.len());
            for op in ops {
                let mut event = s.minted(op);
                event.batch = batch;
                event.undo = undo;
                event.redo = redo;
                written.push(event);
            }
            s.write_all(&written)?;
            Ok(written)
        })
    }

    /// Writes only if `settled` still agrees once the lock is held. Reading the store, deciding,
    /// and then writing lets two callers both decide yes on the same absent thing.
    pub fn append_batch_unless(
        &mut self,
        ops: Vec<Op>,
        settled: impl FnOnce(&[Event]) -> bool,
    ) -> Result<Option<Vec<Event>>> {
        let batch = (ops.len() > 1).then(ulid::Ulid::generate);
        let root = self.root.clone();

        self.locked(|s| {
            let held = read_all(&root)?;
            if settled(&held) {
                return Ok(None);
            }
            // Judged against the whole log, written after the whole log: a clock behind another
            // machine's would otherwise stamp this before the event that let it through, and
            // the replay — sorted by stamp — would let it go everywhere.
            if let Some(newest) = held.iter().map(|one| one.timestamp).max() {
                s.after(newest);
            }
            let mut written = Vec::with_capacity(ops.len());
            for op in ops {
                let mut event = s.minted(op);
                event.batch = batch;
                written.push(event);
            }
            s.write_all(&written)?;
            Ok(Some(written))
        })
    }

    fn after(&mut self, newest: jiff::Timestamp) {
        if newest >= self.head {
            self.head = newest
                .checked_add(jiff::SignedDuration::from_micros(1))
                .unwrap_or(newest);
            self.seq = 0;
        }
    }

    pub fn append_event(&mut self, event: &Event) -> Result<()> {
        self.locked(|s| s.write(event))
    }

    fn write(&mut self, event: &Event) -> Result<()> {
        self.write_all(std::slice::from_ref(event))
    }

    /// One open and one `fsync` for the whole lot: a batch is durable before this returns, but a
    /// pasted list of two hundred does not pay the wait two hundred times over.
    fn write_all(&mut self, events: &[Event]) -> Result<()> {
        let mut at = 0;
        while at < events.len() {
            if self.active_events >= SEGMENT_MAX_EVENTS {
                self.rotate()?;
            }
            let room = SEGMENT_MAX_EVENTS - self.active_events;
            let lot = &events[at..(at + room).min(events.len())];

            let mut said = String::new();
            for event in lot {
                said.push_str(&serde_json::to_string(event)?);
                said.push('\n');
            }

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.dir.join(ACTIVE))?;
            file.write_all(said.as_bytes())?;
            file.sync_all()?;

            self.active_events += lot.len();
            at += lot.len();
        }
        self.seen = active_size(&self.dir.join(ACTIVE));
        Ok(())
    }

    fn rotate(&mut self) -> Result<()> {
        let active = self.dir.join(ACTIVE);
        if active.try_exists()? {
            let next = next_segment_number(&self.dir)?;
            let sealed = self.dir.join(format!("{next:06}.tisty"));
            std::fs::rename(&active, &sealed)?;

            let (lines, _, _) = tail_of(&sealed)?;
            write_atomic(
                &sealed.with_extension("count"),
                lines.to_string().as_bytes(),
            )?;
        }
        self.active_events = 0;
        self.seen = 0;
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<Event>> {
        read_all(&self.root)
    }
}

pub fn read_all(store_root: impl AsRef<Path>) -> Result<Vec<Event>> {
    let root = store_root.as_ref();
    let mut events = Vec::new();

    let devices = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(events),
        Err(e) => return Err(Error::Io(e)),
    };

    for device in devices {
        let device = device?;
        if !device.file_type()?.is_dir() {
            continue;
        }

        let segments = segments_in(&device.path())?;
        contiguous(&segments)?;

        for segment in segments {
            let closed = segment.file_name().is_some_and(|n| n != ACTIVE);
            let found = read_segment(&segment, &mut events)?;

            if closed {
                let declared = declared_count(&segment);
                if found == 0 || declared.is_some_and(|n| n != found) {
                    return Err(Error::TruncatedSegment {
                        file: segment.display().to_string(),
                        found,
                        declared,
                    });
                }
            }
        }
    }

    events.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    events.dedup_by(|a, b| a.sort_key() == b.sort_key());
    Ok(events)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Ledger {
    pub allowed: std::collections::BTreeSet<DeviceId>,
    pub named: std::collections::BTreeSet<DeviceId>,
}

impl Ledger {
    pub fn may_write(&self, who: &DeviceId) -> bool {
        self.allowed.is_empty() || self.allowed.contains(who) || !self.named.contains(who)
    }

    pub fn was_removed(&self, who: &DeviceId) -> bool {
        self.named.contains(who) && !self.allowed.contains(who)
    }
}

pub fn ledger(store_root: impl AsRef<Path>) -> Result<Ledger> {
    let events = read_all(store_root)?;
    let mut assistants: std::collections::BTreeSet<&DeviceId> = std::collections::BTreeSet::new();
    let mut told: Vec<&Event> = Vec::new();
    for one in &events {
        if let Op::DeviceJoin {
            d,
            k: Some(crate::event::DeviceKind::Agent),
        } = &one.op
            && d == &one.device
        {
            assistants.insert(d);
        }
        if !(one.op.destroys() && assistants.contains(&one.device)) {
            told.push(one);
        }
    }
    let gone: std::collections::BTreeSet<&DeviceId> = told
        .iter()
        .filter_map(|one| match &one.op {
            Op::DeviceRemove { d } => Some(d),
            _ => None,
        })
        .collect();

    let mut said = Ledger::default();
    for event in &told {
        match &event.op {
            Op::DeviceJoin { d, .. } => {
                said.named.insert(d.clone());
                if !gone.contains(d) {
                    said.allowed.insert(d.clone());
                }
            }
            Op::DeviceRemove { d } => {
                said.named.insert(d.clone());
                said.allowed.remove(d);
            }
            _ => {}
        }
    }
    Ok(said)
}

pub fn is_store_name(name: &str) -> bool {
    name.len() == 26
        && name
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

pub fn is_device_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 48
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn is_segment(name: &str) -> bool {
    name == ACTIVE
        || name.strip_suffix(".tisty").is_some_and(|stem| {
            (6..=10).contains(&stem.len()) && stem.bytes().all(|b| b.is_ascii_digit())
        })
}

pub fn segments_in(device_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(device_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|at| {
            at.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_segment)
        })
        .collect();
    found.sort();
    Ok(found)
}

pub fn inhabited(store_root: impl AsRef<Path>) -> bool {
    std::fs::read_dir(store_root.as_ref()).is_ok_and(|entries| {
        entries.filter_map(|e| e.ok()).any(|e| {
            e.file_type().is_ok_and(|kind| kind.is_dir())
                && segments_in(&e.path()).is_ok_and(|found| !found.is_empty())
        })
    })
}

pub fn distinct_in(device_dir: &Path) -> Result<usize> {
    let segments = segments_in(device_dir)?;
    let mut events = Vec::new();
    for segment in &segments {
        read_segment(segment, &mut events)?;
    }
    events.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    events.dedup_by(|a, b| a.sort_key() == b.sort_key());
    Ok(events.len())
}

pub fn newest_schema(device_dir: &Path) -> Result<u32> {
    for segment in segments_in(device_dir)?.iter().rev() {
        let Some(line) = last_line(segment)? else {
            continue;
        };
        if let Ok(one) = serde_json::from_str::<Stamped>(&line) {
            return Ok(one.v);
        }
    }
    Ok(0)
}

fn last_line(path: &Path) -> Result<Option<String>> {
    use std::io::{Read, Seek};

    let mut file = File::open(path)?;
    crate::counting::opened();
    let weighs = file.metadata()?.len();
    if weighs == 0 {
        return Ok(None);
    }
    let window = weighs.min(64 * 1024);
    file.seek(std::io::SeekFrom::Start(weighs - window))?;
    let mut read = Vec::new();
    file.read_to_end(&mut read)?;

    let text = String::from_utf8_lossy(&read);
    let whole = match window == weighs {
        true => text.as_ref(),
        false => match text.find('\n') {
            Some(at) => &text[at + 1..],
            None => return Ok(None),
        },
    };
    Ok(whole
        .lines()
        .rfind(|one| !one.trim().is_empty())
        .map(str::to_owned))
}

pub fn check_device(device_dir: &Path) -> Result<usize> {
    let segments = segments_in(device_dir)?;
    contiguous(&segments)?;

    let mut events = Vec::new();
    for segment in &segments {
        let found = read_segment(segment, &mut events)?;
        if segment.file_name().is_some_and(|n| n != ACTIVE) {
            let declared = declared_count(segment);
            if found == 0 || declared.is_some_and(|n| n != found) {
                return Err(Error::TruncatedSegment {
                    file: segment.display().to_string(),
                    found,
                    declared,
                });
            }
        }
    }
    Ok(events.len())
}

fn contiguous(segments: &[PathBuf]) -> Result<()> {
    let mut numbers: Vec<usize> = segments
        .iter()
        .filter_map(|p| p.file_stem()?.to_str()?.parse().ok())
        .collect();
    numbers.sort_unstable();

    for (i, found) in numbers.iter().enumerate() {
        let expected = i + 1;
        if *found != expected {
            return Err(Error::MissingSegment {
                number: expected,
                device: segments
                    .first()
                    .and_then(|p| p.parent()?.file_name()?.to_str())
                    .unwrap_or("?")
                    .to_string(),
            });
        }
    }
    Ok(())
}

fn declared_count(segment: &Path) -> Option<usize> {
    let at = segment.with_extension("count");
    match std::fs::read_to_string(&at).ok()?.trim().parse() {
        Ok(found) => Some(found),
        Err(_) => {
            witness::warn(
                channel::STORE,
                "segment tally unreadable",
                &[("at", Fact::Path(at))],
            );
            None
        }
    }
}

#[derive(serde::Deserialize)]
struct Stamped {
    v: u32,
    #[serde(default)]
    opt: bool,
    #[serde(default)]
    op: String,
}

/// A sealed segment declares lines, not events, so a skipped one must still be counted or the
/// count check reads it as a truncated download.
fn read_segment(path: &Path, out: &mut Vec<Event>) -> Result<usize> {
    read_segment_from(path, 0, out)
}

/// A segment only ever grows, so what sits past a known length is the whole of what is new.
pub fn read_tail(path: &Path, from: u64) -> Result<Vec<Event>> {
    let mut out = Vec::new();
    read_segment_from(path, from, &mut out)?;
    Ok(out)
}

fn read_segment_from(path: &Path, from: u64, out: &mut Vec<Event>) -> Result<usize> {
    let mut file = File::open(path)?;
    crate::counting::opened();
    if from > 0 {
        use std::io::Seek;
        file.seek(std::io::SeekFrom::Start(from))?;
    }
    let mut lines = 0;
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        lines += 1;

        let malformed = |source| Error::MalformedEvent {
            file: path.display().to_string(),
            line: i + 1,
            source,
        };
        let stamp: Stamped = serde_json::from_str(&line).map_err(malformed)?;

        let skipped = |why: String| {
            witness::warn(
                channel::STORE,
                "an optional event this build does not understand was skipped",
                &[
                    ("at", Fact::Path(path.to_path_buf())),
                    ("line", Fact::Count(i + 1)),
                    ("op", Fact::Id(stamp.op.clone())),
                    ("why", Fact::Why(why)),
                ],
            );
        };

        let strange = !KNOWN_OPS.contains(&stamp.op.as_str());
        if stamp.v > SCHEMA_VERSION {
            if !stamp.opt || !strange {
                return Err(Error::UnsupportedVersion(stamp.v));
            }
            skipped(format!("schema {}", stamp.v));
            continue;
        }

        // Only an operation this build has never heard of is forgiven, whatever schema it came
        // under; a known one that fails to parse is corruption, and waving it through would
        // erase what it said.
        match serde_json::from_str::<Event>(&line) {
            Ok(event) if !signed_by_its_directory(path, &event) => witness::warn(
                channel::STORE,
                "an event claims a device other than the one whose segment holds it, and was let go",
                &[
                    ("at", Fact::Path(path.to_path_buf())),
                    ("line", Fact::Count(i + 1)),
                    ("by", Fact::Id(event.device.0.clone())),
                ],
            ),
            Ok(event) => out.push(event),
            Err(source) if stamp.opt && strange => skipped(source.to_string()),
            Err(source) => return Err(malformed(source)),
        }
    }
    Ok(lines)
}

/// A device writes only into its own directory and sync copies directories whole, so the name
/// on the event and the name of the directory agree on everything Tisty ever wrote. One that
/// disagrees was written by a hand outside Tisty in another device's name, and every rule that
/// keys off the device — what an assistant may do — would take that name at its word.
fn signed_by_its_directory(path: &Path, event: &Event) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .is_none_or(|dir| dir == std::ffi::OsStr::new(&event.device.0))
}

pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    static TURN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let turn = TURN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = path.with_extension(format!("{}.{turn}.tmp", std::process::id()));
    let wrote = poured(&tmp, contents).and_then(|()| Ok(renamed(&tmp, path)?));
    if wrote.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    wrote
}

fn renamed(tmp: &Path, path: &Path) -> std::io::Result<()> {
    let mut wait = 10;
    for _ in 0..6 {
        match std::fs::rename(tmp, path) {
            Ok(()) => return Ok(()),
            Err(e) if !for_a_moment(&e) => return Err(e),
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(wait)),
        }
        wait *= 2;
    }
    std::fs::rename(tmp, path)
}

/// The two Windows raises for a file somebody else has open; every other refusal is final.
fn for_a_moment(e: &std::io::Error) -> bool {
    cfg!(windows) && matches!(e.raw_os_error(), Some(5) | Some(32))
}

fn poured(tmp: &Path, contents: &[u8]) -> Result<()> {
    let mut file = File::create(tmp)?;
    let _ = crate::paths::ours_alone(tmp);
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
#[path = "store_atomic_tests.rs"]
mod atomic_tests;

fn active_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

pub struct Alone(File);

impl Drop for Alone {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

pub fn alone(device_dir: &Path) -> Option<Alone> {
    std::fs::create_dir_all(device_dir).ok()?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(device_dir.join(LOCK))
        .ok()?;

    let mut waited = 0;
    while let Err(held) = file.try_lock() {
        if matches!(held, std::fs::TryLockError::Error(_)) || waited >= LOCK_WAIT_MS {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(LOCK_POLL_MS));
        waited += LOCK_POLL_MS;
    }
    Some(Alone(file))
}

/// A line is written whole or not at all, so one that will not parse at the very end of the
/// segment still being written is the half of an event a power cut took. It is set aside rather
/// than read, because refusing it would take every whole event before it down as well. Only ever
/// the last line, only ever this machine's own active segment: a sealed one has its count to
/// answer for, and another machine's history is not ours to mend.
fn mend(dir: &Path) {
    // Behind the same lock every writer takes: mending renames a fresh file over the old one, so
    // whatever somebody appended meanwhile would go down with the inode it was written to. A lock
    // that will not come means a writer is alive, and a live writer's tail is not torn.
    let Ok(guard) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(dir.join(LOCK))
    else {
        return;
    };
    if guard.try_lock().is_err() {
        return;
    }

    let path = &dir.join(ACTIVE);
    // Read as bytes: half of a multi-byte letter is not text, and refusing to look at it would
    // leave the very file this exists to save unreadable.
    let whole = match std::fs::read(path) {
        Ok(whole) => whole,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return,
        Err(why) => {
            witness::warn(
                channel::STORE,
                "the log could not be read to see whether a power cut left it torn",
                &[
                    ("at", Fact::Path(path.clone())),
                    ("why", Fact::Why(why.to_string())),
                ],
            );
            return;
        }
    };
    let shut = whole.iter().rposition(|one| *one == b'\n');
    let tail = match shut {
        Some(at) => &whole[at + 1..],
        None => &whole[..],
    };
    if tail.is_empty() {
        return;
    }

    // A tail that parses is a whole event whose newline the cut took. Give the newline back: an
    // append lands right behind it otherwise, and two events on one line take both down.
    let said = std::str::from_utf8(tail).ok();
    if said.is_some_and(|one| one.trim().is_empty() || serde_json::from_str::<Stamped>(one).is_ok())
    {
        let mut kept = whole.clone();
        kept.push(b'\n');
        if let Err(why) = write_atomic(path, &kept) {
            witness::warn(
                channel::STORE,
                "the last line of the log could not be given its newline back",
                &[
                    ("at", Fact::Path(path.to_path_buf())),
                    ("why", Fact::Why(why.to_string())),
                ],
            );
        }
        return;
    }

    let aside = path.with_extension("torn");
    if let Err(why) = std::fs::write(&aside, tail) {
        witness::error(
            channel::STORE,
            "a torn tail was found and could not be set aside, so the log is left as it is",
            &[
                ("at", Fact::Path(aside.clone())),
                ("bytes", Fact::Bytes(tail.len() as u64)),
                ("why", Fact::Why(why.to_string())),
            ],
        );
        return;
    }
    let kept: &[u8] = match shut {
        Some(at) => &whole[..=at],
        None => &[],
    };
    if let Err(why) = write_atomic(path, kept) {
        witness::warn(
            channel::STORE,
            "the half line a power cut left could not be set aside",
            &[
                ("at", Fact::Path(path.to_path_buf())),
                ("why", Fact::Why(why.to_string())),
            ],
        );
        return;
    }
    witness::warn(
        channel::STORE,
        "a half written event was set aside so the rest of the history could be read",
        &[
            ("at", Fact::Path(path.to_path_buf())),
            ("kept", Fact::Path(aside)),
            ("bytes", Fact::Count(tail.len())),
        ],
    );
}

fn tail_of(path: &Path) -> Result<(usize, jiff::Timestamp, u64)> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok((0, jiff::Timestamp::UNIX_EPOCH, 0));
        }
        Err(e) => return Err(Error::Io(e)),
    };

    let mut lines = 0usize;
    let mut head = jiff::Timestamp::UNIX_EPOCH;
    let mut seq = 0u64;

    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        lines += 1;
        match serde_json::from_str::<Event>(&line) {
            Ok(event) if (event.timestamp, event.seq) > (head, seq) => {
                head = event.timestamp;
                seq = event.seq;
            }
            Ok(_) => {}
            Err(_) => witness::warn(
                channel::STORE,
                "segment line unreadable",
                &[
                    ("at", Fact::Path(path.to_path_buf())),
                    ("line", Fact::Count(lines)),
                ],
            ),
        }
    }
    Ok((lines, head, seq))
}

fn next_segment_number(dir: &Path) -> Result<u32> {
    let highest = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    Ok(highest + 1)
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
