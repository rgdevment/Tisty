use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::{
    Error, Result,
    event::{DeviceId, Event, KNOWN_OPS, Op, SCHEMA_VERSION},
    witness::{self, Fact, channel},
};

const ACTIVE: &str = "active.tisty";
const DISPLACED: &str = ".store-key.was-";
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

pub const MARKER: &str = ".store-id";
/// The identity says which store a parcel came from, and travels inside every one of them.
/// This says it is really that store: it never leaves the machine, and without it nobody
/// can write a parcel that lands here as though it had been born here.
pub const KEEP: &str = ".store-key";

pub fn identity(store_root: impl AsRef<Path>) -> Result<String> {
    if let Some(held) = peek_identity(&store_root) {
        return Ok(held);
    }
    let at = store_root.as_ref().join(MARKER);
    let fresh = ulid::Ulid::generate().to_string();
    std::fs::create_dir_all(store_root.as_ref())?;

    match File::create_new(&at) {
        Ok(mut file) => {
            file.write_all(fresh.as_bytes())?;
            file.sync_all()?;
            let _ = crate::paths::ours_alone(&at);
            Ok(fresh)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if let Some(held) = peek_identity(&store_root) {
                return Ok(held);
            }
            write_atomic(&at, fresh.as_bytes())?;
            Ok(peek_identity(&store_root).unwrap_or(fresh))
        }
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn kept_at(paths: &crate::Paths, named: &str) -> PathBuf {
    paths.private().join(format!("{named}{KEEP}"))
}

pub fn secret_kept(paths: &crate::Paths) -> Option<[u8; 32]> {
    let named = peek_identity(paths.store())?;
    let held = std::fs::read(kept_at(paths, &named)).ok()?;
    <[u8; 32]>::try_from(held.as_slice()).ok()
}

pub fn secret(paths: &crate::Paths) -> Option<[u8; 32]> {
    let named = identity(paths.store()).ok()?;
    let at = kept_at(paths, &named);
    match std::fs::read(&at) {
        Ok(held) => match <[u8; 32]>::try_from(held.as_slice()) {
            Ok(kept) => return Some(kept),
            Err(_) => {
                if !set_aside(paths, &at, &held, "what was kept as the key is not one")
                    || std::fs::remove_file(&at).is_err()
                {
                    return None;
                }
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            witness::error(
                channel::STORE,
                "the key could not be read, so this store cannot prove its own writing",
                &[
                    ("at", Fact::Path(at.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return None;
        }
    }

    if let Ok(inside) = std::fs::read(paths.store().join(KEEP))
        && let Ok(kept) = <[u8; 32]>::try_from(inside.as_slice())
    {
        return Some(kept);
    }

    let mut fresh = [0u8; 32];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut fresh).ok()?;
    std::fs::create_dir_all(paths.private()).ok()?;
    let _ = crate::paths::ours_alone(&paths.private());
    match File::create_new(&at) {
        Ok(mut file) => {
            file.write_all(&fresh).ok()?;
            file.sync_all().ok()?;
            let _ = crate::paths::ours_alone(&at);
            Some(fresh)
        }
        Err(_) => std::fs::read(&at)
            .ok()
            .and_then(|held| <[u8; 32]>::try_from(held.as_slice()).ok()),
    }
}

pub fn brought_home(paths: &crate::Paths) {
    let was = paths.store().join(KEEP);
    let Ok(held) = std::fs::read(&was) else {
        return;
    };
    if <[u8; 32]>::try_from(held.as_slice()).is_err() {
        witness::trace(
            channel::STORE,
            "what was kept where the key used to live is not a key, so it was left alone",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    }
    if !paths.of_one_install() {
        witness::trace(
            channel::STORE,
            "this store was named on its own, so its key was left where it is",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    }
    let Ok(named) = identity(paths.store()) else {
        witness::warn(
            channel::STORE,
            "a key sits in a store that cannot be named, so it was left where it is",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    };

    let now = kept_at(paths, &named);
    match std::fs::read(&now) {
        Ok(there) if <[u8; 32]>::try_from(there.as_slice()).is_err() => {
            if !set_aside(paths, &now, &there, "what was kept as the key is not one") {
                return;
            }
        }
        Ok(there) if there != held => {
            if set_aside(
                paths,
                &now,
                &held,
                "an older build made a second key inside the store",
            ) {
                swept(&was);
            }
            return;
        }
        Ok(_) => {
            swept(&was);
            return;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            witness::error(
                channel::STORE,
                "the key already kept apart could not be read, so nothing was moved over it",
                &[
                    ("at", Fact::Path(now.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return;
        }
    }

    let _ = std::fs::create_dir_all(paths.private());
    let _ = crate::paths::ours_alone(&paths.private());
    if write_atomic(&now, &held).is_err() {
        witness::warn(
            channel::STORE,
            "the key could not be moved out of the store, so it stays where a backup reaches it",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    }
    let _ = crate::paths::ours_alone(&now);
    swept(&was);
}

pub fn displaced(paths: &crate::Paths) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(paths.private()) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_name().to_string_lossy().contains(DISPLACED))
        .map(|one| one.path())
        .collect();
    found.sort();
    found
}

fn set_aside(paths: &crate::Paths, at: &Path, held: &[u8], why: &str) -> bool {
    if displaced(paths)
        .iter()
        .any(|one| std::fs::read(one).is_ok_and(|kept| kept == held))
    {
        return true;
    }
    let stamp = jiff::Zoned::now().strftime("%Y%m%dT%H%M%S").to_string();
    let named = at.file_name().unwrap_or_default().to_string_lossy();
    let mut aside = paths.private().join(format!("{named}.was-{stamp}"));
    for again in 1..100 {
        match File::create_new(&aside) {
            Ok(mut file) => {
                if file.write_all(held).and_then(|()| file.sync_all()).is_err() {
                    let _ = std::fs::remove_file(&aside);
                    break;
                }
                let _ = crate::paths::ours_alone(&aside);
                witness::warn(
                    channel::STORE,
                    "a key was set aside",
                    &[
                        ("at", Fact::Path(aside)),
                        ("why", Fact::Why(why.to_string())),
                    ],
                );
                return true;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                aside = paths.private().join(format!("{named}.was-{stamp}-{again}"));
            }
            Err(_) => break,
        }
    }
    witness::error(
        channel::STORE,
        "a key had to be set aside and could not be, so nothing was changed",
        &[("why", Fact::Why(why.to_string()))],
    );
    false
}

fn swept(at: &Path) {
    if let Err(e) = std::fs::remove_file(at)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        witness::error(
            channel::STORE,
            "the key is still inside the store, where a backup reaches it",
            &[
                ("at", Fact::Path(at.to_path_buf())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
}

pub fn peek_identity(store_root: impl AsRef<Path>) -> Option<String> {
    let held = std::fs::read_to_string(store_root.as_ref().join(MARKER)).ok()?;
    let held = held.trim().to_string();
    (!held.is_empty()).then_some(held)
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
mod atomic_tests {
    use super::*;

    #[test]
    fn an_event_from_a_newer_tisty_says_so_instead_of_looking_broken() {
        let room = tempfile::tempdir().unwrap();
        let at = room.path().join("000001.tisty");
        std::fs::write(
            &at,
            format!(
                "{{\"v\":{},\"ts\":\"2026-08-13T00:00:00Z\",\"by\":\"dev_a\",\"op\":\"folder.colour\",\"id\":\"01J\",\"d\":{{}}}}\n",
                SCHEMA_VERSION + 1
            ),
        )
        .unwrap();

        let mut out = Vec::new();
        let why = read_segment(&at, &mut out).unwrap_err();

        assert!(
            matches!(why, Error::UnsupportedVersion(_)),
            "it read as corruption: {why:?}"
        );
    }

    #[test]
    fn a_store_written_before_the_schema_moved_still_opens() {
        let room = tempfile::tempdir().unwrap();
        std::fs::create_dir(room.path().join("dev_a")).unwrap();
        let at = room.path().join("dev_a/000001.tisty");
        std::fs::write(
            &at,
            "{\"v\":2,\"ts\":\"2026-08-13T00:00:00Z\",\"by\":\"dev_a\",\"seq\":1,\"op\":\"task.done\",\"id\":\"01JBQ0000000000000000000AA\"}\n",
        )
        .unwrap();

        let mut out = Vec::new();
        read_segment(&at, &mut out).expect("an older store is not a broken store");

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].version, 2, "what was written is not rewritten");
    }

    /// A machine whose clock runs behind writes after what it read: sorted by stamp, its
    /// event lands after the one that let it through, not before.
    #[test]
    fn a_write_judged_against_the_log_is_stamped_after_all_of_it() {
        let room = tempfile::tempdir().unwrap();
        let ahead = jiff::Timestamp::now() + jiff::SignedDuration::from_secs(3600);
        let id = ulid::Ulid::generate();
        let mut theirs = Store::open(room.path(), DeviceId("dev_b".into())).unwrap();
        theirs
            .append_event(&Event::new(
                DeviceId("dev_b".into()),
                ahead,
                Op::TaskAdd {
                    id,
                    d: crate::event::TaskAdd::new("opened later", "a0"),
                },
            ))
            .unwrap();

        let mut mine = Store::open(room.path(), DeviceId("dev_a".into())).unwrap();
        let written = mine
            .append_batch_unless(vec![Op::TaskDone { id, filled: false }], |_| false)
            .unwrap()
            .unwrap();

        assert!(
            written[0].timestamp > ahead,
            "{} is not after {ahead}",
            written[0].timestamp
        );
        let all = read_all(room.path()).unwrap();
        assert!(matches!(all.last().unwrap().op, Op::TaskDone { .. }));
    }

    #[test]
    fn an_event_in_another_devices_name_is_let_go_and_the_rest_of_the_segment_read() {
        let room = tempfile::tempdir().unwrap();
        std::fs::create_dir(room.path().join("dev_a")).unwrap();
        let at = room.path().join("dev_a/active.tisty");
        std::fs::write(
            &at,
            format!(
                "{}\n{}\n",
                r#"{"v":6,"ts":"2026-08-01T10:00:00Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{"title":"mine","order":"V"}}"#,
                r#"{"v":6,"ts":"2026-08-01T10:00:01Z","by":"dev_laptop","op":"task.delete","id":"01M14RFT9ECC2B6E4CX4P59XPH"}"#
            ),
        )
        .unwrap();

        let mut out = Vec::new();
        read_segment(&at, &mut out).unwrap();

        assert_eq!(out.len(), 1, "the one in the directory's own name");
        assert_eq!(out[0].device.0, "dev_a");
    }

    #[test]
    fn a_rename_that_fails_takes_its_temporary_with_it() {
        let room = tempfile::tempdir().unwrap();
        let blocked = room.path().join("busy.md");
        std::fs::create_dir(&blocked).unwrap();

        assert!(write_atomic(&blocked, b"x").is_err());

        let left: Vec<_> = std::fs::read_dir(room.path())
            .unwrap()
            .filter_map(|one| one.ok())
            .filter(|one| one.path().extension().is_some_and(|e| e == "tmp"))
            .collect();
        assert!(left.is_empty(), "a temporary was left behind: {left:?}");
    }

    #[cfg(windows)]
    #[test]
    fn a_file_held_open_for_a_moment_is_waited_out_rather_than_refused() {
        use std::os::windows::fs::OpenOptionsExt;

        let room = tempfile::tempdir().unwrap();
        let at = room.path().join("a3f1-0001.md");
        std::fs::write(&at, b"before").unwrap();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&at)
            .unwrap();

        std::thread::scope(|threads| {
            threads.spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(40));
                drop(held);
            });
            write_atomic(&at, b"after").expect("a file let go of is a file that can be written");
        });

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "after");
    }

    #[test]
    fn two_writers_of_one_file_do_not_share_a_temporary() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("a3f1-0001.md");
        std::fs::write(&at, b"before").unwrap();

        std::thread::scope(|threads| {
            let mut hands = Vec::new();
            for n in 0..8 {
                let at = at.clone();
                hands.push(
                    threads.spawn(move || write_atomic(&at, format!("written by {n}").as_bytes())),
                );
            }
            for hand in hands {
                hand.join()
                    .unwrap()
                    .expect("a concurrent save must not fail");
            }
        });

        let kept = std::fs::read_to_string(&at).unwrap();
        assert!(kept.starts_with("written by"), "{kept}");
    }

    #[test]
    fn a_missing_parent_directory_leaves_nothing_temporary_either() {
        let room = tempfile::tempdir().unwrap();
        let at = room.path().join("nope").join("file.md");

        assert!(write_atomic(&at, b"x").is_err());

        let left: Vec<_> = std::fs::read_dir(room.path())
            .unwrap()
            .filter_map(|one| one.ok())
            .collect();
        assert!(
            left.is_empty(),
            "something was created in a directory that was never made: {left:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_read_only_directory_refuses_the_temporary_and_leaves_nothing_behind() {
        use std::os::unix::fs::PermissionsExt;
        let room = tempfile::tempdir().unwrap();
        let locked = room.path().join("locked");
        std::fs::create_dir(&locked).unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

        let result = write_atomic(&locked.join("file.md"), b"x");

        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_err());
        let left: Vec<_> = std::fs::read_dir(&locked)
            .unwrap()
            .filter_map(|one| one.ok())
            .collect();
        assert!(
            left.is_empty(),
            "a temporary survived in a directory with no write permission: {left:?}"
        );
    }

    #[test]
    fn poured_does_not_partially_write_when_the_temporary_path_is_a_directory() {
        let room = tempfile::tempdir().unwrap();
        let dir_as_tmp = room.path().join("oops");
        std::fs::create_dir(&dir_as_tmp).unwrap();

        assert!(poured(&dir_as_tmp, b"x").is_err());
    }

    #[test]
    fn nothing_temporary_is_left_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("a3f1-0001.md");

        write_atomic(&at, b"one").unwrap();
        write_atomic(&at, b"two").unwrap();

        let left: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|one| one.ok())
            .map(|one| one.file_name().to_string_lossy().into_owned())
            .filter(|named| named.contains("tmp"))
            .collect();
        assert!(left.is_empty(), "{left:?}");
    }
}

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
mod tests {

    #[test]
    fn the_segments_of_a_machine_always_come_back_in_the_order_they_were_written() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("dev_a");
        std::fs::create_dir_all(&at).unwrap();
        for leaf in [
            "active.tisty",
            "000002.tisty",
            "000010.tisty",
            "000001.tisty",
        ] {
            std::fs::write(at.join(leaf), b"x").unwrap();
        }

        let found: Vec<String> = segments_in(&at)
            .unwrap()
            .iter()
            .filter_map(|one| one.file_name()?.to_str().map(str::to_string))
            .collect();

        assert_eq!(
            found,
            vec![
                "000001.tisty",
                "000002.tisty",
                "000010.tisty",
                "active.tisty"
            ]
        );
    }
    use super::*;
    use crate::event::TaskAdd;
    use ulid::Ulid;

    fn add(title: &str) -> Op {
        Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(title, "a0"),
        }
    }

    fn seated(at: &Path, who: &str, ops: Vec<Op>) {
        let mut store = Store::open(at, DeviceId(who.into())).unwrap();
        for op in ops {
            store.append(op).unwrap();
        }
    }

    fn poured(from: &Path, into: &Path) {
        for one in std::fs::read_dir(from).unwrap().filter_map(|e| e.ok()) {
            if !one.file_type().unwrap().is_dir() {
                continue;
            }
            let there = into.join(one.file_name());
            std::fs::create_dir_all(&there).unwrap();
            for file in std::fs::read_dir(one.path())
                .unwrap()
                .filter_map(|e| e.ok())
            {
                std::fs::copy(file.path(), there.join(file.file_name())).unwrap();
            }
        }
    }

    #[test]
    fn concatenating_two_histories_locks_nobody_who_was_writing_out() {
        let here = tempfile::tempdir().unwrap();
        let there = tempfile::tempdir().unwrap();
        seated(
            here.path(),
            "dev_here",
            vec![
                Op::DeviceJoin {
                    d: DeviceId("dev_here".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
                add("lo de aqui"),
            ],
        );
        seated(
            there.path(),
            "dev_there",
            vec![
                Op::DeviceJoin {
                    d: DeviceId("dev_there".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
                add("lo de alli"),
            ],
        );

        poured(there.path(), here.path());
        let said = ledger(here.path()).unwrap();

        assert!(said.may_write(&DeviceId("dev_here".into())));
        assert!(said.may_write(&DeviceId("dev_there".into())));
        assert!(!said.was_removed(&DeviceId("dev_here".into())));
        assert!(!said.was_removed(&DeviceId("dev_there".into())));
    }

    #[test]
    fn a_history_that_never_named_anyone_is_not_shut_out_by_one_that_did() {
        let here = tempfile::tempdir().unwrap();
        let there = tempfile::tempdir().unwrap();
        seated(here.path(), "dev_here", vec![add("lo de aqui, sin alta")]);
        seated(
            there.path(),
            "dev_there",
            vec![
                Op::DeviceJoin {
                    d: DeviceId("dev_there".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
                Op::DeviceRemove {
                    d: DeviceId("dev_gone".into()),
                },
            ],
        );

        poured(there.path(), here.path());
        let said = ledger(here.path()).unwrap();

        assert!(
            said.may_write(&DeviceId("dev_here".into())),
            "la maquina que nunca se dio de alta quedo fuera al fusionar"
        );
        assert!(said.may_write(&DeviceId("dev_there".into())));
        assert!(said.was_removed(&DeviceId("dev_gone".into())));
    }

    #[test]
    fn once_a_machine_is_removed_no_ordering_of_the_log_lets_it_back_in() {
        let when: jiff::Timestamp = "2026-08-15T00:00:00Z".parse().unwrap();
        let stamped = |who: &str, seq: u64, op: Op| Event {
            version: SCHEMA_VERSION,
            timestamp: when,
            device: DeviceId(who.into()),
            batch: None,
            undo: false,
            redo: false,
            seq,
            op,
            optional: false,
            zone: None,
            via: None,
        };

        let told = |remover: &str, joiner: &str| {
            let root = tempfile::tempdir().unwrap();
            for (who, seq, op) in [
                (
                    "dev_m",
                    1,
                    Op::DeviceJoin {
                        d: DeviceId("dev_m".into()),
                        k: Some(crate::event::DeviceKind::Machine),
                    },
                ),
                (
                    remover,
                    2,
                    Op::DeviceRemove {
                        d: DeviceId("dev_both".into()),
                    },
                ),
                (
                    joiner,
                    3,
                    Op::DeviceJoin {
                        d: DeviceId("dev_both".into()),
                        k: Some(crate::event::DeviceKind::Machine),
                    },
                ),
            ] {
                let mut store = Store::open(root.path(), DeviceId(who.into())).unwrap();
                store.append_event(&stamped(who, seq, op)).unwrap();
            }
            ledger(root.path())
                .unwrap()
                .may_write(&DeviceId("dev_both".into()))
        };

        assert!(!told("dev_a", "dev_z"), "el alta posterior la resucito");
        assert!(
            !told("dev_z", "dev_a"),
            "el desempate por nombre la resucito"
        );
    }

    #[test]
    fn a_removal_survives_a_clock_that_runs_behind_the_machine_it_removes() {
        let root = tempfile::tempdir().unwrap();
        let earlier: jiff::Timestamp = "2026-08-15T00:00:00Z".parse().unwrap();
        let later: jiff::Timestamp = "2026-08-15T00:00:05Z".parse().unwrap();
        let stamped = |who: &str, when: jiff::Timestamp, seq: u64, op: Op| Event {
            version: SCHEMA_VERSION,
            timestamp: when,
            device: DeviceId(who.into()),
            batch: None,
            undo: false,
            redo: false,
            seq,
            op,
            optional: false,
            zone: None,
            via: None,
        };

        for (who, when, seq, op) in [
            (
                "dev_keeper",
                earlier,
                1,
                Op::DeviceJoin {
                    d: DeviceId("dev_keeper".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
            ),
            (
                "dev_gone",
                later,
                1,
                Op::DeviceJoin {
                    d: DeviceId("dev_gone".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
            ),
            (
                "dev_keeper",
                earlier,
                2,
                Op::DeviceRemove {
                    d: DeviceId("dev_gone".into()),
                },
            ),
        ] {
            let mut store = Store::open(root.path(), DeviceId(who.into())).unwrap();
            store.append_event(&stamped(who, when, seq, op)).unwrap();
        }

        assert!(
            ledger(root.path())
                .unwrap()
                .was_removed(&DeviceId("dev_gone".into())),
            "la baja se deshizo sola porque el reloj de quien la dio iba atrasado"
        );
    }

    #[test]
    fn a_segment_that_arrived_half_written_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());
        let mut store = Store::open(tmp.path(), device.clone()).unwrap();
        for i in 0..4 {
            store.append(add(&format!("task {i}"))).unwrap();
        }
        store.active_events = SEGMENT_MAX_EVENTS;
        store.append(add("one more")).unwrap();

        let dir = tmp.path().join(&device.0);
        let sealed = dir.join("000001.tisty");
        assert_eq!(declared_count(&sealed), Some(4));

        let kept: String = std::fs::read_to_string(&sealed)
            .unwrap()
            .lines()
            .take(2)
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&sealed, kept + "\n").unwrap();

        assert!(matches!(
            read_all(tmp.path()),
            Err(Error::TruncatedSegment {
                found: 2,
                declared: Some(4),
                ..
            })
        ));
    }

    #[test]
    fn an_emptied_segment_is_an_error_not_an_empty_history() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());
        let mut store = Store::open(tmp.path(), device.clone()).unwrap();
        store.append(add("kept")).unwrap();

        let dir = tmp.path().join(&device.0);
        std::fs::rename(dir.join(ACTIVE), dir.join("000001.tisty")).unwrap();
        std::fs::write(dir.join("000001.tisty"), "").unwrap();

        assert!(matches!(
            read_all(tmp.path()),
            Err(Error::TruncatedSegment { .. })
        ));
    }

    #[test]
    fn every_stamp_is_greater_than_the_one_before_it() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        for i in 0..50 {
            store.append(add(&format!("task {i}"))).unwrap();
        }

        let events = read_all(tmp.path()).unwrap();
        let keys: Vec<_> = events
            .iter()
            .map(|e| (e.timestamp, e.seq))
            .collect::<Vec<_>>();
        for pair in keys.windows(2) {
            assert!(
                pair[0] < pair[1],
                "{:?} does not precede {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn a_batch_written_in_one_instant_still_orders() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        let written = store
            .append_batch(vec![add("first"), add("second"), add("third")])
            .unwrap();

        for pair in written.windows(2) {
            assert!(pair[0].sort_key() < pair[1].sort_key());
        }
    }

    #[test]
    fn reopening_does_not_rewind_the_clock() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());

        let mut first = Store::open(tmp.path(), device.clone()).unwrap();
        let before = first.append(add("before")).unwrap();
        drop(first);

        let mut second = Store::open(tmp.path(), device).unwrap();
        let after = second.append(add("after")).unwrap();

        assert!(after.sort_key() > before.sort_key());
    }

    fn at(ms: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_millisecond(ms).unwrap()
    }

    #[test]
    fn appends_and_reads_back() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        store.append(add("first")).unwrap();
        store.append(add("second")).unwrap();

        assert_eq!(store.read_all().unwrap().len(), 2);
    }

    #[test]
    fn survives_reopening() {
        let tmp = tempfile::tempdir().unwrap();
        {
            let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
            store.append(add("persisted")).unwrap();
        }
        let store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        assert_eq!(store.read_all().unwrap().len(), 1);
    }

    #[test]
    fn merges_devices_in_canonical_order() {
        let tmp = tempfile::tempdir().unwrap();
        let mut a = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        let mut b = Store::open(tmp.path(), DeviceId("dev_b".into())).unwrap();

        let same_instant = at(1_000);
        b.append_event(&Event::new(b.device().clone(), same_instant, add("from b")))
            .unwrap();
        a.append_event(&Event::new(a.device().clone(), same_instant, add("from a")))
            .unwrap();

        let from_a = a.read_all().unwrap();
        let from_b = b.read_all().unwrap();

        assert_eq!(from_a, from_b, "both devices see the same order");
        assert_eq!(from_a[0].device, DeviceId("dev_a".into()));
    }

    #[test]
    fn a_writer_that_holds_the_lock_turns_the_other_away() {
        let tmp = tempfile::tempdir().unwrap();
        let mut holding = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        holding.acquire().unwrap();

        let mut other = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        assert!(matches!(
            other.append(add("second")),
            Err(Error::AlreadyRunning)
        ));
    }

    #[test]
    fn reading_stays_possible_while_another_process_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let mut writer = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        writer.append(add("written")).unwrap();

        let reader = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        assert_eq!(reader.read_all().unwrap().len(), 1);
    }

    #[test]
    fn a_different_device_can_write_at_the_same_time() {
        let tmp = tempfile::tempdir().unwrap();
        let mut a = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        let mut b = Store::open(tmp.path(), DeviceId("dev_b".into())).unwrap();

        a.append(add("from a")).unwrap();
        assert!(b.append(add("from b")).is_ok());
    }

    #[test]
    fn rotation_closes_segments_and_keeps_every_event() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();

        store.append(add("before rotation")).unwrap();
        store.active_events = SEGMENT_MAX_EVENTS;
        store.append(add("after rotation")).unwrap();

        let dir = tmp.path().join("dev_a");
        assert!(dir.join("000001.tisty").exists(), "segment was closed");
        assert!(dir.join(ACTIVE).exists(), "a fresh active file took over");
        assert_eq!(store.read_all().unwrap().len(), 2);
    }

    #[test]
    fn rotation_tolerates_a_missing_active_file() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();

        store.active_events = SEGMENT_MAX_EVENTS;
        store.append(add("after a lost file")).unwrap();

        assert_eq!(store.read_all().unwrap().len(), 1);
    }

    #[test]
    fn segments_are_read_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();

        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        for i in 0..3 {
            store
                .append_event(&Event::new(
                    store.device().clone(),
                    at(i + 1),
                    add(&format!("event {i}")),
                ))
                .unwrap();
            store.active_events = SEGMENT_MAX_EVENTS;
        }

        let events = store.read_all().unwrap();
        assert_eq!(events.len(), 3);
        assert!(events.windows(2).all(|w| w[0].sort_key() < w[1].sort_key()));
    }

    #[test]
    fn a_malformed_line_reports_where_it_is() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(ACTIVE), "{\"v\":1}\n{\"not\":\"an event\"}\n").unwrap();

        match read_all(tmp.path()) {
            Err(Error::MalformedEvent { line, .. }) => assert_eq!(line, 1),
            other => panic!("expected MalformedEvent, got {other:?}"),
        }
    }

    #[test]
    fn a_future_schema_version_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();

        let mut future = serde_json::to_value(Event::new(
            DeviceId("dev_a".into()),
            at(1),
            add("from the future"),
        ))
        .unwrap();
        future["v"] = serde_json::json!(SCHEMA_VERSION + 1);
        std::fs::write(dir.join(ACTIVE), format!("{future}\n")).unwrap();

        assert!(matches!(
            read_all(tmp.path()),
            Err(Error::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn an_operation_this_build_does_not_know_is_skipped_when_it_says_it_may_be() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        let known = format!(
            r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T10:00:00Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"title":"still here","order":"V"}}}}"#
        );
        let stranger = format!(
            r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.bless","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"halo":true}}}}"#
        );
        std::fs::write(
            dir.join(ACTIVE),
            format!(
                "{known}
{stranger}
"
            ),
        )
        .unwrap();

        let read = read_all(tmp.path()).unwrap();
        assert_eq!(
            read.len(),
            1,
            "the known event has to survive its unknown neighbour"
        );
    }

    #[test]
    fn an_operation_this_build_does_not_know_stops_the_read_unless_it_says_otherwise() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        let stranger = format!(
            r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","op":"task.erase.everything","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{}}}}"#
        );
        std::fs::write(
            dir.join(ACTIVE),
            format!(
                "{stranger}
"
            ),
        )
        .unwrap();

        assert!(
            matches!(read_all(tmp.path()), Err(Error::MalformedEvent { .. })),
            "an operation that may change what exists cannot be waved through"
        );
    }

    #[test]
    fn a_version_number_below_this_one_never_blocks_a_read() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();

        let mut lines = String::new();
        for v in 1..=SCHEMA_VERSION {
            lines.push_str(&format!(
                r#"{{"v":{v},"ts":"2026-08-28T10:00:{v:02}Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59X{v:02}","d":{{"title":"written by v{v}","order":"V"}}}}"#
            ));
            lines.push('\n');
        }
        std::fs::write(dir.join(ACTIVE), lines).unwrap();

        let read = read_all(tmp.path()).unwrap();
        assert_eq!(read.len(), SCHEMA_VERSION as usize);
        assert!(
            read.iter().all(|e| matches!(&e.op, Op::TaskAdd { .. })),
            "the guard only refuses upwards; what the older shapes look like is another test"
        );
    }

    #[test]
    fn an_event_carries_the_zone_of_whoever_wrote_it() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        let event = store
            .append(Op::TaskAdd {
                id: ulid::Ulid::generate(),
                d: crate::event::TaskAdd::new("where was I", "a0"),
            })
            .unwrap();

        assert!(
            event.zone.is_some(),
            "a written event knows where it was written"
        );
        assert_eq!(
            event.zoned().map(|z| z.timestamp()),
            Some(event.timestamp),
            "reading it back in its own zone is the same instant"
        );
    }

    #[test]
    fn a_skipped_event_still_counts_towards_a_sealed_segment() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        let known = format!(
            r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T10:00:00Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"title":"sealed away","order":"V"}}}}"#
        );
        let stranger = format!(
            r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.bless","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{}}}}"#
        );
        std::fs::write(
            dir.join("000001.tisty"),
            format!(
                "{known}
{stranger}
"
            ),
        )
        .unwrap();
        std::fs::write(dir.join("000001.count"), "2").unwrap();

        let read = read_all(tmp.path()).unwrap();
        assert_eq!(
            read.len(),
            1,
            "a sealed segment declares lines, so skipping one cannot read as a truncated download"
        );
    }

    #[test]
    fn an_event_from_a_newer_schema_is_skipped_when_it_says_it_may_be() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        let ahead = format!(
            r#"{{"v":{},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.bless","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{}}}}"#,
            SCHEMA_VERSION + 1
        );
        std::fs::write(
            dir.join(ACTIVE),
            format!(
                "{ahead}
"
            ),
        )
        .unwrap();
        assert!(
            read_all(tmp.path()).unwrap().is_empty(),
            "the operations this guard exists for will arrive under a newer schema, not this one"
        );

        let sealed = ahead.replace(r#""opt":true,"#, "");
        std::fs::write(
            dir.join(ACTIVE),
            format!(
                "{sealed}
"
            ),
        )
        .unwrap();
        assert!(matches!(
            read_all(tmp.path()),
            Err(Error::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn a_known_operation_that_arrived_corrupt_is_never_waved_through() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        let rotten = format!(
            r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"title":12345,"order":"V"}}}}"#
        );
        std::fs::write(
            dir.join(ACTIVE),
            format!(
                "{rotten}
"
            ),
        )
        .unwrap();

        assert!(
            matches!(read_all(tmp.path()), Err(Error::MalformedEvent { .. })),
            "the mark forgives an operation nobody knows, never one that arrived broken"
        );
    }

    #[test]
    fn the_shape_a_previous_build_wrote_still_projects() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        // Written by hand as a build before v7 would have: no tz, no opt, no k, no filled,
        // no source. A `default` dropped by accident has to fail here, not in someone's store.
        let before = concat!(
            r#"{"v":6,"ts":"2026-08-01T10:00:00Z","by":"dev_a","op":"device.join","d":"dev_a"}"#,
            "
",
            r#"{"v":6,"ts":"2026-08-01T10:00:01Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{"title":"written before v7","order":"V"}}"#,
            "
",
            r#"{"v":6,"ts":"2026-08-01T10:00:02Z","by":"dev_a","op":"task.done","id":"01M14RFT9ECC2B6E4CX4P59XPH"}"#,
            "
",
        );
        std::fs::write(dir.join(ACTIVE), before).unwrap();

        let read = read_all(tmp.path()).unwrap();
        assert_eq!(read.len(), 3);
        assert!(read.iter().all(|e| e.zone.is_none() && !e.optional));

        let state = crate::State::replay(&read);
        let task = &state.tasks[&"01M14RFT9ECC2B6E4CX4P59XPH".parse().unwrap()];
        assert_eq!(task.status, crate::model::Status::Done);
        assert!(
            !task.filled,
            "a closure from before the field is not a backfill"
        );
        assert_eq!(task.source, None);
        assert!(
            state.devices.contains(&DeviceId("dev_a".into())) && state.agents.is_empty(),
            "a join with nothing to say about its kind is not a claim that it is a machine"
        );
    }

    #[test]
    fn a_known_operation_from_a_newer_schema_is_never_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("dev_a");
        std::fs::create_dir_all(&dir).unwrap();
        let ahead = format!(
            r#"{{"v":{},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.delete","id":"01M14RFT9ECC2B6E4CX4P59XPH"}}"#,
            SCHEMA_VERSION + 1
        );
        std::fs::write(
            dir.join(ACTIVE),
            format!(
                "{ahead}
"
            ),
        )
        .unwrap();

        assert!(
            matches!(read_all(tmp.path()), Err(Error::UnsupportedVersion(_))),
            "a deletion this build understands cannot be dropped for wearing a newer number:              the task would stay alive here and be gone everywhere else"
        );
    }

    #[test]
    fn an_empty_store_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_all(tmp.path().join("missing")).unwrap().is_empty());
    }

    #[test]
    fn a_key_kept_inside_the_store_moves_out_and_leaves_nothing_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        let store = paths.store();
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join(KEEP), [7u8; 32]).unwrap();

        brought_home(&paths);

        assert!(
            !store.join(KEEP).exists(),
            "the key stayed where a backup reaches it"
        );
        let named = peek_identity(&store).unwrap();
        assert_eq!(std::fs::read(kept_at(&paths, &named)).unwrap(), [7u8; 32]);
        assert_eq!(secret(&paths).unwrap(), [7u8; 32]);
    }

    #[test]
    fn bringing_the_key_home_when_there_never_was_one_inside_makes_none() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        let store = paths.store();
        let private = paths.private();
        std::fs::create_dir_all(&store).unwrap();

        brought_home(&paths);

        assert!(
            !private.join(KEEP).exists(),
            "a key was made out of nothing"
        );
    }

    #[test]
    fn bringing_the_key_home_twice_changes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        let store = paths.store();
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join(KEEP), [3u8; 32]).unwrap();

        brought_home(&paths);
        brought_home(&paths);

        let named = peek_identity(&store).unwrap();
        assert_eq!(std::fs::read(kept_at(&paths, &named)).unwrap(), [3u8; 32]);
        assert_eq!(displaced(&paths).len(), 0, "the second pass set one aside");
    }

    #[test]
    fn a_key_that_turns_up_inside_an_already_moved_store_does_not_win() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        let store = paths.store();
        std::fs::create_dir_all(paths.private()).unwrap();
        let named = identity(&store).unwrap();
        std::fs::write(kept_at(&paths, &named), [2u8; 32]).unwrap();
        std::fs::write(store.join(KEEP), [1u8; 32]).unwrap();

        brought_home(&paths);

        assert_eq!(
            std::fs::read(kept_at(&paths, &named)).unwrap(),
            [2u8; 32],
            "the newcomer took over from what this store seals with"
        );
        let kept: Vec<Vec<u8>> = displaced(&paths)
            .iter()
            .map(|one| std::fs::read(one).unwrap())
            .collect();
        assert_eq!(
            kept,
            vec![vec![1u8; 32]],
            "the one it displaced was destroyed instead of set aside"
        );
        assert!(!store.join(KEEP).exists());
    }

    #[test]
    fn something_that_is_not_a_key_is_left_where_it_is() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        let store = paths.store();
        let private = paths.private();
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join(KEEP), b"not a key").unwrap();

        brought_home(&paths);

        assert!(
            store.join(KEEP).exists(),
            "it was taken for a key and moved"
        );
        assert!(!private.join(KEEP).exists());
    }

    #[test]
    fn write_atomic_leaves_no_temporary_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("config.toml");

        write_atomic(&target, b"first").unwrap();
        write_atomic(&target, b"second").unwrap();

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "second");
        assert!(!target.with_extension("tmp").exists());
    }

    #[test]
    fn a_write_does_not_keep_the_lock() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());

        let mut gui = Store::open(tmp.path(), device.clone()).unwrap();
        gui.append(add("from the window")).unwrap();

        let mut cli = Store::open(tmp.path(), device).unwrap();
        assert!(cli.append(add("from the terminal")).is_ok());
    }

    #[test]
    fn two_processes_on_one_device_never_stamp_the_same_event() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());

        let mut gui = Store::open(tmp.path(), device.clone()).unwrap();
        let mut cli = Store::open(tmp.path(), device).unwrap();

        let mut stamps = Vec::new();
        for i in 0..8 {
            let who = if i % 2 == 0 { &mut gui } else { &mut cli };
            let event = who.append(add(&format!("task {i}"))).unwrap();
            stamps.push((event.timestamp, event.seq));
        }

        let mut unique = stamps.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), stamps.len(), "collided: {stamps:?}");
    }

    #[test]
    fn catching_up_never_rewinds_the_clock() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());

        let mut first = Store::open(tmp.path(), device.clone()).unwrap();
        first.append(add("one")).unwrap();

        let mut second = Store::open(tmp.path(), device).unwrap();
        second.append(add("two")).unwrap();

        let ahead = second.head;
        first.append(add("three")).unwrap();
        assert!(first.head >= ahead);
    }

    #[test]
    fn rotation_resets_what_the_store_believes_it_has_seen() {
        let tmp = tempfile::tempdir().unwrap();
        let device = DeviceId("dev_a".into());
        let mut store = Store::open(tmp.path(), device).unwrap();

        store.append(add("before")).unwrap();
        store.active_events = SEGMENT_MAX_EVENTS;
        store.append(add("after")).unwrap();

        assert_eq!(store.active_events, 1);
        assert_eq!(store.seen, active_size(&store.dir.join(ACTIVE)));
    }
    #[test]
    fn a_store_keeps_the_same_name_however_often_it_is_asked() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("store");

        let first = identity(&root).unwrap();
        assert!(!first.is_empty());
        assert_eq!(identity(&root).unwrap(), first);

        let other = tempfile::tempdir().unwrap();
        assert_ne!(identity(other.path()).unwrap(), first);
    }

    #[test]
    fn an_empty_marker_is_replaced_rather_than_trusted() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path()).unwrap();
        std::fs::write(
            dir.path().join(MARKER),
            "   
",
        )
        .unwrap();

        let named = identity(dir.path()).unwrap();
        assert!(!named.trim().is_empty());
    }

    #[test]
    fn the_marker_does_not_disturb_reading_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("store");
        let mut store = Store::open(&root, DeviceId("dev_a".into())).unwrap();
        store
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new("comprar pan", "a0"),
            })
            .unwrap();

        identity(&root).unwrap();
        assert_eq!(read_all(&root).unwrap().len(), 1);
    }

    #[test]
    fn what_is_set_aside_is_named_after_the_key_it_replaces() {
        assert!(
            DISPLACED.starts_with(KEEP),
            "{DISPLACED} would not be found beside {KEEP}"
        );
    }

    #[test]
    fn the_key_is_named_after_the_store_it_proves() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
        let named = identity(paths.store()).unwrap();

        let kept = secret(&paths).unwrap();

        assert_eq!(
            std::fs::read(kept_at(&paths, &named)).unwrap(),
            kept,
            "a second store on this machine would write over the first one's key"
        );
    }

    #[test]
    fn two_stores_on_one_machine_do_not_share_a_key() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        let one = crate::Paths::new(dir.path().join("one"), &config);
        let two = crate::Paths::new(dir.path().join("two"), &config);
        identity(one.store()).unwrap();
        identity(two.store()).unwrap();

        let first = secret(&one).unwrap();
        let second = secret(&two).unwrap();

        assert_ne!(first, second, "the second store took over the first's seal");
        assert_eq!(secret(&one).unwrap(), first, "the first store lost its key");
    }

    #[test]
    fn a_key_set_aside_twice_in_the_same_second_keeps_both() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
        std::fs::create_dir_all(paths.private()).unwrap();
        let at = kept_at(&paths, "01ABC");

        assert!(set_aside(&paths, &at, &[1u8; 32], "the first"));
        assert!(set_aside(&paths, &at, &[2u8; 32], "the second"));

        let aside = displaced(&paths);
        assert_eq!(
            aside.len(),
            2,
            "the second write destroyed the first: {aside:?}"
        );
        let held: Vec<Vec<u8>> = aside
            .iter()
            .map(|one| std::fs::read(one).unwrap())
            .collect();
        assert!(held.contains(&vec![1u8; 32]), "the first key is gone");
        assert!(held.contains(&vec![2u8; 32]), "the second key is gone");
    }

    #[test]
    fn a_key_that_is_not_one_is_set_aside_and_a_fresh_one_minted() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
        let named = identity(paths.store()).unwrap();
        std::fs::create_dir_all(paths.private()).unwrap();
        std::fs::write(kept_at(&paths, &named), b"").unwrap();

        let kept = secret(&paths).expect("a store with a broken key can never seal anything again");

        assert_eq!(std::fs::read(kept_at(&paths, &named)).unwrap(), kept);
        assert_eq!(
            displaced(&paths).len(),
            1,
            "what could not be read was destroyed"
        );
    }

    #[test]
    fn a_key_the_migration_left_alone_is_still_what_the_store_seals_with() {
        let dir = tempfile::tempdir().unwrap();
        let mut paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
        paths.unpaired_for_test();
        std::fs::create_dir_all(paths.store()).unwrap();
        std::fs::write(paths.store().join(KEEP), [5u8; 32]).unwrap();

        brought_home(&paths);

        assert_eq!(
            secret(&paths),
            Some([5u8; 32]),
            "the key was left in the store and a brand new one was minted beside it, so every parcel this store handed out is now a stranger's"
        );
    }

    #[test]
    fn settings_kept_somewhere_else_are_still_this_installs_own() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("elsewhere"));
        std::fs::create_dir_all(paths.store()).unwrap();
        std::fs::write(paths.store().join(KEEP), [8u8; 32]).unwrap();

        brought_home(&paths);

        let named = peek_identity(paths.store()).unwrap();
        assert_eq!(
            std::fs::read(kept_at(&paths, &named)).unwrap(),
            [8u8; 32],
            "naming the settings directory made this install look like somebody else's"
        );
    }

    #[test]
    fn a_key_that_cannot_be_removed_is_not_set_aside_again_and_again() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
        std::fs::create_dir_all(paths.private()).unwrap();
        let at = kept_at(&paths, "01ABC");

        for _ in 0..5 {
            assert!(set_aside(&paths, &at, &[4u8; 32], "the same one again"));
        }

        assert_eq!(
            displaced(&paths).len(),
            1,
            "one stuck key grew the private folder on every command"
        );
    }

    #[test]
    fn a_store_opened_with_another_installs_settings_keeps_its_key_where_it_is() {
        let dir = tempfile::tempdir().unwrap();
        let mut paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
        paths.unpaired_for_test();
        let named = identity(paths.store()).unwrap();
        std::fs::write(paths.store().join(KEEP), [3u8; 32]).unwrap();

        brought_home(&paths);

        assert!(
            paths.store().join(KEEP).exists(),
            "somebody else's key was taken off their store"
        );
        assert!(
            !kept_at(&paths, &named).exists(),
            "somebody else's key was installed on this machine"
        );
    }
}
