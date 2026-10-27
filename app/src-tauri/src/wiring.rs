mod catalog;
mod json;
mod places;
mod shape;
mod toml;

use std::path::{Path, PathBuf};

use catalog::{Client, Found, Shape};
use places::Place;

use crate::command;

const NAME: &str = "tisty";
const BEFORE: &str = "before-tisty";
const LAST: &str = "before-tisty-last";
const WSL: &str = "wsl:";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manual {
    pub file: String,
    pub snippet: String,
    pub cli: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Seen {
    pub id: String,
    pub name: String,
    pub also: Option<String>,
    pub place: Option<String>,
    pub at: String,
    /// It points at this Tisty.
    pub wired: bool,
    /// It points at a Tisty that is not this one.
    pub astray: bool,
    /// Told apart from astray so "Repair" can say the old Tisty was deleted, not just moved.
    pub gone: bool,
    pub points: Option<String>,
    /// Only a folder that outlives an uninstall says it is here.
    pub maybe: bool,
    /// Its shape is not one Tisty writes, so it is connected by hand.
    pub by_hand: bool,
    pub manual: Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Distro {
    pub name: String,
    /// Windows programs may not run there, so nothing in it can start Tisty.
    pub shut: bool,
    pub command: Option<String>,
}

#[derive(Debug)]
pub enum Stuck {
    NoSuch,
    Shut,
    Puzzling(String),
    Unborn(String),
    Cannot(String),
}

fn here() -> Option<Place> {
    Place::here(command::calling())
}

pub fn seen() -> Vec<Seen> {
    here().map(|place| seen_in(&place)).unwrap_or_default()
}

/// Every assistant Tisty knows, whether or not it is here, with how to connect it by hand.
pub fn known() -> Vec<Seen> {
    let Some(place) = here() else {
        return Vec::new();
    };
    catalog::all()
        .iter()
        .filter_map(|client| told(client, &place, None))
        .collect()
}

pub fn distros() -> Vec<String> {
    places::distros()
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// Looked into only when asked: reading a distro's files starts it.
pub fn seen_in_distro(name: &str) -> Option<(Distro, Vec<Seen>)> {
    let place = distro(name)?;
    let found = seen_in(&place);
    Some((
        Distro {
            name: name.to_string(),
            shut: !place.open,
            command: Some(place.command.clone()),
        },
        found,
    ))
}

fn distro(name: &str) -> Option<Place> {
    let (_, uid) = places::distros().into_iter().find(|(one, _)| one == name)?;
    places::distro(name, uid, &command::calling())
}

fn seen_in(place: &Place) -> Vec<Seen> {
    catalog::all()
        .iter()
        .filter_map(|client| {
            let found = client.found(place)?;
            told(client, place, Some(found))
        })
        .collect()
}

fn told(client: &Client, place: &Place, found: Option<Found>) -> Option<Seen> {
    let at = client.file(place)?;
    let text = std::fs::read_to_string(&at).unwrap_or_default();
    let points = match client.shape {
        Shape::Toml => toml::reads(&text, &client.key, NAME),
        Shape::Manual => None,
        _ => json::reads(&text, &client.key, NAME),
    };
    let wired = points
        .as_deref()
        .is_some_and(|one| same(one, &place.command));
    let id = match &place.distro {
        Some(named) => format!("{WSL}{named}:{}", client.id),
        None => client.id.clone(),
    };
    Some(Seen {
        id,
        name: client.name.clone(),
        also: client.also.clone(),
        place: place.distro.clone(),
        at: at.display().to_string(),
        wired,
        astray: points.is_some() && !wired,
        gone: !wired
            && points
                .as_deref()
                .and_then(|one| place.reached(one))
                .is_some_and(|at| !at.exists()),
        points,
        maybe: found == Some(Found::Maybe),
        by_hand: client.shape == Shape::Manual,
        manual: Manual {
            file: at.display().to_string(),
            snippet: shape::snippet(client.shape, &client.key, &place.command),
            cli: client
                .cli
                .as_ref()
                .map(|said| said.replace("{command}", &shown(&place.command))),
        },
    })
}

fn shown(command: &str) -> String {
    match command.contains(' ') {
        true => format!("\"{command}\""),
        false => command.to_string(),
    }
}

// Paths on Windows and macOS ignore case, so the same file written two ways is still this Tisty.
fn same(one: &str, other: &str) -> bool {
    let tidy = |it: &str| it.trim().replace('\\', "/").to_lowercase();
    tidy(one) == tidy(other)
}

pub fn wire(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, place, at) = asked(id)?;
    wire_at(&client, &place, &at)?;
    Ok(seen_in(&place))
}

fn wire_at(client: &Client, place: &Place, at: &Path) -> Result<(), Stuck> {
    if !place.open {
        return Err(Stuck::Shut);
    }
    let was = std::fs::read_to_string(at).unwrap_or_default();
    let now = match client.shape {
        Shape::Manual => return Err(Stuck::NoSuch),
        Shape::Toml => toml::set(&was, &client.key, NAME, &shape::toml_entry(&place.command)),
        _ => json::set_fields(
            &was,
            &client.key,
            NAME,
            &shape::fields(client.shape, &place.command),
        ),
    }
    .ok_or_else(|| Stuck::Puzzling(at.display().to_string()))?;
    let aside = place.moved_aside(at);
    if aside && !at.is_file() {
        return Err(Stuck::Unborn(at.display().to_string()));
    }
    kept(at, &was)?;
    laid(at, &now, aside)
}

pub fn unwire(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, place, at) = asked(id)?;
    unwire_at(&client, &place, &at)?;
    Ok(seen_in(&place))
}

fn unwire_at(client: &Client, place: &Place, at: &Path) -> Result<(), Stuck> {
    let was = std::fs::read_to_string(at).unwrap_or_default();
    let now = match client.shape {
        Shape::Manual => None,
        Shape::Toml => toml::unset(&was, &client.key, NAME),
        _ => json::unset(&was, &client.key, NAME),
    };
    if let Some(now) = now {
        kept(at, &was)?;
        laid(at, &now, place.moved_aside(at))?;
    }
    Ok(())
}

fn asked(id: &str) -> Result<(Client, Place, PathBuf), Stuck> {
    let (place, client_id) = match id.strip_prefix(WSL).and_then(|rest| rest.rsplit_once(':')) {
        Some((named, client)) => (distro(named).ok_or(Stuck::NoSuch)?, client),
        None => (here().ok_or(Stuck::NoSuch)?, id),
    };
    let client = catalog::all()
        .into_iter()
        .find(|it| it.id == client_id)
        .ok_or(Stuck::NoSuch)?;
    let at = client.file(&place).ok_or(Stuck::NoSuch)?;
    Ok((client, place, at))
}

/// The first original is kept for good, and the one just before this write beside it.
fn kept(at: &Path, was: &str) -> Result<(), Stuck> {
    if was.is_empty() {
        return Ok(());
    }
    let Some(named) = at.file_name().map(|it| it.to_string_lossy().into_owned()) else {
        return Ok(());
    };
    let first = at.with_file_name(format!("{named}.{BEFORE}"));
    if !first.exists() {
        std::fs::write(&first, was).map_err(sour)?;
    }
    std::fs::write(at.with_file_name(format!("{named}.{LAST}")), was).map_err(sour)
}

fn laid(at: &Path, text: &str, aside: bool) -> Result<(), Stuck> {
    if let Some(folder) = at.parent() {
        std::fs::create_dir_all(folder).map_err(sour)?;
    }
    let wrote = match aside && at.is_file() {
        true => tisty_core::store::write_in_place(at, text.as_bytes()),
        false => tisty_core::store::write_atomic(at, text.as_bytes()),
    };
    wrote.map_err(|e| Stuck::Cannot(e.to_string()))
}

fn sour(why: std::io::Error) -> Stuck {
    Stuck::Cannot(why.to_string())
}

#[cfg(test)]
#[path = "wiring_test.rs"]
mod tests;
