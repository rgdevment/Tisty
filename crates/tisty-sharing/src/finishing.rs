use std::collections::BTreeMap;
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
}

fn page_name(file: &str, at: u64) -> String {
    format!("{file}/{at}")
}

impl Gathered {
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
        let taken = |name: String| html.get(&name).cloned().ok_or(Unshared::Unfinished(name));
        let finish = |tongue: &mut Tongue| {
            let id = &tongue.document.id;
            let file = files
                .get(id)
                .ok_or_else(|| Unshared::Unfinished(id.clone()))?;
            tongue.document.html = taken(file.clone())?;
            for page in tongue.document.pages.iter_mut().flatten() {
                page.html = taken(page_name(file, page.at))?;
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
            ..
        } = self;
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
                (file, was)
            })
            .collect();
        Ok(Sealed {
            going,
            told,
            remembering,
        })
    }
}
