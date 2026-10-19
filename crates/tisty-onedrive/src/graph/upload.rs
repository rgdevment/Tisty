use std::io::Read;
use std::path::Path;

use reqwest::blocking::{RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::{Expect, Hitch, Remote};

use super::OneDrive;
use super::tree::{DriveItem, ITEM_FIELDS};
use super::wire::{given, json, judged, judged_carried};
use crate::{ASKED_WITHIN, unreachable};

const TRIES: u32 = 3;
const PAUSE: std::time::Duration = std::time::Duration::from_secs(1);

// A file that must not be there yet is refused if it is; any other write takes the place of what is there.
fn conflict(expect: &Expect) -> &'static str {
    match expect {
        Expect::Absent => "fail",
        Expect::Revision(_) | Expect::Any => "replace",
    }
}

fn expecting(asked: RequestBuilder, expect: &Expect) -> RequestBuilder {
    match expect {
        Expect::Revision(etag) => asked.header(reqwest::header::IF_MATCH, etag),
        Expect::Absent | Expect::Any => asked,
    }
}

impl OneDrive {
    // A passing fault would otherwise pause a whole round for one small file.
    pub(super) fn whole(
        &self,
        name: &str,
        from: &Path,
        expect: &Expect,
    ) -> Result<DriveItem, Hitch> {
        retried(|failed| {
            if failed > 0 {
                self.spent(self.limits().costs.put);
                if let Some(landed) = self.landed_already(name, from)? {
                    return Ok(landed);
                }
            }
            self.whole_once(name, from, expect)
        })
    }

    // A write whose answer was lost may have landed; it is taken as this one only when the bytes there are these.
    fn landed_already(&self, name: &str, from: &Path) -> Result<Option<DriveItem>, Hitch> {
        self.spent(1);
        let Some(item) = self.item(name, Some(ITEM_FIELDS))? else {
            return Ok(None);
        };
        // Hashing a large file costs far more than one look at its size, which rules most out first.
        if item.size != Some(std::fs::metadata(from)?.len()) {
            return Ok(None);
        }
        let here = self.hash_of(from)?;
        Ok((item.seen(name).hash == here).then_some(item))
    }

    /// One request that either lands whole or leaves the file as it was: what Tisty's own files always take.
    fn whole_once(&self, name: &str, from: &Path, expect: &Expect) -> Result<DriveItem, Hitch> {
        let body = std::fs::read(from)?;
        let url = self.by_path(name, Some("content"))?;
        let answer = self.sent(Some(given(body.len() as u64)), |http| {
            expecting(
                http.put(&url)
                    .query(&[("@microsoft.graph.conflictBehavior", conflict(expect))])
                    .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
                    .body(body.clone()),
                expect,
            )
        })?;
        json(self.landed(name, answer)?)
    }

    // OneDrive checks the revision when the session opens; the files Tisty writes over are small and go whole.
    pub(super) fn in_parts(
        &self,
        name: &str,
        from: &Path,
        expect: &Expect,
        bytes: u64,
    ) -> Result<DriveItem, Hitch> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Session {
            upload_url: String,
        }
        let url = self.by_path(name, Some("createUploadSession"))?;
        // A session holds the conflict it was opened with, so one opened for another write is never taken.
        let key = tisty_carrier::uploads::key_of(
            &url,
            &format!("{name}\u{0}{}", conflict(expect)),
            from,
            bytes,
        );
        let kept = self.uploads.as_ref().zip(key.as_deref());
        if let Some((uploads, key)) = kept
            && let Some(upload) = uploads.session_for(key)
        {
            let asked = match self.carries(&upload) {
                Ok(()) => self.got_to(&upload),
                Err(refused) => Err(refused),
            };
            match asked {
                Ok(Held::At(at)) => {
                    let sent = self.pieces(name, (&upload, Some(key)), from, bytes, at);
                    self.settled(&upload, key, &sent);
                    return sent;
                }
                Ok(Held::Whole) => return Err(assembling()),
                Err(cut) if passing(&cut) => return Err(cut),
                Ok(Held::Gone) | Err(_) => {
                    self.dropped(&upload);
                    uploads.closed(key);
                    if let Some(landed) = self.landed_already(name, from)? {
                        return Ok(landed);
                    }
                }
            }
        }
        let body =
            serde_json::json!({ "item": { "@microsoft.graph.conflictBehavior": conflict(expect) } })
                .to_string();
        let answer = self.sent(Some(ASKED_WITHIN), |http| {
            expecting(
                http.post(&url)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body.clone()),
                expect,
            )
        })?;
        let session: Session = json(self.landed(name, answer)?)?;
        self.carries(&session.upload_url)?;
        if let Some((uploads, key)) = kept {
            uploads.opened(key, &session.upload_url);
        }
        let sent = self.pieces(
            name,
            (&session.upload_url, kept.map(|(_, key)| key)),
            from,
            bytes,
            0,
        );
        match kept {
            Some((_, key)) => self.settled(&session.upload_url, key, &sent),
            None if sent.is_err() => self.dropped(&session.upload_url),
            None => {}
        }
        sent
    }

    // Only a way that was cut keeps the session for next time; anything OneDrive refused is begun again.
    fn settled(&self, upload: &str, key: &str, sent: &Result<DriveItem, Hitch>) {
        let Some(uploads) = &self.uploads else {
            return;
        };
        match sent {
            Err(cut) if passing(cut) => {}
            Err(_) => {
                self.dropped(upload);
                uploads.closed(key);
            }
            Ok(_) => uploads.closed(key),
        }
    }

    fn dropped(&self, upload: &str) {
        let _ = self.http.delete(upload).timeout(ASKED_WITHIN).send();
    }

    fn got_to(&self, upload: &str) -> Result<Held, Hitch> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Status {
            #[serde(default)]
            next_expected_ranges: Vec<String>,
        }
        let asked = self
            .http
            .get(upload)
            .timeout(ASKED_WITHIN)
            .build()
            .map_err(unreachable)?;
        self.spent(1);
        let answer = self.traced(asked, Some("upload")).map_err(unreachable)?;
        match answer.status().as_u16() {
            // An address that carries its own permission and refuses has lapsed, as one OneDrive forgot has.
            401 | 403 | 404 | 410 => Ok(Held::Gone),
            _ => {
                let status: Status = json(judged_carried(answer)?)?;
                // Every byte is there and OneDrive is still putting the file together.
                let Some(range) = status.next_expected_ranges.first() else {
                    return Ok(Held::Whole);
                };
                Ok(range
                    .split('-')
                    .next()
                    .and_then(|first| first.parse().ok())
                    .map_or(Held::Gone, Held::At))
            }
        }
    }

    fn pieces(
        &self,
        name: &str,
        (upload, key): (&str, Option<&str>),
        from: &Path,
        bytes: u64,
        start: u64,
    ) -> Result<DriveItem, Hitch> {
        let mut at = start;
        let mut gone = false;
        retried(|failed| {
            if gone {
                return Err(lapsed());
            }
            if failed > 0 {
                at = match self.got_to(upload)? {
                    Held::At(at) => at,
                    Held::Whole => return Err(assembling()),
                    // The last piece may have landed with its answer lost: the session is gone and the file is there.
                    Held::Gone => {
                        gone = true;
                        if let (Some(uploads), Some(key)) = (&self.uploads, key) {
                            uploads.closed(key);
                        }
                        return self.landed_already(name, from)?.ok_or_else(lapsed);
                    }
                };
            }
            self.pieces_from(name, upload, from, bytes, at)
        })
    }

    // The session's own address carries its permission: OneDrive refuses a piece that also brings the token.
    fn pieces_from(
        &self,
        name: &str,
        upload: &str,
        from: &Path,
        bytes: u64,
        start: u64,
    ) -> Result<DriveItem, Hitch> {
        use std::io::Seek;
        let mut file = std::fs::File::open(from)?;
        file.seek(std::io::SeekFrom::Start(start))?;
        let mut at = start;
        while at < bytes {
            let size = self.chunk.min(bytes - at);
            let mut piece = vec![0u8; usize::try_from(size).unwrap_or(usize::MAX)];
            file.read_exact(&mut piece)?;
            let asked = self
                .http
                .put(upload)
                .timeout(given(size))
                .header(
                    reqwest::header::CONTENT_RANGE,
                    format!("bytes {at}-{}/{bytes}", at + size - 1),
                )
                .body(piece)
                .build()
                .map_err(unreachable)?;
            let answer = self.traced(asked, Some("upload")).map_err(unreachable)?;
            at += size;
            match answer.status().as_u16() {
                202 if at < bytes => {}
                200 | 201 if at == bytes => return json(answer),
                404 => {
                    return Err(Hitch::Troubled(
                        "OneDrive let the upload lapse before it ended".into(),
                    ));
                }
                409 | 412 => return Err(Hitch::Changed(name.to_string())),
                _ => {
                    return Err(judged_carried(answer).err().unwrap_or_else(|| {
                        Hitch::Broke("OneDrive answered an upload oddly".into())
                    }));
                }
            }
        }
        Err(Hitch::Broke(
            "an upload ended with nothing left to send".into(),
        ))
    }

    // A revision that no longer matches is told apart from a file that left in the meantime.
    fn landed(&self, name: &str, answer: Response) -> Result<Response, Hitch> {
        match answer.status().as_u16() {
            409 => Err(Hitch::Changed(name.to_string())),
            412 => {
                self.spent(1);
                match self.item(name, Some(ITEM_FIELDS))? {
                    Some(_) => Err(Hitch::Changed(name.to_string())),
                    None => Err(Hitch::Missing(name.to_string())),
                }
            }
            _ => judged(answer),
        }
    }
}

enum Held {
    At(u64),
    Whole,
    Gone,
}

fn lapsed() -> Hitch {
    Hitch::Troubled("OneDrive let the upload lapse before it ended".into())
}

fn assembling() -> Hitch {
    Hitch::Troubled("OneDrive is still putting an upload together".into())
}

// A way that is down or a passing fault keeps a session for the next round; anything else begins anew.
fn passing(hitch: &Hitch) -> bool {
    matches!(
        hitch,
        Hitch::Unreachable(..) | Hitch::Limited { .. } | Hitch::Troubled(_)
    )
}

// Only a line cut mid-answer or a passing fault is tried again; a way that never opened will not open in a second.
fn retried<T>(mut attempt: impl FnMut(u32) -> Result<T, Hitch>) -> Result<T, Hitch> {
    let mut failed = 0;
    loop {
        match attempt(failed) {
            Err(Hitch::Troubled(_) | Hitch::Unreachable(None, _)) if failed + 1 < TRIES => {}
            other => return other,
        }
        failed += 1;
        std::thread::sleep(PAUSE * failed);
    }
}
