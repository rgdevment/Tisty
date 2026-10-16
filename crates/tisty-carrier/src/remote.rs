use std::io::Write;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Seen {
    pub name: String,
    pub bytes: u64,
    pub hash: String,
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    Absent,
    Revision(String),
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hitch {
    Missing(String),
    Changed(String),
    Limited {
        wait: Duration,
    },
    Busy {
        wait: Duration,
    },
    Spent,
    /// Held by Tisty itself, from afar or on this machine, never by the provider.
    Paused,
    Full,
    Lost,
    Elsewhere {
        found: String,
    },
    /// What stood in the way travels with it, so the window can say why the provider stays out of reach.
    Unreachable(Option<crate::Cause>, String),
    /// The provider failed on its side and gave no wait, so the round is tried again later.
    Troubled(String),
    /// Everything the store held up there is gone at once, deleted outside Tisty, and is never read as each file deleted.
    Emptied,
    Broke(String),
}

impl From<std::io::Error> for Hitch {
    fn from(e: std::io::Error) -> Self {
        Hitch::Broke(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Costs {
    pub list: u64,
    pub fetch: u64,
    pub put: u64,
    pub delete: u64,
    pub about: u64,
    pub changes: u64,
    pub checks: u64,
}

// Only a guard against a copy stuck in a loop: the provider paces by the minute, and a person is never held back.
pub const UNITS_A_DAY: u64 = 2_000_000;
pub const BYTES_A_DAY: u64 = 50 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub provider: &'static str,
    pub units_a_day: u64,
    pub bytes_a_day: u64,
    pub poll_every: Duration,
    pub chunk: u64,
    pub page: usize,
    pub most_per_file: u64,
    pub folds_case: bool,
    pub costs: Costs,
}

impl Limits {
    pub fn requests_to_list(&self, entries: usize) -> u64 {
        entries.div_ceil(self.page.max(1)).max(1) as u64
    }

    pub fn requests_to_put(&self, bytes: u64) -> u64 {
        match bytes <= self.chunk {
            true => 1,
            false => 1 + bytes.div_ceil(self.chunk.max(1)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Changes {
    Whole {
        seen: Vec<Seen>,
        cursor: String,
    },
    Since {
        changed: Vec<Seen>,
        gone: Vec<String>,
        cursor: String,
    },
}

pub trait Watch: Send {
    fn told(&mut self, within: Duration) -> Result<bool, Hitch>;
}

pub fn named_well(name: &str) -> Result<(), Hitch> {
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

pub trait Remote: Send + Sync {
    fn limits(&self) -> Limits;
    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch>;
    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch>;
    /// Never leaves these bytes over a revision `expect` did not name: a write that finds one put back and answers Changed.
    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch>;
    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch>;
    fn hash_of(&self, local: &Path) -> std::io::Result<String>;

    /// Where a provider can only read and write, the bytes go down and up once; one that moves does it in place.
    fn renamed(&self, from: &str, to: &str) -> Result<(), Hitch> {
        named_well(from)?;
        named_well(to)?;
        let held = std::env::temp_dir().join(format!(
            "tisty-renamed-{}-{}",
            std::process::id(),
            to.replace('/', "-")
        ));
        let mut into = std::fs::File::create(&held).map_err(|e| Hitch::Broke(e.to_string()))?;
        let took = self.fetch(from, 0, &mut into);
        drop(into);
        let done = took
            .and_then(|_| self.put(to, &held, Expect::Any))
            .and_then(|_| self.delete(from, None));
        let _ = std::fs::remove_file(&held);
        done
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        named_well(name)?;
        let under = name.rsplit_once('/').map_or("", |(parent, _)| parent);
        let same = |one: &str| match self.limits().folds_case {
            true => one.to_lowercase() == name.to_lowercase(),
            false => one == name,
        };
        Ok(self.list(under)?.into_iter().find(|one| same(&one.name)))
    }

    fn changes(&self, _since: Option<&str>) -> Result<Changes, Hitch> {
        Ok(Changes::Whole {
            seen: self.list("")?,
            cursor: String::new(),
        })
    }

    fn hears(&self, _since: &str) -> Option<Box<dyn Watch>> {
        None
    }

    fn lends(&self, _name: &str) -> Result<Option<String>, Hitch> {
        Ok(None)
    }

    /// Bytes the account can still take, or None where the provider sets no ceiling or does not say.
    fn room(&self) -> Result<Option<u64>, Hitch> {
        Ok(None)
    }

    /// Units spent on requests no verb's cost covers since last asked, so the budget counts them too.
    fn spent_beyond(&self) -> u64 {
        0
    }

    /// Something landed or was found already there, so a meter of asks that came to nothing starts again.
    fn moved_on(&self) {}
}
