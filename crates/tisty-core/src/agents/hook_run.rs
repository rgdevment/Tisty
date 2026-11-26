use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SAID_AT_MOST: usize = 64 * 1024;
const POLL: Duration = Duration::from_millis(20);
pub const BLOCKS: i32 = 2;
pub const EVENT_VAR: &str = "TISTY_HOOK_EVENT";
pub const AGENT_VAR: &str = "TISTY_HOOK_AGENT";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    pub code: Option<i32>,
    pub said: String,
    pub timed_out: bool,
    pub took: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRun {
    NoInterpreter(&'static str),
    Io(String),
}

pub struct Asked<'a> {
    pub name: &'a str,
    pub script: &'a [u8],
    pub scratch: &'a Path,
    pub heard: Vec<u8>,
    pub event: &'a str,
    pub agent: &'a str,
    pub timeout: Duration,
}

impl Ran {
    pub fn blocks(&self) -> bool {
        self.code == Some(BLOCKS) && !self.timed_out
    }
}

pub fn run(asked: Asked) -> Result<Ran, NotRun> {
    let ext = super::extension(asked.name).ok_or(NotRun::NoInterpreter("none"))?;
    let (program, args) = interpreter(ext)?;
    std::fs::create_dir_all(asked.scratch).map_err(|e| NotRun::Io(e.to_string()))?;
    let copy = asked
        .scratch
        .join(format!("{}-{}", ulid::Ulid::generate(), asked.name));
    std::fs::write(&copy, asked.script).map_err(|e| NotRun::Io(e.to_string()))?;
    let ran = started(&program, &args, &copy, &asked);
    let _ = std::fs::remove_file(&copy);
    ran
}

fn started(program: &Path, args: &[String], copy: &Path, asked: &Asked) -> Result<Ran, NotRun> {
    let mut command = Command::new(program);
    command
        .args(args)
        .arg(copy)
        .env(EVENT_VAR, asked.event)
        .env(AGENT_VAR, asked.agent)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    alone(&mut command);
    let began = Instant::now();
    let mut child = command.spawn().map_err(|e| NotRun::Io(e.to_string()))?;
    if let Some(mut into) = child.stdin.take() {
        let heard = asked.heard.clone();
        std::thread::spawn(move || {
            let _ = into.write_all(&heard);
        });
    }
    let said = child.stderr.take().map(|from| {
        std::thread::spawn(move || {
            let mut kept = Vec::new();
            let _ = from.take(SAID_AT_MOST as u64).read_to_end(&mut kept);
            kept
        })
    });
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if began.elapsed() >= asked.timeout => {
                timed_out = true;
                felled(&mut child);
                break child.wait().ok();
            }
            Ok(None) => std::thread::sleep(POLL),
            Err(_) => break None,
        }
    };
    let said = said
        .and_then(|reading| reading.join().ok())
        .unwrap_or_default();
    Ok(Ran {
        code: status.and_then(|it| it.code()),
        said: String::from_utf8_lossy(&said).trim().to_string(),
        timed_out,
        took: began.elapsed(),
    })
}

#[cfg(windows)]
fn alone(command: &mut Command) {
    std::os::windows::process::CommandExt::creation_flags(command, 0x0800_0000);
}

#[cfg(unix)]
fn alone(command: &mut Command) {
    std::os::unix::process::CommandExt::process_group(command, 0);
}

#[cfg(windows)]
fn felled(child: &mut std::process::Child) {
    if let Some(system) = system32() {
        let _ = Command::new(system.join("taskkill.exe"))
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

#[cfg(unix)]
fn felled(child: &mut std::process::Child) {
    let _ = Command::new("/bin/kill")
        .args(["-KILL", &format!("-{}", child.id())])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

pub fn interpreter(ext: &str) -> Result<(PathBuf, Vec<String>), NotRun> {
    let words = |all: &[&str]| all.iter().map(|one| (*one).to_string()).collect();
    match ext {
        "ps1" => powershell().map(|at| {
            let mut args: Vec<String> = words(&["-NoProfile", "-NonInteractive"]);
            if cfg!(windows) {
                args.extend(words(&["-ExecutionPolicy", "Bypass"]));
            }
            args.push("-File".to_string());
            (at, args)
        }),
        "sh" => bash().map(|at| (at, Vec::new())),
        "py" => python(),
        "js" => on_path("node").map(|at| (at, Vec::new())),
        _ => None,
    }
    .ok_or(NotRun::NoInterpreter(ext_word(ext)))
}

fn ext_word(ext: &str) -> &'static str {
    super::EXTENSIONS
        .into_iter()
        .find(|one| *one == ext)
        .unwrap_or("none")
}

#[cfg(windows)]
fn system32() -> Option<PathBuf> {
    let root = std::env::var_os("SystemRoot")?;
    Some(PathBuf::from(root).join("System32"))
}

#[cfg(windows)]
fn powershell() -> Option<PathBuf> {
    let at = system32()?.join("WindowsPowerShell/v1.0/powershell.exe");
    at.is_file().then_some(at)
}

#[cfg(not(windows))]
fn powershell() -> Option<PathBuf> {
    on_path("pwsh")
}

// WSL's bash.exe in System32 runs in another world, so on Windows only Git's own bash is taken.
#[cfg(windows)]
fn bash() -> Option<PathBuf> {
    let beside_git = on_path("git").and_then(|git| {
        let root = git.parent()?.parent()?.to_path_buf();
        Some(root.join("bin").join("bash.exe"))
    });
    let installed = std::env::var_os("ProgramFiles")
        .map(|at| PathBuf::from(at).join("Git").join("bin").join("bash.exe"));
    [beside_git, installed]
        .into_iter()
        .flatten()
        .find(|at| at.is_file())
}

#[cfg(not(windows))]
fn bash() -> Option<PathBuf> {
    on_path("bash")
}

#[cfg(windows)]
fn python() -> Option<(PathBuf, Vec<String>)> {
    if let Some(launcher) = on_path("py") {
        return Some((launcher, vec!["-3".to_string()]));
    }
    on_path("python")
        .filter(|at| !at.to_string_lossy().to_lowercase().contains("windowsapps"))
        .map(|at| (at, Vec::new()))
}

#[cfg(not(windows))]
fn python() -> Option<(PathBuf, Vec<String>)> {
    on_path("python3").map(|at| (at, Vec::new()))
}

pub fn on_path(program: &str) -> Option<PathBuf> {
    let file = match cfg!(windows) {
        true => format!("{program}.exe"),
        false => program.to_string(),
    };
    std::env::split_paths(&std::env::var_os("PATH")?)
        .filter(|dir| dir.is_absolute())
        .map(|dir| dir.join(&file))
        .find(|at| at.is_file())
}

#[cfg(test)]
#[path = "hook_run_test.rs"]
mod tests;
