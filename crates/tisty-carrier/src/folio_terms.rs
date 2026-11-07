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
const UNMET: &str = "folio-terms-unmet";
// GitHub keeps each of Folio's files apart for five minutes, so a print still unmet an hour later is no longer the cache.
const UNMET_FOR: u64 = 60 * 60;

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
    StillAccepted,
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
    Settling,
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

// Read on every press, so newer terms are asked for as soon as they can be checked.
fn heard<F: Fn(&str) -> Option<Vec<u8>>>(home: &Path, bring: &F) -> Option<FolioLimits> {
    match folio_limits::heed_now(home, bring) {
        Some(Ok(limits)) => Some(limits),
        _ => folio_limits::kept(home),
    }
}

fn under<F: Fn(&str) -> Option<Vec<u8>>>(
    terms: &FolioTerms,
    before: Option<&Accepted>,
    tongue: Tongue,
    bring: &F,
) -> Result<Standing, Unsure> {
    if before.is_some_and(|one| one.version == terms.version) {
        return Ok(Standing::Accepted);
    }
    let said = bring(tongue.url()).ok_or(Unsure::NoWord)?;
    if print_of(&said) != tongue.print_in(terms) {
        return Err(Unsure::Unmatched);
    }
    Ok(Standing::ToAccept {
        version: terms.version.clone(),
        text: String::from_utf8(said).map_err(|_| Unsure::Unmatched)?,
        accepted_before: before.map(|one| one.version.clone()),
    })
}

enum Checked {
    Standing(Standing),
    Unmet { long: bool },
}

pub fn standing<F: Fn(&str) -> Option<Vec<u8>>>(
    home: &Path,
    tongue: Tongue,
    bring: F,
) -> Result<Standing, Unsure> {
    let limits = heard(home, &bring);
    let before = accepted(home);
    match checked(home, limits, before.as_ref(), tongue, &bring) {
        Ok(Checked::Standing(standing)) => {
            met(home);
            Ok(standing)
        }
        Ok(Checked::Unmet { long }) => match (before.is_some(), long) {
            (true, false) => Ok(Standing::Accepted),
            (true, true) => Ok(Standing::StillAccepted),
            (false, false) => Err(Unsure::Settling),
            (false, true) => Err(Unsure::Unmatched),
        },
        // Sharing under the terms already accepted needs no text, so one that cannot be brought holds nobody back.
        Err(Unsure::NoWord) if before.is_some() => Ok(Standing::Accepted),
        Err(unsure) => Err(unsure),
    }
}

fn checked<F: Fn(&str) -> Option<Vec<u8>>>(
    home: &Path,
    limits: Option<FolioLimits>,
    before: Option<&Accepted>,
    tongue: Tongue,
    bring: &F,
) -> Result<Checked, Unsure> {
    let limits = limits.ok_or(Unsure::NoWord)?;
    let terms = limits.terms.as_ref().ok_or(Unsure::NotPublished)?;
    match under(terms, before, tongue, bring) {
        Ok(standing) => Ok(Checked::Standing(standing)),
        Err(Unsure::Unmatched) => Ok(Checked::Unmet {
            long: unmet_long(home, tongue.print_in(terms)),
        }),
        Err(unsure) => Err(unsure),
    }
}

fn met(home: &Path) {
    let _ = std::fs::remove_file(home.join(UNMET));
}

// Each print is timed on its own, so a newer publish starts its own hour.
fn unmet_long(home: &Path, wanted: &str) -> bool {
    let kept = home.join(UNMET);
    let now = policy::now();
    let since = std::fs::read_to_string(&kept).ok().and_then(|said| {
        let (since, print) = said.trim().split_once(' ')?;
        (print == wanted).then(|| since.parse::<u64>().ok())?
    });
    if let Some(since) = since {
        return now.saturating_sub(since) >= UNMET_FOR;
    }
    let _ = std::fs::create_dir_all(home);
    let _ = std::fs::write(&kept, format!("{now} {wanted}"));
    false
}

pub fn accepted_version(home: &Path) -> Option<String> {
    accepted(home).map(|one| one.version)
}

// Folio refused the terms this computer holds as accepted, so the hour since the last reading counts for nothing.
pub fn read_again<F: Fn(&str) -> Option<Vec<u8>>>(
    home: &Path,
    tongue: Tongue,
    bring: F,
    asked: Option<&str>,
) -> Result<Standing, Unsure> {
    let limits = heard(home, &bring);
    // Only terms that check out can stand in for the ones Folio refused.
    let long = match checked(home, limits, accepted(home).as_ref(), tongue, &bring)? {
        Checked::Standing(standing @ Standing::ToAccept { .. }) => {
            met(home);
            return Ok(standing);
        }
        Checked::Standing(_) => unmet_long(home, &format!("asked:{}", asked.unwrap_or_default())),
        Checked::Unmet { long } => long,
    };
    Err(if long {
        Unsure::Unmatched
    } else {
        Unsure::Settling
    })
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
