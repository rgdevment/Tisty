use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tisty_core::witness::{self, Fact, channel};

const PATIENCE: std::time::Duration = std::time::Duration::from_secs(6);
/// A head is at the top of the page: past this the rest is body we would only throw away.
const READS: usize = 256 * 1024;
const SHOT_AT_MOST: usize = 400 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glimpse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub said: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot: Option<String>,
}

impl Glimpse {
    pub fn bare(&self) -> bool {
        self.title.is_none() && self.said.is_none() && self.shot.is_none()
    }
}

pub fn kept(cache: &Path, at: &str) -> Option<Glimpse> {
    let said = std::fs::read_to_string(file(cache, at)).ok()?;
    serde_json::from_str(&said).ok()
}

pub fn keep(cache: &Path, at: &str, one: &Glimpse) {
    let file = file(cache, at);
    if let Some(parent) = file.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let Ok(said) = serde_json::to_string(one) else {
        return;
    };
    if let Err(why) = std::fs::write(&file, said) {
        witness::warn(
            channel::WINDOW,
            "a glimpse of a link could not be kept",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
}

fn file(cache: &Path, at: &str) -> PathBuf {
    let mut digest = Sha256::new();
    digest.update(at.as_bytes());
    let named = format!("{:x}", digest.finalize());
    cache.join("glimpses").join(format!("{named}.json"))
}

pub fn worldly(at: &str) -> bool {
    at.starts_with("http://") || at.starts_with("https://")
}

pub fn fetch(at: &str) -> Option<Glimpse> {
    if !worldly(at) {
        return None;
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let page = client.get(at).send().ok()?;
    if !page.status().is_success() {
        return None;
    }
    let body = page.text().ok()?;
    let head = &body[..body.len().min(READS)];

    let mut one = Glimpse {
        title: named(head, "og:title")
            .or_else(|| named(head, "twitter:title"))
            .or_else(|| titled(head)),
        said: named(head, "og:description").or_else(|| named(head, "description")),
        shot: None,
    };
    if let Some(shot) = named(head, "og:image").or_else(|| named(head, "twitter:image")) {
        one.shot = pictured(&client, &whole(at, &shot));
    }
    (!one.bare()).then_some(one)
}

fn pictured(client: &reqwest::blocking::Client, at: &str) -> Option<String> {
    if !worldly(at) {
        return None;
    }
    let asked = client.get(at).send().ok()?;
    if !asked.status().is_success() {
        return None;
    }
    let kind = asked
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|one| one.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if !kind.starts_with("image/") {
        return None;
    }
    let bytes = asked.bytes().ok()?;
    if bytes.len() > SHOT_AT_MOST {
        return None;
    }
    Some(format!("data:{kind};base64,{}", encoded(&bytes)))
}

/// The picture travels inside the glimpse rather than beside it: nothing of the world ends up in
/// the store, and the card never asks the network again to draw itself.
fn encoded(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut said = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for lot in bytes.chunks(3) {
        let held = [lot[0], *lot.get(1).unwrap_or(&0), *lot.get(2).unwrap_or(&0)];
        let bits = u32::from(held[0]) << 16 | u32::from(held[1]) << 8 | u32::from(held[2]);
        for n in 0..4 {
            if n <= lot.len() {
                said.push(ALPHABET[(bits >> (18 - n * 6)) as usize & 63] as char);
            } else {
                said.push('=');
            }
        }
    }
    said
}

fn whole(page: &str, shot: &str) -> String {
    if worldly(shot) {
        return shot.to_string();
    }
    let root = page.split('/').take(3).collect::<Vec<_>>().join("/");
    match shot.strip_prefix('/') {
        Some(rest) => format!("{root}/{rest}"),
        None => format!("{root}/{shot}"),
    }
}

fn titled(head: &str) -> Option<String> {
    let at = head.to_lowercase().find("<title")?;
    let rest = &head[at..];
    let opens = rest.find('>')? + 1;
    let shuts = rest.to_lowercase().find("</title>")?;
    (opens < shuts).then(|| tidy(&rest[opens..shuts]))?
}

fn named(head: &str, want: &str) -> Option<String> {
    let low = head.to_lowercase();
    let mut from = 0;
    while let Some(at) = low[from..].find("<meta") {
        let start = from + at;
        let end = low[start..].find('>').map(|one| start + one)?;
        let tag = &head[start..end];
        if holds(&tag.to_lowercase(), want)
            && let Some(said) = valued(tag, "content")
        {
            return tidy(&said);
        }
        from = end + 1;
    }
    None
}

fn holds(tag: &str, want: &str) -> bool {
    ["property", "name", "itemprop"]
        .iter()
        .filter_map(|key| valued(tag, key))
        .any(|said| said.trim().eq_ignore_ascii_case(want))
}

fn valued(tag: &str, key: &str) -> Option<String> {
    let low = tag.to_lowercase();
    let at = low.find(&format!("{key}="))?;
    let rest = &tag[at + key.len() + 1..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return rest.split_whitespace().next().map(str::to_string);
    }
    let shuts = rest[1..].find(quote)?;
    Some(rest[1..1 + shuts].to_string())
}

fn tidy(said: &str) -> Option<String> {
    let whole = said
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    let trimmed: String = whole.split_whitespace().collect::<Vec<_>>().join(" ");
    (!trimmed.is_empty()).then(|| trimmed.chars().take(300).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_says_its_name_through_open_graph_before_its_title_tag() {
        let head = r#"<title>Lo de siempre</title><meta property="og:title" content="Lo que quiere que veas">"#;

        assert_eq!(
            named(head, "og:title").as_deref(),
            Some("Lo que quiere que veas")
        );
        assert_eq!(titled(head).as_deref(), Some("Lo de siempre"));
    }

    #[test]
    fn a_meta_that_names_something_else_is_not_taken_for_this_one() {
        let head = r#"<meta name="author" content="Alguien"><meta name="description" content="De esto va">"#;

        assert_eq!(named(head, "description").as_deref(), Some("De esto va"));
        assert_eq!(named(head, "og:title"), None);
    }

    #[test]
    fn what_a_page_writes_in_its_head_comes_back_as_plain_words() {
        let head = r#"<meta property="og:title" content="Uno &amp; otro
   con   aire">"#;

        assert_eq!(
            named(head, "og:title").as_deref(),
            Some("Uno & otro con aire")
        );
    }

    #[test]
    fn a_picture_named_by_its_path_is_asked_for_at_the_same_host() {
        assert_eq!(
            whole("https://ejemplo.org/uno/dos", "/cover.png"),
            "https://ejemplo.org/cover.png"
        );
        assert_eq!(
            whole("https://ejemplo.org/uno", "https://otro.example/x.png"),
            "https://otro.example/x.png"
        );
    }

    #[test]
    fn nothing_of_the_machine_is_ever_asked_for() {
        assert!(!worldly("file:///etc/passwd"));
        assert!(!worldly("/etc/passwd"));
        assert!(!worldly("tisty:doc/mac0-0001"));
        assert!(worldly("https://ejemplo.org"));
    }

    #[test]
    fn bytes_travel_as_the_letters_that_stand_for_them() {
        assert_eq!(encoded(b"Ma"), "TWE=");
        assert_eq!(encoded(b"Man"), "TWFu");
        assert_eq!(encoded(b"M"), "TQ==");
        assert_eq!(encoded(b""), "");
    }

    #[test]
    fn a_glimpse_written_down_is_read_back_the_same() {
        let room = tempfile::tempdir().unwrap();
        let one = Glimpse {
            title: Some("Tisty".into()),
            said: Some("Notas y tareas".into()),
            shot: None,
        };

        keep(room.path(), "https://ejemplo.org", &one);

        assert_eq!(kept(room.path(), "https://ejemplo.org"), Some(one));
        assert_eq!(kept(room.path(), "https://otra.example"), None);
    }
}
