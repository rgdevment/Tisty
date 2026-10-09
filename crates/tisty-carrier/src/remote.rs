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
    Broke(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Costs {
    pub list: u64,
    pub fetch: u64,
    pub put: u64,
    pub delete: u64,
    pub changes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub units_a_day: u64,
    pub bytes_a_day: u64,
    pub poll_every: Duration,
    pub chunk: u64,
    pub most_per_file: u64,
    pub costs: Costs,
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

pub trait Remote: Send + Sync {
    fn limits(&self) -> Limits;
    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch>;
    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch>;
    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch>;
    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch>;
    fn hash_of(&self, local: &Path) -> std::io::Result<String>;

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        let under = name.rsplit_once('/').map_or("", |(parent, _)| parent);
        Ok(self.list(under)?.into_iter().find(|one| one.name == name))
    }

    fn changes(&self, _since: Option<&str>) -> Result<Changes, Hitch> {
        Ok(Changes::Whole {
            seen: self.list("")?,
            cursor: String::new(),
        })
    }

    fn append(&self, name: &str, from: &Path, _at: u64, expect: Expect) -> Result<Seen, Hitch> {
        self.put(name, from, expect)
    }

    fn hears(&self) -> Option<Box<dyn Watch>> {
        None
    }

    fn lends(&self, _name: &str) -> Result<Option<String>, Hitch> {
        Ok(None)
    }
}
