use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use fs4::fs_std::FileExt;

use crate::{
    Error, Result,
    event::{DeviceId, Event, Op, SCHEMA_VERSION},
};

const ACTIVE: &str = "active.jsonl";
const LOCK: &str = ".lock";
const SEGMENT_MAX_EVENTS: usize = 5_000;

/// Writes only to this device's directory: that turns merging into
/// concatenation instead of conflict.
#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    dir: PathBuf,
    device: DeviceId,
    active_events: usize,
    lock: Option<File>,
}

impl Store {
    pub fn open(store_root: impl AsRef<Path>, device: DeviceId) -> Result<Self> {
        let root = store_root.as_ref().to_path_buf();
        let dir = root.join(&device.0);
        std::fs::create_dir_all(&dir)?;

        Ok(Self {
            active_events: count_lines(&dir.join(ACTIVE))?,
            root,
            dir,
            device,
            lock: None,
        })
    }

    /// Taken on first write, not on open: reading must stay possible while the
    /// GUI holds this device's store.
    fn acquire(&mut self) -> Result<()> {
        if self.lock.is_some() {
            return Ok(());
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(self.dir.join(LOCK))?;
        // fs4 signals failure with `Ok(false)`, not `Err`.
        if !file.try_lock_exclusive()? {
            return Err(Error::AlreadyRunning);
        }
        self.lock = Some(file);
        Ok(())
    }

    pub fn device(&self) -> &DeviceId {
        &self.device
    }

    pub fn append(&mut self, op: Op) -> Result<Event> {
        let event = Event::new(self.device.clone(), jiff::Timestamp::now(), op);
        self.append_event(&event)?;
        Ok(event)
    }

    /// One user action, however many events it takes.
    pub fn append_batch(&mut self, ops: Vec<Op>) -> Result<Vec<Event>> {
        let batch = (ops.len() > 1).then(ulid::Ulid::generate);
        let mut written = Vec::with_capacity(ops.len());

        for op in ops {
            let mut event = Event::new(self.device.clone(), jiff::Timestamp::now(), op);
            event.batch = batch;
            self.append_event(&event)?;
            written.push(event);
        }
        Ok(written)
    }

    pub fn append_event(&mut self, event: &Event) -> Result<()> {
        self.acquire()?;
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
        Ok(())
    }

    /// Closed segments never change again, so Git stores each one once.
    fn rotate(&mut self) -> Result<()> {
        let active = self.dir.join(ACTIVE);
        if active.try_exists()? {
            let next = next_segment_number(&self.dir)?;
            std::fs::rename(&active, self.dir.join(format!("{next:06}.jsonl")))?;
        }
        self.active_events = 0;
        Ok(())
    }

    /// Reads every device, not just this one.
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

        let mut segments: Vec<PathBuf> = std::fs::read_dir(device.path())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .collect();
        segments.sort();

        for segment in segments {
            read_segment(&segment, &mut events)?;
        }
    }

    events.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(events)
}

fn read_segment(path: &Path, out: &mut Vec<Event>) -> Result<()> {
    for (i, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let event: Event = serde_json::from_str(&line).map_err(|source| Error::MalformedEvent {
            file: path.display().to_string(),
            line: i + 1,
            source,
        })?;

        if event.version > SCHEMA_VERSION {
            return Err(Error::UnsupportedVersion(event.version));
        }
        out.push(event);
    }
    Ok(())
}

/// Temp plus rename: a crash mid-write leaves the previous contents.
pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(contents)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn count_lines(path: &Path) -> Result<usize> {
    match File::open(path) {
        Ok(file) => Ok(BufReader::new(file).lines().count()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(Error::Io(e)),
    }
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
    fn a_second_writer_on_the_same_device_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let mut first = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        first.append(add("first")).unwrap();

        let mut second = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        assert!(matches!(
            second.append(add("second")),
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
        assert!(dir.join("000001.jsonl").exists(), "segment was closed");
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
}
