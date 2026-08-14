use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Gravity {
    Note,
    Warn,
    Error,
    Fatal,
}

impl Gravity {
    fn worded(self) -> &'static str {
        match self {
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

pub fn file(paths: &crate::paths::Paths) -> PathBuf {
    paths.private().join("tisty.log")
}

fn rolled(at: &Path) -> PathBuf {
    at.with_extension("log.1")
}

#[cfg(test)]
fn stops() {
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
    if gravity < Gravity::Warn && !ALL.load(Ordering::Relaxed) {
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
    use fs4::fs_std::FileExt;
    if gate.try_lock_exclusive().is_err() {
        return;
    }
    if std::fs::metadata(at).map(|now| now.len()).unwrap_or(0) >= ROLLS_AT {
        let _ = std::fs::rename(at, rolled(at));
    }
    let _ = FileExt::unlock(&gate);
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
mod tests {
    use super::*;

    fn now() -> jiff::Zoned {
        "2026-08-11T17:04:03-04[America/Santiago]".parse().unwrap()
    }

    #[test]
    fn a_line_carries_the_moment_the_gravity_the_channel_and_the_words() {
        let line = lined(
            now(),
            Gravity::Warn,
            channel::SYNC,
            "folder unreachable",
            &[],
        );

        assert!(line.starts_with("2026-08-11 17:04:03-04"), "{line}");
        assert!(line.contains("WARN"), "{line}");
        assert!(line.contains("sync"), "{line}");
        assert!(line.contains("folder unreachable"), "{line}");
        assert!(line.ends_with('\n'));
    }

    #[test]
    fn facts_are_named_beside_the_words() {
        let line = lined(
            now(),
            Gravity::Error,
            channel::STORE,
            "segment unreadable",
            &[("line", Fact::Count(41)), ("code", Fact::Code("badJson"))],
        );

        assert!(line.contains("line=41"), "{line}");
        assert!(line.contains("code=badJson"), "{line}");
    }

    #[test]
    fn the_account_name_never_reaches_the_file() {
        assert_eq!(
            without(
                r"store C:\Users\rgdevment\tisty, sync G:\rgdevment\copies",
                "rgdevment"
            ),
            r"store C:\Users\···\tisty, sync G:\···\copies"
        );
    }

    #[test]
    fn a_reason_that_spans_lines_is_flattened_into_one() {
        let line = lined(
            now(),
            Gravity::Error,
            channel::CONFIG,
            "config unreadable",
            &[("why", Fact::Why("expected a table\n  at line 3".into()))],
        );

        assert_eq!(line.matches('\n').count(), 1, "{line}");
        assert!(line.contains("expected a table at line 3"), "{line}");
    }

    #[test]
    fn nothing_under_attachments_is_named() {
        let at: PathBuf = ["store", "attachments", "2026-08", "severance-juan.pdf"]
            .iter()
            .collect();

        let said = kept_short(&at);

        assert!(!said.contains("severance"), "{said}");
        assert!(said.contains("attachments"), "{said}");
        assert!(said.ends_with('…'), "{said}");
    }

    #[test]
    fn nothing_under_attachments_is_named_however_it_is_handed_over() {
        let said = Fact::Id("attachments/ab/severance-juan-perez-91f2.pdf".into()).shown();

        assert!(!said.contains("severance"), "{said}");
        assert!(said.contains("attachments"), "{said}");
    }

    #[test]
    fn an_identifier_that_names_nobody_is_still_readable() {
        let said = Fact::Id("01JBQ0000000000000000000AA".into()).shown();

        assert_eq!(said, "01JBQ0000000000000000000AA");
    }

    #[test]
    fn a_path_that_names_nobody_is_left_whole() {
        let at: PathBuf = ["store", "dev_a", "000001.jsonl"].iter().collect();

        assert!(kept_short(&at).ends_with("000001.jsonl"), "{at:?}");
    }

    #[test]
    fn a_one_letter_account_is_left_alone() {
        assert_eq!(without(r"C:\data\attachments", "a"), r"C:\data\attachments");
    }

    #[test]
    fn notes_go_nowhere_until_somewhere_is_named() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
        warn(channel::STORE, "nobody is listening", &[]);
    }

    #[test]
    fn gravity_sorts_the_way_it_reads() {
        assert!(Gravity::Note < Gravity::Warn);
        assert!(Gravity::Warn < Gravity::Error);
        assert!(Gravity::Error < Gravity::Fatal);
    }

    #[test]
    fn the_rolled_file_sits_beside_the_live_one() {
        assert_eq!(
            rolled(&PathBuf::from("/somewhere/tisty.log")),
            PathBuf::from("/somewhere/tisty.log.1")
        );
    }

    static ALONE: Mutex<()> = Mutex::new(());

    #[test]
    fn what_is_written_can_be_read_back_newest_last() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        keeps(file(&paths), false);

        warn(channel::SYNC, "first", &[]);
        error(channel::SYNC, "second", &[]);
        note(channel::SYNC, "quiet by default", &[]);

        let seen = recent(&paths, 10);
        assert_eq!(seen.len(), 2, "{seen:?}");
        assert!(seen[0].contains("first"), "{seen:?}");
        assert!(seen[1].contains("second"), "{seen:?}");
        assert!(weighs(&paths) > 0);

        forget(&paths).unwrap();
        assert!(recent(&paths, 10).is_empty());
        assert_eq!(weighs(&paths), 0);
        stops();
    }

    #[test]
    fn a_panic_leaves_a_line_behind() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        keeps(file(&paths), false);

        let quiet = std::panic::take_hook();
        catches(channel::WINDOW);
        let _ = std::panic::catch_unwind(|| panic!("the sky fell"));
        let _ = std::panic::take_hook();
        std::panic::set_hook(quiet);

        let seen = recent(&paths, 10);
        let said = seen.last().expect("a line");
        assert!(said.contains("FATAL"), "{said}");
        assert!(said.contains("panicked"), "{said}");
        assert!(said.contains("witness.rs:"), "{said}");
        assert!(!said.contains("the sky fell"), "{said}");
        stops();
    }

    #[test]
    fn the_account_name_goes_whatever_case_it_is_written_in() {
        let said = without(r"D:\Dropbox\MARIO\tisty and C:\Users\mario\x", "Mario");

        assert!(!said.to_lowercase().contains("mario"), "{said}");
    }

    #[test]
    fn a_name_too_short_to_replace_safely_is_left_whole() {
        assert_eq!(
            without(r"C:\data\attachments", "ab"),
            r"C:\data\attachments"
        );
    }

    #[test]
    fn a_message_with_a_line_break_still_makes_one_note() {
        let line = lined(now(), Gravity::Warn, channel::STORE, "a\nb", &[]);

        assert_eq!(line.matches('\n').count(), 1, "{line}");
    }

    #[test]
    fn the_moment_is_written_with_its_offset_in_full() {
        let line = lined(now(), Gravity::Warn, channel::STORE, "x", &[]);

        assert!(line.starts_with("2026-08-11 17:04:03-04:00"), "{line}");
    }

    #[test]
    fn a_torn_character_does_not_blank_the_whole_file() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        keeps(file(&paths), false);
        warn(channel::STORE, "before the tear", &[]);
        let mut raw = std::fs::read(file(&paths)).unwrap();
        raw.extend_from_slice(&[0xff, 0xfe, b'\n']);
        std::fs::write(file(&paths), raw).unwrap();

        let seen = recent(&paths, 10);

        assert!(
            seen.iter().any(|one| one.contains("before the tear")),
            "{seen:?}"
        );
        stops();
    }
}
