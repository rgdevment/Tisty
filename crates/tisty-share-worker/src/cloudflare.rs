//! What only Workers has: D1 and R2 behind the server's `Store`, its randomness, and the events it calls.

use std::net::IpAddr;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tisty_share_server::keys::Randomness;
use tisty_share_server::moderation::Report;
use tisty_share_server::share::{Policy, Share};
use tisty_share_server::store::{Doc, Fault, Kept, Saved, Store};
use worker::js_sys::{self, Function, Reflect, Uint8Array};
use worker::wasm_bindgen::{JsCast, JsValue};
use worker::{
    Bucket, Context, D1Database, D1PreparedStatement, Date, Env, Headers, Request, Response,
    ScheduleContext, ScheduledEvent, event,
};

use crate::{api, sql};

struct Cloud {
    db: D1Database,
    envelopes: Bucket,
}

fn fault(error: impl std::fmt::Display) -> Fault {
    Fault(error.to_string())
}

/// D1 takes numbers as JavaScript does, which holds every count and second the server keeps.
fn number(value: i64) -> JsValue {
    JsValue::from_f64(value as f64)
}

fn bound(statement: D1PreparedStatement, values: &[JsValue]) -> Result<D1PreparedStatement, Fault> {
    statement.bind(values).map_err(fault)
}

fn envelope_key(id: &str, kept: &Kept) -> String {
    format!(
        "{id}/{}.{}",
        kept.version,
        URL_SAFE_NO_PAD.encode(kept.digest)
    )
}

#[derive(Deserialize)]
struct Body {
    body: String,
}

fn parsed<T: DeserializeOwned>(text: &str) -> Result<T, Fault> {
    serde_json::from_str(text).map_err(fault)
}

impl Cloud {
    async fn first<T: DeserializeOwned>(
        &self,
        query: &str,
        values: &[JsValue],
    ) -> Result<Option<T>, Fault> {
        bound(self.db.prepare(query), values)?
            .first(None)
            .await
            .map_err(fault)
    }

    async fn all<T: DeserializeOwned>(
        &self,
        query: &str,
        values: &[JsValue],
    ) -> Result<Vec<T>, Fault> {
        let done = bound(self.db.prepare(query), values)?
            .all()
            .await
            .map_err(fault)?;
        done.results().map_err(fault)
    }

    async fn changes(&self, query: &str, values: &[JsValue]) -> Result<usize, Fault> {
        let done = bound(self.db.prepare(query), values)?
            .run()
            .await
            .map_err(fault)?;
        Ok(done
            .meta()
            .map_err(fault)?
            .and_then(|meta| meta.changes)
            .unwrap_or(0))
    }
}

impl Store for Cloud {
    async fn doc(&self, id: &str) -> Result<Option<Doc>, Fault> {
        let held: Option<Body> = self.first(sql::DOC, &[id.into()]).await?;
        held.map(|held| parsed(&held.body)).transpose()
    }

    async fn save(&self, doc: &Doc, revision: Option<u64>) -> Result<Saved, Fault> {
        let body = serde_json::to_string(doc).map_err(fault)?;
        let mut values = vec![
            doc.id.as_str().into(),
            number(doc.revision as i64),
            number(doc.ends),
            body.into(),
        ];
        let query = match revision {
            None => sql::INSERT_DOC,
            Some(was) => {
                values.push(number(was as i64));
                sql::UPDATE_DOC
            }
        };
        Ok(match self.changes(query, &values).await? {
            0 => Saved::Raced,
            _ => Saved::Done,
        })
    }

    async fn forget(&self, id: &str) -> Result<(), Fault> {
        self.changes(sql::FORGET, &[id.into()]).await?;
        let listed = self
            .envelopes
            .list()
            .prefix(format!("{id}/"))
            .execute()
            .await
            .map_err(fault)?;
        let keys: Vec<String> = listed.objects().iter().map(|object| object.key()).collect();
        if !keys.is_empty() {
            self.envelopes.delete_multiple(keys).await.map_err(fault)?;
        }
        Ok(())
    }

    async fn envelope(&self, id: &str, kept: &Kept) -> Result<Option<Vec<u8>>, Fault> {
        let Some(object) = self
            .envelopes
            .get(envelope_key(id, kept))
            .execute()
            .await
            .map_err(fault)?
        else {
            return Ok(None);
        };
        let Some(body) = object.body() else {
            return Ok(None);
        };
        Ok(Some(body.bytes().await.map_err(fault)?))
    }

    async fn keep(&self, id: &str, kept: &Kept, bytes: &[u8]) -> Result<(), Fault> {
        self.envelopes
            .put(envelope_key(id, kept), bytes.to_vec())
            .execute()
            .await
            .map_err(fault)?;
        Ok(())
    }

    async fn drop_envelope(&self, id: &str, kept: &Kept) -> Result<(), Fault> {
        self.envelopes
            .delete(envelope_key(id, kept))
            .await
            .map_err(fault)
    }

    async fn ending(&self, now: i64) -> Result<Vec<String>, Fault> {
        #[derive(Deserialize)]
        struct Id {
            id: String,
        }
        let ids: Vec<Id> = self.all(sql::ENDING, &[number(now)]).await?;
        Ok(ids.into_iter().map(|one| one.id).collect())
    }

    async fn count(&self, key: &str, by: u64, until: i64, now: i64) -> Result<u64, Fault> {
        #[derive(Deserialize)]
        struct Value {
            value: u64,
        }
        let values = [key.into(), number(by as i64), number(until), number(now)];
        let counted: Option<Value> = self.first(sql::COUNT, &values).await?;
        counted
            .map(|counted| counted.value)
            .ok_or_else(|| Fault("a counter came back empty".into()))
    }

    async fn hold(&self, key: &str, until: i64) -> Result<(), Fault> {
        self.changes(sql::HOLD, &[key.into(), number(until)])
            .await?;
        Ok(())
    }

    async fn held(&self, key: &str, now: i64) -> Result<Option<i64>, Fault> {
        #[derive(Deserialize)]
        struct Until {
            until: i64,
        }
        let held: Option<Until> = self.first(sql::HELD, &[key.into(), number(now)]).await?;
        Ok(held.map(|held| held.until))
    }

    async fn file(&self, report: &Report) -> Result<(), Fault> {
        let body = serde_json::to_string(report).map_err(fault)?;
        let values = [report.id.as_str().into(), number(report.until), body.into()];
        self.changes(sql::FILE, &values).await?;
        Ok(())
    }

    async fn reports(&self, id: &str, now: i64) -> Result<Vec<Report>, Fault> {
        let held: Vec<Body> = self.all(sql::REPORTS, &[id.into(), number(now)]).await?;
        held.iter().map(|held| parsed(&held.body)).collect()
    }

    async fn lapse(&self, now: i64) -> Result<(), Fault> {
        let statements = sql::LAPSE
            .iter()
            .map(|query| bound(self.db.prepare(*query), &[number(now)]))
            .collect::<Result<Vec<_>, Fault>>()?;
        self.db.batch(statements).await.map_err(fault)?;
        Ok(())
    }
}

/// The runtime's `crypto.getRandomValues`, reached without code of our own that is unsafe.
struct Crypto;

impl Randomness for Crypto {
    fn fill(&mut self, into: &mut [u8]) -> bool {
        drawn(into).is_ok()
    }
}

fn drawn(into: &mut [u8]) -> Result<(), JsValue> {
    let crypto = Reflect::get(&js_sys::global(), &"crypto".into())?;
    let draw: Function = Reflect::get(&crypto, &"getRandomValues".into())?.dyn_into()?;
    let array = Uint8Array::new_with_length(into.len() as u32);
    draw.call1(&crypto, &array)?;
    array.copy_to(into);
    Ok(())
}

/// The server's secret, kept among the Worker's secrets as 64 hexadecimal digits.
fn secret(env: &Env) -> worker::Result<[u8; 32]> {
    let said = env.secret("SHARE_SECRET")?.to_string();
    let mut secret = [0u8; 32];
    if said.len() != 64 {
        return Err("SHARE_SECRET is not 32 bytes in hexadecimal".into());
    }
    for (at, byte) in secret.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&said[at * 2..at * 2 + 2], 16)
            .map_err(|_| worker::Error::from("SHARE_SECRET is not hexadecimal"))?;
    }
    Ok(secret)
}

fn share(env: &Env) -> worker::Result<Share<Cloud, Crypto>> {
    let cloud = Cloud {
        db: env.d1("DB")?,
        envelopes: env.bucket("ENVELOPES")?,
    };
    Ok(Share::new(cloud, Crypto, Policy::default(), secret(env)?))
}

fn now() -> i64 {
    (Date::now().as_millis() / 1000) as i64
}

fn answered(answer: api::Answer) -> worker::Result<Response> {
    if let Some(fault) = &answer.fault {
        worker::console_error!("{fault}");
    }
    let headers = Headers::new();
    for (name, value) in &answer.headers {
        headers.append(name, value)?;
    }
    Ok(Response::from_bytes(answer.body)?
        .with_status(answer.status)
        .with_headers(headers))
}

#[event(fetch)]
async fn fetch(mut request: Request, env: Env, _: Context) -> worker::Result<Response> {
    let mut share = share(&env)?;
    let headers = request.headers();
    let length = headers
        .get("Content-Length")?
        .and_then(|length| length.parse().ok());
    if let Some(refused) = length.and_then(|length| api::too_large(&share.policy.limits, length)) {
        return answered(refused);
    }
    // Cloudflare sets it on every request and a caller cannot, so the allowances rest on it.
    let caller: IpAddr = headers
        .get("CF-Connecting-IP")?
        .and_then(|ip| ip.parse().ok())
        .ok_or_else(|| worker::Error::from("the request came without its address"))?;
    let token = headers
        .get("Authorization")?
        .and_then(|said| said.strip_prefix("Bearer ").map(str::to_owned));
    let password = headers.get("Tisty-Password")?;
    let work = headers.get("Tisty-Work")?;
    let asked = api::Asked {
        method: request.method().to_string(),
        path: request.path(),
        token,
        password,
        work,
        body: request.bytes().await?,
        caller,
        now: now(),
    };
    answered(api::answer(&mut share, asked).await)
}

#[event(scheduled)]
async fn scheduled(_: ScheduledEvent, env: Env, _: ScheduleContext) {
    let swept = match share(&env) {
        Ok(share) => share
            .sweep(now())
            .await
            .map_err(|refused| refused.to_string()),
        Err(error) => Err(error.to_string()),
    };
    match swept {
        Ok(gone) => worker::console_log!("the sweep took {gone} documents"),
        Err(why) => worker::console_error!("the sweep failed: {why}"),
    }
}
