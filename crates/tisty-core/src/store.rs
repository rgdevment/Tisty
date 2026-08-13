use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use fs4::fs_std::FileExt;

use crate::{
    Error, Result,
    event::{DeviceId, Event, Op, SCHEMA_VERSION},
    witness::{self, Fact, channel},
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
        while !file.try_lock_exclusive()? {
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
            let (timestamp, seq) = s.stamp();
            let mut event = Event::new(s.device.clone(), timestamp, op);
            event.seq = seq;
            s.write(&event)?;
            Ok(event)
        })
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
                let (timestamp, seq) = s.stamp();
                let mut event = Event::new(s.device.clone(), timestamp, op);
                event.batch = batch;
                event.undo = undo;
                event.redo = redo;
                event.seq = seq;
                s.write(&event)?;
                written.push(event);
            }
            Ok(written)
        })
    }

    pub fn append_event(&mut self, event: &Event) -> Result<()> {
        self.locked(|s| s.write(event))
    }

    fn write(&mut self, event: &Event) -> Result<()> {
        if self.active_events >= SEGMENT_MAX_EVENTS {
            self.rotate()?;
        }

        let mut line = serde_json::to_string(event)?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(ACTIVE))?;
        file.write_all(line.as_bytes())?;
        file.sync_all()?;

        self.active_events += 1;
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

        let mut segments = segments_in(&device.path())?;
        segments.sort();
        contiguous(&segments)?;

        for segment in segments {
            let closed = segment.file_name().is_some_and(|n| n != ACTIVE);
            let before = events.len();
            read_segment(&segment, &mut events)?;

            if closed {
                let found = events.len() - before;
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

pub fn is_segment(name: &str) -> bool {
    name == ACTIVE
        || name.strip_suffix(".tisty").is_some_and(|stem| {
            (6..=10).contains(&stem.len()) && stem.bytes().all(|b| b.is_ascii_digit())
        })
}

pub fn segments_in(device_dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(device_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|at| {
            at.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_segment)
        })
        .collect())
}

pub fn inhabited(store_root: impl AsRef<Path>) -> bool {
    std::fs::read_dir(store_root.as_ref()).is_ok_and(|entries| {
        entries.filter_map(|e| e.ok()).any(|e| {
            e.file_type().is_ok_and(|kind| kind.is_dir())
                && segments_in(&e.path()).is_ok_and(|found| !found.is_empty())
        })
    })
}

pub fn check_device(device_dir: &Path) -> Result<usize> {
    let mut segments = segments_in(device_dir)?;
    segments.sort();
    contiguous(&segments)?;

    let mut events = Vec::new();
    for segment in &segments {
        let before = events.len();
        read_segment(segment, &mut events)?;
        if segment.file_name().is_some_and(|n| n != ACTIVE) {
            let found = events.len() - before;
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
}

fn read_segment(path: &Path, out: &mut Vec<Event>) -> Result<()> {
    for (i, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let stamp: Stamped =
            serde_json::from_str(&line).map_err(|source| Error::MalformedEvent {
                file: path.display().to_string(),
                line: i + 1,
                source,
            })?;
        if stamp.v > SCHEMA_VERSION {
            return Err(Error::UnsupportedVersion(stamp.v));
        }

        let event: Event = serde_json::from_str(&line).map_err(|source| Error::MalformedEvent {
            file: path.display().to_string(),
            line: i + 1,
            source,
        })?;
        out.push(event);
    }
    Ok(())
}

pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    static TURN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let turn = TURN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = path.with_extension(format!("{}.{turn}.tmp", std::process::id()));
    {
        let mut file = File::create(&tmp)?;
        let _ = crate::paths::ours_alone(&tmp);
        file.write_all(contents)?;
        file.sync_all()?;
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
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
    use super::*;
    use crate::event::TaskAdd;
    use ulid::Ulid;

    fn add(title: &str) -> Op {
        Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(title, "a0"),
        }
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
    fn an_empty_store_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_all(tmp.path().join("missing")).unwrap().is_empty());
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
}
