use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use tisty_core::witness::{self, Fact};

const GREETED_WITHIN: Duration = Duration::from_secs(10);

type Mark = (u64, SystemTime);

/// The binary this door was started from, as it was then, so an update that lays a new one in
/// its place is noticed and the client is carried over instead of cut off.
pub(super) struct Born {
    at: Option<PathBuf>,
    was: Option<Mark>,
    failed: Option<Mark>,
}

impl Born {
    pub(super) fn now() -> Self {
        let at = std::env::current_exe().ok();
        let was = at.as_deref().and_then(mark);
        Self {
            at,
            was,
            failed: None,
        }
    }

    fn replaced(&self) -> Option<(&Path, Mark)> {
        let at = self.at.as_deref()?;
        let now = mark(at)?;
        (self.was.is_some() && Some(now) != self.was && Some(now) != self.failed)
            .then_some((at, now))
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
    let mut child = Command::new(at)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .ok()?;
    let (mut into, out) = (child.stdin.take()?, child.stdout.take()?);
    let Some(greeting) = greeting else {
        return Some((child, into, BufReader::new(out)));
    };
    let (tell, heard) = mpsc::channel();
    std::thread::spawn(move || {
        let mut from = BufReader::new(out);
        let mut first = String::new();
        let read = from.read_line(&mut first).is_ok_and(|n| n > 0);
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
    serde_json::from_str::<serde_json::Value>(line)
        .is_ok_and(|asked| asked.get("method").and_then(|m| m.as_str()) == Some("initialize"))
}
