use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tisty_core::event::{Shared, Signs};
use tisty_markdown::front;
use tisty_share_proto::config::{Config, Field};
use tisty_share_proto::envelope::{self, Cited, Draft, Kind, LinkKey, Refused as Unsealed, Signer};
use tisty_share_proto::identity::{REVOCATIONS_AT_ONCE, Revocation};
use tisty_share_proto::limits::Limits;
use tisty_share_proto::work::{self, Challenge};

use crate::handed::Voucher;
use crate::refusing::Refusal;

/// Tisty Share as Tisty knows it; each share keeps the server it went to, so a later one can live elsewhere.
/// A build can name another with `TISTY_SHARE_SERVER`, so losing the domain costs one release and no code.
pub(crate) const SERVER: &str = match option_env!("TISTY_SHARE_SERVER") {
    Some(server) => server,
    None => "https://tisty.cl",
};
const API: &str = "/api/v1/";
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(30);

/// What came back from one request.
pub(crate) struct Said {
    pub(crate) status: u16,
    pub(crate) body: Vec<u8>,
    /// The `Tisty-Revoked` the server sends with a document, for the reader to weigh the signature by.
    pub(crate) revoked: Option<String>,
    /// Whether the cache in front of the server may have answered it, so the reader tells the read itself.
    pub(crate) cached: bool,
}

/// One request as the API takes it, so the same client speaks to a server or, in its tests, to the API itself.
pub(crate) trait Wire {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal>;
}

pub(crate) struct Asking<'a> {
    pub(crate) method: &'a str,
    /// Below `/api/v1/`.
    pub(crate) path: String,
    /// The edit or management token.
    pub(crate) token: Option<&'a str>,
    /// The proof of work, as JSON.
    pub(crate) work: Option<String>,
    /// A reader's password as its UTF-8 in base64url, which the server takes only in `Tisty-Password`.
    pub(crate) password: Option<String>,
    pub(crate) body: Vec<u8>,
}

pub(crate) struct Net {
    client: reqwest::blocking::Client,
    server: String,
}

impl Net {
    pub(crate) fn to(server: &str) -> Result<Self, Refusal> {
        let client = reqwest::blocking::Client::builder()
            .timeout(PATIENCE)
            // A server that never answers is told apart in seconds, not after the whole patience.
            .connect_timeout(std::time::Duration::from_secs(8))
            .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| Refusal::of("shareOffline"))?;
        Ok(Self {
            client,
            server: server.trim_end_matches('/').to_owned(),
        })
    }
}

impl Wire for Net {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal> {
        let url = format!("{}{API}{}", self.server, asked.path);
        let method = reqwest::Method::from_bytes(asked.method.as_bytes())
            .map_err(|_| Refusal::of("internal"))?;
        let mut request = self.client.request(method, url).body(asked.body);
        if let Some(token) = asked.token {
            request = request.bearer_auth(token);
        }
        if let Some(work) = asked.work {
            request = request.header("Tisty-Work", work);
        }
        if let Some(password) = asked.password {
            request = request.header("Tisty-Password", password);
        }
        let answer = request.send().map_err(|_| Refusal::of("shareOffline"))?;
        let status = answer.status().as_u16();
        let revoked = answer
            .headers()
            .get("Tisty-Revoked")
            .and_then(|one| one.to_str().ok())
            .map(str::to_owned);
        let cached = answer
            .headers()
            .get("Cache-Control")
            .and_then(|one| one.to_str().ok())
            .is_some_and(|one| one.starts_with("public"));
        let body = answer.bytes().map_err(|_| Refusal::of("shareOffline"))?;
        Ok(Said {
            status,
            body: body.to_vec(),
            revoked,
            cached,
        })
    }
}

/// What the person chose in the «Share» dialog.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Chosen {
    /// A password the reader must give, or none for a link alone.
    pub(crate) password: Option<String>,
    /// Whether the edit link is handed out (decision 67).
    pub(crate) editable: bool,
    /// Days without activity before it goes, when shorter than the server's (decision 50).
    pub(crate) idle_days: Option<u32>,
    /// The print of the words the dialog looked through for keys, so what goes out is what was looked at.
    #[serde(default)]
    pub(crate) seen: Option<String>,
    /// The name it goes out signed with (decision 89).
    #[serde(default)]
    pub(crate) signs: Signs,
    /// Kept as it is now rather than following what is saved (decision 48).
    #[serde(default)]
    pub(crate) still: bool,
    /// What it cites besides its pages goes too, each in its own envelope (decision 64).
    #[serde(default)]
    pub(crate) cites: bool,
}

/// A print of a text, to tell whether it changed between two reads of it.
pub(crate) fn print_of(text: &str) -> String {
    use sha2::Digest;
    URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(text.as_bytes()))
}

/// What the owner sees of a share on its card, beside the links the store keeps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Info {
    pub(crate) latest: Option<u64>,
    pub(crate) reads: u64,
    pub(crate) ends: i64,
    pub(crate) paused: bool,
    pub(crate) blocked: bool,
    pub(crate) gated: bool,
}

#[derive(Deserialize)]
struct Reserved {
    id: String,
    edit: String,
    admin: String,
}

/// A server's no, said as a refusal the window words: its own word rides along as the name.
pub(crate) fn refused(said: &Said) -> Refusal {
    let word = serde_json::from_slice::<serde_json::Value>(&said.body)
        .ok()
        .and_then(|value| value["refused"].as_str().map(str::to_owned));
    match word.as_deref() {
        // The one no the person can act on: this machine's key was taken back or never held.
        Some("unsigned") => Refusal::of("shareUnsigned"),
        Some("too_large") => Refusal::of("shareTooBig"),
        Some("slow") => Refusal::of("shareSlow"),
        // The server's own trouble, or no word at all: nothing the person can do but try again.
        Some("store" | "crowded" | "work_refused" | "no_randomness") | None => {
            Refusal::of("shareFailed")
        }
        Some(_) if said.status >= 500 => Refusal::of("shareFailed"),
        Some(word) => Refusal::about("shareRefused", word.to_owned()),
    }
}

fn answered<T: for<'de> Deserialize<'de>>(said: Said) -> Result<T, Refusal> {
    if !(200..300).contains(&said.status) {
        return Err(refused(&said));
    }
    serde_json::from_slice(&said.body).map_err(|_| Refusal::of("shareFailed"))
}

fn done(said: Said) -> Result<(), Refusal> {
    match (200..300).contains(&said.status) {
        true => Ok(()),
        false => Err(refused(&said)),
    }
}

pub(crate) fn get(wire: &impl Wire, path: String, token: Option<&str>) -> Result<Said, Refusal> {
    wire.send(Asking {
        method: "GET",
        path,
        token,
        work: None,
        password: None,
        body: Vec::new(),
    })
}

// Each piece of work answers one ask only, so it is asked for right before it is spent.
pub(crate) fn worked(wire: &impl Wire, ask: serde_json::Value) -> Result<String, Refusal> {
    let said = wire.send(Asking {
        method: "POST",
        path: "challenge".into(),
        token: None,
        work: None,
        password: None,
        body: ask.to_string().into_bytes(),
    })?;
    let challenge: Challenge = answered(said)?;
    let proof = work::solve(&challenge).ok_or(Refusal::of("shareFailed"))?;
    serde_json::to_string(&proof).map_err(|_| Refusal::of("internal"))
}

fn managed(wire: &impl Wire, shared: &Shared, what: serde_json::Value) -> Result<(), Refusal> {
    done(wire.send(Asking {
        method: "POST",
        path: format!("docs/{}/manage", shared.link),
        token: Some(&shared.admin),
        work: None,
        password: None,
        body: what.to_string().into_bytes(),
    })?)
}

/// Where a picture the words name sits in an envelope, which the reader turns back into the picture (decision 79).
pub(crate) const PICTURE: &str = "tisty:picture/";

/// The words with each picture they name from the attachments put in its place in the envelope, and those pictures
/// written again, so nothing but the pixels travels. What is no picture, or sits in code, stays as it was.
pub(crate) fn pictured(
    text: &str,
    read: impl Fn(&str) -> Option<Vec<u8>>,
) -> (String, Vec<(Kind, Vec<u8>)>) {
    let code = tisty_core::beside::written_as_code(text);
    let in_code = |at: usize| code.get(at).copied().unwrap_or(false);
    let mut said = String::with_capacity(text.len());
    let mut pictures: Vec<(Kind, Vec<u8>)> = Vec::new();
    let mut placed: Vec<String> = Vec::new();
    let mut rest = 0;
    let mut from = 0;
    while let Some(found) = text[from..].find("![") {
        let at = from + found;
        from = at + 2;
        let Some((opens, closes)) = destination(text, at) else {
            continue;
        };
        if in_code(at) || in_code(opens) {
            continue;
        }
        let inner = &text[opens..closes];
        let starts = opens + (inner.len() - inner.trim_start_matches([' ', '\t']).len());
        let named = &text[starts..closes];
        // Within `<…>` a destination may hold spaces; bare, the first space ends it.
        let written = match named.strip_prefix('<').and_then(|inside| inside.find('>')) {
            Some(ends) => &named[..ends + 2],
            None => named.split([' ', '\t']).next().unwrap_or_default(),
        };
        let target = written.trim_start_matches('<').trim_end_matches('>');
        if !tisty_core::attach::names_an_attachment(target) {
            continue;
        }
        let place = match placed.iter().position(|one| one == target) {
            Some(place) => place,
            None => {
                let made = read(target).and_then(|bytes| {
                    tisty_share_proto::pictures::recompressed(&bytes, &Limits::default()).ok()
                });
                let Some(made) = made else {
                    continue;
                };
                pictures.push(made);
                placed.push(target.to_owned());
                placed.len() - 1
            }
        };
        said.push_str(&text[rest..starts]);
        said.push_str(&format!("{PICTURE}{place}"));
        rest = starts + written.len();
        from = closes;
    }
    said.push_str(&text[rest..]);
    (said, pictures)
}

/// The bytes of an attachment the words name, read where the store keeps them; called off the window's thread.
pub(crate) fn attachment(target: &str, data: &std::path::Path) -> Option<Vec<u8>> {
    std::fs::read(tisty_core::attach::resolve(target, data).ok()?).ok()
}

// The `(` and `)` of an image's destination: right after the `]` that closes its alt text, all on one line.
fn destination(text: &str, image: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut at = image + 2;
    loop {
        match bytes.get(at)? {
            b'\n' => return None,
            b'\\' => at += 1,
            b'[' => depth += 1,
            b']' if depth == 0 => break,
            b']' => depth -= 1,
            _ => {}
        }
        at += 1;
    }
    if bytes.get(at + 1) != Some(&b'(') {
        return None;
    }
    let opens = at + 2;
    // Within `<…>` a `)` is part of the name, so the destination closes only after the `>`.
    let named = opens + (text[opens..].len() - text[opens..].trim_start_matches([' ', '\t']).len());
    let after = match bytes.get(named) {
        Some(b'<') => named + text[named..].find(['>', '\n'])?,
        _ => opens,
    };
    let closes = after + text[after..].find([')', '\n'])?;
    (bytes[closes] == b')').then_some((opens, closes))
}

/// Shares a document's words for the first time, as Tisty did before what it cites went with it.
#[cfg(test)]
pub(crate) fn share(
    wire: &impl Wire,
    server: &str,
    text: &str,
    pictures: &[(Kind, Vec<u8>)],
    chosen: &Chosen,
    voucher: &Voucher,
    name: Option<&str>,
) -> Result<Shared, Refusal> {
    let shared = reserve(wire, server, chosen)?;
    let first = Words {
        text,
        cited: &[],
        pictures,
        voucher,
        name,
    };
    first_up(wire, &shared, &first, chosen)?;
    Ok(shared)
}

/// Reserves a document's links and mints the key that will seal it, so what cites it can name it before it is up.
pub(crate) fn reserve(wire: &impl Wire, server: &str, chosen: &Chosen) -> Result<Shared, Refusal> {
    let work = worked(wire, json!({ "for": "reserve" }))?;
    let reserved: Reserved = answered(wire.send(Asking {
        method: "POST",
        path: "docs".into(),
        token: None,
        work: Some(work),
        password: None,
        body: Vec::new(),
    })?)?;
    let link_key = LinkKey::minted().ok_or(Refusal::of("internal"))?;
    Ok(Shared {
        server: server.to_owned(),
        link: reserved.id,
        key: link_key.to_text().to_string(),
        edit: reserved.edit,
        admin: reserved.admin,
        editable: chosen.editable,
        gated: chosen.password.is_some(),
        version: 1,
        still: chosen.still,
        taken: false,
        print: String::new(),
        signs: chosen.signs,
        holds: Vec::new(),
        held: false,
        cites: chosen.cites,
    })
}

/// Puts a reserved document's first version up with what was chosen for it, or lets its links go.
pub(crate) fn first_up(
    wire: &impl Wire,
    shared: &Shared,
    words: &Words,
    chosen: &Chosen,
) -> Result<(), Refusal> {
    if let Err(refused) = put_up(wire, shared, None, words) {
        // An answer lost on the way back may still have left it up, and a link nobody kept is a link nobody can stop.
        let _ = stop(wire, shared);
        return Err(refused);
    }
    if let Err(refused) = guarded(wire, shared, chosen) {
        // Up without the password or the shorter life it was asked for is not what the person shared, so it goes.
        let _ = stop(wire, shared);
        return Err(refused);
    }
    Ok(())
}

/// The share takes up a document it cites, so that one lives as long as it does (decision 64).
pub(crate) fn hold(
    wire: &impl Wire,
    holder: &Shared,
    cited: &Shared,
    lives: bool,
) -> Result<(), Refusal> {
    let with = json!({ "id": cited.link, "admin": cited.admin, "lives": lives });
    managed(wire, holder, json!({ "do": "hold", "with": with }))
}

/// The share lets go of a document it no longer cites; one shared only through it goes on the server.
pub(crate) fn release(wire: &impl Wire, holder: &Shared, link: &str) -> Result<(), Refusal> {
    managed(wire, holder, json!({ "do": "release", "with": link }))
}

/// What goes up as a version: the words, the documents they lead to, their pictures, and who signs them.
pub(crate) struct Words<'a> {
    pub(crate) text: &'a str,
    pub(crate) cited: &'a [Cited],
    pub(crate) pictures: &'a [(Kind, Vec<u8>)],
    pub(crate) voucher: &'a Voucher,
    pub(crate) name: Option<&'a str>,
}

/// Puts the words up as the version after the one this store last saw, and says which it became (decision 48).
/// One put up since by anyone is never written over: the server says `behind`, and its words are brought first.
pub(crate) fn update(wire: &impl Wire, shared: &Shared, words: &Words) -> Result<u64, Refusal> {
    let next = shared
        .version
        .checked_add(1)
        .ok_or(Refusal::of("shareFailed"))?;
    put_up(wire, shared, Some(shared.version), words)?;
    Ok(next)
}

/// Whether the server turned an upload away because a newer version is there.
pub(crate) fn behind(refused: &Refusal) -> bool {
    refused.name.as_deref() == Some("behind")
}

/// The newest version's number, asked with the edit token, which a share behind a password needs.
pub(crate) fn latest(wire: &impl Wire, shared: &Shared) -> Result<Option<u64>, Refusal> {
    #[derive(Deserialize)]
    struct Latest {
        latest: Option<u64>,
    }
    let said: Latest = answered(get(
        wire,
        format!("docs/{}/latest", shared.link),
        Some(&shared.edit),
    )?)?;
    Ok(said.latest)
}

/// One version as the server keeps it, sealed, brought with the edit token so no password is asked.
pub(crate) fn version(wire: &impl Wire, shared: &Shared, number: u64) -> Result<Said, Refusal> {
    let said = get(
        wire,
        format!("docs/{}/versions/{number}", shared.link),
        Some(&shared.edit),
    )?;
    match (200..300).contains(&said.status) {
        true => Ok(said),
        false => Err(refused(&said)),
    }
}

// Seals the words with the link's own key against the server's limits, signs them and puts them up after `from`.
fn put_up(
    wire: &impl Wire,
    shared: &Shared,
    from: Option<u64>,
    words: &Words,
) -> Result<(), Refusal> {
    let limits: Limits = answered(get(wire, "limits".into(), None)?)?;
    let link_key = LinkKey::from_text(&shared.key).ok_or(Refusal::of("internal"))?;
    let (key, certificate) = words.voucher;
    // Everything shared is signed: the key is the only identity there is, since any alias can be taken.
    let signer = Signer {
        key,
        certificate,
        at: jiff::Timestamp::now(),
        name: words.name,
    };
    let pictures: Vec<(Kind, &[u8])> = words
        .pictures
        .iter()
        .map(|(kind, bytes)| (*kind, bytes.as_slice()))
        .collect();
    // The web shows only the page drawn here, so one past the server's limits is too big to share, not words alone.
    let html = tisty_markdown::html(&tisty_markdown::read(words.text));
    let draft = Draft {
        id: &shared.link,
        version: from.map_or(1, |one| one + 1),
        from,
        text: words.text,
        cited: words.cited,
        html: &html,
        config: configured(words.text),
        pictures: &pictures,
    };
    let sealed =
        envelope::seal(&link_key, &draft, Some(&signer), &limits).map_err(|why| match why {
            // Sealed against the server's own limits, so a document over them is told here, never by the server.
            Unsealed::TextTooBig
            | Unsealed::HtmlTooBig
            | Unsealed::PictureTooBig
            | Unsealed::PicturesTooBig
            | Unsealed::TooManyPictures => Refusal::of("shareTooBig"),
            _ => Refusal::of("shareFailed"),
        })?;
    let work = worked(
        wire,
        json!({
            "for": "upload",
            "id": shared.link,
            "digest": URL_SAFE_NO_PAD.encode(sealed.digest()),
            "bytes": sealed.as_bytes().len(),
        }),
    )?;
    wire.send(Asking {
        method: "PUT",
        path: format!("docs/{}", shared.link),
        token: Some(&shared.edit),
        work: Some(work),
        password: None,
        body: sealed.into_bytes(),
    })
    .and_then(done)
}

/// How this Tisty reads the words, so the reader shows the title, paper and properties without reading Markdown.
fn configured(text: &str) -> Config {
    let laid = front::laid(text);
    Config {
        title: Some(tisty_core::docs::titled(text)).filter(|one| !one.is_empty()),
        tags: front::tags(text),
        lang: front::lang(text),
        paper: laid.paper.map(str::to_owned),
        sheets: laid.sheets,
        landscape: laid.landscape,
        fields: front::fields(text)
            .into_iter()
            .map(|one| Field {
                key: one.key,
                values: one.values,
            })
            .collect(),
    }
    .trimmed()
}

/// The password and the shorter life the person chose, set once the first version is up.
fn guarded(wire: &impl Wire, shared: &Shared, chosen: &Chosen) -> Result<(), Refusal> {
    if let Some(password) = &chosen.password {
        let with = Some(password.clone());
        changed(wire, shared, &Change::Password { with })?;
    }
    if let Some(days) = chosen.idle_days {
        changed(wire, shared, &Change::Idle { days })?;
    }
    Ok(())
}

/// What the owner changes on a share already up, from its row in the column.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", tag = "do")]
pub(crate) enum Change {
    /// A new password, or none to open it with the link alone.
    Password {
        with: Option<String>,
    },
    /// Days without activity before it goes; the server keeps it within its own.
    Idle {
        days: u32,
    },
    /// Nobody can open it until it is resumed, and the server forgets it after its trash days (decision 51).
    Pause,
    Resume,
}

pub(crate) fn changed(wire: &impl Wire, shared: &Shared, change: &Change) -> Result<(), Refusal> {
    let what = match change {
        Change::Password { with } => json!({ "do": "password", "with": with }),
        Change::Idle { days } => json!({ "do": "idle", "with": i64::from(*days) * 86_400 }),
        Change::Pause => json!({ "do": "pause" }),
        Change::Resume => json!({ "do": "restore" }),
    };
    managed(wire, shared, what)
}

/// How it stands on the server: its reads, when it ends, and whether it is paused or blocked.
pub(crate) fn info(wire: &impl Wire, shared: &Shared) -> Result<Info, Refusal> {
    answered(get(
        wire,
        format!("docs/{}/info", shared.link),
        Some(&shared.admin),
    )?)
}

/// Its links stop leading anywhere; the document itself stays as it is (decision 48).
pub(crate) fn stop(wire: &impl Wire, shared: &Shared) -> Result<(), Refusal> {
    managed(wire, shared, json!({ "do": "delete" }))
}

/// The server turned it away for one of these reasons, by the name it gave.
pub(crate) fn refused_as(refused: &Refusal, names: &[&str]) -> bool {
    refused.code == "shareRefused"
        && refused
            .name
            .as_deref()
            .is_some_and(|name| names.contains(&name))
}

/// The server no longer holds it: it ran out, or was stopped from another machine.
pub(crate) fn already_gone(refused: &Refusal) -> bool {
    refused.code == "shareRefused" && matches!(refused.name.as_deref(), Some("missing" | "gone"))
}

/// Hands the server a root's word taking keys back, so readers stop seeing what they signed as holding (decision 73).
pub(crate) fn revoke(wire: &impl Wire, revoked: &[Revocation]) -> Result<(), Refusal> {
    // A few at once, so even a person who took back many machines asks the server only a few times.
    for some in revoked.chunks(REVOCATIONS_AT_ONCE) {
        let body = serde_json::to_vec(some).map_err(|_| Refusal::of("internal"))?;
        done(wire.send(Asking {
            method: "POST",
            path: "revocations".into(),
            token: None,
            work: None,
            password: None,
            body,
        })?)?;
    }
    Ok(())
}

const TOLD: &str = "revocations-told.json";

/// Every key this person took back, the root's own signed word on each.
pub(crate) fn taken_back_by(
    persons: &tisty_core::person::Persons,
    me: &tisty_share_proto::identity::Person,
) -> Vec<Revocation> {
    persons
        .revocations
        .iter()
        .filter(|one| one.unchecked().is_some_and(|(person, _)| person == *me))
        .cloned()
        .collect()
}

// A revocation is named by what it signs, so the same word said twice is told once.
fn told_name(revoked: &Revocation) -> String {
    serde_json::to_string(revoked).unwrap_or_default()
}

/// What of these the server has not heard yet, by the list this machine keeps of what it told.
pub(crate) fn untold(cache: &std::path::Path, revoked: Vec<Revocation>) -> Vec<Revocation> {
    let told: std::collections::HashSet<String> = std::fs::read(cache.join(TOLD))
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default();
    revoked
        .into_iter()
        .filter(|one| !told.contains(&told_name(one)))
        .collect()
}

fn heard(cache: &std::path::Path, revoked: &[Revocation]) {
    let at = cache.join(TOLD);
    let mut told: std::collections::BTreeSet<String> = std::fs::read(&at)
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default();
    told.extend(revoked.iter().map(told_name));
    if let Ok(said) = serde_json::to_vec(&told) {
        let _ = std::fs::write(at, said);
    }
}

/// Tells the server in the background what it has not heard; what does not reach it is tried again the next round.
pub(crate) fn revoke_untold(cache: std::path::PathBuf, revoked: Vec<Revocation>) {
    // The told list is read off the caller's thread, which may be holding the session.
    tauri::async_runtime::spawn_blocking(move || {
        let revoked = untold(&cache, revoked);
        if revoked.is_empty() {
            return;
        }
        match Net::to(SERVER).and_then(|wire| revoke(&wire, &revoked)) {
            Ok(()) => heard(&cache, &revoked),
            Err(refused) => tisty_core::witness::note(
                tisty_core::witness::channel::WINDOW,
                "Tisty Share did not hear that a key was taken back; the next round tells it",
                &[(
                    "why",
                    tisty_core::witness::Fact::Why(refused.code.to_string()),
                )],
            ),
        }
    });
}

/// The link to read it, and the one to edit it when it was handed out (decision 66).
pub(crate) fn links(shared: &Shared) -> (String, Option<String>) {
    let read = format!(
        "{}/{}#{}",
        shared.server.trim_end_matches('/'),
        shared.link,
        shared.key
    );
    let edit = shared.editable.then(|| format!("{read}.{}", shared.edit));
    (read, edit)
}

#[cfg(test)]
#[path = "sharing_here.rs"]
pub(crate) mod here;

#[cfg(test)]
#[path = "sharing_test.rs"]
mod tests;
