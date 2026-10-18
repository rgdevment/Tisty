use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use reqwest::Method;
use tisty_carrier::Hitch;

use super::Drive;
use super::sessions::key_of;
use super::tree::{FILE_FIELDS, File};
use super::wire::{given, json, judged, traced};
use crate::ASKED_WITHIN;

// Google takes a multipart upload up to 5 MB; anything larger goes through a session.
const WHOLE_AT_MOST: u64 = 5 * 1000 * 1000;
const TRIES: u32 = 3;
const PAUSE: std::time::Duration = std::time::Duration::from_secs(1);

impl Drive {
    pub(super) fn sent(
        &self,
        name: &str,
        from: &Path,
        file: Option<&File>,
        parent: &str,
        carried: &mut u64,
    ) -> Result<File, Hitch> {
        let bytes = std::fs::metadata(from)?.len();
        let leaf = name.rsplit('/').next().unwrap_or(name);
        let meta = match file {
            Some(_) => serde_json::json!({}),
            None => serde_json::json!({ "name": leaf, "parents": [parent] }),
        };
        let (url, method) = match file {
            Some(one) => (
                format!("{}/files/{}", self.at.upload, one.id),
                Method::PATCH,
            ),
            None => (format!("{}/files", self.at.upload), Method::POST),
        };
        match bytes <= WHOLE_AT_MOST.min(self.chunk) {
            true => self.sent_whole_again(name, (&url, method), &meta, from, carried),
            false => self.sent_in_parts(&url, method, &meta, (name, from, bytes), carried),
        }
    }

    // A passing fault would otherwise pause a whole move for one small file.
    fn sent_whole_again(
        &self,
        name: &str,
        (url, method): (&str, Method),
        meta: &serde_json::Value,
        from: &Path,
        carried: &mut u64,
    ) -> Result<File, Hitch> {
        let mut failed = 0;
        loop {
            match self.sent_whole(url, method.clone(), meta, from, carried) {
                Err(Hitch::Troubled(_) | Hitch::Unreachable(..)) if failed + 1 < TRIES => {}
                other => return other,
            }
            failed += 1;
            std::thread::sleep(PAUSE * failed);
            // A create that failed on its way back may have landed, and sending it again would leave a twin.
            if method == Method::POST
                && let Ok(found) = self.looked_up(name)
                && let Some(landed) = found.into_iter().find(|one| sent_from(one, from))
            {
                return Ok(landed);
            }
        }
    }

    // A boundary of its own each time, so no file can ever hold the line that ends it.
    fn sent_whole(
        &self,
        url: &str,
        method: Method,
        meta: &serde_json::Value,
        from: &Path,
        carried: &mut u64,
    ) -> Result<File, Hitch> {
        let boundary = format!("tisty-{}", tisty_carrier::loopback::random_text(18)?);
        let mut body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{boundary}\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .into_bytes();
        body.extend(std::fs::read(from)?);
        body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());
        let weighs = body.len() as u64;
        *carried += 1;
        json(self.call(Some(given(weighs)), |http| {
            http.request(method.clone(), url)
                .query(&[("uploadType", "multipart"), ("fields", FILE_FIELDS)])
                .header(
                    reqwest::header::CONTENT_TYPE,
                    format!("multipart/related; boundary={boundary}"),
                )
                .body(body.clone())
        })?)
    }

    // The session is written down, so a long upload cut short goes on from where it was next round.
    fn sent_in_parts(
        &self,
        url: &str,
        method: Method,
        meta: &serde_json::Value,
        (name, from, bytes): (&str, &Path, u64),
        carried: &mut u64,
    ) -> Result<File, Hitch> {
        let key = key_of(url, &format!("{name}\u{0}{meta}"), from, bytes);
        let (session, at) = match self.reopened(key.as_deref(), bytes)? {
            Reopened::At(session, at) => (session, at),
            Reopened::Done(file) => return Ok(*file),
            Reopened::Fresh => {
                let session = self.opened(url, method, meta, bytes)?;
                if let Some(key) = &key {
                    self.session_opened(key, &session);
                }
                (session, 0)
            }
        };
        let done = self.sent_from(&session, from, (bytes, at), carried);
        if let (Ok(_), Some(key)) = (&done, &key) {
            self.session_closed(key);
        }
        done
    }

    fn opened(
        &self,
        url: &str,
        method: Method,
        meta: &serde_json::Value,
        bytes: u64,
    ) -> Result<String, Hitch> {
        let opened = self.call(Some(ASKED_WITHIN), |http| {
            http.request(method.clone(), url)
                .query(&[("uploadType", "resumable"), ("fields", FILE_FIELDS)])
                .header("X-Upload-Content-Length", bytes.to_string())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(meta.to_string())
        })?;
        Ok(opened
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|one| one.to_str().ok())
            .ok_or_else(|| Hitch::Broke("Google opened no upload session".into()))?
            .to_string())
    }

    fn reopened(&self, key: Option<&str>, bytes: u64) -> Result<Reopened, Hitch> {
        let Some((key, session)) = key.and_then(|key| Some((key, self.session_for(key)?))) else {
            return Ok(Reopened::Fresh);
        };
        match self.kept_so_far(&session, bytes) {
            Ok(Kept::At(at)) if at < bytes => Ok(Reopened::At(session, at)),
            Ok(Kept::Done(file)) => {
                self.session_closed(key);
                Ok(Reopened::Done(file))
            }
            Err(waits @ (Hitch::Unreachable(..) | Hitch::Troubled(_) | Hitch::Limited { .. })) => {
                Err(waits)
            }
            _ => {
                self.session_closed(key);
                Ok(Reopened::Fresh)
            }
        }
    }

    fn sent_from(
        &self,
        session: &str,
        from: &Path,
        (bytes, mut at): (u64, u64),
        carried: &mut u64,
    ) -> Result<File, Hitch> {
        let mut file = std::fs::File::open(from)?;
        let mut failed = 0;
        loop {
            let take = self.chunk.min(bytes - at);
            file.seek(SeekFrom::Start(at))?;
            let mut part = vec![0u8; usize::try_from(take).unwrap_or(usize::MAX)];
            file.read_exact(&mut part)?;
            let began = std::time::Instant::now();
            *carried += 1;
            let sent = self
                .http
                .put(session)
                .timeout(given(take))
                .header(
                    reqwest::header::CONTENT_RANGE,
                    format!("bytes {at}-{}/{bytes}", at + take - 1),
                )
                .body(part)
                .send();
            traced(
                &Method::PUT,
                "upload-chunk",
                sent.as_ref().ok(),
                began,
                take,
            );
            // Google may have kept part of a cut chunk, so resending from our own offset would misalign the file.
            let broke = match &sent {
                Ok(answer) => answer.status().is_server_error(),
                Err(_) => true,
            };
            if broke && failed + 1 < TRIES {
                failed += 1;
                match self.resumed(session, bytes, &mut failed)? {
                    Kept::At(next) => at = next,
                    Kept::Done(file) => return Ok(*file),
                    Kept::Lost => {
                        return Err(Hitch::Troubled(
                            "Google let go of the upload session, so it starts over".into(),
                        ));
                    }
                }
                if at >= bytes {
                    return Err(Hitch::Broke(
                        "Google kept every byte but never closed the upload".into(),
                    ));
                }
                continue;
            }
            let answer = sent.map_err(crate::health::unreachable)?;
            match answer.status().as_u16() {
                200 | 201 => return json(answer),
                308 => {
                    at = stored(&answer);
                    failed = 0;
                }
                _ => {
                    return Err(judged(answer)
                        .err()
                        .unwrap_or_else(|| Hitch::Broke("the upload ended early".into())));
                }
            }
            if at >= bytes {
                return Err(Hitch::Broke(
                    "Google kept every byte but never closed the upload".into(),
                ));
            }
        }
    }

    // The question itself can be cut too; it shares the same budget of failures as the chunks.
    fn resumed(&self, session: &str, bytes: u64, failed: &mut u32) -> Result<Kept, Hitch> {
        loop {
            match self.kept_so_far(session, bytes) {
                Ok(kept) => return Ok(kept),
                Err(Hitch::Unreachable(..) | Hitch::Troubled(_) | Hitch::Limited { .. })
                    if *failed + 1 < TRIES =>
                {
                    *failed += 1;
                    std::thread::sleep(PAUSE * *failed);
                }
                Err(other) => return Err(other),
            }
        }
    }

    fn kept_so_far(&self, session: &str, bytes: u64) -> Result<Kept, Hitch> {
        let began = std::time::Instant::now();
        let answer = self
            .http
            .put(session)
            .timeout(ASKED_WITHIN)
            .header(reqwest::header::CONTENT_RANGE, format!("bytes */{bytes}"))
            .send();
        traced(
            &Method::PUT,
            "upload-status",
            answer.as_ref().ok(),
            began,
            0,
        );
        let answer = answer.map_err(crate::health::unreachable)?;
        match answer.status().as_u16() {
            308 => Ok(Kept::At(stored(&answer))),
            200 | 201 => Ok(Kept::Done(Box::new(json(answer)?))),
            // Google's word for a session it let go of: the upload starts over, the file is not missing.
            404 | 410 => Ok(Kept::Lost),
            _ => Err(judged(answer)
                .err()
                .unwrap_or_else(|| Hitch::Broke("the upload session ended".into()))),
        }
    }
}

enum Kept {
    At(u64),
    Done(Box<File>),
    Lost,
}

enum Reopened {
    At(String, u64),
    Done(Box<File>),
    Fresh,
}

fn stored(answer: &reqwest::blocking::Response) -> u64 {
    answer
        .headers()
        .get(reqwest::header::RANGE)
        .and_then(|one| one.to_str().ok())
        .and_then(|one| one.rsplit('-').next())
        .and_then(|last| last.trim().parse::<u64>().ok())
        .map_or(0, |last| last + 1)
}

fn sent_from(file: &File, from: &Path) -> bool {
    let Ok((hex, bytes)) = tisty_core::attach::hashed(from) else {
        return false;
    };
    match file.sha256_checksum.as_deref() {
        Some(theirs) => theirs == hex,
        None => file.size.as_deref().and_then(|one| one.parse().ok()) == Some(bytes),
    }
}
