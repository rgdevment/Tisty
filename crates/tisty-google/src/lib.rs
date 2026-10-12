use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use tisty_carrier::{Hitch, Renew, Renewed};

const CONSENT_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const LISTENED_EVERY: Duration = Duration::from_millis(100);
const ASKED_WITHIN: Duration = Duration::from_secs(30);
const HEARD_WITHIN: Duration = Duration::from_secs(2);
const HEARD_AT_MOST: u64 = 16 * 1024;
const WAITED_AT_LEAST: Duration = Duration::from_secs(1);
const WAITED_UNTOLD: Duration = Duration::from_secs(60);
const CONNECTED: &str = "Tisty is connected. You can close this tab.";
const NOT_CONNECTED: &str =
    "Tisty could not connect. You can close this tab and try again from Tisty.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    pub id: &'static str,
    pub secret: &'static str,
}

// Fixed when the release is built, so a fork or a local build without them offers no cloud.
pub fn client() -> Option<Client> {
    match (
        option_env!("TISTY_GOOGLE_CLIENT_ID"),
        option_env!("TISTY_GOOGLE_CLIENT_SECRET"),
    ) {
        (Some(id), Some(secret)) if !id.is_empty() && !secret.is_empty() => {
            Some(Client { id, secret })
        }
        _ => None,
    }
}

pub fn offered() -> bool {
    client().is_some()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    pub token: String,
    pub revoke: String,
}

impl Endpoints {
    pub fn google() -> Self {
        Self {
            token: "https://oauth2.googleapis.com/token".into(),
            revoke: "https://oauth2.googleapis.com/revoke".into(),
        }
    }
}

pub fn challenge_of(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn random_text(bytes: usize) -> Result<String, Hitch> {
    use rand_core::TryRngCore;
    let mut fresh = vec![0u8; bytes];
    rand_core::OsRng
        .try_fill_bytes(&mut fresh)
        .map_err(|e| Hitch::Broke(format!("no randomness for the sign-in: {e}")))?;
    Ok(URL_SAFE_NO_PAD.encode(fresh))
}

pub struct Granted {
    pub refresh: String,
    pub access: String,
    pub lasts: Duration,
}

pub struct Consent {
    pub url: String,
    client: Client,
    state: String,
    verifier: String,
    redirect: String,
    listener: TcpListener,
}

impl Consent {
    pub fn begin(client: Client) -> Result<Self, Hitch> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let redirect = format!("http://127.0.0.1:{}/", listener.local_addr()?.port());
        let verifier = random_text(48)?;
        let state = random_text(24)?;
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", client.id)
            .append_pair("redirect_uri", &redirect)
            .append_pair("response_type", "code")
            .append_pair("scope", SCOPE)
            .append_pair("code_challenge", &challenge_of(&verifier))
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &state)
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .finish();
        Ok(Self {
            url: format!("{CONSENT_URL}?{query}"),
            client,
            state,
            verifier,
            redirect,
            listener,
        })
    }

    // The browser is told how it ended only once the code is traded and the grant is known good.
    pub fn finish(self, within: Duration, at: &Endpoints) -> Result<Granted, Hitch> {
        let (code, mut browser) = self.answered(within)?;
        let granted = self.traded(&code, at);
        answer(
            &mut browser,
            "200 OK",
            if granted.is_ok() {
                CONNECTED
            } else {
                NOT_CONNECTED
            },
        );
        granted
    }

    fn traded(&self, code: &str, at: &Endpoints) -> Result<Granted, Hitch> {
        let http = http()?;
        let answer = post_form(
            &http,
            &at.token,
            &[
                ("client_id", self.client.id),
                ("client_secret", self.client.secret),
                ("code", code),
                ("code_verifier", &self.verifier),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &self.redirect),
            ],
        )?;
        if !only_drive_file(answer.scope.as_deref()) {
            revoke(&http, at, &answer);
            return Err(Hitch::Broke(
                "Google did not grant access to just the files Tisty creates, so the sign-in was refused"
                    .into(),
            ));
        }
        let Some(refresh) = answer.refresh_token.clone() else {
            revoke(&http, at, &answer);
            return Err(Hitch::Broke("Google gave no lasting access to keep".into()));
        };
        Ok(Granted {
            refresh,
            access: answer.access_token,
            lasts: Duration::from_secs(answer.expires_in),
        })
    }

    fn answered(&self, within: Duration) -> Result<(String, TcpStream), Hitch> {
        self.listener.set_nonblocking(true)?;
        let until = Instant::now().checked_add(within);
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => match self.heard(stream) {
                    Heard::Code(code, browser) => return Ok((code, browser)),
                    Heard::Refused(hitch) => return Err(hitch),
                    Heard::Elsewhere => {}
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if until.is_some_and(|until| Instant::now() >= until) {
                        return Err(Hitch::Lost);
                    }
                    std::thread::sleep(LISTENED_EVERY);
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::ConnectionAborted
                            | std::io::ErrorKind::ConnectionReset
                            | std::io::ErrorKind::Interrupted
                    ) => {}
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
            (_, Some(error)) => Hitch::Broke(format!("Google refused the sign-in: {error}")),
            (None, None) => Hitch::Broke("the sign-in came back without a code".into()),
        };
        answer(&mut stream, "200 OK", NOT_CONNECTED);
        Heard::Refused(refused)
    }
}

enum Heard {
    Code(String, TcpStream),
    Refused(Hitch),
    Elsewhere,
}

fn asked_for(stream: &TcpStream) -> Option<String> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(HEARD_WITHIN)).ok()?;
    let until = Instant::now() + HEARD_WITHIN;
    let mut reader = BufReader::new(stream.take(HEARD_AT_MOST));
    let mut first = String::new();
    reader.read_line(&mut first).ok()?;
    // The headers are read through, or closing with them unread resets the browser's connection.
    let mut line = String::new();
    while let Some(left) = until
        .checked_duration_since(Instant::now())
        .filter(|left| !left.is_zero())
    {
        stream.set_read_timeout(Some(left)).ok()?;
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

fn only_drive_file(scope: Option<&str>) -> bool {
    scope.is_some_and(|granted| {
        granted.split_whitespace().next().is_some()
            && granted.split_whitespace().all(|one| one == SCOPE)
    })
}

#[derive(serde::Deserialize)]
struct TokenAnswer {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    scope: Option<String>,
}

#[derive(serde::Deserialize)]
struct TokenRefusal {
    error: Option<String>,
}

// Blocking on purpose: every carrier call already runs off the window's async runtime.
fn http() -> Result<reqwest::blocking::Client, Hitch> {
    reqwest::blocking::Client::builder()
        .timeout(ASKED_WITHIN)
        .build()
        .map_err(|e| Hitch::Broke(told(&e.without_url())))
}

fn told(error: &dyn std::error::Error) -> String {
    let mut said = error.to_string();
    let mut under = error.source();
    while let Some(one) = under {
        said.push_str(": ");
        said.push_str(&one.to_string());
        under = one.source();
    }
    said
}

fn waited(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|one| one.to_str().ok())
        .and_then(|one| one.trim().parse::<u64>().ok())
        .map(|secs| Duration::from_secs(secs).max(WAITED_AT_LEAST))
}

fn post_form(
    http: &reqwest::blocking::Client,
    url: &str,
    fields: &[(&str, &str)],
) -> Result<TokenAnswer, Hitch> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(fields)
        .finish();
    let answer = http
        .post(url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
    let status = answer.status();
    let wait = waited(answer.headers());
    let text = answer
        .text()
        .map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
    match status.as_u16() {
        200 => serde_json::from_str(&text).map_err(|_| {
            Hitch::Broke("Google answered the sign-in in a shape Tisty does not read".into())
        }),
        429 => Err(Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        }),
        500.. => match wait {
            Some(wait) => Err(Hitch::Limited { wait }),
            None => Err(Hitch::Unreachable(format!("Google answered {status}"))),
        },
        _ => {
            let error = serde_json::from_str::<TokenRefusal>(&text)
                .ok()
                .and_then(|one| one.error)
                .unwrap_or_default();
            match error.as_str() {
                "invalid_grant" => Err(Hitch::Lost),
                other => Err(Hitch::Broke(format!("Google refused the token: {other}"))),
            }
        }
    }
}

// A grant Tisty turns down is handed back, so the account never lists access Tisty said it refused.
fn revoke(http: &reqwest::blocking::Client, at: &Endpoints, granted: &TokenAnswer) {
    let token = granted
        .refresh_token
        .as_deref()
        .unwrap_or(&granted.access_token);
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("token", token)
        .finish();
    let _ = http
        .post(&at.revoke)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send();
}

pub struct GoogleRenew {
    client: Client,
    at: Endpoints,
    http: reqwest::blocking::Client,
}

impl GoogleRenew {
    pub fn new(client: Client, at: Endpoints) -> Result<Self, Hitch> {
        Ok(Self {
            client,
            at,
            http: http()?,
        })
    }
}

impl Renew for GoogleRenew {
    fn renew(&self, refresh: &str) -> Result<Renewed, Hitch> {
        let answer = post_form(
            &self.http,
            &self.at.token,
            &[
                ("client_id", self.client.id),
                ("client_secret", self.client.secret),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh),
            ],
        )?;
        Ok(Renewed {
            access: answer.access_token,
            lasts: Duration::from_secs(answer.expires_in),
            refresh: answer.refresh_token,
        })
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
