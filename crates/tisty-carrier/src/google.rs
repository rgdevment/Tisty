use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::{Hitch, Renew, Renewed};

const CONSENT_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const LISTENED_EVERY: Duration = Duration::from_millis(100);
const ASKED_WITHIN: Duration = Duration::from_secs(30);
const HEARD_WITHIN: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    pub id: &'static str,
    pub secret: &'static str,
}

/// Fixed when the release is built; a build without them, a fork's included, offers no cloud.
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
            state,
            verifier,
            redirect,
            listener,
        })
    }

    pub fn finish(
        self,
        client: Client,
        within: Duration,
        token_url: &str,
    ) -> Result<Granted, Hitch> {
        let code = self.answered(within)?;
        let answer = post_form(
            &reqwest::blocking::Client::new(),
            token_url,
            &[
                ("client_id", client.id),
                ("client_secret", client.secret),
                ("code", &code),
                ("code_verifier", &self.verifier),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &self.redirect),
            ],
        )?;
        if !only_drive_file(answer.scope.as_deref()) {
            return Err(Hitch::Broke(
                "Google granted more than the files Tisty creates, so the sign-in was refused"
                    .into(),
            ));
        }
        let refresh = answer
            .refresh_token
            .ok_or_else(|| Hitch::Broke("Google gave no lasting access to keep".into()))?;
        Ok(Granted {
            refresh,
            access: answer.access_token,
            lasts: Duration::from_secs(answer.expires_in),
        })
    }

    fn answered(&self, within: Duration) -> Result<String, Hitch> {
        self.listener.set_nonblocking(true)?;
        let until = Instant::now() + within;
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => match self.heard(stream) {
                    Heard::Code(code) => return Ok(code),
                    Heard::Refused(hitch) => return Err(hitch),
                    Heard::Elsewhere => {}
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= until {
                        return Err(Hitch::Unreachable("the sign-in was never finished".into()));
                    }
                    std::thread::sleep(LISTENED_EVERY);
                }
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
        let heard = match (said.get("code"), said.get("error")) {
            (_, Some(error)) => Heard::Refused(match error.as_str() {
                "access_denied" => Hitch::Lost,
                other => Hitch::Broke(format!("Google refused the sign-in: {other}")),
            }),
            (Some(code), None) => Heard::Code(code.clone()),
            (None, None) => {
                Heard::Refused(Hitch::Broke("the sign-in came back without a code".into()))
            }
        };
        let page = match heard {
            Heard::Code(_) => "Tisty is connected. You can close this tab.",
            _ => "Tisty could not connect. You can close this tab and try again from Tisty.",
        };
        answer(&mut stream, "200 OK", page);
        heard
    }
}

enum Heard {
    Code(String),
    Refused(Hitch),
    Elsewhere,
}

fn asked_for(stream: &TcpStream) -> Option<String> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(HEARD_WITHIN)).ok()?;
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    reader.read_line(&mut first).ok()?;
    // The headers are read through, or closing with them unread resets the browser's connection.
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|read| read > 2) {
        line.clear();
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
    let _ = stream.shutdown(std::net::Shutdown::Write);
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
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .timeout(ASKED_WITHIN)
        .send()
        .map_err(|e| Hitch::Unreachable(e.without_url().to_string()))?;
    let status = answer.status();
    let wait = answer
        .headers()
        .get("Retry-After")
        .and_then(|one| one.to_str().ok())
        .and_then(|one| one.parse::<u64>().ok())
        .map_or(Duration::from_secs(60), Duration::from_secs);
    let text = answer
        .text()
        .map_err(|e| Hitch::Unreachable(e.without_url().to_string()))?;
    match status.as_u16() {
        200 => serde_json::from_str(&text).map_err(|_| {
            Hitch::Broke("Google answered the sign-in in a shape Tisty does not read".into())
        }),
        429 => Err(Hitch::Limited { wait }),
        500.. => Err(Hitch::Unreachable(format!("Google answered {status}"))),
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

pub struct GoogleRenew {
    client: Client,
    token_url: String,
    http: reqwest::blocking::Client,
}

impl GoogleRenew {
    pub fn new(client: Client, token_url: impl Into<String>) -> Self {
        Self {
            client,
            token_url: token_url.into(),
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl Renew for GoogleRenew {
    fn renew(&self, refresh: &str) -> Result<Renewed, Hitch> {
        let answer = post_form(
            &self.http,
            &self.token_url,
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
#[path = "google_test.rs"]
mod tests;
