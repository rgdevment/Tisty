use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::tags::{Tag, escaped, unescaped, walk};

// Written only by Tisty, for itself: a task's id and what finds a language again say nothing to whoever reads.
const KEPT_TO_ITSELF: [&str; 2] = ["task", "translationKey"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub at: u64,
    pub width: u64,
    pub height: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    Page(u64),
    Cited(u64),
}

impl Reached {
    fn href(self) -> String {
        match self {
            Self::Page(at) => format!("tisty:page/{at}"),
            Self::Cited(at) => format!("tisty:cited/{at}"),
        }
    }
}

/// Where each thing a document names sits in what travels. What is not here travels as its words.
#[derive(Debug, Clone, Default)]
pub struct Places {
    pub pictures: BTreeMap<String, Placed>,
    pub files: BTreeMap<String, u64>,
    pub documents: BTreeMap<String, Reached>,
}

/// What the HTML names, to go and find before calling `ready`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Named {
    pub pictures: Vec<String>,
    pub links: Vec<String>,
    pub documents: Vec<String>,
}

fn opens_a_doc(href: &str) -> Option<&str> {
    href.strip_prefix("tisty:doc/")
}

fn leads_outside(href: &str) -> bool {
    let lowered = href.trim().to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|one| lowered.starts_with(one))
}

fn leads_to_a_heading(href: &str) -> bool {
    href.len() > 1 && href.starts_with('#')
}

pub fn named(html: &str) -> Named {
    let mut out = Named::default();
    walk(html, |_, tag| {
        let Some(tag) = tag.filter(|one| !one.closes) else {
            return;
        };
        match tag.name {
            "img" => out.pictures.extend(tag.said("src").map(unescaped)),
            "a" => {
                if let Some(doc) = tag.said("data-doc") {
                    out.documents.push(unescaped(doc));
                }
                let href = tag.said("href").map(unescaped).unwrap_or_default();
                if let Some(doc) = opens_a_doc(&href) {
                    out.documents.push(doc.to_owned());
                } else if !href.is_empty() && !href.starts_with('#') && !leads_outside(&href) {
                    out.links.push(href);
                }
            }
            "span" => out.documents.extend(tag.said("data-ref").map(unescaped)),
            _ => {}
        }
    });
    out
}

fn kept_to_itself_gone(html: &str) -> String {
    let mut out = html.to_owned();
    for key in KEPT_TO_ITSELF {
        let open = format!("<div><dt>{key}</dt>");
        while let Some(from) = out.find(&open) {
            let Some(length) = out[from..].find("</dd></div>") else {
                break;
            };
            out.replace_range(from..from + length + "</dd></div>".len(), "");
        }
    }
    out
}

fn written_back(tag: &Tag, leaving: &[&str], out: &mut String) {
    let _ = write!(out, "<{}", tag.name);
    for (name, value) in &tag.attributes {
        if !leaving.contains(name) {
            let _ = write!(out, " {name}=\"{value}\"");
        }
    }
    out.push('>');
}

fn card(href: &str, out: &mut String) {
    let _ = write!(out, "<a class=\"card\" href=\"{href}\">");
}

struct Ready<'a> {
    places: &'a Places,
    out: String,
    // What the next closing tag of each kind turns into, when the opening one changed.
    closing_a: Option<&'static str>,
    closing_span: Option<&'static str>,
}

impl Ready<'_> {
    fn picture(&mut self, tag: &Tag) {
        let src = tag.said("src").map(unescaped).unwrap_or_default();
        let alt = tag.said("alt").unwrap_or_default();
        let as_file = tag.said("data-as") == Some("file");
        let picture = self.places.pictures.get(&src).filter(|_| !as_file);
        if let Some(placed) = picture {
            let _ = write!(
                self.out,
                "<img src=\"tisty:picture/{}\" alt=\"{alt}\" width=\"{}\" height=\"{}\" loading=\"lazy\">",
                placed.at, placed.width, placed.height
            );
            return;
        }
        let shown = if alt.is_empty() {
            escaped(&src)
        } else {
            alt.to_owned()
        };
        match self.places.files.get(&src) {
            Some(at) => {
                let _ = write!(
                    self.out,
                    "<a class=\"card\" href=\"tisty:file/{at}\">{shown}</a>"
                );
            }
            // Nothing is asked of anyone else to read it: a picture on the web stays a link to it.
            None if leads_outside(&src) => {
                let _ = write!(self.out, "<a href=\"{}\">{shown}</a>", escaped(&src));
            }
            None => self.out.push_str(&shown),
        }
    }

    fn link(&mut self, tag: &Tag) {
        self.closing_a = None;
        let href = tag.said("href").map(unescaped);
        let doc = tag
            .said("data-doc")
            .map(unescaped)
            .or_else(|| href.as_deref().and_then(opens_a_doc).map(str::to_owned));
        if let Some(doc) = doc {
            match self.places.documents.get(&doc) {
                Some(reached) => card(&reached.href(), &mut self.out),
                None => self.closing_a = Some(""),
            }
            return;
        }
        if tag.said("data-task").is_some() {
            self.closing_a = Some("");
            return;
        }
        let Some(href) = href else {
            written_back(tag, &["data-at"], &mut self.out);
            return;
        };
        if leads_outside(&href) || leads_to_a_heading(&href) {
            written_back(tag, &["data-at"], &mut self.out);
        } else if let Some(at) = self.places.files.get(&href) {
            card(&format!("tisty:file/{at}"), &mut self.out);
        } else {
            self.closing_a = Some("");
        }
    }

    fn reference(&mut self, tag: &Tag) -> bool {
        let Some(name) = tag.said("data-ref") else {
            return false;
        };
        match self.places.documents.get(&unescaped(name)) {
            Some(reached) => {
                card(&reached.href(), &mut self.out);
                self.closing_span = Some("</a>");
            }
            None => self.closing_span = Some(""),
        }
        true
    }

    fn opening(&mut self, tag: &Tag) {
        match tag.name {
            "img" => self.picture(tag),
            "a" => self.link(tag),
            "span" if self.reference(tag) => {}
            "area"
                if !tag
                    .said("href")
                    .is_some_and(|one| leads_outside(&unescaped(one))) =>
            {
                written_back(tag, &["data-at", "href"], &mut self.out);
            }
            _ => {
                let cited = tag
                    .said("cite")
                    .is_some_and(|one| !leads_outside(&unescaped(one)));
                let leaving: &[&str] = if cited {
                    &["data-at", "cite"]
                } else {
                    &["data-at"]
                };
                written_back(tag, leaving, &mut self.out);
            }
        }
    }

    fn closing(&mut self, tag: &Tag) {
        let turned = match tag.name {
            "a" => self.closing_a.take(),
            "span" => self.closing_span.take(),
            _ => None,
        };
        match turned {
            Some(said) => self.out.push_str(said),
            None => {
                let _ = write!(self.out, "</{}>", tag.name);
            }
        }
    }
}

/// The HTML of a document with everything it names in place, so reading it takes nothing else. Colour, icons and drawings come after, in the window.
pub fn ready(html: &str, places: &Places) -> String {
    let html = kept_to_itself_gone(html);
    let mut ready = Ready {
        places,
        out: String::with_capacity(html.len()),
        closing_a: None,
        closing_span: None,
    };
    walk(&html, |words, tag| {
        ready.out.push_str(words);
        match tag {
            Some(tag) if tag.closes => ready.closing(&tag),
            Some(tag) => ready.opening(&tag),
            None => {}
        }
    });
    ready.out
}

#[cfg(test)]
#[path = "travelling_test.rs"]
mod tests;
