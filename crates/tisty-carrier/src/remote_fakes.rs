use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Watch, named_well};

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

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
    fn fail_next(&self, hitch: Hitch) {
        self.fail_after(0, hitch);
    }

    fn who(&self) -> &'static str;
    fn counts(&self) -> Counts;
    fn forget_counts(&self);
    fn fail_after(&self, passing: usize, hitch: Hitch);
    fn forget_feed(&self);
    fn native_changes(&self) -> bool;
    fn hears_pushed(&self) -> bool;
    fn lends_links(&self) -> bool;
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
    floor: u64,
    touched: Vec<(u64, String)>,
    failing: VecDeque<Option<Hitch>>,
    counts: Counts,
}

pub struct Fake {
    kind: Kind,
    page: Option<usize>,
    chunk: Option<u64>,
    shelf: Arc<Mutex<Shelf>>,
}

impl Fake {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            page: None,
            chunk: None,
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

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_chunk(mut self, chunk: u64) -> Self {
        self.chunk = Some(chunk);
        self
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
        match self.limits().folds_case {
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

    fn gate(&self, shelf: &mut Shelf, cost: u64) -> Result<(), Hitch> {
        match shelf.failing.pop_front().flatten() {
            Some(hitch) => {
                self.spend(shelf, 1, cost);
                Err(hitch)
            }
            None => Ok(()),
        }
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

impl Remote for Fake {
    fn limits(&self) -> Limits {
        let (chunk, page, most_per_file, poll_every, costs) = match self.kind {
            Kind::Drive => (
                8 * MIB,
                1000,
                5 * 1024 * GIB,
                300,
                Costs {
                    list: 100,
                    fetch: 200,
                    put: 50,
                    delete: 50,
                    about: 5,
                    changes: 100,
                },
            ),
            Kind::OneDrive => (
                10 * MIB,
                200,
                250 * GIB,
                30,
                Costs {
                    list: 2,
                    fetch: 1,
                    put: 2,
                    delete: 2,
                    about: 1,
                    changes: 1,
                },
            ),
            Kind::Dropbox => (
                4 * MIB,
                2000,
                350 * GIB,
                30,
                Costs {
                    list: 1,
                    fetch: 1,
                    put: 1,
                    delete: 1,
                    about: 1,
                    changes: 1,
                },
            ),
        };
        Limits {
            units_a_day: 100_000,
            bytes_a_day: 5 * GIB,
            poll_every: Duration::from_secs(poll_every),
            chunk: self.chunk.unwrap_or(chunk),
            page: self.page.unwrap_or(page),
            most_per_file,
            folds_case: self.kind != Kind::Drive,
            costs,
        }
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        let limits = self.limits();
        let mut shelf = self.lock();
        self.gate(&mut shelf, limits.costs.list)?;
        let seen = self.listed(&shelf, under);
        self.spend(
            &mut shelf,
            limits.requests_to_list(seen.len()),
            limits.costs.list,
        );
        Ok(seen)
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        named_well(name)?;
        let cost = self.limits().costs.fetch;
        let mut shelf = self.lock();
        self.gate(&mut shelf, cost)?;
        self.spend(&mut shelf, 1, cost);
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
        let limits = self.limits();
        let mut shelf = self.lock();
        self.gate(&mut shelf, limits.costs.put)?;
        let at = match self.allowed(&shelf, name, &expect) {
            Ok(at) => at,
            Err(refused) => {
                self.spend(&mut shelf, 1, limits.costs.put);
                return Err(refused);
            }
        };
        let requests = limits.requests_to_put(body.len() as u64);
        self.spend(&mut shelf, requests, limits.costs.put);
        shelf.counts.sent += body.len() as u64;
        Ok(self.written(&mut shelf, name, body, at))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        named_well(name)?;
        let cost = self.limits().costs.delete;
        let mut shelf = self.lock();
        self.gate(&mut shelf, cost)?;
        self.spend(&mut shelf, 1, cost);
        let at = self
            .winner(&shelf, name)
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        if expect.is_some_and(|rev| self.seen(&shelf.files[at]).revision != rev) {
            return Err(Hitch::Changed(name.to_string()));
        }
        shelf.last += 1;
        let stamp = shelf.last;
        let (key, shown) = (self.key(name), shelf.files[at].name.clone());
        shelf.files.retain(|one| self.key(&one.name) != key);
        shelf.touched.push((stamp, shown));
        Ok(())
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        Ok(self.hash(&std::fs::read(local)?))
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        named_well(name)?;
        let cost = self.limits().costs.about;
        let mut shelf = self.lock();
        self.gate(&mut shelf, cost)?;
        self.spend(&mut shelf, 1, cost);
        Ok(self
            .winner(&shelf, name)
            .map(|at| self.seen(&shelf.files[at])))
    }

    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        let limits = self.limits();
        let mut shelf = self.lock();
        self.gate(&mut shelf, limits.costs.changes)?;
        let cursor = shelf.last.to_string();
        let known = since
            .and_then(|one| one.parse::<u64>().ok())
            .filter(|one| (shelf.floor..=shelf.last).contains(one));
        let Some(since) = known else {
            let seen = self.listed(&shelf, "");
            let requests = limits.requests_to_list(seen.len());
            self.spend(&mut shelf, requests, limits.costs.changes);
            return Ok(Changes::Whole { seen, cursor });
        };
        let mut spellings: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (_, name) in shelf.touched.iter().filter(|(when, _)| *when > since) {
            spellings
                .entry(self.key(name))
                .or_default()
                .insert(name.clone());
        }
        let (mut changed, mut gone) = (Vec::new(), Vec::new());
        for (key, names) in spellings {
            let held = self
                .winner(&shelf, &key)
                .map(|at| self.seen(&shelf.files[at]));
            gone.extend(
                names
                    .into_iter()
                    .filter(|name| held.as_ref().is_none_or(|seen| seen.name != *name)),
            );
            changed.extend(held);
        }
        let requests = limits.requests_to_list(changed.len() + gone.len());
        self.spend(&mut shelf, requests, limits.costs.changes);
        Ok(Changes::Since {
            changed,
            gone,
            cursor,
        })
    }

    fn hears(&self, since: &str) -> Option<Box<dyn Watch>> {
        if self.kind != Kind::Dropbox {
            return None;
        }
        Some(Box::new(Told {
            shelf: Arc::clone(&self.shelf),
            last: since.parse().unwrap_or(0),
        }))
    }

    fn lends(&self, name: &str) -> Result<Option<String>, Hitch> {
        named_well(name)?;
        let cost = self.limits().costs.about;
        let mut shelf = self.lock();
        self.gate(&mut shelf, cost)?;
        self.spend(&mut shelf, 1, cost);
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

    fn fail_after(&self, passing: usize, hitch: Hitch) {
        let mut shelf = self.lock();
        shelf.failing.extend(std::iter::repeat_n(None, passing));
        shelf.failing.push_back(Some(hitch));
    }

    fn forget_feed(&self) {
        let mut shelf = self.lock();
        shelf.last += 1;
        shelf.floor = shelf.last;
    }

    fn native_changes(&self) -> bool {
        true
    }

    fn hears_pushed(&self) -> bool {
        self.kind == Kind::Dropbox
    }

    fn lends_links(&self) -> bool {
        true
    }
}

pub struct Bare(Fake);

impl Bare {
    pub fn over(fake: Fake) -> Self {
        Self(fake)
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
        match self.0.kind {
            Kind::Drive => "bare-drive",
            Kind::OneDrive => "bare-onedrive",
            Kind::Dropbox => "bare-dropbox",
        }
    }

    fn counts(&self) -> Counts {
        self.0.counts()
    }

    fn forget_counts(&self) {
        self.0.forget_counts();
    }

    fn fail_after(&self, passing: usize, hitch: Hitch) {
        self.0.fail_after(passing, hitch);
    }

    fn forget_feed(&self) {}

    fn native_changes(&self) -> bool {
        false
    }

    fn hears_pushed(&self) -> bool {
        false
    }

    fn lends_links(&self) -> bool {
        false
    }
}

pub fn every() -> Vec<Box<dyn Counting>> {
    every_with(|fake| fake)
}

pub fn every_with(shape: impl Fn(Fake) -> Fake) -> Vec<Box<dyn Counting>> {
    vec![
        Box::new(shape(Fake::drive())),
        Box::new(shape(Fake::onedrive())),
        Box::new(shape(Fake::dropbox())),
        Box::new(Bare::over(shape(Fake::drive()))),
        Box::new(Bare::over(shape(Fake::dropbox()))),
    ]
}
