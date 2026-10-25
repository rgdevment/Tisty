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

#[derive(Clone, PartialEq, Eq)]
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
    Barred(tisty_sync::Trouble),
    Broke(String),
}

impl Hitch {
    pub fn code(&self) -> &'static str {
        match self {
            Hitch::Missing(_) => "missing",
            Hitch::Changed(_) => "changed",
            Hitch::Limited { .. } => "limited",
            Hitch::Busy { .. } => "busy",
            Hitch::Spent => "spent",
            Hitch::Paused => "paused",
            Hitch::Full => "full",
            Hitch::Lost => "lost",
            Hitch::Elsewhere { .. } => "elsewhere",
            Hitch::Unreachable(..) => "unreachable",
            Hitch::Troubled(_) => "troubled",
            Hitch::Emptied => "emptied",
            Hitch::Barred(_) => "barred",
            Hitch::Broke(_) => "broke",
        }
    }
}

/// The account found is the person's address, and what is printed may end up in a shared log.
impl std::fmt::Debug for Hitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hitch::Missing(name) => f.debug_tuple("Missing").field(name).finish(),
            Hitch::Changed(name) => f.debug_tuple("Changed").field(name).finish(),
            Hitch::Limited { wait } => f.debug_struct("Limited").field("wait", wait).finish(),
            Hitch::Busy { wait } => f.debug_struct("Busy").field("wait", wait).finish(),
            Hitch::Elsewhere { .. } => f.debug_struct("Elsewhere").finish_non_exhaustive(),
            Hitch::Unreachable(cause, why) => f
                .debug_tuple("Unreachable")
                .field(cause)
                .field(why)
                .finish(),
            Hitch::Troubled(why) => f.debug_tuple("Troubled").field(why).finish(),
            Hitch::Barred(trouble) => f.debug_tuple("Barred").field(&trouble.code()).finish(),
            Hitch::Broke(why) => f.debug_tuple("Broke").field(why).finish(),
            Hitch::Spent => f.write_str("Spent"),
            Hitch::Paused => f.write_str("Paused"),
            Hitch::Full => f.write_str("Full"),
            Hitch::Lost => f.write_str("Lost"),
            Hitch::Emptied => f.write_str("Emptied"),
        }
    }
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

pub(crate) fn folded(name: &str) -> String {
    name.to_lowercase()
}

pub(crate) fn keyed(folds: bool, name: &str) -> String {
    match folds {
        true => folded(name),
        false => name.to_string(),
    }
}

pub(crate) fn same_name(folds: bool, one: &str, other: &str) -> bool {
    match folds {
        true => folded(one) == folded(other),
        false => one == other,
    }
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

pub fn renamed_by_copy<R: Remote + ?Sized>(remote: &R, from: &str, to: &str) -> Result<(), Hitch> {
    named_well(from)?;
    named_well(to)?;
    if from == to {
        return remote
            .about(from)?
            .map(|_| ())
            .ok_or_else(|| Hitch::Missing(from.to_string()));
    }
    if same_name(remote.limits().folds_case, from, to) {
        return Err(Hitch::Broke(format!(
            "{from} cannot be renamed by copy to {to} where the provider folds case"
        )));
    }
    let held = crate::temp::Temp::new();
    let mut into = held.create().map_err(|e| Hitch::Broke(e.to_string()))?;
    let took = remote.fetch(from, 0, &mut into);
    drop(into);
    took?;
    remote.put(to, &held.0, Expect::Any)?;
    remote.delete(from, None)
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
        renamed_by_copy(self, from, to)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        named_well(name)?;
        let under = name.rsplit_once('/').map_or("", |(parent, _)| parent);
        let folds = self.limits().folds_case;
        Ok(self
            .list(under)?
            .into_iter()
            .find(|one| same_name(folds, &one.name, name)))
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

    fn root(&self) -> &str {
        ""
    }

    fn mark_up(&self) -> Result<(), Hitch> {
        Ok(())
    }
}
