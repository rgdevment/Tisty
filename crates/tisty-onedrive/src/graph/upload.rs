use std::io::Read;
use std::path::Path;

use reqwest::blocking::{RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::{Expect, Hitch};

use super::OneDrive;
use super::tree::{DriveItem, ITEM_FIELDS};
use super::wire::{given, json, judged};
use crate::{ASKED_WITHIN, unreachable};

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
    /// One request that either lands whole or leaves the file as it was: what Tisty's own files always take.
    pub(super) fn whole(
        &self,
        name: &str,
        from: &Path,
        expect: &Expect,
    ) -> Result<DriveItem, Hitch> {
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
        let sent = self.pieces(name, &session.upload_url, from, bytes);
        if sent.is_err() {
            let _ = self
                .http
                .delete(&session.upload_url)
                .timeout(ASKED_WITHIN)
                .send();
        }
        sent
    }

    // The session's own address carries its permission: OneDrive refuses a piece that also brings the token.
    fn pieces(
        &self,
        name: &str,
        upload: &str,
        from: &Path,
        bytes: u64,
    ) -> Result<DriveItem, Hitch> {
        let mut file = std::fs::File::open(from)?;
        let mut at = 0;
        while at < bytes {
            let size = self.chunk.min(bytes - at);
            let mut piece = vec![0u8; usize::try_from(size).unwrap_or(usize::MAX)];
            file.read_exact(&mut piece)?;
            let answer = self
                .http
                .put(upload)
                .timeout(given(size))
                .header(
                    reqwest::header::CONTENT_RANGE,
                    format!("bytes {at}-{}/{bytes}", at + size - 1),
                )
                .body(piece)
                .send()
                .map_err(unreachable)?;
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
                    return Err(judged(answer).err().unwrap_or_else(|| {
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
            412 => match self.item(name, Some(ITEM_FIELDS))? {
                Some(_) => Err(Hitch::Changed(name.to_string())),
                None => Err(Hitch::Missing(name.to_string())),
            },
            _ => judged(answer),
        }
    }
}
