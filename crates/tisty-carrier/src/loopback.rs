use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::Hitch;

const LISTENED_EVERY: Duration = Duration::from_millis(100);
const HEARD_WITHIN: Duration = Duration::from_secs(2);
const HEARD_AT_MOST: u64 = 16 * 1024;
pub const CONNECTED: &str = "Tisty is connected. You can close this tab.";
pub const NOT_CONNECTED: &str =
    "Tisty could not connect. You can close this tab and try again from Tisty.";

pub fn challenge_of(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn random_text(bytes: usize) -> Result<String, Hitch> {
    use rand_core::TryRngCore;
    let mut fresh = vec![0u8; bytes];
    rand_core::OsRng
        .try_fill_bytes(&mut fresh)
        .map_err(|e| Hitch::Broke(format!("no randomness for the sign-in: {e}")))?;
    Ok(URL_SAFE_NO_PAD.encode(fresh))
}

/// How a sign-in that gave no grant ended, so the window tells waiting too long from saying no.
#[derive(Debug)]
pub enum Ended {
    NoAnswer,
    /// Let go of here: by the person, or by a newer sign-in.
    Dropped,
    Failed(Hitch),
}

impl From<Hitch> for Ended {
    fn from(hitch: Hitch) -> Self {
        Ended::Failed(hitch)
    }
}

impl From<std::io::Error> for Ended {
    fn from(e: std::io::Error) -> Self {
        Ended::Failed(e.into())
    }
}

/// The port on this machine the browser comes back to, with the state only this sign-in knows.
pub struct Loopback {
    pub state: String,
    pub port: u16,
    who: &'static str,
    listener: TcpListener,
    dropped: Arc<AtomicBool>,
}

impl Loopback {
    pub fn open(who: &'static str) -> Result<Self, Hitch> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            state: random_text(24)?,
            port,
            who,
            listener,
            dropped: Arc::default(),
        })
    }

    /// Set from anywhere, it ends the wait for the browser at once.
    pub fn dropping(&self) -> Arc<AtomicBool> {
        self.dropped.clone()
    }

    pub fn answered(&self, within: Duration) -> Result<(String, TcpStream), Ended> {
        self.listener.set_nonblocking(true)?;
        let until = Instant::now().checked_add(within);
        loop {
            if self.dropped.load(Ordering::Relaxed) {
                return Err(Ended::Dropped);
            }
            match self.listener.accept() {
                Ok((stream, _)) => match self.heard(stream) {
                    Heard::Code(code, browser) => return Ok((code, browser)),
                    Heard::Refused(hitch) => return Err(hitch.into()),
                    Heard::Elsewhere => {}
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if until.is_some_and(|until| Instant::now() >= until) {
                        return Err(Ended::NoAnswer);
                    }
                    std::thread::sleep(LISTENED_EVERY);
                }
                Err(e) if passing(e.kind()) => {}
                Err(e) => return Err(e.into()),
            }
        }
    }

    // Only our own state ends the wait: a stale tab, a probe or an idle socket may reach the port first.
    fn heard(&self, mut stream: TcpStream) -> Heard {
        let Some(query) = asked_for(&stream) else {
            answer(&mut stream, "404 Not Found", "");
            return Heard::Elsewhere;
        };
        let said: std::collections::HashMap<String, String> =
            url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();
        if said.get("state") != Some(&self.state) {
            answer(&mut stream, "404 Not Found", "");
            return Heard::Elsewhere;
        }
        let refused = match (said.get("code"), said.get("error")) {
            (Some(code), None) => return Heard::Code(code.clone(), stream),
            (_, Some(error)) if error == "access_denied" => Hitch::Lost,
            (_, Some(error)) => Hitch::Broke(format!("{} refused the sign-in: {error}", self.who)),
            (None, None) => Hitch::Broke("the sign-in came back without a code".into()),
        };
        answer(&mut stream, "200 OK", NOT_CONNECTED);
        Heard::Refused(refused)
    }
}

/// The browser is told how it ended only once the code is traded and the grant is known good.
pub fn tell_browser(browser: &mut TcpStream, connected: bool) {
    answer(
        browser,
        "200 OK",
        if connected { CONNECTED } else { NOT_CONNECTED },
    );
}

enum Heard {
    Code(String, TcpStream),
    Refused(Hitch),
    Elsewhere,
}

// A socket someone dropped before it was taken is theirs to lose, not the sign-in's.
fn passing(kind: std::io::ErrorKind) -> bool {
    matches!(
        kind,
        std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::Interrupted
    )
}

fn asked_for(stream: &TcpStream) -> Option<String> {
    stream.set_nonblocking(false).ok()?;
    let mut patience = |left: Duration| stream.set_read_timeout(Some(left)).is_ok();
    query_of(stream, Instant::now, &mut patience)
}

// The headers are read through, or closing with them unread resets the browser's connection.
pub(crate) fn query_of(
    request: impl Read,
    mut now: impl FnMut() -> Instant,
    patience: &mut impl FnMut(Duration) -> bool,
) -> Option<String> {
    let until = now() + HEARD_WITHIN;
    let mut reader = BufReader::new(request.take(HEARD_AT_MOST));
    let mut first = String::new();
    if !patience(HEARD_WITHIN) {
        return None;
    }
    reader.read_line(&mut first).ok()?;
    let mut line = String::new();
    while let Some(left) = until
        .checked_duration_since(now())
        .filter(|left| !left.is_zero())
    {
        if !patience(left) {
            break;
        }
        match reader.read_line(&mut line) {
            Ok(read) if read > 2 => line.clear(),
            _ => break,
        }
    }
    let target = first.split_whitespace().nth(1)?;
    target.strip_prefix("/?").map(str::to_owned)
}

fn answer(stream: &mut TcpStream, status: &str, page: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}",
        page.len()
    );
    let _ = stream.shutdown(Shutdown::Write);
}

#[cfg(test)]
#[path = "loopback_test.rs"]
mod tests;
