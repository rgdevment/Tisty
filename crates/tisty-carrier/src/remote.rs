use std::io::Write;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Limited { wait: Duration },
    Spent,
    Full,
    Lost,
    Elsewhere { found: String },
    Unreachable(String),
    Broke(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Costs {
    pub list: u64,
    pub fetch: u64,
    pub put: u64,
    pub delete: u64,
    pub about: u64,
    pub changes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
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
    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch>;
    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch>;
    fn hash_of(&self, local: &Path) -> std::io::Result<String>;

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
}
