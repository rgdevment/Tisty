use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::blocking::{Body, Client as Http, Response};
use reqwest::header::{CONTENT_RANGE, CONTENT_TYPE, ETAG, IF_MATCH, IF_NONE_MATCH, RANGE};
use tisty_carrier::net::unreachable;
use tisty_carrier::{Costs, Expect, Hitch, Limits, Remote, Seen, Tokens};

mod multistatus;
mod wire;

use multistatus::{ASKED, Found};
use wire::{given, judged, refusal, strong, transfer_http, verb};

use crate::{ASKED_WITHIN, DavAt, names};

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;
// Without a way to send a file in parts that every server shares, one request carries it whole.
const WHOLE_AT_MOST: u64 = 2 * GIB;

pub struct WebDav {
    tokens: Arc<Tokens>,
    http: Http,
    at: DavAt,
    made: Mutex<BTreeSet<String>>,
}

enum Depth {
    Itself,
    Children,
}

impl WebDav {
    pub fn new(tokens: Arc<Tokens>, at: DavAt) -> Result<Self, Hitch> {
        Ok(Self {
            tokens,
            http: transfer_http()?,
            at,
            made: Mutex::new(BTreeSet::new()),
        })
    }

    /// What a server answered for at an address, or None when nothing is there.
    fn found(&self, at: &url::Url, depth: Depth) -> Result<Option<Vec<Found>>, Hitch> {
        let depth = match depth {
            Depth::Itself => "0",
            Depth::Children => "1",
        };
        let answer = self.sent("read", Some(ASKED_WITHIN), |http| {
            Ok(http
                .request(verb("PROPFIND"), at.clone())
                .header("Depth", depth)
                .header(CONTENT_TYPE, "application/xml; charset=utf-8")
                .body(ASKED))
        })?;
        if answer.status().as_u16() == 404 {
            return Ok(None);
        }
        let body = judged(answer)?.text().map_err(unreachable)?;
        multistatus::read(&body).map(Some)
    }

    fn seen(name: &str, found: &Found) -> Seen {
        Seen {
            name: name.to_string(),
            bytes: found.bytes.unwrap_or(0),
            hash: String::new(),
            revision: found.revision(),
        }
    }

    fn known(&self, folder: &str) -> bool {
        self.made.lock().is_ok_and(|made| made.contains(folder))
    }

    fn knew(&self, folder: &str) {
        if let Ok(mut made) = self.made.lock() {
            made.insert(folder.to_string());
        }
    }

    fn forget_folders(&self) {
        if let Ok(mut made) = self.made.lock() {
            made.clear();
        }
    }

    /// Every folder above a name made from the top down, Tisty's own first; one already there is taken as it is.
    fn folders_for(&self, name: &str) -> Result<(), Hitch> {
        let parts: Vec<&str> = name.split('/').collect();
        let mut folder = String::new();
        for depth in 0..parts.len() {
            if depth > 0 {
                folder = parts[..depth].join("/");
            }
            if self.known(&folder) {
                continue;
            }
            let at = self.url_of(&folder, true)?;
            let answer = self.sent("make", Some(ASKED_WITHIN), |http| {
                Ok(http.request(verb("MKCOL"), at.clone()))
            })?;
            match answer.status().as_u16() {
                // 405 is a folder already there.
                200 | 201 | 204 | 405 => self.knew(&folder),
                status => return Err(refusal(status, None)),
            }
        }
        Ok(())
    }

    /// A refused condition names what is there now: nothing, or another revision.
    fn missing_or_changed(&self, name: &str) -> Hitch {
        match self.about(name) {
            Ok(None) => Hitch::Missing(name.to_string()),
            Ok(Some(_)) => Hitch::Changed(name.to_string()),
            Err(other) => other,
        }
    }

    // The server's word for what landed; only a strong ETag in its answer saves asking again.
    fn landed(&self, name: &str, bytes: u64, answer: &Response) -> Result<Seen, Hitch> {
        if let Some(etag) = header(answer, ETAG).filter(|etag| strong(etag)) {
            return Ok(Seen {
                name: name.to_string(),
                bytes,
                hash: String::new(),
                revision: etag,
            });
        }
        self.about(name)?
            .ok_or_else(|| Hitch::Troubled(format!("«{name}» was written and is not there")))
    }

    fn written(
        &self,
        name: &str,
        from: &Path,
        expect: &Expect,
        bytes: u64,
    ) -> Result<Response, Hitch> {
        let at = self.url_of(name, false)?;
        self.sent("upload", Some(given(bytes)), |http| {
            let file = std::fs::File::open(from)?;
            let asked = http.put(at.clone()).body(Body::sized(file, bytes));
            Ok(match expect {
                Expect::Absent => asked.header(IF_NONE_MATCH, "*"),
                Expect::Revision(revision) => asked.header(IF_MATCH, revision.as_str()),
                Expect::Any => asked,
            })
        })
    }

    fn poured(
        name: &str,
        mut answer: Response,
        mut skipped: u64,
        into: &mut dyn Write,
    ) -> Result<(), Hitch> {
        let mut block = vec![0u8; 64 * 1024];
        loop {
            let read = answer.read(&mut block).map_err(|_| {
                Hitch::Unreachable(None, format!("the line was cut while «{name}» was coming"))
            })?;
            if read == 0 {
                return Ok(());
            }
            let passed = usize::try_from(skipped).unwrap_or(usize::MAX).min(read);
            skipped -= passed as u64;
            into.write_all(&block[passed..read])?;
        }
    }
}

impl Remote for WebDav {
    fn limits(&self) -> Limits {
        Limits {
            provider: "WebDAV",
            units_a_day: tisty_carrier::UNITS_A_DAY,
            bytes_a_day: tisty_carrier::BYTES_A_DAY,
            // No server says what changed, so every look is a whole listing and is asked for less often.
            poll_every: Duration::from_secs(120),
            chunk: WHOLE_AT_MOST,
            page: 1000,
            most_per_file: WHOLE_AT_MOST,
            // Some servers keep their files where case folds; Tisty writes lowercase names anyway.
            folds_case: true,
            costs: Costs {
                list: 1,
                fetch: 1,
                put: 1,
                delete: 1,
                about: 1,
                changes: 1,
                checks: 0,
            },
        }
    }

    /// Folder by folder, since many servers refuse to answer for a whole tree at once.
    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        let mut seen = Vec::new();
        let mut waiting = vec![under.trim_matches('/').to_string()];
        while let Some(folder) = waiting.pop() {
            let Some(all) = self.found(&self.url_of(&folder, true)?, Depth::Children)? else {
                continue;
            };
            self.knew(&folder);
            for one in &all {
                let Some(name) = self.name_of(&one.href) else {
                    continue;
                };
                match (one.collection, name == folder) {
                    (_, true) => {}
                    (true, false) => waiting.push(name),
                    (false, false) => seen.push(Self::seen(&name, one)),
                }
            }
        }
        Ok(seen)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        names::fitting(name)?;
        Ok(self
            .found(&self.url_of(name, false)?, Depth::Itself)?
            .and_then(|all| all.into_iter().next())
            .filter(|one| !one.collection)
            .map(|one| Self::seen(name, &one)))
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        names::fitting(name)?;
        let at = self.url_of(name, false)?;
        let answer = self.sent("download", None, |http| {
            let asked = http.get(at.clone());
            Ok(match from {
                0 => asked,
                from => asked.header(RANGE, format!("bytes={from}-")),
            })
        })?;
        let (skipped, bytes) = match answer.status().as_u16() {
            206 => match range_of(&answer) {
                Some((start, whole)) if start == from => (0, whole),
                _ => {
                    return Err(Hitch::Broke(format!(
                        "the WebDAV server sent another part of «{name}» than the one asked for"
                    )));
                }
            },
            // A server that cannot cut a file sends it whole, and what was already here is passed over.
            200 => (from, answer.content_length()),
            // Asked from its very end: everything is already here.
            416 => {
                return self
                    .about(name)?
                    .filter(|seen| from >= seen.bytes)
                    .ok_or_else(|| Hitch::Changed(name.to_string()));
            }
            404 => return Err(Hitch::Missing(name.to_string())),
            status => {
                return Err(refusal(
                    status,
                    tisty_carrier::net::waited(answer.headers()),
                ));
            }
        };
        let etag = header(&answer, ETAG);
        Self::poured(name, answer, skipped, into)?;
        match (etag, bytes) {
            (Some(revision), Some(bytes)) => Ok(Seen {
                name: name.to_string(),
                bytes,
                hash: String::new(),
                revision,
            }),
            _ => self
                .about(name)?
                .ok_or_else(|| Hitch::Missing(name.to_string())),
        }
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        names::fitting(name)?;
        let bytes = std::fs::metadata(from)?.len();
        if bytes > WHOLE_AT_MOST {
            return Err(Hitch::Broke(format!(
                "«{name}» is larger than a WebDAV server takes in one piece"
            )));
        }
        if let Expect::Revision(revision) = &expect
            && !strong(revision)
        {
            return Err(unsafe_server());
        }
        self.folders_for(name)?;
        let mut tried_again = false;
        loop {
            let answer = self.written(name, from, &expect, bytes)?;
            match answer.status().as_u16() {
                200 | 201 | 204 => return self.landed(name, bytes, &answer),
                // A folder above it went away since it was made: made again once.
                409 if !tried_again => {
                    tried_again = true;
                    self.forget_folders();
                    self.folders_for(name)?;
                }
                412 => return Err(self.missing_or_changed(name)),
                status => {
                    return Err(refusal(
                        status,
                        tisty_carrier::net::waited(answer.headers()),
                    ));
                }
            }
        }
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        names::fitting(name)?;
        if expect.is_some_and(|revision| !strong(revision)) {
            return Err(unsafe_server());
        }
        let at = self.url_of(name, false)?;
        let answer = self.sent("delete", Some(ASKED_WITHIN), |http| {
            let asked = http.delete(at.clone());
            Ok(match expect {
                Some(revision) => asked.header(IF_MATCH, revision),
                None => asked,
            })
        })?;
        match answer.status().as_u16() {
            200 | 204 => Ok(()),
            404 => Err(Hitch::Missing(name.to_string())),
            412 => Err(self.missing_or_changed(name)),
            status => Err(refusal(
                status,
                tisty_carrier::net::waited(answer.headers()),
            )),
        }
    }

    // Moved on the server over whatever bears the name already, as the copy it replaces would have done.
    fn renamed(&self, from: &str, to: &str) -> Result<(), Hitch> {
        names::fitting(from)?;
        names::fitting(to)?;
        self.folders_for(to)?;
        let (at, destination) = (self.url_of(from, false)?, self.url_of(to, false)?);
        let answer = self.sent("move", Some(ASKED_WITHIN), |http| {
            Ok(http
                .request(verb("MOVE"), at.clone())
                .header("Destination", destination.as_str())
                .header("Overwrite", "T"))
        })?;
        match answer.status().as_u16() {
            200 | 201 | 204 => Ok(()),
            404 => Err(Hitch::Missing(from.to_string())),
            // A server that cannot move between those two places is written to and read from instead.
            409 | 502 => tisty_carrier::renamed_by_copy(self, from, to),
            status => Err(refusal(
                status,
                tisty_carrier::net::waited(answer.headers()),
            )),
        }
    }

    fn room(&self) -> Result<Option<u64>, Hitch> {
        let at = self.url_of("", true)?;
        Ok(self
            .found(&at, Depth::Itself)?
            .and_then(|all| all.into_iter().next())
            .and_then(|one| one.room)
            .and_then(|left| u64::try_from(left).ok()))
    }

    // No fingerprint every server shares exists, so none is given here either and size and revision stand in.
    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        std::fs::metadata(local).map(|_| String::new())
    }
}

fn unsafe_server() -> Hitch {
    Hitch::Broke(
        "the WebDAV server keeps no strong ETag, so Tisty cannot write without overwriting another copy".into(),
    )
}

fn header(answer: &Response, name: reqwest::header::HeaderName) -> Option<String> {
    answer
        .headers()
        .get(name)
        .and_then(|one| one.to_str().ok())
        .map(|one| one.trim().to_string())
        .filter(|one| !one.is_empty())
}

// Where the piece sent begins, and how long the whole file is.
fn range_of(answer: &Response) -> Option<(u64, Option<u64>)> {
    let said = header(answer, CONTENT_RANGE)?;
    let (span, whole) = said.strip_prefix("bytes ")?.split_once('/')?;
    let start = span.split_once('-')?.0.trim().parse().ok()?;
    Some((start, whole.trim().parse().ok()))
}

#[cfg(test)]
#[path = "dav_test.rs"]
mod tests;
