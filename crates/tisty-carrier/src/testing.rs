use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;

use crate::{Hitch, Renew, Renewed, Secrets};

/// One request as a test server heard it: its first line, its whole head and its body.
#[derive(Debug, Default)]
pub struct Asked {
    pub line: String,
    pub head: String,
    pub body: Vec<u8>,
}

impl Asked {
    /// A header's value, its name matched whatever its case.
    pub fn header(&self, name: &str) -> Option<String> {
        let wanted = format!("{}:", name.to_ascii_lowercase());
        self.head
            .lines()
            .find(|one| one.to_ascii_lowercase().starts_with(&wanted))
            .and_then(|one| one.split_once(':'))
            .map(|(_, value)| value.trim().to_string())
    }
}

/// Reads a request through its body, as its `Content-Length` says; one cut short comes back as far as it got.
pub fn heard(stream: &mut TcpStream) -> Asked {
    let mut got = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let read = stream.read(&mut chunk).unwrap_or(0);
        got.extend_from_slice(&chunk[..read]);
        if let Some(end) = got.windows(4).position(|one| one == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&got[..end]).to_string();
            let wants = head
                .lines()
                .find_map(|one| {
                    one.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|n| n.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if got.len() >= end + 4 + wants || read == 0 {
                let line = head.lines().next().unwrap_or_default().to_string();
                let body = got[end + 4..got.len().min(end + 4 + wants)].to_vec();
                return Asked { line, head, body };
            }
        }
        if read == 0 {
            return Asked {
                line: String::from_utf8_lossy(&got)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                head: String::from_utf8_lossy(&got).to_string(),
                body: Vec::new(),
            };
        }
    }
}

/// A keychain held in memory.
pub struct Kept(pub Mutex<HashMap<String, String>>);

impl Kept {
    pub fn holding(key: &str, value: &str) -> Self {
        Self(Mutex::new(HashMap::from([(
            key.to_string(),
            value.to_string(),
        )])))
    }

    fn held(&self) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Secrets for Kept {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch> {
        Ok(self.held().get(key).cloned())
    }
    fn write(&self, key: &str, value: &str) -> Result<(), Hitch> {
        self.held().insert(key.into(), value.into());
        Ok(())
    }
    fn forget(&self, key: &str) -> Result<(), Hitch> {
        self.held().remove(key);
        Ok(())
    }
}

/// Hands out `access-1`, `access-2`… lasting an hour, and never a new refresh token.
#[derive(Default)]
pub struct Fresh(pub Mutex<u32>);

impl Renew for Fresh {
    fn renew(&self, _: &str) -> Result<Renewed, Hitch> {
        let mut given = self.0.lock().unwrap_or_else(|e| e.into_inner());
        *given += 1;
        Ok(Renewed {
            access: format!("access-{given}"),
            lasts: Duration::from_secs(3600),
            refresh: None,
        })
    }
}
