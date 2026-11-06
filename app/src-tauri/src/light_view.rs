use std::io::Read as _;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Component, Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use percent_encoding::percent_decode_str;
use tauri::{AppHandle, Manager};

use crate::viewing::{Shown, Views, named, place_of, shown_in};
use crate::{Answer, Refusal};

const PIECE_ABOUT: usize = 64 * 1024;

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Heading {
    level: u8,
    said: String,
    id: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Reading {
    Drawn {
        name: String,
        place: String,
        path: String,
        first: String,
        more: bool,
        heads: Vec<Heading>,
    },
    Missed {
        name: String,
        place: String,
        path: String,
        refusal: Refusal,
    },
}

fn file_of(app: &AppHandle, label: &str) -> Answer<std::path::PathBuf> {
    match shown_in(&app.state::<Views>(), label)? {
        Shown::File(at) => Ok(at),
        _ => Err(Refusal::of("notAllowed")),
    }
}

fn body_of(at: &Path) -> Answer<String> {
    tisty_core::docs::read_to_show(at).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooBig", crate::weighed(limit))
        }
        _ => Refusal::about("cannotRead", at.display().to_string()),
    })
}

// What is drawn here goes nowhere else: a stranger's file stays out of the cache of the person's own pages.
fn drawn_from(body: &str) -> (Vec<String>, Vec<Heading>) {
    let doc = tisty_markdown::read_file(body);
    let heads = tisty_markdown::outline(&doc)
        .into_iter()
        .map(|head| Heading {
            level: head.level,
            said: head.said,
            id: head.id,
        })
        .collect();
    (tisty_markdown::pieces(&doc, PIECE_ABOUT), heads)
}

#[tauri::command(async)]
pub fn view_read(app: AppHandle, window: tauri::WebviewWindow) -> Answer<Reading> {
    let label = window.label();
    let at = file_of(&app, label)?;
    let (name, place, path) = (named(&at), place_of(&at), at.display().to_string());
    let body = match body_of(&at) {
        Ok(body) => body,
        Err(refusal) => {
            return Ok(Reading::Missed {
                name,
                place,
                path,
                refusal,
            });
        }
    };
    let (mut pieces, heads) = drawn_from(&body);
    let first = if pieces.is_empty() {
        String::new()
    } else {
        pieces.remove(0)
    };
    let more = !pieces.is_empty();
    if let Ok(mut held) = app.state::<Views>().waiting_pieces.lock() {
        held.insert(label.to_string(), pieces);
    }
    Ok(Reading::Drawn {
        name,
        place,
        path,
        first,
        more,
        heads,
    })
}

#[tauri::command(async)]
pub fn view_rest(app: AppHandle, window: tauri::WebviewWindow) -> Vec<String> {
    app.state::<Views>()
        .waiting_pieces
        .lock()
        .ok()
        .and_then(|mut held| held.remove(window.label()))
        .unwrap_or_default()
}

#[tauri::command(async)]
pub fn view_source(app: AppHandle, window: tauri::WebviewWindow) -> Answer<String> {
    body_of(&file_of(&app, window.label())?)
}

const PICTURES: [(&str, &str); 9] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("bmp", "image/bmp"),
    ("ico", "image/x-icon"),
    ("svg", "image/svg+xml"),
];
const PICTURE_AT_MOST: u64 = 20 * 1024 * 1024;
const FAR_PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);
const FAR_REDIRECTS: usize = 5;

fn as_data(kind: &str, bytes: &[u8]) -> String {
    format!("data:{kind};base64,{}", BASE64.encode(bytes))
}

fn undone(said: &str) -> String {
    percent_decode_str(said).decode_utf8_lossy().into_owned()
}

fn beside(file: &Path, said: &str) -> PathBuf {
    let mut at = file.parent().map(Path::to_path_buf).unwrap_or_default();
    for one in Path::new(said).components() {
        match one {
            Component::ParentDir => {
                at.pop();
            }
            Component::CurDir => {}
            _ => at.push(one),
        }
    }
    at
}

// A file from a stranger may name any path, so a picture is read only from its own folder or below.
fn near_picture(file: &Path, src: &str) -> Answer<String> {
    let refused = || Refusal::of("notAllowed");
    let root = file
        .parent()
        .ok_or_else(refused)?
        .canonicalize()
        .map_err(|_| refused())?;
    let at = beside(file, &undone(src))
        .canonicalize()
        .map_err(|_| refused())?;
    if !at.starts_with(&root) {
        return Err(refused());
    }
    let extension = at
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (_, kind) = PICTURES
        .iter()
        .find(|(one, _)| *one == extension)
        .ok_or_else(refused)?;
    let weighs = std::fs::metadata(&at).map_err(|_| refused())?.len();
    if weighs > PICTURE_AT_MOST {
        return Err(refused());
    }
    Ok(as_data(kind, &std::fs::read(&at).map_err(|_| refused())?))
}

#[tauri::command(async)]
pub fn view_picture(app: AppHandle, window: tauri::WebviewWindow, src: String) -> Answer<String> {
    near_picture(&file_of(&app, window.label())?, &src)
}

fn far_kind(said: Option<&str>) -> Option<String> {
    let kind = said?.split(';').next()?.trim().to_ascii_lowercase();
    kind.starts_with("image/").then_some(kind)
}

#[tauri::command]
pub async fn view_picture_far(
    app: AppHandle,
    window: tauri::WebviewWindow,
    src: String,
) -> Answer<String> {
    file_of(&app, window.label())?;
    tauri::async_runtime::spawn_blocking(move || far_picture(&src))
        .await
        .map_err(|_| Refusal::of("notAllowed"))?
}

fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(one) => {
            let [first, second, ..] = one.octets();
            !(one.is_loopback()
                || one.is_private()
                || one.is_link_local()
                || one.is_unspecified()
                || one.is_broadcast()
                || one.is_multicast()
                || one.is_documentation()
                || first == 0
                || (first == 100 && (64..128).contains(&second)))
        }
        IpAddr::V6(one) => match one.to_ipv4_mapped() {
            Some(mapped) => public_address(IpAddr::V4(mapped)),
            None => {
                let head = one.segments()[0];
                !(one.is_loopback()
                    || one.is_unspecified()
                    || one.is_multicast()
                    || (head & 0xfe00) == 0xfc00
                    || (head & 0xffc0) == 0xfe80)
            }
        },
    }
}

// A stranger's file chooses the address, and a press here must not reach this machine or its network.
fn reaches_outside(address: &reqwest::Url) -> bool {
    let Some(port) = address.port_or_known_default() else {
        return false;
    };
    match address.host() {
        Some(url::Host::Ipv4(one)) => public_address(IpAddr::V4(one)),
        Some(url::Host::Ipv6(one)) => public_address(IpAddr::V6(one)),
        Some(url::Host::Domain(name)) => (name, port).to_socket_addrs().is_ok_and(|mut all| {
            let mut any = false;
            all.all(|one| {
                any = true;
                public_address(one.ip())
            }) && any
        }),
        None => false,
    }
}

fn far_picture(src: &str) -> Answer<String> {
    let refused = || Refusal::of("notAllowed");
    if !crate::linking::on_the_web(src) {
        return Err(refused());
    }
    let address = reqwest::Url::parse(src).map_err(|_| refused())?;
    if !reaches_outside(&address) {
        return Err(refused());
    }
    let answer = reqwest::blocking::Client::builder()
        .timeout(FAR_PATIENCE)
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() < FAR_REDIRECTS && reaches_outside(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| refused())?
        .get(src)
        .send()
        .map_err(|_| refused())?;
    let kind = far_kind(
        answer
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|one| one.to_str().ok()),
    )
    .ok_or_else(refused)?;
    let mut bytes = Vec::new();
    answer
        .take(PICTURE_AT_MOST + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| refused())?;
    if bytes.len() as u64 > PICTURE_AT_MOST {
        return Err(refused());
    }
    Ok(as_data(&kind, &bytes))
}

fn neighbour(file: &Path, said: &str) -> Option<PathBuf> {
    let at = beside(file, &undone(said));
    crate::viewing::readable(&at).then_some(at)
}

#[tauri::command(async)]
pub fn view_follow(app: AppHandle, window: tauri::WebviewWindow, to: String) -> Answer<()> {
    let label = window.label();
    let at = neighbour(&file_of(&app, label)?, &to).ok_or_else(|| Refusal::of("notAllowed"))?;
    crate::viewing::reshow(&app.state::<Views>(), label, Shown::File(at));
    Ok(())
}

#[cfg(test)]
#[path = "light_view_test.rs"]
mod tests;
