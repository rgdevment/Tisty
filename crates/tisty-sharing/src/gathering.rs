use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tisty_contracts::{
    Attachment, Cited, Document, Orientation, Page, Paper, Sealing, Sheet, Translation,
};
use tisty_core::model::{DocId, Kept};
use tisty_core::{Paths, State, docs, person};
use tisty_envelope::clean;
use tisty_envelope::family::{Family, Tongue};
use tisty_envelope::sealing::link_key;
use tisty_envelope::travelling::{Named, Places, Reached, body_ready, named, ready, words_in};
use tisty_share_proto::identity::{Certificate, ShareKey};

use crate::packing::{Packed, Packing, Told};
use crate::{CARRIED_CITED, Remembered, Unshared, remembered};

const ID_CHARACTERS: usize = 22;
const TAGS_AT_MOST: usize = 64;
const LANGUAGES_AT_MOST: usize = 20;

pub struct Desk<'a> {
    pub paths: &'a Paths,
    pub state: &'a State,
    pub also: Option<&'a Path>,
    pub turned: &'a dyn Fn(&Path) -> Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    pub doc: DocId,
    pub public: bool,
    pub cited: Vec<DocId>,
    pub signed_as: Option<String>,
}

#[derive(Default, Deserialize)]
struct Carried {
    lang: Option<String>,
    languages: Option<Vec<Translation>>,
    cited: Option<Vec<Cited>>,
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
}

impl Gathered {
    pub fn told(&self) -> &[Told] {
        &self.told
    }
}

// A machine signs only as the root it answers to (72).
pub fn signer(
    paths: &Paths,
    state: &State,
    device: &tisty_core::DeviceId,
) -> Option<(ShareKey, Certificate)> {
    // Asked last: a machine makes its key only once the log names a person.
    let root = state.persons.person(device)?;
    let vouched = state.persons.vouched_by(device, &root)?;
    let key = person::mine(paths, device)?;
    if vouched.share != key.public() {
        return None;
    }
    let certificate = state.persons.certificates.iter().find(|one| {
        one.unchecked().is_some_and(|said| {
            said.person == root && said.share == vouched.share && said.since == vouched.since
        })
    })?;
    Some((key, certificate.clone()))
}

fn sharable<'a>(
    state: &'a State,
    id: DocId,
    shelf: &BTreeMap<String, Remembered>,
) -> Result<&'a Kept, Unshared> {
    let kept = state.docs.get(&id).ok_or(Unshared::NoSuchDoc)?;
    if kept.page_of.is_some() || kept.task_of.is_some() {
        return Err(Unshared::NotADocument);
    }
    let edits_through_a_link = shelf
        .get(&kept.file)
        .is_some_and(|one| one.edit_token.is_some());
    if kept.guest && !edits_through_a_link {
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

fn languages_of<'a>(state: &'a State, kept: &'a Kept) -> Vec<&'a Kept> {
    let group = state.tongues_of(kept.id);
    if group.iter().any(|one| one.id == kept.id) {
        group
    } else {
        vec![kept]
    }
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

// Files stay in the envelope already sent only under the key that sealed it: another, or turning public or private, sends them again.
fn sealing_for(
    was: Option<&Remembered>,
    public: bool,
    print: Option<&str>,
    under: &Under,
    editable: bool,
) -> Result<(Sealing, bool), Unshared> {
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
    // Folio never changes who may edit once a document is there, so only a first version can grant it.
    let token = match was {
        Some(one) => one.edit_token.clone(),
        None if editable && !public => Some(link_key()?),
        None => None,
    };
    Ok((
        Sealing {
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
        kept.is_some(),
    ))
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
    editable_anchor: Option<DocId>,
    shelf: BTreeMap<String, Remembered>,
    public: bool,
    signed_as: Option<String>,
    told: Vec<Told>,
    remembering: Vec<(String, Remembered)>,
    titles: BTreeMap<String, String>,
    cited: Vec<Family>,
    gathered_cited: BTreeMap<DocId, usize>,
    cites: BTreeMap<String, Vec<usize>>,
}

impl Gathering<'_> {
    fn pages_of(&mut self, kept: &Kept, body: &str, shown: &str) -> Vec<WrittenPage> {
        let state = self.desk.state;
        let root = self.desk.paths.docs();
        let mut out = Vec::new();
        let edits_through_a_link = self
            .shelf
            .get(&kept.file)
            .is_some_and(|one| one.edit_token.is_some());
        for page in state.pages_read(kept.id, body) {
            let not_ours = page.guest && !edits_through_a_link;
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
        let editable = self.editable_anchor == Some(kept.id);
        let (sealing, keeps_files) =
            sealing_for(was, self.public, print.as_deref(), self.under, editable)?;
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
        };
        Ok(Tongue {
            document,
            pictures: packed.pictures,
            files: if keeps_files { vec![] } else { packed.files },
            sealing,
            cited: vec![],
        })
    }

    fn family(&mut self, doc: DocId, chosen: &[Chosen]) -> Result<Family, Unshared> {
        let state = self.desk.state;
        let kept = sharable(state, doc, &self.shelf)?;
        let mut going = Vec::new();
        for one in languages_of(state, kept) {
            if sharable(state, one.id, &self.shelf).is_ok() {
                going.push(one);
            } else {
                self.told.push(Told::LanguageLeftBehind {
                    language: language_said(one),
                });
            }
        }
        // An anchor that stays here hands the map to the first language that goes; `kept` always goes.
        let anchor_id = state.anchor_of(kept);
        // Folio counts what it already holds whether it goes again or not, so those keep their place before anything new.
        let held_there = |one: &Kept| self.shelf.contains_key(&one.file);
        let must_go = |one: &Kept| one.id == kept.id || one.id == anchor_id || held_there(one);
        let taken = going.iter().filter(|one| must_go(one)).count()
            + languages_of(state, kept)
                .into_iter()
                .filter(|one| held_there(one) && !going.iter().any(|goes| goes.id == one.id))
                .count();
        let top = self.under.languages_at_most.unwrap_or(LANGUAGES_AT_MOST);
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
        Ok(Family { anchor, others })
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
    let top = sharable(state, asked.doc, &shelf)?;
    let editable_anchor = under.editable.then(|| state.anchor_of(top));
    let wanted = asked
        .signed_as
        .clone()
        .filter(|one| !one.trim().is_empty())
        .or_else(|| state.signed.alias.clone());
    let signed_as = wanted.as_deref().and_then(clean::name);
    let mut gathering = Gathering {
        desk,
        under,
        editable_anchor,
        shelf,
        public: asked.public,
        signed_as: signed_as.clone(),
        told: Vec::new(),
        remembering: Vec::new(),
        titles: BTreeMap::new(),
        cited: Vec::new(),
        gathered_cited: BTreeMap::new(),
        cites: BTreeMap::new(),
    };
    if wanted.is_some() && signed_as.is_none() {
        gathering.told.push(Told::NameLeftOut);
    }
    // One family goes once, whichever of its languages was ticked and however many times.
    let mut seen = BTreeSet::from([state.anchor_of(top)]);
    let mut chosen = Vec::new();
    for doc in &asked.cited {
        let Some(kept) = state.docs.get(doc) else {
            continue;
        };
        // What cannot go stays as its words, as a language or a page that cannot: it never stops the rest.
        if sharable(state, *doc, &gathering.shelf).is_err() {
            gathering.told.push(Told::CitedLeftBehind {
                document: kept.title.clone().unwrap_or_default(),
            });
        } else if seen.insert(state.anchor_of(kept)) {
            chosen.push(gathering.chosen(kept));
        }
    }
    let family = gathering.family(asked.doc, &chosen)?;
    Ok(Gathered {
        family,
        cited: gathering.cited,
        cites: gathering.cites,
        told: gathering.told,
        remembering: gathering.remembering,
        titles: gathering.titles,
    })
}

// Only what can go along is offered, and a page already goes with its document.
pub fn citable(desk: &Desk, doc: DocId) -> Result<Vec<DocId>, Unshared> {
    let state = desk.state;
    let shelf = remembered(state);
    let kept = sharable(state, doc, shelf)?;
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
                && sharable(state, one.id, shelf).is_ok()
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
