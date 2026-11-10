use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::folio_client::{self, Door, Fetched};
use tisty_carrier::{folio_domains, folio_limits};
use tisty_contracts::{EnvelopeKind, Opening, Served, Verdict};
use tisty_core::Op;
use tisty_core::attach::{self, Via};
use tisty_core::witness::{self, Fact, channel};
use tisty_envelope::Translation;
use tisty_envelope::reading::{self, Unopened};
use tisty_sharing::{Asked, Desk, Received, Remembered};

use super::folio::{ToFolio, said};
use super::folio_standing::{Party, standing_of};
use super::sharing::turned;
use crate::viewing::{LinkQuestion, Views};
use crate::{Answer, Refusal, Session, held};

pub(super) fn guest_page(
    session: &mut Session,
    body: &str,
    by: Option<String>,
    up: tisty_core::model::DocId,
) -> Answer<()> {
    let file = written(session, body)?;
    guest_added(session, file, body, by, Some(up)).map(|_| ())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub id: String,
    pub key: String,
    pub token: Option<String>,
}

pub fn link_of(web: &url::Url) -> Option<Link> {
    let id = web.path().trim_start_matches('/').to_owned();
    if id.len() != 8 || !id.chars().all(|one| one.is_ascii_alphanumeric()) {
        return None;
    }
    let fragment = web.fragment()?;
    let (key, token) = match fragment.split_once('.') {
        Some((key, token)) => (key, Some(token.to_owned())),
        None => (fragment, None),
    };
    (!key.is_empty()).then(|| Link {
        id,
        key: key.to_owned(),
        token,
    })
}

fn served_by(fetched: &Fetched) -> Served {
    let received = jiff::Timestamp::from_millisecond(fetched.arrived_ms)
        .ok()
        .map(|at| at.to_string());
    let revocations = fetched
        .revocations
        .as_deref()
        .and_then(|said| serde_json::from_str(said).ok());
    Served {
        received,
        revocations,
    }
}

fn unopened(wrong: &Unopened) -> Refusal {
    witness::warn(
        channel::WINDOW,
        "a shared document could not be opened",
        &[("why", Fact::Why(wrong.to_string()))],
    );
    match wrong {
        Unopened::Newer => Refusal::of("folioOutdated"),
        _ => Refusal::of("folioUnopened"),
    }
}

#[derive(Clone)]
pub(super) struct Opened {
    pub document: serde_json::Value,
    pub pictures: Vec<Vec<u8>>,
    pub files: Vec<Vec<u8>>,
    pub owner: Option<String>,
    pub owner_root: Option<String>,
    pub signer: String,
    pub version: u64,
    pub verdict: Verdict,
    pub contract: Option<String>,
}

impl Opened {
    pub(super) fn doubtful(&self) -> bool {
        self.verdict != Verdict::Holds
    }
}

pub(super) fn opened_from(door: &dyn Door, id: &str, key: &str) -> Answer<Opened> {
    opened_with(door, id, key, true)
}

fn opened_head(door: &dyn Door, id: &str, key: &str) -> Answer<Opened> {
    opened_with(door, id, key, false)
}

fn opened_with(door: &dyn Door, id: &str, key: &str, with_files: bool) -> Answer<Opened> {
    let fetched = folio_client::fetch(door, id).map_err(|stopped| said(&stopped))?;
    let served = served_by(&fetched);
    let opening = |kind| Opening {
        id: id.to_owned(),
        kind,
        link_key: Some(key.to_owned()),
    };
    let main = opening(EnvelopeKind::Main);
    let sealed = reading::opened(&fetched.bytes, &main, &served).map_err(|e| unopened(&e))?;
    let owner = reading::owner_block(&sealed);
    let contract = reading::contract_of(&sealed);
    let read = reading::reading(&fetched.bytes, &main, &served).map_err(|e| unopened(&e))?;
    let files = match read.document["files_hash"].as_str().filter(|_| with_files) {
        Some(hash) => {
            let bytes =
                folio_client::fetch_files(door, id, hash).map_err(|stopped| said(&stopped))?;
            reading::files_for(
                &read.document,
                &bytes,
                &opening(EnvelopeKind::Files),
                &served,
            )
            .map_err(|e| unopened(&e))?
            .parts
        }
        None => Vec::new(),
    };
    Ok(Opened {
        document: read.document,
        pictures: read.pictures,
        files,
        owner,
        owner_root: read.owner.clone(),
        signer: read.signer,
        version: fetched.version,
        verdict: read.verdict,
        contract,
    })
}

pub(super) fn kept_here(data: &std::path::Path, opened: &Opened) -> (Received, Vec<Op>) {
    let room = std::env::temp_dir().join(format!("tisty-received-{}", ulid::Ulid::generate()));
    let mut told = Vec::new();
    let received = tisty_sharing::received(
        &opened.document,
        &opened.pictures,
        &opened.files,
        &mut |name, bytes| {
            let leaf = std::path::Path::new(name).file_name()?;
            std::fs::create_dir_all(&room).ok()?;
            let beside = room.join(leaf);
            std::fs::write(&beside, bytes).ok()?;
            let kept = attach::keep(&beside, data, attach::COPIED_IN_DOC).ok()?;
            told.push(kept.told(Via::Window));
            Some(attach::leaf_of(&kept.at)?.to_owned())
        },
    );
    let _ = std::fs::remove_dir_all(&room);
    (received, told)
}

fn guest_added(
    session: &mut Session,
    file: String,
    body: &str,
    by: Option<String>,
    page_of: Option<tisty_core::model::DocId>,
) -> Answer<tisty_core::model::DocId> {
    // Only the guide Tisty sows speaks as Tisty; a name like it from Folio is kept as no name.
    let by = by.filter(|one| !tisty_core::state::same_name(one, tisty_core::untouched::SOWN_BY));
    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.page_of == page_of && one.folder.is_none())
            .map(|one| one.order.as_str()),
    );
    let id = ulid::Ulid::generate();
    session.commit(Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: true,
            sown: false,
            made: None,
            by,
            file,
            order,
            said: Some(tisty_core::event::Said::of(body)),
            folder: None,
            page_of,
            task_of: None,
        },
    })?;
    Ok(id)
}

fn written(session: &Session, body: &str) -> Answer<String> {
    tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body)
        .map(|made| made.id)
        .map_err(|_| Refusal::of("folioGarbled"))
}

// Pages are written first, so the document's words can point at them as this store does.
pub(super) fn guests_kept(
    session: &mut Session,
    received: &Received,
    by: Option<String>,
) -> Answer<(tisty_core::model::DocId, String)> {
    let files = received
        .pages
        .iter()
        .map(|page| written(session, page))
        .collect::<Answer<Vec<_>>>()?;
    let body = files
        .iter()
        .enumerate()
        .rev()
        .fold(received.body.clone(), |text, (at, file)| {
            text.replace(
                &format!("tisty:page/{}", at + 1),
                &format!("tisty:doc/{file}"),
            )
        });
    let file = written(session, &body)?;
    let doc = guest_added(session, file.clone(), &body, by.clone(), None)?;
    for (page, its) in received.pages.iter().zip(files) {
        guest_added(session, its, page, by.clone(), Some(doc))?;
    }
    Ok((doc, file))
}

fn entry_for(link: &Link, opened: &Opened, received: &Received) -> Remembered {
    let files_hash = opened.document["files_hash"].as_str().map(str::to_owned);
    Remembered {
        id: link.id.clone(),
        version: opened.version,
        link_key: Some(link.key.clone()),
        files_print: None,
        files_key: files_hash.as_ref().map(|_| link.key.clone()),
        files_hash,
        admin_token: None,
        edit_token: link.token.clone(),
        words_print: None,
        signed_as: None,
        owner: opened.owner.clone(),
        owner_name: received.owner_name.clone(),
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: received.carried.clone(),
        sealed_under: opened.contract.clone(),
        version_by: owner_apart(opened).and(received.signed_as.clone()),
        version_root: owner_apart(opened).map(|_| opened.signer.clone()),
        left_out: Vec::new(),
    }
}

// Reading a document whole takes a while, so it never holds the session, and it is read once for all its languages.
fn with_their_prints(
    app: &AppHandle,
    doc: tisty_core::model::DocId,
    entries: Vec<(String, Remembered)>,
) {
    if entries.is_empty() {
        return;
    }
    let session = app.state::<Mutex<Session>>();
    let (paths, state, also, device) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.state.clone(),
            session.dest(),
            session.config.device_id.clone(),
        )
    };
    let desk = Desk {
        paths: &paths,
        state: &state,
        also: also.as_deref(),
        turned: &turned,
        maker: super::sharing::the_maker_here(&paths, &state, &device),
    };
    let asked = Asked {
        doc,
        public: false,
        cited: Vec::new(),
        signed_as: None,
        left_out: Vec::new(),
    };
    let gathered = tisty_sharing::gathered(&desk, &asked).ok();
    let printed: Vec<(String, Remembered)> = entries
        .into_iter()
        .map(|(file, mut entry)| {
            entry.words_print = gathered.as_ref().and_then(|one| one.print_of(&file));
            (file, entry)
        })
        .collect();
    let _ = held(&session).commit_all(tisty_sharing::sent(&printed));
}

// A token the owner's block does not name is no edit link: it is turned away, never kept as one to read in silence.
fn editable(link: &Link, opened: &Opened) -> bool {
    link.token.as_deref().is_some_and(|token| {
        super::folio_editing::named_by_the_block(opened.owner.as_deref(), token)
    })
}

/// A document has at most twenty languages, its anchor among them (69 bis).
const OTHER_LANGUAGES_AT_MOST: usize = 19;

/// Every language the anchor's map names that is not here yet and Folio still serves, read before anything is kept.
/// One its owner has not reviewed stays out: it is someone else's word in the owner's name (166).
fn languages_brought(
    door: &dyn Door,
    opened: &Opened,
    here: &std::collections::BTreeSet<String>,
) -> Vec<(Translation, Opened)> {
    let map: Vec<Translation> =
        serde_json::from_value(opened.document["languages"].clone()).unwrap_or_default();
    map.into_iter()
        .filter(|one| !here.contains(&one.lang))
        .filter_map(|one| {
            let key = one.key.clone()?;
            match opened_from(door, &one.document, &key) {
                Ok(its) if belongs(opened, &its) => Some((one, its)),
                Ok(_) => {
                    witness::note(
                        channel::SYNC,
                        "a language that is not this anchor's, or that its owner has not reviewed, was left in Folio",
                        &[],
                    );
                    None
                }
                Err(refused) => {
                    witness::warn(
                        channel::SYNC,
                        "a language of a document arrived without being read, so it was left behind",
                        &[("code", Fact::Code(refused.code))],
                    );
                    None
                }
            }
        })
        .take(OTHER_LANGUAGES_AT_MOST)
        .collect()
}

fn reviewed(anchor: &Opened, its: &Opened) -> bool {
    anchor.owner_root.is_some() && anchor.owner_root == its.owner_root
}

// A map could name any document of its owner, so a language counts only when it names this anchor itself.
fn belongs(anchor: &Opened, its: &Opened) -> bool {
    let named = its.document["anchor"].as_str();
    named.is_some() && named == anchor.document["id"].as_str() && reviewed(anchor, its)
}

// Only the anchor's edit link opens a language's token, and only one the language's owner names (166).
fn language_link(link: &Link, one: &Translation, its: &Opened) -> Option<Link> {
    let anchor = link.token.as_deref()?;
    let key = one.key.clone()?;
    let token = tisty_envelope::tokens::opened_with(anchor, &one.document, one.edit.as_deref()?)?;
    super::folio_editing::named_by_the_block(its.owner.as_deref(), &token).then(|| Link {
        id: one.document.clone(),
        key,
        token: Some(token),
    })
}

// Languages join by what each says it is, so one that arrived without saying it in its words says it now.
fn saying_its_language(mut arrived: Received, lang: Option<&str>) -> Received {
    if let Some(lang) = lang.filter(|_| tisty_markdown::front::lang(&arrived.body).is_none()) {
        arrived.body = tisty_markdown::front::with_field(&arrived.body, "lang", lang);
    }
    arrived
}

fn speaks(lang: Option<&str>, locale: Option<&str>) -> bool {
    let first = |said: &str| {
        said.split(['-', '_'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    let locale = tisty_core::model::spoken(locale);
    match (lang, Some(locale.as_str())) {
        (Some(lang), Some(locale)) => first(lang) == first(locale),
        _ => false,
    }
}

fn taken_in(app: &AppHandle, door: &dyn Door, link: &Link, opened: &Opened) -> Answer<String> {
    let session = app.state::<Mutex<Session>>();
    let data = held(&session).paths.data().to_path_buf();
    let languages = languages_brought(door, opened, &std::collections::BTreeSet::new());
    let (received, told) = kept_here(&data, opened);
    let received = match languages.is_empty() {
        true => received,
        false => saying_its_language(received, opened.document["lang"].as_str()),
    };
    let by = received.owner_name.clone().or(received.signed_as.clone());
    let editable = editable(link, opened);
    let (doc, file) = {
        let mut session = held(&session);
        session.commit_all(told)?;
        let (doc, file) = guests_kept(&mut session, &received, by)?;
        // A link to read only leaves a copy of one's own to change: nothing of it goes back (80).
        if editable {
            let entry = (file.clone(), entry_for(link, opened, &received));
            session.commit_all(tisty_sharing::sent(&[entry]))?;
        }
        (doc, file)
    };
    let mut printing = Vec::new();
    if editable {
        printing.push((file.clone(), entry_for(link, opened, &received)));
    }
    let (shown, theirs) = languages_kept(app, link, opened, doc, languages)?;
    printing.extend(theirs);
    with_their_prints(app, doc, printing);
    Ok(shown.unwrap_or(file))
}

/// The language Tisty speaks, when one arrived, and the edit links kept for the languages.
type Kept = (Option<String>, Vec<(String, Remembered)>);

/// Keeps each language beside the anchor already here, and says which one Tisty speaks and which edit links it holds.
fn languages_kept(
    app: &AppHandle,
    link: &Link,
    opened: &Opened,
    doc: tisty_core::model::DocId,
    languages: Vec<(Translation, Opened)>,
) -> Answer<Kept> {
    let session = app.state::<Mutex<Session>>();
    let data = held(&session).paths.data().to_path_buf();
    let locale = held(&session).locale.clone();
    let editable = editable(link, opened);
    let mut shown = None;
    let mut printing = Vec::new();
    for (one, its) in languages {
        // One language that cannot be kept leaves the others, and what was kept is still printed.
        let kept = (|| {
            let (arrived, told) = kept_here(&data, &its);
            let arrived = saying_its_language(arrived, Some(&one.lang));
            let by = arrived.owner_name.clone().or(arrived.signed_as.clone());
            let mut session = held(&session);
            session.commit_all(told)?;
            let (its_doc, its_file) = guests_kept(&mut session, &arrived, by)?;
            let joined = session.state.joining(its_doc, doc);
            session.commit_all(joined)?;
            let entry = match editable.then(|| language_link(link, &one, &its)).flatten() {
                Some(its_link) => {
                    let mut entry = entry_for(&its_link, &its, &arrived);
                    entry.held_by = Some(link.id.clone());
                    session
                        .commit_all(tisty_sharing::sent(&[(its_file.clone(), entry.clone())]))?;
                    Some(entry)
                }
                None => None,
            };
            Answer::Ok((its_file, entry))
        })();
        let (its_file, entry) = match kept {
            Ok(kept) => kept,
            Err(refused) => {
                witness::warn(
                    channel::SYNC,
                    "a language of a document could not be kept here, and the others were",
                    &[("code", Fact::Code(refused.code))],
                );
                continue;
            }
        };
        if let Some(entry) = entry {
            printing.push((its_file.clone(), entry));
        }
        if speaks(Some(&one.lang), locale.as_deref())
            && !speaks(opened.document["lang"].as_str(), locale.as_deref())
        {
            shown = Some(its_file);
        }
    }
    Ok((shown, printing))
}

// A link opened again asks before Folio hears of it, brings the languages added since, and keeps the copy here as it is.
fn languages_added_since(
    app: &AppHandle,
    door: &dyn Door,
    link: &Link,
    file: &str,
) -> Option<String> {
    if !question(app, LinkQuestion::Look) {
        return None;
    }
    let (doc, here) = {
        let session = app.state::<Mutex<Session>>();
        let session = held(&session);
        let kept = session.state.docs.values().find(|one| one.file == file)?;
        let here: std::collections::BTreeSet<String> = session
            .state
            .tongues_of(kept.id)
            .into_iter()
            .filter_map(|one| one.lang.clone())
            .collect();
        (session.state.anchor_of(kept), here)
    };
    let opened = opened_head(door, &link.id, &link.key).ok()?;
    let languages = languages_brought(door, &opened, &here);
    if languages.is_empty() {
        return None;
    }
    let (shown, mut theirs) = languages_kept(app, link, &opened, doc, languages).ok()?;
    // The anchor's map gained them, so its print is taken again with theirs.
    let anchor_entry = {
        let session = app.state::<Mutex<Session>>();
        let session = held(&session);
        tisty_sharing::remembered(&session.state).get(file).cloned()
    };
    theirs.extend(anchor_entry.map(|one| (file.to_owned(), one)));
    with_their_prints(app, doc, theirs);
    shown
}

const DOMAIN_PATIENCE: std::time::Duration = std::time::Duration::from_secs(2);

type Checked = std::collections::BTreeSet<(String, String)>;

// A checked domain is a word added beside a name: every lookup shares one short deadline, so a slow network never holds the question up.
fn checked_within(home: &std::path::Path, wanted: &[(&str, Option<&str>)]) -> Checked {
    let (said, heard) = std::sync::mpsc::channel();
    for (root, alias) in wanted {
        let Some(domain) = alias.filter(|one| folio_domains::a_domain(one)) else {
            continue;
        };
        let (home, root, domain, said) = (
            home.to_path_buf(),
            root.to_string(),
            domain.to_owned(),
            said.clone(),
        );
        std::thread::spawn(move || {
            let held =
                folio_domains::checked(&home, &root, &domain, crate::reaching::brought_from_folio);
            let _ = said.send(held.then_some((root, domain)));
        });
    }
    drop(said);
    let deadline = std::time::Instant::now() + DOMAIN_PATIENCE;
    let mut checked = Checked::new();
    while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
        match heard.recv_timeout(left) {
            Ok(Some(found)) => {
                checked.insert(found);
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }
    checked
}

// Owner and signer are told apart by their roots: two people can sign with one name.
pub(super) fn owner_apart(opened: &Opened) -> Option<&str> {
    opened
        .owner_root
        .as_deref()
        .filter(|root| *root != opened.signer)
}

fn asked_to_keep(app: &AppHandle, opened: &Opened) -> bool {
    let said = |field: &str| opened.document[field].as_str();
    let home = held(&app.state::<Mutex<Session>>()).paths.private();
    let maker = folio_limits::maker(&home);
    let owner = owner_apart(opened);
    let mut wanted = vec![(opened.signer.as_str(), said("signed_as"))];
    wanted.extend(owner.map(|root| (root, said("owner_name"))));
    let checked = checked_within(&home, &wanted);
    let party = |root: &str, alias: Option<&str>| Party {
        name: alias.unwrap_or_default().to_owned(),
        standing: standing_of(
            root,
            opened.verdict,
            alias,
            maker.as_ref(),
            |root, domain| checked.contains(&(root.to_owned(), domain.to_owned())),
        ),
    };
    question(
        app,
        LinkQuestion::Keep {
            title: said("title").unwrap_or_default().to_owned(),
            owner: owner.map(|root| party(root, said("owner_name"))),
            signer: party(&opened.signer, said("signed_as")),
            print: opened.signer.clone(),
            doubtful: opened.doubtful(),
        },
    )
}

// It waits for the person, and the window is what asks: a link can arrive while Tisty is out of sight.
fn question(app: &AppHandle, asked: LinkQuestion) -> bool {
    if app.get_webview_window("main").is_none() {
        return false;
    }
    crate::tray::surface(app);
    let answer = app.state::<Views>().ask_of_a_link(asked);
    let _ = app.emit_to("main", "linkquestion", ());
    answer.recv().unwrap_or(false)
}

// Any page can fire a link, and the request tells Folio this computer's network address, so nothing goes out unasked.
fn looked_for(
    door: &dyn Door,
    link: &Link,
    asked_first: impl FnOnce() -> bool,
) -> Answer<Option<Opened>> {
    if !asked_first() {
        return Ok(None);
    }
    opened_from(door, &link.id, &link.key).map(Some)
}

fn already_here(app: &AppHandle, id: &str) -> Option<String> {
    let session = app.try_state::<Mutex<Session>>()?;
    let session = held(&session);
    tisty_sharing::remembered(&session.state)
        .iter()
        .find(|(_, one)| one.id == id)
        .map(|(file, _)| file.clone())
}

fn brought(app: &AppHandle, web: &url::Url) -> Answer<Option<String>> {
    let link = link_of(web).ok_or_else(|| Refusal::of("folioUnopened"))?;
    let home = held(&app.state::<Mutex<Session>>()).paths.private();
    if let Some(file) = already_here(app, &link.id) {
        let added = ToFolio::new(&home)
            .ok()
            .and_then(|door| languages_added_since(app, &door, &link, &file));
        return Ok(Some(added.unwrap_or(file)));
    }
    let door = ToFolio::new(&home)?;
    let Some(opened) = looked_for(&door, &link, || question(app, LinkQuestion::Look))? else {
        return Ok(None);
    };
    if link.token.is_some() && !editable(&link, &opened) {
        return Err(Refusal::of("folioEditLinkMismatch"));
    }
    if !asked_to_keep(app, &opened) {
        return Ok(None);
    }
    taken_in(app, &door, &link, &opened).map(Some)
}

fn bring_one(app: &AppHandle, web: &url::Url) {
    match brought(app, web) {
        Ok(Some(file)) => {
            let _ = app.emit_to("main", "showdoc", file);
            crate::tray::surface(app);
        }
        Ok(None) => {}
        Err(refused) => {
            question(
                app,
                LinkQuestion::Failed {
                    code: refused.code.to_owned(),
                },
            );
        }
    }
}

pub fn bring_apart(app: &AppHandle, web: url::Url) {
    if !app.state::<Views>().link_arrived(web) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Views>()
            .go_through_links(|web| bring_one(&app, web));
    });
}

#[cfg(test)]
#[path = "folio_receiving_test.rs"]
mod tests;
