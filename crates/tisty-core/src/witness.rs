use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once, OnceLock};

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
            Fact::Why(why) => format!("{:?}", hidden(&one_line(why))),
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

fn without(text: &str, who: &str) -> String {
    if who.chars().count() < 3 {
        return text.to_string();
    }
    let hay = text.to_lowercase();
    let needle = who.to_lowercase();
    if !hay.contains(&needle) {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.to_lowercase().find(&needle) {
        out.push_str(&rest[..at]);
        out.push_str("···");
        rest = &rest[at + needle.len()..];
    }
    out.push_str(rest);
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

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
#[path = "witness_test.rs"]
mod tests;
