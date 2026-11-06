use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::folio_limits::{self, FolioLimits, FolioTerms};
use crate::policy;

const TERMS_ES_URL: &str =
    "https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/main/docs/TERMINOS.md";
const TERMS_EN_URL: &str =
    "https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/main/docs/TERMS.md";
const ACCEPTED: &str = "folio-terms-accepted.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tongue {
    Es,
    En,
}

impl Tongue {
    fn url(self) -> &'static str {
        match self {
            Self::Es => TERMS_ES_URL,
            Self::En => TERMS_EN_URL,
        }
    }

    fn print_in(self, terms: &FolioTerms) -> &str {
        match self {
            Self::Es => &terms.es,
            Self::En => &terms.en,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Accepted {
    version: String,
    print: String,
    tongue: Tongue,
    at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    Accepted,
    ToAccept {
        version: String,
        text: String,
        accepted_before: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unsure {
    NoWord,
    NotPublished,
    Unmatched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unkept {
    Moved,
    Unwritten,
}

fn print_of(said: &[u8]) -> String {
    Sha256::digest(said)
        .iter()
        .map(|one| format!("{one:02x}"))
        .collect()
}

fn accepted(home: &Path) -> Option<Accepted> {
    policy::kept_at(&home.join(ACCEPTED))
}

fn heard<F: Fn(&str) -> Option<Vec<u8>>>(home: &Path, bring: &F) -> Option<FolioLimits> {
    match folio_limits::heed(home, bring) {
        Some(Ok(limits)) => Some(limits),
        _ => folio_limits::kept(home).or_else(|| folio_limits::heed_now(home, bring)?.ok()),
    }
}

fn under<F: Fn(&str) -> Option<Vec<u8>>>(
    terms: &FolioTerms,
    before: Option<&Accepted>,
    tongue: Tongue,
    bring: &F,
) -> Option<Standing> {
    if before.is_some_and(|one| one.version == terms.version) {
        return Some(Standing::Accepted);
    }
    let said = bring(tongue.url())?;
    if print_of(&said) != tongue.print_in(terms) {
        return None;
    }
    Some(Standing::ToAccept {
        version: terms.version.clone(),
        text: String::from_utf8(said).ok()?,
        accepted_before: before.map(|one| one.version.clone()),
    })
}

pub fn standing<F: Fn(&str) -> Option<Vec<u8>>>(
    home: &Path,
    tongue: Tongue,
    bring: F,
) -> Result<Standing, Unsure> {
    let limits = heard(home, &bring).ok_or(Unsure::NoWord)?;
    let terms = limits.terms.as_ref().ok_or(Unsure::NotPublished)?;
    let before = accepted(home);
    if let Some(standing) = under(terms, before.as_ref(), tongue, &bring) {
        return Ok(standing);
    }
    // The limits kept here may be up to an hour older than the text just read.
    folio_limits::heed_now(home, &bring)
        .and_then(Result::ok)
        .and_then(|limits| {
            let terms = limits.terms.as_ref()?;
            under(terms, before.as_ref(), tongue, &bring)
        })
        .ok_or(Unsure::Unmatched)
}

pub fn accepted_version(home: &Path) -> Option<String> {
    accepted(home).map(|one| one.version)
}

// Folio refused the terms this computer holds as accepted, so the hour since the last reading counts for nothing.
pub fn read_again<F: Fn(&str) -> Option<Vec<u8>>>(
    home: &Path,
    tongue: Tongue,
    bring: F,
) -> Result<Standing, Unsure> {
    let _ = folio_limits::heed_now(home, &bring);
    standing(home, tongue, bring)
}

pub fn accept(home: &Path, tongue: Tongue, version: &str) -> Result<(), Unkept> {
    let terms = folio_limits::kept(home)
        .and_then(|limits| limits.terms)
        .filter(|terms| terms.version == version)
        .ok_or(Unkept::Moved)?;
    let accepted = Accepted {
        version: terms.version.clone(),
        print: tongue.print_in(&terms).to_owned(),
        tongue,
        at: policy::now(),
    };
    let bytes = serde_json::to_vec(&accepted).map_err(|_| Unkept::Unwritten)?;
    std::fs::create_dir_all(home).map_err(|_| Unkept::Unwritten)?;
    tisty_core::store::write_atomic(&home.join(ACCEPTED), &bytes).map_err(|_| Unkept::Unwritten)
}

#[cfg(test)]
#[path = "folio_terms_test.rs"]
mod tests;
