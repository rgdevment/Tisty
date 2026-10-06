use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use tisty_core::witness::{self, Fact};

const GREETED_WITHIN: Duration = Duration::from_secs(10);
const PROBE: &str = r#"{"jsonrpc":"2.0","id":"tisty-handover","method":"ping"}"#;

type Mark = (u64, SystemTime);

/// The binary this door started from, as it was then, so one laid in its place is noticed.
pub(super) struct Born {
    from: Option<(PathBuf, Mark)>,
    failed: Option<Mark>,
}

impl Born {
    pub(super) fn now() -> Self {
        let from = std::env::current_exe()
            .ok()
            .and_then(|at| mark(&at).map(|was| (at, was)));
        Self { from, failed: None }
    }

    fn replaced(&self) -> Option<(&Path, Mark)> {
        let (at, was) = self.from.as_ref()?;
        let now = mark(at)?;
        (now != *was && Some(now) != self.failed).then_some((at.as_path(), now))
    }

    pub(super) fn hand_over(
        &mut self,
        greeting: Option<&str>,
        line: &str,
        stdin: &mut dyn BufRead,
    ) -> Option<anyhow::Result<ExitCode>> {
        let (at, now) = self.replaced()?;
        match greeted(at, greeting) {
            Some((child, into, from)) => {
                witness::note(
                    witness::channel::AGENT,
                    "a newer Tisty was laid in place, so the door carries on through it",
                    &[],
                );
                Some(relay(child, into, from, line, stdin))
            }
            None => {
                witness::warn(
                    witness::channel::AGENT,
                    "a newer Tisty was laid in place but would not take the door, so this one serves on",
                    &[("at", Fact::Path(at.to_path_buf()))],
                );
                self.failed = Some(now);
                None
            }
        }
    }
}

fn mark(at: &Path) -> Option<Mark> {
    let told = std::fs::metadata(at).ok()?;
    Some((told.len(), told.modified().ok()?))
}

// The client greeted the old door only, so the new one is greeted the same way and its answer dropped.
fn greeted(
    at: &Path,
    greeting: Option<&str>,
) -> Option<(Child, ChildStdin, BufReader<ChildStdout>)> {
    let mut command = Command::new(at);
    command
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0800_0000);
    let mut child = command.spawn().ok()?;
    let (mut into, out) = (child.stdin.take()?, child.stdout.take()?);
    let greeting = greeting.unwrap_or(PROBE);
    let (tell, heard) = mpsc::channel();
    let asked = greeting.to_string();
    std::thread::spawn(move || {
        let mut from = BufReader::new(out);
        let mut first = String::new();
        let read = from.read_line(&mut first).is_ok() && answers(&asked, &first);
        let _ = tell.send(read.then_some(from));
    });
    let sent = writeln!(into, "{}", greeting.trim_end()).and_then(|_| into.flush());
    match (sent, heard.recv_timeout(GREETED_WITHIN)) {
        (Ok(()), Ok(Some(from))) => Some((child, into, from)),
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            None
        }
    }
}

fn relay(
    mut child: Child,
    mut into: ChildStdin,
    mut from: BufReader<ChildStdout>,
    line: &str,
    stdin: &mut dyn BufRead,
) -> anyhow::Result<ExitCode> {
    let back = std::thread::spawn(move || -> std::io::Result<()> {
        let mut raw = Vec::new();
        while from.read_until(b'\n', &mut raw)? > 0 {
            let mut out = std::io::stdout().lock();
            out.write_all(&raw)?;
            out.flush()?;
            raw.clear();
        }
        Ok(())
    });
    writeln!(into, "{}", line.trim_end())?;
    into.flush()?;
    let mut raw = Vec::new();
    while stdin.read_until(b'\n', &mut raw)? > 0 {
        into.write_all(&raw)?;
        into.flush()?;
        raw.clear();
    }
    drop(into);
    let _ = back.join();
    child.wait()?;
    Ok(ExitCode::SUCCESS)
}

pub(super) fn greets(line: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(line).is_ok_and(|asked| {
        matches!(
            asked.get("method").and_then(|m| m.as_str()),
            Some("initialize" | "server/discover")
        )
    })
}

fn answers(asked: &str, said: &str) -> bool {
    let (Ok(asked), Ok(said)) = (
        serde_json::from_str::<serde_json::Value>(asked),
        serde_json::from_str::<serde_json::Value>(said),
    ) else {
        return false;
    };
    said.get("id") == asked.get("id")
}
