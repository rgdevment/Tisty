use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Watch};

const PAGE: usize = 1000;
const GIB: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Drive,
    OneDrive,
    Dropbox,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Counts {
    pub requests: u64,
    pub units: u64,
    pub sent: u64,
    pub received: u64,
}

pub trait Counting: Remote {
    fn who(&self) -> &'static str;
    fn counts(&self) -> Counts;
    fn forget_counts(&self);
    fn native_changes(&self) -> bool;
    fn native_append(&self) -> bool;
    fn folds_case(&self) -> bool;
}

struct Entry {
    id: u64,
    name: String,
    body: Vec<u8>,
    revision: u64,
}

#[derive(Default)]
struct Shelf {
    files: Vec<Entry>,
    last: u64,
    touched: Vec<(u64, String)>,
    counts: Counts,
}

pub struct Fake {
    kind: Kind,
    shelf: Arc<Mutex<Shelf>>,
}

impl Fake {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            shelf: Arc::default(),
        }
    }

    pub fn drive() -> Self {
        Self::new(Kind::Drive)
    }

    pub fn onedrive() -> Self {
        Self::new(Kind::OneDrive)
    }

    pub fn dropbox() -> Self {
        Self::new(Kind::Dropbox)
    }

    fn lock(&self) -> MutexGuard<'_, Shelf> {
        self.shelf.lock().unwrap()
    }

    pub fn plant(&self, name: &str, body: &[u8]) {
        assert_eq!(self.kind, Kind::Drive, "only Drive keeps two of one name");
        let mut shelf = self.lock();
        shelf.last += 1;
        let id = shelf.last;
        shelf.files.push(Entry {
            id,
            name: name.to_string(),
            body: body.to_vec(),
            revision: id,
        });
        shelf.touched.push((id, name.to_string()));
    }

    pub fn copies_of(&self, name: &str) -> usize {
        self.lock()
            .files
            .iter()
            .filter(|one| one.name == name)
            .count()
    }

    fn key(&self, name: &str) -> String {
        match self.folds_case() {
            true => name.to_lowercase(),
            false => name.to_string(),
        }
    }

    fn within(&self, name: &str, under: &str) -> bool {
        let under = under.trim_end_matches('/');
        under.is_empty() || self.key(name).starts_with(&format!("{}/", self.key(under)))
    }

    fn winner(&self, shelf: &Shelf, name: &str) -> Option<usize> {
        let wanted = self.key(name);
        shelf
            .files
            .iter()
            .enumerate()
            .filter(|(_, one)| self.key(&one.name) == wanted)
            .min_by_key(|(_, one)| one.id)
            .map(|(at, _)| at)
    }

    fn seen(&self, one: &Entry) -> Seen {
        Seen {
            name: one.name.clone(),
            bytes: one.body.len() as u64,
            hash: self.hash(&one.body),
            revision: format!("r{}", one.revision),
        }
    }

    fn hash(&self, body: &[u8]) -> String {
        let kind = match self.kind {
            Kind::Drive => "md5",
            Kind::OneDrive => "quickxor",
            Kind::Dropbox => "content",
        };
        format!("{kind}:{}", tisty_core::attach::printed(body))
    }

    fn listed(&self, shelf: &Shelf, under: &str) -> Vec<Seen> {
        let mut firsts: BTreeMap<String, &Entry> = BTreeMap::new();
        for one in shelf
            .files
            .iter()
            .filter(|one| self.within(&one.name, under))
        {
            let slot = firsts.entry(self.key(&one.name)).or_insert(one);
            if one.id < slot.id {
                *slot = one;
            }
        }
        firsts.values().map(|one| self.seen(one)).collect()
    }

    fn spend(&self, shelf: &mut Shelf, requests: u64, cost: u64) {
        shelf.counts.requests += requests;
        shelf.counts.units += requests * cost;
    }

    fn pages(&self, wholes: usize, per: usize) -> u64 {
        wholes.div_ceil(per).max(1) as u64
    }

    fn chunks(&self, bytes: u64) -> u64 {
        bytes.div_ceil(self.limits().chunk).max(1)
    }

    fn written(&self, shelf: &mut Shelf, name: &str, body: Vec<u8>, at: Option<usize>) -> Seen {
        shelf.last += 1;
        let stamp = shelf.last;
        let seen = match at {
            Some(at) => {
                shelf.files[at].body = body;
                shelf.files[at].revision = stamp;
                self.seen(&shelf.files[at])
            }
            None => {
                let one = Entry {
                    id: stamp,
                    name: name.to_string(),
                    body,
                    revision: stamp,
                };
                let seen = self.seen(&one);
                shelf.files.push(one);
                seen
            }
        };
        shelf.touched.push((stamp, seen.name.clone()));
        seen
    }

    fn allowed(&self, shelf: &Shelf, name: &str, expect: &Expect) -> Result<Option<usize>, Hitch> {
        let at = self.winner(shelf, name);
        match (expect, at) {
            (Expect::Absent, Some(_)) => Err(Hitch::Changed(name.to_string())),
            (Expect::Revision(_), None) => Err(Hitch::Missing(name.to_string())),
            (Expect::Revision(rev), Some(at)) if self.seen(&shelf.files[at]).revision != *rev => {
                Err(Hitch::Changed(name.to_string()))
            }
            _ => Ok(at),
        }
    }

    fn read_local(&self, from: &Path) -> Result<Vec<u8>, Hitch> {
        let body = std::fs::read(from).map_err(|e| Hitch::Broke(e.to_string()))?;
        match body.len() as u64 > self.limits().most_per_file {
            true => Err(Hitch::Broke("past what the provider takes".to_string())),
            false => Ok(body),
        }
    }
}

fn named_well(name: &str) -> Result<(), Hitch> {
    let bad = name.is_empty()
        || name.contains('\\')
        || name.chars().any(char::is_control)
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..");
    match bad {
        true => Err(Hitch::Broke(format!("not a name: {name}"))),
        false => Ok(()),
    }
}

impl Remote for Fake {
    fn limits(&self) -> Limits {
        let (chunk, most_per_file, costs) = match self.kind {
            Kind::Drive => (
                8 * 1024 * 1024,
                5 * 1024 * GIB,
                Costs {
                    list: 100,
                    fetch: 200,
                    put: 50,
                    delete: 50,
                    changes: 100,
                },
            ),
            Kind::OneDrive => (
                10 * 1024 * 1024,
                250 * GIB,
                Costs {
                    list: 2,
                    fetch: 1,
                    put: 2,
                    delete: 2,
                    changes: 1,
                },
            ),
            Kind::Dropbox => (
                4 * 1024 * 1024,
                350 * GIB,
                Costs {
                    list: 1,
                    fetch: 1,
                    put: 1,
                    delete: 1,
                    changes: 1,
                },
            ),
        };
        Limits {
            units_a_day: 100_000,
            bytes_a_day: 5 * GIB,
            poll_every: Duration::from_secs(30),
            chunk,
            most_per_file,
            costs,
        }
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        let mut shelf = self.lock();
        let seen = self.listed(&shelf, under);
        let requests = self.pages(seen.len(), PAGE);
        self.spend(&mut shelf, requests, self.limits().costs.list);
        Ok(seen)
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        named_well(name)?;
        let mut shelf = self.lock();
        self.spend(&mut shelf, 1, self.limits().costs.fetch);
        let at = self
            .winner(&shelf, name)
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        let one = &shelf.files[at];
        let tail = usize::try_from(from)
            .ok()
            .and_then(|from| one.body.get(from..))
            .ok_or_else(|| Hitch::Broke(format!("past the end of {name}")))?;
        into.write_all(tail)
            .map_err(|e| Hitch::Broke(e.to_string()))?;
        let (seen, received) = (self.seen(one), tail.len() as u64);
        shelf.counts.received += received;
        Ok(seen)
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        named_well(name)?;
        let body = self.read_local(from)?;
        let mut shelf = self.lock();
        let costs = self.limits().costs;
        let at = match self.allowed(&shelf, name, &expect) {
            Ok(at) => at,
            Err(refused) => {
                self.spend(&mut shelf, 1, costs.put);
                return Err(refused);
            }
        };
        self.spend(&mut shelf, self.chunks(body.len() as u64), costs.put);
        shelf.counts.sent += body.len() as u64;
        Ok(self.written(&mut shelf, name, body, at))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        named_well(name)?;
        let mut shelf = self.lock();
        self.spend(&mut shelf, 1, self.limits().costs.delete);
        let at = self
            .winner(&shelf, name)
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        if expect.is_some_and(|rev| self.seen(&shelf.files[at]).revision != rev) {
            return Err(Hitch::Changed(name.to_string()));
        }
        shelf.last += 1;
        let stamp = shelf.last;
        let gone = shelf.files.remove(at);
        shelf.touched.push((stamp, gone.name));
        Ok(())
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        Ok(self.hash(&std::fs::read(local)?))
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        named_well(name)?;
        let mut shelf = self.lock();
        self.spend(&mut shelf, 1, self.limits().costs.list);
        Ok(self
            .winner(&shelf, name)
            .map(|at| self.seen(&shelf.files[at])))
    }

    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        let mut shelf = self.lock();
        let cursor = shelf.last.to_string();
        let Some(since) = since else {
            let seen = self.listed(&shelf, "");
            let requests = self.pages(seen.len(), PAGE);
            self.spend(&mut shelf, requests, self.limits().costs.changes);
            return Ok(Changes::Whole { seen, cursor });
        };
        let since: u64 = since
            .parse()
            .map_err(|_| Hitch::Broke(format!("not a cursor: {since}")))?;
        let mut latest: BTreeMap<String, String> = BTreeMap::new();
        for (when, name) in &shelf.touched {
            if *when > since {
                latest.insert(self.key(name), name.clone());
            }
        }
        let (mut changed, mut gone) = (Vec::new(), Vec::new());
        for name in latest.into_values() {
            match self.winner(&shelf, &name) {
                Some(at) => changed.push(self.seen(&shelf.files[at])),
                None => gone.push(name),
            }
        }
        let requests = self.pages(changed.len() + gone.len(), PAGE);
        self.spend(&mut shelf, requests, self.limits().costs.changes);
        Ok(Changes::Since {
            changed,
            gone,
            cursor,
        })
    }

    fn append(&self, name: &str, from: &Path, at: u64, expect: Expect) -> Result<Seen, Hitch> {
        if self.kind != Kind::Dropbox {
            return self.put(name, from, expect);
        }
        named_well(name)?;
        let body = self.read_local(from)?;
        let mut shelf = self.lock();
        let costs = self.limits().costs;
        let held = match self.allowed(&shelf, name, &expect) {
            Ok(held) if held.map_or(0, |held| shelf.files[held].body.len() as u64) == at => held,
            Ok(_) => {
                self.spend(&mut shelf, 1, costs.put);
                return Err(Hitch::Changed(name.to_string()));
            }
            Err(refused) => {
                self.spend(&mut shelf, 1, costs.put);
                return Err(refused);
            }
        };
        let tail = (body.len() as u64).saturating_sub(at);
        self.spend(&mut shelf, self.chunks(tail), costs.put);
        shelf.counts.sent += tail;
        Ok(self.written(&mut shelf, name, body, held))
    }

    fn hears(&self) -> Option<Box<dyn Watch>> {
        if self.kind != Kind::Dropbox {
            return None;
        }
        let last = self.lock().last;
        Some(Box::new(Told {
            shelf: Arc::clone(&self.shelf),
            last,
        }))
    }

    fn lends(&self, name: &str) -> Result<Option<String>, Hitch> {
        let mut shelf = self.lock();
        self.spend(&mut shelf, 1, self.limits().costs.list);
        Ok(self
            .winner(&shelf, name)
            .map(|_| format!("https://links.invalid/{name}")))
    }
}

struct Told {
    shelf: Arc<Mutex<Shelf>>,
    last: u64,
}

impl Watch for Told {
    fn told(&mut self, _within: Duration) -> Result<bool, Hitch> {
        let now = self.shelf.lock().unwrap().last;
        let moved = now > self.last;
        self.last = now;
        Ok(moved)
    }
}

impl Counting for Fake {
    fn who(&self) -> &'static str {
        match self.kind {
            Kind::Drive => "drive",
            Kind::OneDrive => "onedrive",
            Kind::Dropbox => "dropbox",
        }
    }

    fn counts(&self) -> Counts {
        self.lock().counts.clone()
    }

    fn forget_counts(&self) {
        self.lock().counts = Counts::default();
    }

    fn native_changes(&self) -> bool {
        true
    }

    fn native_append(&self) -> bool {
        self.kind == Kind::Dropbox
    }

    fn folds_case(&self) -> bool {
        self.kind != Kind::Drive
    }
}

pub struct Bare(Fake);

impl Bare {
    pub fn over_drive() -> Self {
        Self(Fake::drive())
    }
}

impl Remote for Bare {
    fn limits(&self) -> Limits {
        self.0.limits()
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.0.list(under)
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        self.0.fetch(name, from, into)
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        self.0.put(name, from, expect)
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.0.delete(name, expect)
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.0.hash_of(local)
    }
}

impl Counting for Bare {
    fn who(&self) -> &'static str {
        "bare"
    }

    fn counts(&self) -> Counts {
        self.0.counts()
    }

    fn forget_counts(&self) {
        self.0.forget_counts();
    }

    fn native_changes(&self) -> bool {
        false
    }

    fn native_append(&self) -> bool {
        false
    }

    fn folds_case(&self) -> bool {
        false
    }
}

pub fn every() -> Vec<Box<dyn Counting>> {
    vec![
        Box::new(Fake::drive()),
        Box::new(Fake::onedrive()),
        Box::new(Fake::dropbox()),
        Box::new(Bare::over_drive()),
    ]
}
