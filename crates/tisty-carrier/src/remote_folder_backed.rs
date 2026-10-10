use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use crate::fakes::{Counting, Counts};
use crate::{Costs, Expect, Hitch, Limits, Remote, Seen, named_well};

#[derive(Default)]
struct Book {
    counts: Counts,
    uploads: Vec<String>,
    failing: VecDeque<Option<Hitch>>,
}

/// A provider whose files are a real folder: what a test plants or tampers with there is what the
/// cloud holds, and the folder a cloud keeps is laid out byte for byte as a shared folder is.
pub struct FolderBacked {
    root: PathBuf,
    book: Mutex<Book>,
}

impl FolderBacked {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        std::fs::create_dir_all(&root).unwrap();
        Self {
            root,
            book: Mutex::default(),
        }
    }

    fn at_name(&self, name: &str) -> Result<PathBuf, Hitch> {
        named_well(name)?;
        Ok(name
            .split('/')
            .fold(self.root.clone(), |at, part| at.join(part)))
    }

    fn gate(&self) -> Result<(), Hitch> {
        let mut book = self.book.lock().unwrap();
        book.counts.requests += 1;
        match book.failing.pop_front().flatten() {
            Some(hitch) => Err(hitch),
            None => Ok(()),
        }
    }

    fn seen(&self, name: &str, at: &Path) -> Result<Option<Seen>, Hitch> {
        match std::fs::read(at) {
            Ok(body) => {
                let hash = format!("sha256:{}", tisty_core::attach::printed(&body));
                Ok(Some(Seen {
                    name: name.to_string(),
                    bytes: body.len() as u64,
                    revision: hash.clone(),
                    hash,
                }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Hitch::Broke(e.to_string())),
        }
    }

    fn walk(&self, at: &Path, named: &str, into: &mut Vec<Seen>) -> Result<(), Hitch> {
        let Ok(entries) = std::fs::read_dir(at) else {
            return Ok(());
        };
        for entry in entries.filter_map(Result::ok) {
            let Some(leaf) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let name = match named {
                "" => leaf,
                named => format!("{named}/{leaf}"),
            };
            let path = entry.path();
            if path.is_dir() {
                self.walk(&path, &name, into)?;
            } else if let Some(seen) = self.seen(&name, &path)? {
                into.push(seen);
            }
        }
        Ok(())
    }

    fn matches(&self, name: &str, at: &Path, expect: &Expect) -> Result<(), Hitch> {
        let held = self.seen(name, at)?;
        match (expect, held) {
            (Expect::Absent, Some(_)) => Err(Hitch::Changed(name.to_string())),
            (Expect::Revision(_), None) => Err(Hitch::Missing(name.to_string())),
            (Expect::Revision(rev), Some(seen)) if seen.revision != *rev => {
                Err(Hitch::Changed(name.to_string()))
            }
            _ => Ok(()),
        }
    }
}

impl Remote for FolderBacked {
    fn limits(&self) -> Limits {
        Limits {
            units_a_day: u64::MAX,
            bytes_a_day: u64::MAX,
            poll_every: Duration::from_secs(30),
            chunk: u64::MAX,
            page: usize::MAX,
            most_per_file: u64::MAX,
            folds_case: cfg!(any(windows, target_os = "macos")),
            costs: Costs {
                list: 1,
                fetch: 1,
                put: 1,
                delete: 1,
                about: 1,
                changes: 1,
            },
        }
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.gate()?;
        self.book.lock().unwrap().counts.lists += 1;
        let under = under.trim_end_matches('/');
        let at = match under {
            "" => self.root.clone(),
            under => self.at_name(under)?,
        };
        let mut seen = Vec::new();
        self.walk(&at, under, &mut seen)?;
        Ok(seen)
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        let at = self.at_name(name)?;
        self.gate()?;
        let seen = self
            .seen(name, &at)?
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        let body = std::fs::read(&at).map_err(|e| Hitch::Broke(e.to_string()))?;
        let tail = usize::try_from(from)
            .ok()
            .and_then(|from| body.get(from..))
            .ok_or_else(|| Hitch::Broke(format!("past the end of {name}")))?;
        into.write_all(tail)
            .map_err(|e| Hitch::Broke(e.to_string()))?;
        let mut book = self.book.lock().unwrap();
        book.counts.fetches += 1;
        book.counts.received += tail.len() as u64;
        Ok(seen)
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        let at = self.at_name(name)?;
        self.gate()?;
        self.matches(name, &at, &expect)?;
        let body = std::fs::read(from).map_err(|e| Hitch::Broke(e.to_string()))?;
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Hitch::Broke(e.to_string()))?;
        }
        tisty_core::store::write_atomic(&at, &body).map_err(|e| Hitch::Broke(e.to_string()))?;
        let mut book = self.book.lock().unwrap();
        book.counts.puts += 1;
        book.counts.sent += body.len() as u64;
        book.uploads.push(name.to_string());
        drop(book);
        self.seen(name, &at)?
            .ok_or_else(|| Hitch::Broke(format!("{name} went away as it was written")))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        let at = self.at_name(name)?;
        self.gate()?;
        let held = self
            .seen(name, &at)?
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        if expect.is_some_and(|rev| held.revision != rev) {
            return Err(Hitch::Changed(name.to_string()));
        }
        std::fs::remove_file(&at).map_err(|e| Hitch::Broke(e.to_string()))?;
        self.book.lock().unwrap().counts.deletes += 1;
        Ok(())
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        Ok(format!(
            "sha256:{}",
            tisty_core::attach::printed(&std::fs::read(local)?)
        ))
    }
}

impl Counting for FolderBacked {
    fn who(&self) -> &'static str {
        "folder-backed"
    }

    fn counts(&self) -> Counts {
        self.book.lock().unwrap().counts.clone()
    }

    fn forget_counts(&self) {
        self.book.lock().unwrap().counts = Counts::default();
    }

    fn uploaded(&self) -> Vec<String> {
        self.book.lock().unwrap().uploads.clone()
    }

    fn fail_after(&self, passing: usize, hitch: Hitch) {
        let mut book = self.book.lock().unwrap();
        book.failing.extend(std::iter::repeat_n(None, passing));
        book.failing.push_back(Some(hitch));
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
