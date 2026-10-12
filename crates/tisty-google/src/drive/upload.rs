use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use reqwest::Method;
use tisty_carrier::Hitch;

use super::Drive;
use super::tree::{FILE_FIELDS, File};
use super::wire::{given, json, judged};
use crate::ASKED_WITHIN;
use crate::told;

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
            true => self.sent_whole(&url, method, &meta, from),
            false => self.sent_in_parts(&url, method, &meta, from, bytes),
        }
    }

    // A boundary of its own each time, so no file can ever hold the line that ends it.
    fn sent_whole(
        &self,
        url: &str,
        method: Method,
        meta: &serde_json::Value,
        from: &Path,
    ) -> Result<File, Hitch> {
        let boundary = format!("tisty-{}", crate::random_text(18)?);
        let mut body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{boundary}\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .into_bytes();
        body.extend(std::fs::read(from)?);
        body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());
        let weighs = body.len() as u64;
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

    fn sent_in_parts(
        &self,
        url: &str,
        method: Method,
        meta: &serde_json::Value,
        from: &Path,
        bytes: u64,
    ) -> Result<File, Hitch> {
        let opened = self.call(Some(ASKED_WITHIN), |http| {
            http.request(method.clone(), url)
                .query(&[("uploadType", "resumable"), ("fields", FILE_FIELDS)])
                .header("X-Upload-Content-Length", bytes.to_string())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(meta.to_string())
        })?;
        let session = opened
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|one| one.to_str().ok())
            .ok_or_else(|| Hitch::Broke("Google opened no upload session".into()))?
            .to_string();
        let mut file = std::fs::File::open(from)?;
        let mut at = 0u64;
        let mut failed = 0;
        loop {
            let take = self.chunk.min(bytes - at);
            file.seek(SeekFrom::Start(at))?;
            let mut part = vec![0u8; usize::try_from(take).unwrap_or(usize::MAX)];
            file.read_exact(&mut part)?;
            let sent = self
                .http
                .put(&session)
                .timeout(given(take))
                .header(
                    reqwest::header::CONTENT_RANGE,
                    format!("bytes {at}-{}/{bytes}", at + take - 1),
                )
                .body(part)
                .send();
            // Google may have kept part of a cut chunk, so resending from our own offset would misalign the file.
            let broke = match &sent {
                Ok(answer) => answer.status().is_server_error(),
                Err(_) => true,
            };
            if broke && failed + 1 < TRIES {
                failed += 1;
                match self.resumed(&session, bytes, &mut failed)? {
                    Kept::At(next) => at = next,
                    Kept::Done(file) => return Ok(file),
                }
                if at >= bytes {
                    return Err(Hitch::Broke(
                        "Google kept every byte but never closed the upload".into(),
                    ));
                }
                continue;
            }
            let answer = sent.map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
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
                Err(Hitch::Unreachable(_) | Hitch::Limited { .. }) if *failed + 1 < TRIES => {
                    *failed += 1;
                    std::thread::sleep(PAUSE * *failed);
                }
                Err(other) => return Err(other),
            }
        }
    }

    fn kept_so_far(&self, session: &str, bytes: u64) -> Result<Kept, Hitch> {
        let answer = self
            .http
            .put(session)
            .timeout(ASKED_WITHIN)
            .header(reqwest::header::CONTENT_RANGE, format!("bytes */{bytes}"))
            .send()
            .map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
        match answer.status().as_u16() {
            308 => Ok(Kept::At(stored(&answer))),
            200 | 201 => Ok(Kept::Done(json(answer)?)),
            _ => Err(judged(answer)
                .err()
                .unwrap_or_else(|| Hitch::Broke("the upload session ended".into()))),
        }
    }
}

enum Kept {
    At(u64),
    Done(File),
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
