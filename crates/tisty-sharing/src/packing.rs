use std::path::PathBuf;

use serde::Serialize;
use tisty_contracts::{Attachment, Picture, PictureKind};
use tisty_core::attach;
use tisty_envelope::clean;
use tisty_envelope::pictures::{Kind, Made, looks_like_a_picture, made_or_turned};
use tisty_envelope::travelling::{Placed, Places, leads_outside};

use crate::gathering::Desk;

const NO_NAME: &str = "file";
// This machine's own and no rule of the contract's: all a language carries is held in memory to seal it.
const HELD_AT_MOST: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "what", rename_all = "camelCase")]
pub enum Told {
    PictureAsFile {
        document: String,
        name: String,
    },
    AsAttachment {
        document: String,
        name: String,
        plays: Option<String>,
    },
    PictureAsWords {
        document: String,
        name: String,
    },
    PictureNotKept {
        document: String,
        name: String,
    },
    PictureAsLink {
        document: String,
        name: String,
    },
    FileAsWords {
        document: String,
        name: String,
    },
    NotDrawn {
        document: String,
        many: usize,
    },
    PageLeftBehind {
        document: String,
    },
    LanguageLeftBehind {
        language: String,
    },
    LanguageOverTheTop {
        language: String,
        top: usize,
    },
    CitedLeftBehind {
        document: String,
    },
    NameLeftOut,
    TakenBackNotKept,
}

#[derive(Debug, Default)]
pub(crate) struct Packed {
    pub places: Places,
    pub pictures: Vec<Vec<u8>>,
    pub said_pictures: Vec<Picture>,
    pub files: Vec<Vec<u8>>,
    pub said_files: Vec<Attachment>,
}

pub(crate) struct Packing<'a> {
    desk: &'a Desk<'a>,
    document: &'a str,
    told: &'a mut Vec<Told>,
    pub out: Packed,
}

// An attachment is kept under its name and a stamp of its bytes, and only the name is its own.
fn own_name(reference: &str) -> String {
    let leaf = attach::leaf_of(reference).unwrap_or(reference);
    let stamp = attach::stamped_by(leaf);
    let bare = leaf.replacen(&format!("-{stamp}"), "", 1);
    let named = if bare == leaf {
        leaf.replacen(stamp, NO_NAME, 1)
    } else {
        bare
    };
    clean::file_name(&named).unwrap_or_else(|| NO_NAME.to_owned())
}

// What is no address and no path has nothing a person would recognise, as a picture written in the words themselves.
fn name_of_what_is_not_kept(name: &str) -> String {
    if name == "#" || name.trim().to_ascii_lowercase().starts_with("data:") {
        return String::new();
    }
    clean::file_name(name).unwrap_or_default()
}

fn kind_of(kind: Kind) -> PictureKind {
    match kind {
        Kind::Png => PictureKind::ImagePng,
        Kind::Jpeg => PictureKind::ImageJpeg,
    }
}

impl<'a> Packing<'a> {
    pub fn new(desk: &'a Desk<'a>, document: &'a str, told: &'a mut Vec<Told>) -> Self {
        Self {
            desk,
            document,
            told,
            out: Packed::default(),
        }
    }

    fn found(&self, name: &str) -> Option<(PathBuf, Vec<u8>)> {
        let at = attach::found(name, self.desk.paths.data(), self.desk.also).ok()?;
        let held: u64 = (self.out.pictures.iter())
            .chain(&self.out.files)
            .map(|one| one.len() as u64)
            .sum();
        if held + std::fs::metadata(&at).ok()?.len() > HELD_AT_MOST {
            return None;
        }
        let bytes = std::fs::read(&at).ok()?;
        Some((at, bytes))
    }

    fn document(&self) -> String {
        self.document.to_owned()
    }

    pub fn picture(&mut self, name: &str) {
        let places = &self.out.places;
        if places.pictures.contains_key(name) || places.files.contains_key(name) {
            return;
        }
        if !attach::names_an_attachment(name) {
            let document = self.document();
            self.told.push(if leads_outside(name) {
                Told::PictureAsLink {
                    document,
                    name: name.to_owned(),
                }
            } else {
                Told::PictureNotKept {
                    document,
                    name: name_of_what_is_not_kept(name),
                }
            });
            return;
        }
        let shown = own_name(name);
        let Some((at, bytes)) = self.found(name) else {
            self.told.push(Told::PictureAsWords {
                document: self.document(),
                name: shown,
            });
            return;
        };
        match made_or_turned(&bytes, || (self.desk.turned)(&at)) {
            Ok(made) => self.place_picture(name, made),
            Err(_) => {
                let document = self.document();
                // What never was a picture is an attachment, and the web plays a sound or a video as its first bytes say.
                self.told.push(if looks_like_a_picture(&bytes) {
                    Told::PictureAsFile {
                        document,
                        name: shown,
                    }
                } else {
                    Told::AsAttachment {
                        document,
                        name: shown,
                        plays: tisty_contracts::playable_as(&bytes).map(|one| one.player),
                    }
                });
                self.place_file(name, bytes);
            }
        }
    }

    // Laid in as a file it is a card, even when the same attachment is drawn as a picture elsewhere.
    pub fn file(&mut self, name: &str) {
        if self.out.places.files.contains_key(name) || !attach::names_an_attachment(name) {
            return;
        }
        match self.found(name) {
            Some((_, bytes)) => self.place_file(name, bytes),
            None => self.told.push(Told::FileAsWords {
                document: self.document(),
                name: own_name(name),
            }),
        }
    }

    // A link reaches a picture that already travels, so its bytes need not go twice.
    pub fn link(&mut self, name: &str) {
        if !self.out.places.pictures.contains_key(name) {
            self.file(name);
        }
    }

    // The same picture travels once, however many names lead to it.
    fn place_picture(&mut self, name: &str, made: Made) {
        let at = match self.out.pictures.iter().position(|one| *one == made.bytes) {
            Some(at) => at,
            None => {
                let at = self.out.pictures.len();
                self.out.said_pictures.push(Picture {
                    at: at as u64,
                    name: format!("picture-{at}.{}", made.kind.ending()),
                    kind: kind_of(made.kind),
                    size: made.bytes.len() as u64,
                    width: u64::from(made.width),
                    height: u64::from(made.height),
                });
                self.out.pictures.push(made.bytes);
                at
            }
        };
        let placed = Placed {
            at: at as u64,
            width: self.out.said_pictures[at].width,
            height: self.out.said_pictures[at].height,
        };
        self.out.places.pictures.insert(name.to_owned(), placed);
    }

    fn place_file(&mut self, name: &str, bytes: Vec<u8>) {
        let at = match self.out.files.iter().position(|one| *one == bytes) {
            Some(at) => at,
            None => {
                let at = self.out.files.len();
                self.out.said_files.push(Attachment {
                    at: at as u64,
                    name: own_name(name),
                    kind: None,
                    size: Some(bytes.len() as u64),
                });
                self.out.files.push(bytes);
                at
            }
        };
        self.out.places.files.insert(name.to_owned(), at as u64);
    }
}

#[cfg(test)]
#[path = "packing_test.rs"]
mod tests;
