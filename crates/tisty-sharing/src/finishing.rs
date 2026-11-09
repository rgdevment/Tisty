use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tisty_envelope::family::{Family, Going, Tongue, sealed_family};
use tisty_envelope::sealing::Writer;

use crate::gathering::Gathered;
use crate::{Remembered, Told, Unshared};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unfinished {
    pub name: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    pub name: String,
    pub html: String,
    pub not_drawn: usize,
}

#[derive(Debug)]
pub struct Sealed {
    pub going: Vec<Going>,
    pub told: Vec<Told>,
    pub remembering: Vec<(String, Remembered)>,
    pub anchors: BTreeMap<String, String>,
    pub changed: BTreeSet<String>,
}

impl Sealed {
    // Each envelope sent is a new object Folio pays for, and the others are reached by id, never by version.
    #[must_use]
    pub fn only_what_changed(mut self) -> Self {
        self.going.retain(|one| self.changed.contains(&one.id));
        self.remembering
            .retain(|(_, one)| self.changed.contains(&one.id));
        self
    }
}

fn anchors_of<'a>(families: impl Iterator<Item = &'a Family>) -> BTreeMap<String, String> {
    families
        .flat_map(|family| {
            let anchor = &family.anchor.sealing.id;
            family
                .others
                .iter()
                .map(|one| (one.sealing.id.clone(), anchor.clone()))
        })
        .collect()
}

fn page_name(file: &str, at: u64) -> String {
    format!("{file}/{at}")
}

// Each thing `finished` in tisty-editor acts on. A word that only looks like one costs a window and nothing else.
fn asks_for_a_window(html: &str) -> bool {
    ["class=\"diagram\"", "language-", "data-ico="]
        .iter()
        .any(|one| html.contains(one))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Carrying {
    pub pages: usize,
    pub pictures: usize,
    pub pictures_bytes: u64,
    pub largest_picture: u64,
    pub largest_picture_name: Option<String>,
    pub main_bytes: u64,
    pub files: usize,
    pub files_bytes: u64,
}

fn weight(bytes: &[Vec<u8>]) -> u64 {
    bytes.iter().map(|one| one.len() as u64).sum()
}

impl Gathered {
    pub fn carrying(&self) -> Carrying {
        let anchor = &self.family.anchor;
        Carrying {
            pages: anchor.document.pages.as_ref().map_or(0, Vec::len),
            pictures: anchor.pictures.len(),
            pictures_bytes: weight(&anchor.pictures),
            largest_picture: anchor
                .pictures
                .iter()
                .map(|one| one.len() as u64)
                .max()
                .unwrap_or(0),
            largest_picture_name: anchor
                .document
                .pictures
                .iter()
                .flatten()
                .max_by_key(|one| one.size)
                .map(|one| one.name.clone()),
            // The words, the pages and the pictures travel in one envelope; the attachments go in another.
            main_bytes: serde_json::to_vec(&anchor.document).map_or(0, |said| said.len() as u64)
                + weight(&anchor.pictures),
            files: anchor.files.len(),
            files_bytes: weight(&anchor.files),
        }
    }

    pub fn whole_print(&self) -> String {
        self.remembering
            .iter()
            .filter_map(|(_, one)| one.words_print.as_deref())
            .collect()
    }

    pub fn print_of(&self, file: &str) -> Option<String> {
        let (_, now) = self.remembering.iter().find(|(at, _)| at == file)?;
        now.words_print.clone()
    }

    pub fn same_as_sent(&self) -> bool {
        self.changed.is_empty()
    }

    fn files_by_id(&self) -> BTreeMap<String, String> {
        self.remembering
            .iter()
            .map(|(file, was)| (was.id.clone(), file.clone()))
            .collect()
    }

    pub fn unfinished(&self) -> Vec<Unfinished> {
        let files = self.files_by_id();
        let mut out = Vec::new();
        let tongues = std::iter::once(&self.family)
            .chain(&self.cited)
            .flat_map(|one| std::iter::once(&one.anchor).chain(&one.others));
        for tongue in tongues {
            let Some(file) = files.get(&tongue.document.id) else {
                continue;
            };
            out.push(Unfinished {
                name: file.clone(),
                html: tongue.document.html.clone(),
            });
            out.extend(
                tongue
                    .document
                    .pages
                    .iter()
                    .flatten()
                    .map(|page| Unfinished {
                        name: page_name(file, page.at),
                        html: page.html.clone(),
                    }),
            );
        }
        out.retain(|one| asks_for_a_window(&one.html));
        out
    }

    // Sealing looks at the result again: a window's word counts for nothing either.
    pub fn finished(&mut self, answers: Vec<Finished>) -> Result<(), Unshared> {
        let files = self.files_by_id();
        let mut not_drawn: BTreeMap<String, usize> = BTreeMap::new();
        let mut html = BTreeMap::new();
        for one in answers {
            let file = one.name.split('/').next().unwrap_or_default().to_owned();
            *not_drawn.entry(file).or_default() += one.not_drawn;
            html.insert(one.name, one.html);
        }
        let taken = |name: String, was: &mut String| match html.get(&name) {
            Some(one) => {
                one.clone_into(was);
                Ok(())
            }
            None if asks_for_a_window(was) => Err(Unshared::Unfinished(name)),
            None => Ok(()),
        };
        let finish = |tongue: &mut Tongue| {
            let id = &tongue.document.id;
            let file = files
                .get(id)
                .ok_or_else(|| Unshared::Unfinished(id.clone()))?;
            taken(file.clone(), &mut tongue.document.html)?;
            for page in tongue.document.pages.iter_mut().flatten() {
                taken(page_name(file, page.at), &mut page.html)?;
            }
            Ok::<(), Unshared>(())
        };
        let tongues = std::iter::once(&mut self.family)
            .chain(&mut self.cited)
            .flat_map(|one| std::iter::once(&mut one.anchor).chain(&mut one.others));
        for tongue in tongues {
            finish(tongue)?;
        }
        for (file, many) in not_drawn.into_iter().filter(|(_, many)| *many > 0) {
            self.told.push(Told::NotDrawn {
                document: self.titles.get(&file).cloned().unwrap_or_default(),
                many,
            });
        }
        Ok(())
    }

    // Nothing is remembered yet: nothing has gone anywhere.
    pub fn sealed(self, writer: &mut Writer) -> Result<Sealed, Unshared> {
        let Self {
            mut family,
            cited,
            cites,
            told,
            remembering,
            changed,
            ..
        } = self;
        let anchors = anchors_of(std::iter::once(&family).chain(&cited));
        let top = family.anchor.sealing.id.clone();
        let cited: Vec<Arc<Family>> = cited.into_iter().map(Arc::new).collect();
        for tongue in std::iter::once(&mut family.anchor).chain(&mut family.others) {
            let said = cites.get(&tongue.document.id).into_iter().flatten();
            tongue.cited = said.map(|at| Arc::clone(&cited[*at])).collect();
        }
        // Whoever holds one last hands its bytes on without copying them.
        drop(cited);
        let going = sealed_family(writer, family)?;
        let remembering = remembering
            .into_iter()
            .map(|(file, mut was)| {
                let hash = going
                    .iter()
                    .find(|one| one.id == was.id)
                    .and_then(|one| one.sealed.files_hash.clone());
                was.files_key = hash.as_ref().and_then(|_| was.link_key.clone());
                was.files_hash = hash;
                was.held_by = (was.id != top).then(|| top.clone());
                (file, was)
            })
            .collect();
        Ok(Sealed {
            going,
            told,
            remembering,
            anchors,
            changed,
        })
    }
}
