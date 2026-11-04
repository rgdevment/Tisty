use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tisty_envelope::family::{Family, Going, Tongue, sealed_family};
use tisty_envelope::sealing::Writer;

use crate::gathering::Gathered;
use crate::{Remembered, Told, Unshared};

/// HTML with everything it names in place, for a window to colour and draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unfinished {
    pub name: String,
    pub html: String,
}

/// What a window answers, as `finished` in `tisty-editor` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    pub name: String,
    pub html: String,
    pub not_drawn: usize,
}

/// A document in its envelopes, and what the person has to be told about them.
#[derive(Debug)]
pub struct Sealed {
    pub going: Vec<Going>,
    pub told: Vec<Told>,
    /// Handed to `sent` once they have reached where they go.
    pub remembering: Vec<(String, Remembered)>,
}

fn page_name(file: &str, at: u64) -> String {
    format!("{file}/{at}")
}

fn each_tongue<'a>(family: &'a Family, out: &mut Vec<&'a Tongue>) {
    out.push(&family.anchor);
    out.extend(&family.others);
    for tongue in std::iter::once(&family.anchor).chain(&family.others) {
        for cited in &tongue.cited {
            each_tongue(cited, out);
        }
    }
}

fn each_tongue_mut(
    family: &mut Family,
    change: &mut impl FnMut(&mut Tongue) -> Result<(), Unshared>,
) -> Result<(), Unshared> {
    change(&mut family.anchor)?;
    for tongue in &mut family.others {
        change(tongue)?;
    }
    let cited = std::iter::once(&mut family.anchor)
        .chain(&mut family.others)
        .flat_map(|tongue| &mut tongue.cited);
    for one in cited {
        each_tongue_mut(one, change)?;
    }
    Ok(())
}

impl Gathered {
    /// All a window has to finish, once each however many cite it.
    pub fn unfinished(&self) -> Vec<Unfinished> {
        let mut tongues = Vec::new();
        each_tongue(&self.family, &mut tongues);
        let mut out = BTreeMap::new();
        for tongue in tongues {
            let Some(file) = self.file_of(&tongue.document.id) else {
                continue;
            };
            out.insert(file.to_owned(), tongue.document.html.clone());
            for page in tongue.document.pages.iter().flatten() {
                out.insert(page_name(file, page.at), page.html.clone());
            }
        }
        out.into_iter()
            .map(|(name, html)| Unfinished { name, html })
            .collect()
    }

    /// Takes what a window answered. Sealing looks at it again: a window's word counts for nothing either.
    pub fn finished(&mut self, answers: Vec<Finished>) -> Result<(), Unshared> {
        let mut not_drawn: BTreeMap<String, usize> = BTreeMap::new();
        let mut html = BTreeMap::new();
        for one in answers {
            let file = one.name.split('/').next().unwrap_or_default().to_owned();
            *not_drawn.entry(file).or_default() += one.not_drawn;
            html.insert(one.name, one.html);
        }
        let files = self.files.clone();
        let taken = |name: String| html.get(&name).cloned().ok_or(Unshared::Unfinished(name));
        each_tongue_mut(&mut self.family, &mut |tongue| {
            let file = files
                .get(&tongue.document.id)
                .ok_or_else(|| Unshared::Unfinished(tongue.document.id.clone()))?;
            tongue.document.html = taken(file.clone())?;
            for page in tongue.document.pages.iter_mut().flatten() {
                page.html = taken(page_name(file, page.at))?;
            }
            Ok(())
        })?;
        for (file, many) in not_drawn.into_iter().filter(|(_, many)| *many > 0) {
            self.told.push(Told::NotDrawn {
                document: self.titles.get(&file).cloned().unwrap_or_default(),
                many,
            });
        }
        Ok(())
    }

    /// Seals what the window finished. Nothing is remembered yet: nothing has gone anywhere.
    pub fn sealed(self, writer: &mut Writer) -> Result<Sealed, Unshared> {
        let going = sealed_family(writer, self.family)?;
        let remembering = self
            .remembering
            .into_iter()
            .map(|(file, mut was)| {
                let hash = going
                    .iter()
                    .find(|one| one.id == was.id)
                    .and_then(|one| one.sealed.files_hash.clone());
                was.files_key = hash.as_ref().and_then(|_| was.link_key.clone());
                was.files_hash = hash;
                (file, was)
            })
            .collect();
        Ok(Sealed {
            going,
            told: self.told,
            remembering,
        })
    }

    fn file_of(&self, id: &str) -> Option<&str> {
        self.files.get(id).map(String::as_str)
    }
}
