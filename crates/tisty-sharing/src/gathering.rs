use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tisty_contracts::{
    Attachment, Cited, Document, Editor, Orientation, Page, Paper, Review, Sealing, Sheet,
    Translation,
};
use tisty_core::model::{DocId, Kept};
use tisty_core::person::Persons;
use tisty_core::{Paths, State, docs, person};
use tisty_envelope::clean;
use tisty_envelope::family::{Family, Mine, Tongue, untouched};
use tisty_envelope::sealing::link_key;
use tisty_envelope::travelling::{Named, Places, Reached, body_ready, named, ready, words_in};
use tisty_share_proto::identity::{Certificate, Person, ShareKey};

use crate::packing::{Packed, Packing, Told};
use crate::{CARRIED_CITED, Remembered, Unshared, remembered};

const ID_CHARACTERS: usize = 22;
const TAGS_AT_MOST: usize = 64;

pub struct Desk<'a> {
    pub paths: &'a Paths,
    pub state: &'a State,
    pub also: Option<&'a Path>,
    pub turned: &'a dyn Fn(&Path) -> Option<Vec<u8>>,
    pub maker: Option<Person>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    pub doc: DocId,
    pub public: bool,
    pub cited: Vec<DocId>,
    pub signed_as: Option<String>,
    pub left_out: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offered {
    pub id: DocId,
    pub lang: String,
    pub anchor: bool,
    pub own: bool,
    pub left_out: bool,
}

#[derive(Default, Deserialize)]
struct Carried {
    #[serde(default, deserialize_with = "each_alone")]
    lang: Option<String>,
    #[serde(default, deserialize_with = "each_alone")]
    languages: Option<Vec<Translation>>,
    #[serde(default, deserialize_with = "each_alone")]
    cited: Option<Vec<Cited>>,
    #[serde(default, deserialize_with = "each_alone")]
    editors: Option<Vec<Editor>>,
    #[serde(default, deserialize_with = "each_alone")]
    reviews: Option<Vec<Review>>,
}

// One field that does not read leaves only itself out, never what the others carry.
fn each_alone<'de, D, T>(said: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let said = serde_json::Value::deserialize(said)?;
    Ok(serde_json::from_value(said).ok())
}

// A document shared again under a new link carries words others wrote under the last one, in any of its languages (140).
fn others_who_edited(
    desk: &Desk,
    kept: &Kept,
    shelf: &BTreeMap<String, Remembered>,
) -> Option<Told> {
    let state = desk.state;
    let mut names: Vec<String> = Vec::new();
    for one in languages_of(state, kept) {
        if one.guest || shelf.contains_key(&one.file) || sharable(desk, one.id, shelf).is_err() {
            continue;
        }
        let Some(said) = state.sharing_editors.get(&one.file) else {
            continue;
        };
        let editors: Vec<Editor> = serde_json::from_str(said).unwrap_or_default();
        for name in editors
            .into_iter()
            .filter(|one| one.owned != Some(true))
            .map(|one| one.name)
        {
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    (!names.is_empty()).then_some(Told::OthersEdited { editors: names })
}

fn carried_by(was: Option<&Remembered>) -> Carried {
    was.and_then(|one| one.carried.as_deref())
        .and_then(|said| serde_json::from_str(said).ok())
        .unwrap_or_default()
}

pub struct Under<'a> {
    pub mint: &'a dyn Fn() -> Result<String, Unshared>,
    pub editable: bool,
    pub terms: Option<&'a str>,
    pub languages_at_most: Option<usize>,
}

#[derive(Debug)]
pub struct Gathered {
    pub(crate) family: Family,
    pub(crate) cited: Vec<Family>,
    // Kept apart until sealing, so a window finishes each once however many languages name it.
    pub(crate) cites: BTreeMap<String, Vec<usize>>,
    pub(crate) told: Vec<Told>,
    pub(crate) remembering: Vec<(String, Remembered)>,
    pub(crate) titles: BTreeMap<String, String>,
    pub(crate) changed: BTreeSet<String>,
    pub(crate) granted: BTreeSet<String>,
    pub(crate) sealed_before: BTreeMap<String, Option<String>>,
}

impl Gathered {
    pub fn told(&self) -> &[Told] {
        &self.told
    }

    // Folio stops taking or serving what sits under its floor, so that goes again with no change of its own (155).
    pub fn sent_again_below(mut self, floors: &[String]) -> Self {
        for (id, sealed) in &self.sealed_before {
            if floors
                .iter()
                .any(|floor| tisty_envelope::reading::under_the_floor(sealed.as_deref(), floor))
            {
                self.changed.insert(id.clone());
            }
        }
        self
    }
}

pub fn signer(
    paths: &Paths,
    state: &State,
    device: &tisty_core::DeviceId,
) -> Option<(ShareKey, Certificate)> {
    signer_among(paths, &state.persons, device)
}

pub fn signer_among(
    paths: &Paths,
    persons: &Persons,
    device: &tisty_core::DeviceId,
) -> Option<(ShareKey, Certificate)> {
    person::signer(paths, persons, device)
}

fn goes_as_mine(desk: &Desk, kept: &Kept, edits_through_a_link: bool) -> bool {
    desk.state.mine(kept)
        || edits_through_a_link
        || desk.maker.is_some_and(|root| sown_by(desk, kept, root))
}

// A store can hold another person's machines, and one of them could write the mark too.
fn sown_by(desk: &Desk, kept: &Kept, root: Person) -> bool {
    kept.sown
        && kept
            .made_by
            .as_ref()
            .is_some_and(|device| desk.state.persons.answers_to(device).contains(&root))
}

fn sharable<'a>(
    desk: &Desk<'a>,
    id: DocId,
    shelf: &BTreeMap<String, Remembered>,
) -> Result<&'a Kept, Unshared> {
    let state = desk.state;
    let kept = state.docs.get(&id).ok_or(Unshared::NoSuchDoc)?;
    if kept.page_of.is_some() || kept.task_of.is_some() {
        return Err(Unshared::NotADocument);
    }
    let edits_through_a_link = shelf
        .get(&kept.file)
        .is_some_and(Remembered::edited_through_a_link);
    if !goes_as_mine(desk, kept, edits_through_a_link) {
        return Err(Unshared::NotOwn);
    }
    if state.shut(id) {
        return Err(Unshared::Shut);
    }
    if state.held_away(kept) {
        return Err(Unshared::Away);
    }
    Ok(kept)
}

const LANGUAGES_AT_MOST: usize = 20;

fn languages_of<'a>(state: &'a State, kept: &'a Kept) -> Vec<&'a Kept> {
    let group = state.tongues_of(kept.id);
    if group.iter().any(|one| one.id == kept.id) {
        group
    } else {
        vec![kept]
    }
}

fn carries_sown(state: &State, asked: &Asked) -> bool {
    std::iter::once(&asked.doc)
        .chain(&asked.cited)
        .filter_map(|id| state.docs.get(id))
        .flat_map(|kept| languages_of(state, kept))
        .any(|one| one.sown)
}

fn language_said(one: &Kept) -> String {
    one.lang
        .clone()
        .or_else(|| one.title.clone())
        .unwrap_or_default()
}

fn unread(e: impl std::fmt::Display) -> Unshared {
    Unshared::Unread(e.to_string())
}

fn sheet_of(body: &str) -> Option<Sheet> {
    let laid = tisty_markdown::front::laid(body);
    let paper: Option<Paper> = laid
        .paper
        .and_then(|one| serde_json::from_value(json!(one)).ok());
    let sheet = Sheet {
        paper,
        on: laid.sheets.then_some(true),
        orientation: laid.landscape.then_some(Orientation::Landscape),
    };
    (sheet.paper.is_some() || sheet.on.is_some()).then_some(sheet)
}

fn tags_of(body: &str) -> Option<Vec<String>> {
    let tags: Vec<String> = tisty_markdown::tags::tags_in(body)
        .into_iter()
        .filter(|one| clean::is_a_tag(one))
        .take(TAGS_AT_MOST)
        .collect();
    (!tags.is_empty()).then_some(tags)
}

// Read off the words as they travel: a first line holds whatever its words point at, and only its words go in a title.
fn title_of(travelling: &str) -> Option<String> {
    let said = match tisty_markdown::front::title(travelling) {
        Some(said) => said,
        None => words_in(&drawn(&docs::titled(travelling))),
    };
    clean::line(&said)
}

fn drawn(body: &str) -> String {
    tisty_markdown::html(&tisty_markdown::read(body))
}

fn print_of(said: &[Attachment], files: &[Vec<u8>]) -> Option<String> {
    if files.is_empty() {
        return None;
    }
    let mut print = Sha256::new();
    for (file, bytes) in said.iter().zip(files) {
        print.update(file.name.as_bytes());
        print.update((bytes.len() as u64).to_be_bytes());
        print.update(bytes);
    }
    Some(
        print
            .finalize()
            .iter()
            .map(|one| format!("{one:02x}"))
            .collect(),
    )
}

fn hex_of(said: impl AsRef<[u8]>) -> String {
    said.as_ref()
        .iter()
        .map(|one| format!("{one:02x}"))
        .collect()
}

fn words_print_of(
    body: &str,
    pages: &[WrittenPage],
    pictures: &[Vec<u8>],
    files: Option<&str>,
) -> String {
    let mut print = Sha256::new();
    print.update(body.as_bytes());
    for page in pages {
        print.update(page.body.as_bytes());
    }
    for picture in pictures {
        print.update(picture);
    }
    print.update(files.unwrap_or_default().as_bytes());
    hex_of(print.finalize())
}

fn new_id() -> Result<String, Unshared> {
    Ok(link_key()?.chars().take(ID_CHARACTERS).collect())
}

/// How a document is sealed this time, whether it keeps the files already sent, and whether Folio must
/// first learn the permit of a token it gets now, though Folio already holds it.
struct Prepared {
    sealing: Sealing,
    keeps_files: bool,
    granted: bool,
}

// Files stay in the envelope already sent only under the key that sealed it: another, or turning public or private, sends them again.
fn sealing_for(
    was: Option<&Remembered>,
    public: bool,
    print: Option<&str>,
    under: &Under,
    editable: bool,
) -> Result<Prepared, Unshared> {
    let key = match (public, was.and_then(|one| one.link_key.clone())) {
        (true, _) => None,
        (false, Some(kept)) => Some(kept),
        (false, None) => Some(link_key()?),
    };
    let kept = was
        .filter(|one| {
            print.is_some()
                && one.files_print.as_deref() == print
                && one.files_hash.is_some()
                && one.files_key == key
        })
        .and_then(|one| one.files_hash.clone());
    // A language already in Folio gets its token from its owner, who tells Folio its permit before sending (166).
    let owns_it = was.is_none_or(|one| one.admin_token.is_some());
    let (token, granted) = match was.and_then(|one| one.edit_token.clone()) {
        Some(kept) => (Some(kept), false),
        None if editable && !public && owns_it => (Some(link_key()?), was.is_some()),
        None => (None, false),
    };
    Ok(Prepared {
        sealing: Sealing {
            id: was.map_or_else(under.mint, |one| Ok(one.id.clone()))?,
            version: was.map_or(1, |one| one.version + 1),
            from: was.map(|one| one.version),
            link_key: key.clone(),
            files_key: kept.as_ref().and(key),
            files_hash: kept.clone(),
            owner: was
                .and_then(|one| one.owner.as_deref())
                .and_then(|said| serde_json::from_str(said).ok()),
            token,
            terms: under.terms.map(str::to_owned),
        },
        keeps_files: kept.is_some(),
        granted,
    })
}

// How the words of another find a document chosen to go along, which is gathered only once they name it.
struct Chosen {
    doc: DocId,
    files: Vec<String>,
    titles: Vec<String>,
}

impl Chosen {
    fn answers_to(&self, name: &str) -> bool {
        self.files.iter().any(|one| one == name)
            || self.titles.iter().any(|one| one.eq_ignore_ascii_case(name))
    }
}

struct WrittenPage {
    file: String,
    body: String,
    html: String,
}

fn pages_going(pages: &[WrittenPage], id: &str, places: &Places) -> Vec<Page> {
    pages
        .iter()
        .enumerate()
        .map(|(at, page)| {
            let travelling = body_ready(&page.body, places);
            Page {
                id: format!("p{}", at + 1),
                title: title_of(&travelling),
                tags: tags_of(&page.body),
                lang: tisty_markdown::front::lang(&page.body),
                sheet: sheet_of(&page.body),
                made: None,
                wrote: None,
                body: travelling,
                html: ready(&page.html, places),
                page_of: id.to_owned(),
                at: at as u64 + 1,
            }
        })
        .collect()
}

struct Gathering<'a> {
    desk: &'a Desk<'a>,
    under: &'a Under<'a>,
    editable_family: Option<DocId>,
    granted: BTreeSet<String>,
    shelf: BTreeMap<String, Remembered>,
    public: bool,
    signed_as: Option<String>,
    told: Vec<Told>,
    remembering: Vec<(String, Remembered)>,
    titles: BTreeMap<String, String>,
    cited: Vec<Family>,
    gathered_cited: BTreeMap<DocId, usize>,
    cites: BTreeMap<String, Vec<usize>>,
    left_out: Vec<String>,
    group: BTreeSet<String>,
}

impl Gathering<'_> {
    fn pages_of(&mut self, kept: &Kept, body: &str, shown: &str) -> Vec<WrittenPage> {
        let state = self.desk.state;
        let root = self.desk.paths.docs();
        let mut out = Vec::new();
        let edits_through_a_link = self
            .shelf
            .get(&kept.file)
            .is_some_and(Remembered::edited_through_a_link);
        for page in state.pages_read(kept.id, body) {
            let not_ours = !goes_as_mine(self.desk, page, edits_through_a_link);
            let stays = not_ours || state.shut(page.id) || state.held_away(page);
            let read = (!stays)
                .then(|| docs::read(&root, &page.file).ok())
                .flatten();
            match read {
                Some(body) => out.push(WrittenPage {
                    file: page.file.clone(),
                    html: drawn(&body),
                    body,
                }),
                None => self.told.push(Told::PageLeftBehind {
                    document: shown.to_owned(),
                }),
            }
        }
        out
    }

    fn packed(&mut self, shown: &str, all_named: &[Named]) -> Packed {
        let desk = self.desk;
        let mut packing = Packing::new(desk, shown, &mut self.told);
        for one in all_named {
            one.pictures.iter().for_each(|name| packing.picture(name));
        }
        for one in all_named {
            one.files.iter().for_each(|name| packing.file(name));
        }
        for one in all_named {
            one.links.iter().for_each(|name| packing.link(name));
        }
        packing.out
    }

    fn cited_place(
        &mut self,
        name: &str,
        chosen: &[Chosen],
        cites: &mut Vec<usize>,
    ) -> Result<Option<u64>, Unshared> {
        let Some(one) = chosen.iter().find(|one| one.answers_to(name)) else {
            return Ok(None);
        };
        let at = match self.gathered_cited.get(&one.doc) {
            Some(at) => *at,
            None => {
                let family = self.family(one.doc, &[])?;
                self.cited.push(family);
                let at = self.cited.len() - 1;
                self.gathered_cited.insert(one.doc, at);
                at
            }
        };
        let place = cites.iter().position(|was| *was == at).unwrap_or_else(|| {
            cites.push(at);
            cites.len() - 1
        });
        Ok(Some(place as u64))
    }

    fn tongue(&mut self, kept: &Kept, chosen: &[Chosen]) -> Result<Tongue, Unshared> {
        let body = docs::read(&self.desk.paths.docs(), &kept.file).map_err(unread)?;
        let shown = title_of(&body_ready(&body, &Places::default())).unwrap_or_default();
        let pages = self.pages_of(kept, &body, &shown);
        let html = drawn(&body);
        let mut all_named = vec![named(&html)];
        all_named.extend(pages.iter().map(|one| named(&one.html)));
        let mut packed = self.packed(&shown, &all_named);

        let mut cites = Vec::new();
        for name in all_named.iter().flat_map(|one| &one.documents) {
            let page = pages.iter().position(|page| {
                page.file == *name || docs::titled(&page.body).eq_ignore_ascii_case(name)
            });
            let carried = name
                .strip_prefix(CARRIED_CITED)
                .and_then(|at| at.parse().ok());
            let reached = match (page, carried) {
                (Some(at), _) => Some(Reached::Page(at as u64 + 1)),
                (None, Some(at)) => Some(Reached::Cited(at)),
                (None, None) => self
                    .cited_place(name, chosen, &mut cites)?
                    .map(Reached::Cited),
            };
            if let Some(reached) = reached {
                packed.places.documents.insert(name.clone(), reached);
            }
        }

        let print = print_of(&packed.said_files, &packed.files);
        let words_print = words_print_of(&body, &pages, &packed.pictures, print.as_deref());
        let was = self.shelf.get(&kept.file);
        if was
            .and_then(|one| one.sealed_under.as_deref())
            .is_some_and(tisty_envelope::reading::later_than_this)
        {
            return Err(Unshared::Newer);
        }
        let editable = self.editable_family == Some(self.desk.state.anchor_of(kept));
        let Prepared {
            sealing,
            keeps_files,
            granted,
        } = sealing_for(was, self.public, print.as_deref(), self.under, editable)?;
        if granted {
            self.granted.insert(sealing.id.clone());
        }
        let admin_token = match was {
            Some(one) => one.admin_token.clone(),
            None => Some(link_key()?),
        };
        let its_owner = was.and_then(|one| one.owner.clone());
        let carried = carried_by(was);
        let owner_name = match (&its_owner, was) {
            (Some(_), Some(one)) => one.owner_name.clone(),
            _ => self.signed_as.clone().filter(|_| !self.public),
        };
        self.remembering.push((
            kept.file.clone(),
            Remembered {
                id: sealing.id.clone(),
                version: sealing.version,
                link_key: sealing.link_key.clone(),
                files_print: print,
                files_hash: None,
                files_key: None,
                admin_token,
                owner: its_owner,
                owner_name: owner_name.clone(),
                cited: was.map_or_else(Vec::new, |one| one.cited.clone()),
                held_by: None,
                ttl_days: was.and_then(|one| one.ttl_days),
                carried: was.and_then(|one| one.carried.clone()),
                edit_token: sealing.token.clone(),
                words_print: Some(words_print),
                signed_as: self.signed_as.clone(),
                sealed_under: Some(tisty_contracts::VERSION.to_owned()),
                version_by: None,
                version_root: None,
                left_out: if self.group.contains(&kept.file) {
                    self.left_out.clone()
                } else {
                    Vec::new()
                },
            },
        ));
        self.titles.insert(kept.file.clone(), shown);
        if !cites.is_empty() {
            self.cites.insert(sealing.id.clone(), cites);
        }

        let places = &packed.places;
        let pages = pages_going(&pages, &sealing.id, places);
        let travelling = body_ready(&body, places);
        let document = Document {
            id: sealing.id.clone(),
            title: title_of(&travelling),
            tags: tags_of(&body),
            lang: tisty_markdown::front::lang(&body).or(carried.lang),
            sheet: sheet_of(&body),
            signed_as: self.signed_as.clone(),
            owner_name,
            made: None,
            wrote: None,
            body: travelling,
            html: ready(&html, places),
            pages: (!pages.is_empty()).then_some(pages),
            pictures: (!packed.said_pictures.is_empty()).then_some(packed.said_pictures),
            files: (!packed.said_files.is_empty()).then_some(packed.said_files),
            files_hash: None,
            cited: carried.cited,
            languages: carried.languages,
            anchor: None,
            reviews: carried.reviews,
            editors: carried.editors,
        };
        Ok(Tongue {
            document,
            pictures: packed.pictures,
            files: if keeps_files { vec![] } else { packed.files },
            sealing,
            cited: vec![],
        })
    }

    // The map also names languages others added, which this Tisty never seals, and those it keeps hold their place too.
    fn held_by_others(&self, kept: &Kept, anchor_id: DocId) -> usize {
        let group = languages_of(self.desk.state, kept);
        let anchor = group.iter().find(|one| one.id == anchor_id);
        let anchor_shared = anchor.and_then(|one| self.shelf.get(&one.file));
        let owner_writes = anchor_shared.is_none_or(|one| !one.edited_through_a_link());
        let mine: Vec<Mine> = group
            .iter()
            .filter(|one| one.id != anchor_id)
            .map(|one| Mine {
                id: self.shelf.get(&one.file).map(|shared| shared.id.as_str()),
                lang: one.lang.as_deref(),
            })
            .collect();
        let held = carried_by(anchor_shared).languages.unwrap_or_default();
        let anchor_lang = anchor.and_then(|one| one.lang.as_deref());
        untouched(held, anchor_lang, &mine, owner_writes).len()
    }

    fn family(&mut self, doc: DocId, chosen: &[Chosen]) -> Result<Family, Unshared> {
        let state = self.desk.state;
        let kept = sharable(self.desk, doc, &self.shelf)?;
        let anchor_id = state.anchor_of(kept);
        let mut going = Vec::new();
        for one in languages_of(state, kept) {
            let chosen_out =
                self.left_out.contains(&one.file) && one.id != kept.id && one.id != anchor_id;
            if chosen_out {
                continue;
            }
            if sharable(self.desk, one.id, &self.shelf).is_ok() {
                going.push(one);
            } else {
                self.told.push(Told::LanguageLeftBehind {
                    language: language_said(one),
                });
            }
        }
        // An anchor that stays here hands the map to the first language that goes; `kept` always goes.
        // Folio counts what it already holds whether it goes again or not, so those keep their place before anything new.
        let held_there = |one: &Kept| self.shelf.contains_key(&one.file);
        let must_go = |one: &Kept| one.id == kept.id || one.id == anchor_id || held_there(one);
        let taken = going.iter().filter(|one| must_go(one)).count()
            + languages_of(state, kept)
                .into_iter()
                .filter(|one| held_there(one) && !going.iter().any(|goes| goes.id == one.id))
                .count()
            + self.held_by_others(kept, anchor_id);
        let top = self
            .under
            .languages_at_most
            .map_or(LANGUAGES_AT_MOST, |most| most.min(LANGUAGES_AT_MOST));
        if taken > top {
            return Err(Unshared::LanguagesFull);
        }
        let mut room = top - taken;
        let (within, over): (Vec<&Kept>, Vec<&Kept>) = going.into_iter().partition(|one| {
            let goes = must_go(one) || room > 0;
            if goes && !must_go(one) {
                room -= 1;
            }
            goes
        });
        self.told
            .extend(over.into_iter().map(|one| Told::LanguageOverTheTop {
                language: language_said(one),
                top,
            }));
        let mut going = within;
        let at = going
            .iter()
            .position(|one| one.id == anchor_id)
            .unwrap_or(0);
        let anchor = self.tongue(going.remove(at), chosen)?;
        let mut others = Vec::with_capacity(going.len());
        for one in going {
            others.push(self.tongue(one, chosen)?);
        }
        self.printed_with_its_family(&anchor, &others);
        Ok(Family { anchor, others })
    }

    // The anchor's map names each language and every language wears the anchor's sheet, so a change there is one in them.
    fn printed_with_its_family(&mut self, anchor: &Tongue, others: &[Tongue]) {
        if others.is_empty() {
            return;
        }
        let map: Vec<(Option<&String>, &Option<String>, Option<&String>)> = others
            .iter()
            .map(|one| {
                let file = self.file_of(&one.sealing.id);
                (
                    file,
                    &one.document.lang,
                    file.and_then(|at| self.titles.get(at)),
                )
            })
            .collect();
        let map = serde_json::to_string(&map).unwrap_or_default();
        let sheet = serde_json::to_string(&anchor.document.sheet).unwrap_or_default();
        self.printed_with(&anchor.sealing.id, &map);
        for one in others {
            self.printed_with(&one.sealing.id, &sheet);
        }
    }

    fn file_of(&self, id: &str) -> Option<&String> {
        let found = self.remembering.iter().find(|(_, one)| one.id == id);
        found.map(|(file, _)| file)
    }

    fn printed_with(&mut self, id: &str, said: &str) {
        let Some((_, now)) = self.remembering.iter_mut().find(|(_, one)| one.id == id) else {
            return;
        };
        now.words_print = now.words_print.as_deref().map(|print| {
            let mut both = Sha256::new();
            both.update(print.as_bytes());
            both.update(said.as_bytes());
            hex_of(both.finalize())
        });
    }

    fn sealed_before(&self) -> BTreeMap<String, Option<String>> {
        self.remembering
            .iter()
            .filter_map(|(file, now)| {
                let was = self.shelf.get(file).filter(|was| was.id == now.id)?;
                Some((now.id.clone(), was.sealed_under.clone()))
            })
            .collect()
    }

    fn changed(&self, family: &Family) -> BTreeSet<String> {
        let mut fresh = BTreeSet::new();
        let mut changed = BTreeSet::new();
        for (file, now) in &self.remembering {
            match self.shelf.get(file).filter(|was| was.id == now.id) {
                None => {
                    fresh.insert(now.id.clone());
                    changed.insert(now.id.clone());
                }
                Some(was) if now.words_print.is_none() || was.words_print != now.words_print => {
                    changed.insert(now.id.clone());
                }
                Some(_) => {}
            }
        }
        changed.extend(self.granted.iter().cloned());
        // The map lives in the anchor, so a language new to it, or with a token new to it, sends the anchor too.
        for one in std::iter::once(family).chain(&self.cited) {
            if one.others.iter().any(|tongue| {
                fresh.contains(&tongue.sealing.id) || self.granted.contains(&tongue.sealing.id)
            }) {
                changed.insert(one.anchor.sealing.id.clone());
            }
        }
        for (citing, ats) in &self.cites {
            let names_a_change = ats
                .iter()
                .any(|at| changed.contains(&self.cited[*at].anchor.sealing.id));
            if names_a_change {
                changed.insert(citing.clone());
            }
        }
        changed
    }

    fn chosen(&self, kept: &Kept) -> Chosen {
        let state = self.desk.state;
        let root = self.desk.paths.docs();
        let members = languages_of(state, kept);
        Chosen {
            doc: kept.id,
            files: members.iter().map(|one| one.file.clone()).collect(),
            titles: members
                .iter()
                .filter_map(|one| docs::read(&root, &one.file).ok())
                .map(|body| docs::titled(&body))
                .collect(),
        }
    }
}

pub fn gathered(desk: &Desk, asked: &Asked) -> Result<Gathered, Unshared> {
    gathered_within(desk, asked, None)
}

pub fn gathered_within(
    desk: &Desk,
    asked: &Asked,
    languages_at_most: Option<usize>,
) -> Result<Gathered, Unshared> {
    gathered_under(
        desk,
        asked,
        &Under {
            mint: &new_id,
            editable: false,
            terms: None,
            languages_at_most,
        },
    )
}

// It sends nothing and remembers nothing.
pub fn gathered_under(desk: &Desk, asked: &Asked, under: &Under) -> Result<Gathered, Unshared> {
    let state = desk.state;
    let shelf = remembered(state).clone();
    let top = sharable(desk, asked.doc, &shelf)?;
    let anchor = state.anchor_of(top);
    let was = state.docs.get(&anchor).and_then(|one| shelf.get(&one.file));
    // Each language of what its owner shares to edit has its own token (166); what went out to read stays to read (148).
    let owner_edits = was.is_some_and(|one| one.admin_token.is_some() && one.edit_token.is_some());
    let editable_family = (owner_edits || (under.editable && was.is_none())).then_some(anchor);
    // What Tisty sowed goes out under the maker's alias, so nothing reads as the app speaking.
    let wanted = asked
        .signed_as
        .clone()
        .filter(|one| !one.trim().is_empty() && !carries_sown(state, asked))
        .or_else(|| state.signed.alias.clone());
    let signed_as = wanted.as_deref().and_then(clean::name);
    let mut gathering = Gathering {
        desk,
        under,
        editable_family,
        granted: BTreeSet::new(),
        shelf,
        public: asked.public,
        signed_as: signed_as.clone(),
        told: Vec::new(),
        remembering: Vec::new(),
        titles: BTreeMap::new(),
        cited: Vec::new(),
        gathered_cited: BTreeMap::new(),
        cites: BTreeMap::new(),
        left_out: asked.left_out.clone(),
        group: languages_of(state, top)
            .into_iter()
            .map(|one| one.file.clone())
            .collect(),
    };
    if wanted.is_some() && signed_as.is_none() {
        gathering.told.push(Told::NameLeftOut);
    }
    if let Some(others) = others_who_edited(desk, top, &gathering.shelf) {
        gathering.told.push(others);
    }
    // One family goes once, whichever of its languages was ticked and however many times.
    let mut seen = BTreeSet::from([state.anchor_of(top)]);
    let mut chosen = Vec::new();
    for doc in &asked.cited {
        let Some(kept) = state.docs.get(doc) else {
            continue;
        };
        // What cannot go stays as its words, as a language or a page that cannot: it never stops the rest.
        if sharable(desk, *doc, &gathering.shelf).is_err() {
            gathering.told.push(Told::CitedLeftBehind {
                document: kept.title.clone().unwrap_or_default(),
            });
        } else if seen.insert(state.anchor_of(kept)) {
            chosen.push(gathering.chosen(kept));
        }
    }
    let family = gathering.family(asked.doc, &chosen)?;
    let changed = gathering.changed(&family);
    let sealed_before = gathering.sealed_before();
    Ok(Gathered {
        changed,
        granted: gathering.granted.clone(),
        sealed_before,
        family,
        cited: gathering.cited,
        cites: gathering.cites,
        told: gathering.told,
        remembering: gathering.remembering,
        titles: gathering.titles,
    })
}

// Each language is one's own or not on its own (68), and the one asked for and the anchor always go.
pub fn languages_offered(
    desk: &Desk,
    doc: DocId,
    left_out: &[String],
) -> Result<Vec<Offered>, Unshared> {
    let shelf = remembered(desk.state);
    let kept = sharable(desk, doc, shelf)?;
    let anchor = desk.state.anchor_of(kept);
    Ok(languages_of(desk.state, kept)
        .into_iter()
        .map(|one| {
            let fixed = one.id == anchor || one.id == kept.id;
            Offered {
                id: one.id,
                lang: language_said(one),
                anchor: fixed,
                own: sharable(desk, one.id, shelf).is_ok(),
                left_out: !fixed && left_out.contains(&one.file),
            }
        })
        .collect())
}

// The group's choice is kept with whichever of its languages is shared, so any of them asked again starts from it.
pub fn left_out_of(state: &State, doc: DocId) -> Vec<String> {
    let Some(kept) = state.docs.get(&doc) else {
        return Vec::new();
    };
    let shelf = remembered(state);
    languages_of(state, kept)
        .into_iter()
        .filter_map(|one| shelf.get(&one.file))
        .map(|one| one.left_out.clone())
        .find(|one| !one.is_empty())
        .unwrap_or_default()
}

// Only what can go along is offered, and a page already goes with its document.
pub fn citable(desk: &Desk, doc: DocId) -> Result<Vec<DocId>, Unshared> {
    let state = desk.state;
    let shelf = remembered(state);
    let kept = sharable(desk, doc, shelf)?;
    let body = docs::read(&desk.paths.docs(), &kept.file).map_err(unread)?;
    let names = named(&drawn(&body)).documents;
    let pages = state.pages_read(kept.id, &body);
    let group = languages_of(state, kept);
    Ok(state
        .docs
        .values()
        .filter(|one| {
            !group.iter().any(|same| same.id == one.id)
                && !pages.iter().any(|page| page.id == one.id)
                && sharable(desk, one.id, shelf).is_ok()
                && names.iter().any(|name| {
                    *name == one.file
                        || one
                            .title
                            .as_deref()
                            .is_some_and(|title| title.eq_ignore_ascii_case(name))
                })
        })
        .map(|one| one.id)
        .collect())
}

#[cfg(test)]
#[path = "gathering_test.rs"]
mod tests;
