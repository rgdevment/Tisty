use std::path::PathBuf;

use serde::Serialize;
use tisty_contracts::{Attachment, Picture, PictureKind};
use tisty_core::attach;
use tisty_envelope::clean;
use tisty_envelope::pictures::{Kind, Made, made_or_turned};
use tisty_envelope::travelling::{Placed, Places};

use crate::gathering::Desk;

const NO_NAME: &str = "file";

/// What did not travel as the person wrote it, which they are told.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "what", rename_all = "camelCase")]
pub enum Told {
    /// A picture that could not be made, or an SVG, went as a file with its card.
    PictureAsFile { document: String, name: String },
    /// A picture that could go neither way went as its alt.
    PictureAsWords { document: String, name: String },
    /// A picture on the web is never asked for, so it went as a link to it.
    PictureAsLink { document: String, name: String },
    /// A file that is no longer here went as its words.
    FileAsWords { document: String, name: String },
    /// Diagrams that could not be drawn went as code.
    NotDrawn { document: String, many: usize },
    /// A page could not be read, so its document went without it.
    PageLeftBehind { document: String },
    /// A language that is locked, or someone else's, stayed here.
    LanguageLeftBehind { document: String },
    /// The name chosen can be no name, so it went without one.
    NameLeftOut,
}

/// What a language names, once found: where each sits and the bytes that go.
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

fn leads_outside(name: &str) -> bool {
    let lowered = name.trim().to_ascii_lowercase();
    lowered.starts_with("http://") || lowered.starts_with("https://")
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
        let bytes = std::fs::read(&at).ok()?;
        Some((at, bytes))
    }

    fn placed(&self, name: &str) -> bool {
        self.out.places.pictures.contains_key(name) || self.out.places.files.contains_key(name)
    }

    fn document(&self) -> String {
        self.document.to_owned()
    }

    /// A picture the HTML draws. It goes made anew, or as a file with its card, or as its alt.
    pub fn picture(&mut self, name: &str) {
        if self.placed(name) {
            return;
        }
        if !attach::names_an_attachment(name) {
            let name = name.to_owned();
            let document = self.document();
            self.told.push(if leads_outside(&name) {
                Told::PictureAsLink { document, name }
            } else {
                Told::PictureAsWords {
                    document,
                    name: String::new(),
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
                self.told.push(Told::PictureAsFile {
                    document: self.document(),
                    name: shown,
                });
                self.place_file(name, bytes);
            }
        }
    }

    /// What the person laid in as a file, or a link to something kept here. A link to anything else goes as its words.
    pub fn file(&mut self, name: &str) {
        if self.placed(name) || !attach::names_an_attachment(name) {
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
