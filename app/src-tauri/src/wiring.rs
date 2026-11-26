pub mod agents;
mod catalog;
mod files;
pub(crate) mod hooks;
mod json;
mod leads;
mod patch;
mod places;
mod running;
mod shape;
mod toml;
mod user_hook_table;
pub mod user_hooks;

use std::path::{Path, PathBuf};

use catalog::{Client, Found, Shape};
use files::{Read, Writes};
use leads::{Leads, leads};
use places::Place;

use crate::command;

const NAME: &str = "tisty";
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
    /// It holds an entry of Tisty's that nothing here can tell, so it is left as the person wrote it.
    pub apart: bool,
    pub unread: bool,
    pub installed: bool,
    pub points: Option<String>,
    /// Only a folder that outlives an uninstall says it is here.
    pub maybe: bool,
    /// Its shape is not one Tisty writes, so it is connected by hand.
    pub by_hand: bool,
    pub manual: Manual,
    /// Only the assistants that run hooks have them.
    pub hooks: Option<hooks::Hooks>,
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
    Unread(String),
    Moved(String),
    Plugged,
    Unplugged,
    PluginUnread(String),
    Puzzling(String),
    Unfixable(String),
    HooksApart(String),
    HooksPuzzling(String),
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
    let (text, unread) = match files::read(&at) {
        Read::Text(text) => (text, false),
        Read::Missing => (String::new(), false),
        Read::Unread => (String::new(), true),
    };
    let points = client.reads(&text);
    let leads = points.as_deref().map(|one| leads(place, one));
    let wired = leads == Some(Leads::Here);
    let astray = leads.is_some_and(Leads::astray);
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
        astray,
        gone: leads == Some(Leads::Gone),
        apart: !wired && !astray && client.holds(&text),
        unread,
        installed: client.installed(place),
        points,
        maybe: found == Some(Found::Maybe),
        by_hand: client.shape == Shape::Manual,
        manual: Manual {
            file: at.display().to_string(),
            snippet: shape::snippet(client.shape, &client.key, NAME, &place.command),
            cli: client
                .cli
                .as_ref()
                .map(|said| said.replace("{command}", &shown(&place.command))),
        },
        hooks: hooks::kind(&client.id).and_then(|kind| hooks::seen(kind, place)),
    })
}

fn shown(command: &str) -> String {
    match command.contains(' ') {
        true => format!("\"{command}\""),
        false => command.to_string(),
    }
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
    let was = files::text(at)?;
    let there = client.holds(&was);
    let told = client.reads(&was).map(|one| leads(place, &one));
    if there && matches!(told, None | Some(Leads::Unsaid)) {
        return Err(Stuck::Moved(at.display().to_string()));
    }
    let puzzling = || match there {
        true => Stuck::Unfixable(at.display().to_string()),
        false => Stuck::Puzzling(at.display().to_string()),
    };
    let now = match client.shape {
        Shape::Manual => return Err(Stuck::NoSuch),
        Shape::Toml => toml::set(&was, &client.key, NAME, &place.command),
        Shape::Patch => patch::set(&was, &client.key, NAME, &place.command),
        _ => json::set_fields(
            &was,
            &client.key,
            NAME,
            &shape::fields(client.shape, &place.command),
        )
        .filter(|now| client.sound(&was, now)),
    }
    .ok_or_else(puzzling)?;
    if client.reads(&now).as_deref() != Some(place.command.as_str()) {
        return Err(puzzling());
    }
    let aside = place.moved_aside(at);
    if aside && !at.is_file() {
        return Err(Stuck::Unborn(at.display().to_string()));
    }
    Writes::default().change(at, &was, &now, aside)
}

pub fn unwire(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, place, at) = asked(id)?;
    unwire_at(&client, &place, &at)?;
    Ok(seen_in(&place))
}

// Both texts are worked out before either is written, so a file that cannot be read leaves both as they were.
fn unwire_at(client: &Client, place: &Place, at: &Path) -> Result<(), Stuck> {
    let was = files::text(at)?;
    let now = match client.holds(&was) {
        true => {
            disconnected(client, &was).ok_or_else(|| Stuck::Unfixable(at.display().to_string()))?
        }
        false => was.clone(),
    };
    let mut writes = Writes::default();
    let hooked = hooks::kind(&client.id).and_then(|kind| Some((kind, kind.file(place)?)));
    let Some((kind, beside)) = hooked else {
        return writes.change(at, &was, &now, place.moved_aside(at));
    };
    if beside == at {
        let now = unhooked(kind, at, &now)?;
        return writes.change(at, &was, &now, place.moved_aside(at));
    }
    let hooks_was = files::text(&beside)?;
    let hooks_now = unhooked(kind, &beside, &hooks_was)?;
    writes.change(at, &was, &now, place.moved_aside(at))?;
    unhook_written(kind, place, &beside, &hooks_was, &hooks_now, &mut writes)
}

fn disconnected(client: &Client, was: &str) -> Option<String> {
    match client.shape {
        Shape::Manual => None,
        Shape::Toml => toml::unset(was, &client.key, NAME),
        Shape::Patch => patch::unset(was, &client.key, NAME),
        _ => json::unset(was, &client.key, NAME).filter(|now| client.sound(was, now)),
    }
}

pub fn hook(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, place, _) = asked(id)?;
    unplugged(&client, homes_of(&place))?;
    hook_at(&client, &place)?;
    Ok(seen_in(&place))
}

fn hook_at(client: &Client, place: &Place) -> Result<(), Stuck> {
    if !place.open {
        return Err(Stuck::Shut);
    }
    let (kind, at) = hooks_of(client, place)?;
    let was = files::text(&at)?;
    if hooks::apart(kind, &was) {
        return Err(Stuck::HooksApart(at.display().to_string()));
    }
    let now = hooks::with(kind, &was, place)
        .ok_or_else(|| Stuck::HooksPuzzling(at.display().to_string()))?;
    Writes::default().change(&at, &was, &now, place.moved_aside(&at))
}

fn homes_of(place: &Place) -> Option<agents::Homes> {
    match place.distro {
        Some(_) => agents::Homes::of(&place.home),
        None => agents::Homes::here(),
    }
}

// The plugin's own hooks would run beside these, so without its records read none is written.
fn unplugged(client: &Client, homes: Option<agents::Homes>) -> Result<(), Stuck> {
    if hooks::kind(&client.id) != Some(hooks::Kind::Claude) {
        return Ok(());
    }
    match brought(homes)? {
        false => Ok(()),
        true => Err(Stuck::Plugged),
    }
}

fn brought(homes: Option<agents::Homes>) -> Result<bool, Stuck> {
    homes
        .map_or(Ok(false), |homes| homes.plugin_brings_hooks())
        .map_err(|at| Stuck::PluginUnread(at.display().to_string()))
}

fn still_plugged(client: &Client, homes: Option<agents::Homes>) -> Result<(), Stuck> {
    let claude = hooks::kind(&client.id) == Some(hooks::Kind::Claude);
    match claude && brought(homes)? {
        true => Ok(()),
        false => Err(Stuck::Unplugged),
    }
}

fn hooks_of(client: &Client, place: &Place) -> Result<(hooks::Kind, PathBuf), Stuck> {
    let kind = hooks::kind(&client.id).ok_or(Stuck::NoSuch)?;
    let at = kind.file(place).ok_or(Stuck::NoSuch)?;
    Ok((kind, at))
}

pub fn unhook(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, place, _) = asked(id)?;
    unhook_at(&client, &place)?;
    Ok(seen_in(&place))
}

fn unhook_at(client: &Client, place: &Place) -> Result<(), Stuck> {
    let (kind, at) = hooks_of(client, place)?;
    let was = files::text(&at)?;
    let now = unhooked(kind, &at, &was)?;
    unhook_written(kind, place, &at, &was, &now, &mut Writes::default())
}

fn unhooked(kind: hooks::Kind, at: &Path, was: &str) -> Result<String, Stuck> {
    match hooks::hooked(kind, was) {
        true => {
            hooks::without(kind, was).ok_or_else(|| Stuck::HooksPuzzling(at.display().to_string()))
        }
        false => Ok(was.to_string()),
    }
}

fn unhook_written(
    kind: hooks::Kind,
    place: &Place,
    at: &Path,
    was: &str,
    now: &str,
    writes: &mut Writes,
) -> Result<(), Stuck> {
    match now != was && hooks::alone(kind) && hooks::emptied(now) {
        true => writes.remove(at, was),
        false => writes.change(at, was, now, place.moved_aside(at)),
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Mend {
    /// What the connection was said to point at, so nothing that changed since is written over.
    pub connection: Option<String>,
    pub hooks: Vec<String>,
    /// The plugin brings them too, so the ones left behind go instead of running twice.
    pub hooks_out: bool,
}

#[derive(Debug)]
pub struct Mended {
    pub seen: Vec<Seen>,
    pub connection: Option<Stuck>,
    pub hooks: Option<Stuck>,
}

pub fn mend(id: &str, ask: &Mend) -> Result<Mended, Stuck> {
    let (client, place, at) = asked(id)?;
    let (connection, hooks) = mend_guarded(&client, &place, &at, ask, homes_of(&place));
    Ok(Mended {
        seen: seen_in(&place),
        connection,
        hooks,
    })
}

fn mend_guarded(
    client: &Client,
    place: &Place,
    at: &Path,
    ask: &Mend,
    homes: Option<agents::Homes>,
) -> (Option<Stuck>, Option<Stuck>) {
    let refused = match (ask.hooks.is_empty(), ask.hooks_out) {
        (true, _) => Ok(()),
        (false, true) => still_plugged(client, homes),
        (false, false) => unplugged(client, homes),
    };
    let Err(why) = refused else {
        return mend_at(client, place, at, ask);
    };
    let connection = Mend {
        hooks: Vec::new(),
        ..ask.clone()
    };
    (mend_at(client, place, at, &connection).0, Some(why))
}

fn mend_at(
    client: &Client,
    place: &Place,
    at: &Path,
    ask: &Mend,
) -> (Option<Stuck>, Option<Stuck>) {
    if !place.open {
        return (Some(Stuck::Shut), Some(Stuck::Shut));
    }
    let mut writes = Writes::default();
    let connection = ask
        .connection
        .as_deref()
        .and_then(|named| repointed(client, place, at, named, &mut writes).err());
    let hooks = match ask.hooks.is_empty() {
        true => None,
        false => hooks_repointed(client, place, ask, &mut writes).err(),
    };
    (connection, hooks)
}

fn repointed(
    client: &Client,
    place: &Place,
    at: &Path,
    named: &str,
    writes: &mut Writes,
) -> Result<(), Stuck> {
    let was = files::text(at)?;
    if client.reads(&was).as_deref() != Some(named) || !leads(place, named).astray() {
        return Err(Stuck::Moved(at.display().to_string()));
    }
    let now = match client.shape {
        Shape::Manual => None,
        Shape::Toml => toml::repoint(&was, &client.key, NAME, &place.command),
        Shape::Patch => patch::repoint(&was, &client.key, NAME, &place.command),
        _ => json::repoint(&was, &client.key, NAME, &place.command)
            .filter(|now| client.sound(&was, now)),
    }
    .ok_or_else(|| Stuck::Unfixable(at.display().to_string()))?;
    writes.change(at, &was, &now, place.moved_aside(at))
}

fn hooks_repointed(
    client: &Client,
    place: &Place,
    ask: &Mend,
    writes: &mut Writes,
) -> Result<(), Stuck> {
    let (kind, at) = hooks_of(client, place)?;
    let shown = at.display().to_string();
    let was = files::text(&at)?;
    let now = match ask.hooks_out {
        true => hooks::without_those(kind, &was, place, &ask.hooks),
        false => hooks::repointed(kind, &was, place, &ask.hooks),
    }
    .ok_or_else(|| Stuck::HooksPuzzling(shown.clone()))?;
    match now == was {
        true => Err(Stuck::Moved(shown)),
        false if ask.hooks_out => unhook_written(kind, place, &at, &was, &now, writes),
        false => writes.change(&at, &was, &now, place.moved_aside(&at)),
    }
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

#[cfg(test)]
#[path = "wiring_test.rs"]
mod tests;

#[cfg(test)]
#[path = "wiring_shapes_test.rs"]
mod shapes_tests;
