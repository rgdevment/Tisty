use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once, OnceLock};

use crate::text::one_line;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Gravity {
    Trace,
    Note,
    Warn,
    Error,
    Fatal,
}

impl Gravity {
    fn worded(self) -> &'static str {
        match self {
            Gravity::Trace => "TRACE",
            Gravity::Note => "NOTE",
            Gravity::Warn => "WARN",
            Gravity::Error => "ERROR",
            Gravity::Fatal => "FATAL",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Fact {
    Count(usize),
    Bytes(u64),
    Id(String),
    Code(&'static str),
    Path(PathBuf),
    Why(String),
    Word(&'static str),
}

impl Fact {
    fn shown(&self) -> String {
        match self {
            Fact::Count(n) => n.to_string(),
            Fact::Bytes(n) => n.to_string(),
            Fact::Id(id) => one_line(&kept_short(Path::new(id))),
            Fact::Code(code) => (*code).to_string(),
            Fact::Path(at) => format!("{:?}", hidden(&one_line(&kept_short(at)))),
            Fact::Why(why) => format!("{:?}", hidden(&crate::masked::masked(&one_line(why)))),
            Fact::Word(word) => (*word).to_string(),
        }
    }
}

pub mod channel {
    pub const STORE: &str = "store";
    pub const CACHE: &str = "cache";
    pub const SYNC: &str = "sync";
    pub const ATTACH: &str = "attach";
    pub const CONFIG: &str = "config";
    pub const HERALD: &str = "herald";
    pub const WINDOW: &str = "window";
    pub const TERMINAL: &str = "terminal";
    pub const AGENT: &str = "agent";
    pub const BACKUP: &str = "backup";
    pub const CLOUD: &str = "cloud";
}

struct Kept {
    at: PathBuf,
}

static KEPT: OnceLock<Mutex<Option<Kept>>> = OnceLock::new();
static ALL: AtomicBool = AtomicBool::new(false);
static HOOKED: Once = Once::new();

thread_local! {
    static INSIDE: Cell<bool> = const { Cell::new(false) };
}

/// A replay made only to read the past again has nothing new to tell.
pub(crate) fn hushed<T>(work: impl FnOnce() -> T) -> T {
    let was = INSIDE.with(|inside| inside.replace(true));
    let out = work();
    INSIDE.with(|inside| inside.set(was));
    out
}

fn held() -> &'static Mutex<Option<Kept>> {
    KEPT.get_or_init(|| Mutex::new(None))
}

pub const ROLLS_AT: u64 = 256 * 1024;

#[cfg(test)]
pub(crate) static ALONE: Mutex<()> = Mutex::new(());

pub fn file(paths: &crate::paths::Paths) -> PathBuf {
    paths.private().join("tisty.log")
}

fn rolled(at: &Path) -> PathBuf {
    at.with_extension("log.1")
}

pub fn kept_files(paths: &crate::paths::Paths) -> Vec<PathBuf> {
    let at = file(paths);
    vec![rolled(&at), at]
}

#[cfg(test)]
pub(crate) fn stops() {
    *held().lock().unwrap_or_else(|e| e.into_inner()) = None;
    ALL.store(false, Ordering::Relaxed);
}

pub fn keeps(at: PathBuf, all: bool) {
    if let Some(parent) = at.parent() {
        let _ = std::fs::create_dir_all(parent);
        let _ = crate::paths::ours_alone(parent);
    }
    ALL.store(all, Ordering::Relaxed);
    *held().lock().unwrap_or_else(|e| e.into_inner()) = Some(Kept { at });
}

pub fn keeping_all() -> bool {
    ALL.load(Ordering::Relaxed)
}

pub fn wants_all() -> bool {
    std::env::var("TISTY_LOG")
        .map(|said| said.eq_ignore_ascii_case("all"))
        .unwrap_or(false)
}

pub fn trace(channel: &'static str, said: &'static str, facts: &[(&'static str, Fact)]) {
    write(Gravity::Trace, channel, said, facts);
}

pub fn note(channel: &'static str, said: &'static str, facts: &[(&'static str, Fact)]) {
    write(Gravity::Note, channel, said, facts);
}

/// A note a round finds again every time, written again only when what it says changes or a while has passed.
pub fn standing(channel: &'static str, said: &'static str, facts: &[(&'static str, Fact)]) {
    written(Gravity::Note, true, channel, said, facts);
}

pub fn warn(channel: &'static str, said: &'static str, facts: &[(&'static str, Fact)]) {
    write(Gravity::Warn, channel, said, facts);
}

pub fn error(channel: &'static str, said: &'static str, facts: &[(&'static str, Fact)]) {
    write(Gravity::Error, channel, said, facts);
}

pub fn fatal(channel: &'static str, said: &'static str, facts: &[(&'static str, Fact)]) {
    write(Gravity::Fatal, channel, said, facts);
}

fn write(
    gravity: Gravity,
    channel: &'static str,
    said: &'static str,
    facts: &[(&'static str, Fact)],
) {
    written(gravity, gravity == Gravity::Warn, channel, said, facts);
}

fn written(
    gravity: Gravity,
    quiet: bool,
    channel: &'static str,
    said: &'static str,
    facts: &[(&'static str, Fact)],
) {
    if gravity < Gravity::Note && !ALL.load(Ordering::Relaxed) {
        return;
    }
    if INSIDE.with(|inside| inside.replace(true)) {
        return;
    }

    let at = {
        let kept = held().lock().unwrap_or_else(|e| e.into_inner());
        kept.as_ref().map(|one| one.at.clone())
    };
    if let Some(at) = at {
        roll(&at);
        let line = lined(jiff::Zoned::now(), gravity, channel, said, facts);
        if quiet && said_lately(&at, &line) {
            INSIDE.with(|inside| inside.set(false));
            return;
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&at)
        {
            let _ = crate::paths::ours_alone(&at);
            let _ = file.write_all(line.as_bytes());
        }
    }
    INSIDE.with(|inside| inside.set(false));
}

const QUIET_FOR: std::time::Duration = std::time::Duration::from_secs(15 * 60);

/// A round that keeps failing the same way would push the line that explains it out of the log.
fn said_lately(at: &Path, line: &str) -> bool {
    use std::hash::{Hash, Hasher};

    static LATELY: OnceLock<Mutex<std::collections::HashMap<u64, std::time::Instant>>> =
        OnceLock::new();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    at.hash(&mut hasher);
    line.split_once("  ")
        .map_or(line, |(_, rest)| rest)
        .hash(&mut hasher);
    let key = hasher.finish();
    let now = std::time::Instant::now();
    let mut seen = LATELY
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if seen
        .get(&key)
        .is_some_and(|then| now.duration_since(*then) < QUIET_FOR)
    {
        return true;
    }
    if seen.len() >= 512 {
        seen.clear();
    }
    seen.insert(key, now);
    false
}

fn lined(
    now: jiff::Zoned,
    gravity: Gravity,
    channel: &'static str,
    said: &'static str,
    facts: &[(&'static str, Fact)],
) -> String {
    let mut line = format!(
        "{}  {:<5}  {:<8}  {}",
        now.strftime("%Y-%m-%d %H:%M:%S%:z"),
        gravity.worded(),
        channel,
        one_line(said),
    );
    for (name, fact) in facts {
        line.push_str(&format!("  {name}={}", fact.shown()));
    }
    line.push('\n');
    line
}

fn roll(at: &Path) {
    let Ok(meta) = std::fs::metadata(at) else {
        return;
    };
    if meta.len() < ROLLS_AT {
        return;
    }

    let Ok(gate) = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(at.with_extension("log.lock"))
    else {
        return;
    };
    if gate.try_lock().is_err() {
        return;
    }
    if std::fs::metadata(at).map(|now| now.len()).unwrap_or(0) >= ROLLS_AT {
        let _ = std::fs::rename(at, rolled(at));
    }
    let _ = gate.unlock();
}

pub fn catches(channel: &'static str) {
    HOOKED.call_once(|| {
        let before = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let at = info
                .location()
                .map(|one| format!("{}:{}", one.file(), one.line()))
                .unwrap_or_default();
            fatal(channel, "panicked", &[("at", Fact::Id(at))]);
            before(info);
        }));
    });
}

pub fn recent(paths: &crate::paths::Paths, most: usize) -> Vec<String> {
    let at = file(paths);
    let older = readable(&rolled(&at));
    let live = readable(&at);
    let mut lines: Vec<String> = older
        .lines()
        .chain(live.lines())
        .map(str::to_owned)
        .collect();
    if lines.len() > most {
        lines.drain(..lines.len() - most);
    }
    lines
}

fn readable(at: &Path) -> String {
    std::fs::read(at)
        .map(|raw| String::from_utf8_lossy(&raw).into_owned())
        .unwrap_or_default()
}

pub fn weighs(paths: &crate::paths::Paths) -> u64 {
    let at = file(paths);
    [at.clone(), rolled(&at)]
        .iter()
        .filter_map(|one| std::fs::metadata(one).ok())
        .map(|meta| meta.len())
        .sum()
}

pub fn forget(paths: &crate::paths::Paths) -> crate::Result<()> {
    let at = file(paths);
    for one in [at.with_extension("log.lock"), rolled(&at), at] {
        match std::fs::remove_file(&one) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(crate::Error::Io(e)),
        }
    }
    Ok(())
}

pub fn hidden(text: &str) -> String {
    match who() {
        Some(name) => without(text, &name),
        None => text.to_string(),
    }
}

/// Letter by letter: lowering a whole text can change its length and cut a letter in half.
fn without(text: &str, who: &str) -> String {
    let needle: Vec<char> = who.chars().collect();
    if needle.len() < 3 {
        return text.to_string();
    }
    let alike =
        |one: char, other: char| one == other || one.to_lowercase().eq(other.to_lowercase());
    let letters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < letters.len() {
        let here = letters[at..].len() >= needle.len()
            && needle
                .iter()
                .zip(&letters[at..])
                .all(|(n, l)| alike(*n, *l));
        match here {
            true => {
                out.push_str("···");
                at += needle.len();
            }
            false => {
                out.push(letters[at]);
                at += 1;
            }
        }
    }
    out
}

fn kept_short(at: &Path) -> String {
    let mut said = std::path::PathBuf::new();
    for part in at.components() {
        let hidden = matches!(
            part.as_os_str().to_str(),
            Some("attachments") | Some("docs")
        );
        said.push(part);
        if hidden {
            said.push("…");
            break;
        }
    }
    said.display().to_string()
}

#[cfg(target_os = "macos")]
fn who() -> Option<String> {
    let name = std::env::var("USER").ok().filter(|one| !one.is_empty());
    name.or_else(|| {
        let home = std::env::var("HOME").ok()?;
        Path::new(&home)
            .file_name()
            .and_then(|one| one.to_str())
            .map(str::to_owned)
    })
}

#[cfg(not(target_os = "macos"))]
fn who() -> Option<String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    Path::new(&home)
        .file_name()
        .and_then(|one| one.to_str())
        .map(str::to_owned)
}

#[cfg(test)]
#[path = "witness_test.rs"]
mod tests;
