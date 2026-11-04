use std::collections::BTreeMap;
use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};
use tisty_contracts::{Attachment, Document, Orientation, Page, Paper, Sealing, Sheet};
use tisty_core::model::{DocId, Kept};
use tisty_core::{Paths, State, docs, person};
use tisty_envelope::clean;
use tisty_envelope::family::{Family, Tongue};
use tisty_envelope::sealing::link_key;
use tisty_envelope::travelling::{Named, Reached, body_ready, named, ready, words_in};
use tisty_share_proto::identity::{Certificate, ShareKey};

use crate::packing::{Packed, Packing, Told};
use crate::{Remembered, Unshared, remembered};

const ID_CHARACTERS: usize = 22;
const TAGS_AT_MOST: usize = 64;

/// Where a document is found.
pub struct Desk<'a> {
    pub paths: &'a Paths,
    pub state: &'a State,
    /// The other folder attachments may rest in, when the person keeps them apart.
    pub also: Option<&'a Path>,
    /// Asks the machine for what only it can read, as a HEIC.
    pub turned: &'a dyn Fn(&Path) -> Option<Vec<u8>>,
}

/// What the person chose to share.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    pub doc: DocId,
    pub public: bool,
    /// What goes along. Anything else a document names travels as its words.
    pub cited: Vec<DocId>,
    /// The name it is signed with, instead of the alias.
    pub signed_as: Option<String>,
}

/// A document and all it holds, with everything it names in place, waiting for a window.
#[derive(Debug)]
pub struct Gathered {
    pub(crate) family: Family,
    pub(crate) told: Vec<Told>,
    pub(crate) remembering: Vec<(String, Remembered)>,
    pub(crate) files: BTreeMap<String, String>,
    pub(crate) titles: BTreeMap<String, String>,
}

impl Gathered {
    pub fn told(&self) -> &[Told] {
        &self.told
    }
}

/// This machine's key and the certificate of the root it answers to, if it answers to one yet. Sign only as that root (72).
pub fn signer(
    paths: &Paths,
    state: &State,
    device: &tisty_core::DeviceId,
) -> Option<(ShareKey, Certificate)> {
    let key = person::mine(paths, device)?;
    let root = state.persons.person(device)?;
    let vouched = state.persons.vouched_by(device, &root)?;
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

fn sharable(state: &State, id: DocId) -> Result<&Kept, Unshared> {
    let kept = state.docs.get(&id).ok_or(Unshared::NoSuchDoc)?;
    if kept.page_of.is_some() || kept.task_of.is_some() {
        return Err(Unshared::NotADocument);
    }
    if kept.guest {
        return Err(Unshared::NotOwn);
    }
    if state.shut(id) {
        return Err(Unshared::Shut);
    }
    Ok(kept)
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

fn new_id() -> Result<String, Unshared> {
    Ok(link_key()?.chars().take(ID_CHARACTERS).collect())
}

// Files stay in the envelope already sent only under the key that sealed it: another, or turning public or private, sends them again.
fn sealing_for(
    was: Option<&Remembered>,
    public: bool,
    print: Option<&str>,
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
    Ok((
        Sealing {
            id: was.map_or_else(new_id, |one| Ok(one.id.clone()))?,
            version: was.map_or(1, |one| one.version + 1),
            from: was.map(|one| one.version),
            link_key: key.clone(),
            files_key: kept.as_ref().and(key),
            files_hash: kept.clone(),
        },
        kept.is_some(),
    ))
}

// A document shared along, and how the words of another find it.
struct Chosen {
    files: Vec<String>,
    titles: Vec<String>,
    family: Family,
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

struct Gathering<'a> {
    desk: &'a Desk<'a>,
    shelf: BTreeMap<String, Remembered>,
    public: bool,
    signed_as: Option<String>,
    told: Vec<Told>,
    remembering: Vec<(String, Remembered)>,
    files: BTreeMap<String, String>,
    titles: BTreeMap<String, String>,
}

impl Gathering<'_> {
    fn pages_of(&mut self, kept: &Kept, body: &str, shown: &str) -> Vec<WrittenPage> {
        let root = self.desk.paths.docs();
        let mut out = Vec::new();
        for page in self.desk.state.pages_read(kept.id, body) {
            match docs::read(&root, &page.file) {
                Ok(body) => out.push(WrittenPage {
                    file: page.file.clone(),
                    html: drawn(&body),
                    body,
                }),
                Err(_) => self.told.push(Told::PageLeftBehind {
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
            one.links.iter().for_each(|name| packing.file(name));
        }
        packing.out
    }

    fn tongue(&mut self, kept: &Kept, chosen: &[Chosen]) -> Result<Tongue, Unshared> {
        let body = docs::read(&self.desk.paths.docs(), &kept.file).map_err(unread)?;
        let shown = docs::titled(&body);
        let pages = self.pages_of(kept, &body, &shown);
        let html = drawn(&body);
        let mut all_named = vec![named(&html)];
        all_named.extend(pages.iter().map(|one| named(&one.html)));
        let mut packed = self.packed(&shown, &all_named);

        let mut cited = Vec::new();
        for name in all_named.iter().flat_map(|one| &one.documents) {
            let reached = pages
                .iter()
                .position(|page| {
                    page.file == *name || docs::titled(&page.body).eq_ignore_ascii_case(name)
                })
                .map(|at| Reached::Page(at as u64 + 1))
                .or_else(|| cited_at(name, chosen, &mut cited).map(Reached::Cited));
            if let Some(reached) = reached {
                packed.places.documents.insert(name.clone(), reached);
            }
        }

        let print = print_of(&packed.said_files, &packed.files);
        let was = self.shelf.get(&kept.file);
        let (sealing, keeps_files) = sealing_for(was, self.public, print.as_deref())?;
        self.remembering.push((
            kept.file.clone(),
            Remembered {
                id: sealing.id.clone(),
                version: sealing.version,
                link_key: sealing.link_key.clone(),
                files_print: print,
                files_hash: None,
                files_key: None,
            },
        ));
        self.files.insert(sealing.id.clone(), kept.file.clone());
        self.titles.insert(kept.file.clone(), shown);

        let places = &packed.places;
        let pages: Vec<Page> = pages
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
                    page_of: sealing.id.clone(),
                    at: at as u64 + 1,
                }
            })
            .collect();
        let travelling = body_ready(&body, places);
        let document = Document {
            id: sealing.id.clone(),
            title: title_of(&travelling),
            tags: tags_of(&body),
            lang: tisty_markdown::front::lang(&body),
            sheet: sheet_of(&body),
            signed_as: self.signed_as.clone(),
            made: None,
            wrote: None,
            body: travelling,
            html: ready(&html, places),
            pages: (!pages.is_empty()).then_some(pages),
            pictures: (!packed.said_pictures.is_empty()).then_some(packed.said_pictures),
            files: (!packed.said_files.is_empty()).then_some(packed.said_files),
            files_hash: None,
            cited: None,
            languages: None,
        };
        Ok(Tongue {
            document,
            pictures: packed.pictures,
            files: if keeps_files { vec![] } else { packed.files },
            sealing,
            cited,
        })
    }

    fn family(&mut self, doc: DocId, chosen: &[Chosen]) -> Result<Family, Unshared> {
        let state = self.desk.state;
        let kept = sharable(state, doc)?;
        let in_group = state.tongues_of(doc);
        let members = if in_group.iter().any(|one| one.id == doc) {
            in_group
        } else {
            vec![kept]
        };
        let anchor_id = state.anchor_of(kept);
        let (anchors, others): (Vec<&Kept>, Vec<&Kept>) =
            members.into_iter().partition(|one| one.id == anchor_id);
        let mut rest = others.into_iter();
        let Some(anchor) = anchors.into_iter().next().or_else(|| rest.next()) else {
            return Err(Unshared::NoSuchDoc);
        };
        let anchor = self.tongue(anchor, chosen)?;
        let mut others = Vec::new();
        for one in rest {
            if sharable(state, one.id).is_ok() {
                others.push(self.tongue(one, chosen)?);
            } else {
                self.told.push(Told::LanguageLeftBehind {
                    document: one.title.clone().unwrap_or_default(),
                });
            }
        }
        Ok(Family { anchor, others })
    }

    fn chosen(&mut self, doc: DocId) -> Result<Chosen, Unshared> {
        let state = self.desk.state;
        let root = self.desk.paths.docs();
        let in_group = state.tongues_of(doc);
        let kept = sharable(state, doc)?;
        let members: Vec<&Kept> = if in_group.is_empty() {
            vec![kept]
        } else {
            in_group
        };
        let titles = members
            .iter()
            .filter_map(|one| docs::read(&root, &one.file).ok())
            .map(|body| docs::titled(&body))
            .collect();
        Ok(Chosen {
            files: members.iter().map(|one| one.file.clone()).collect(),
            titles,
            family: self.family(doc, &[])?,
        })
    }
}

// Each tongue says its own list, in the order its words first name them.
fn cited_at(name: &str, chosen: &[Chosen], cited: &mut Vec<Family>) -> Option<u64> {
    let family = &chosen.iter().find(|one| one.answers_to(name))?.family;
    let at = match cited.iter().position(|one| one == family) {
        Some(at) => at,
        None => {
            cited.push(family.clone());
            cited.len() - 1
        }
    };
    Some(at as u64)
}

/// Finds all a document holds and names, puts it in place and leaves it ready for a window. It sends nothing and remembers nothing.
pub fn gathered(desk: &Desk, asked: &Asked) -> Result<Gathered, Unshared> {
    let state = desk.state;
    let wanted = asked
        .signed_as
        .clone()
        .or_else(|| state.signed.alias.clone());
    let signed_as = wanted.as_deref().and_then(clean::name);
    let mut gathering = Gathering {
        desk,
        shelf: remembered(desk.paths)?,
        public: asked.public,
        signed_as: signed_as.clone(),
        told: Vec::new(),
        remembering: Vec::new(),
        files: BTreeMap::new(),
        titles: BTreeMap::new(),
    };
    if wanted.is_some() && signed_as.is_none() {
        gathering.told.push(Told::NameLeftOut);
    }
    let own = state.anchor_of(sharable(state, asked.doc)?);
    let mut chosen = Vec::new();
    for doc in &asked.cited {
        if state
            .docs
            .get(doc)
            .is_none_or(|one| state.anchor_of(one) != own)
        {
            chosen.push(gathering.chosen(*doc)?);
        }
    }
    let family = gathering.family(asked.doc, &chosen)?;
    Ok(Gathered {
        family,
        told: gathering.told,
        remembering: gathering.remembering,
        files: gathering.files,
        titles: gathering.titles,
    })
}
