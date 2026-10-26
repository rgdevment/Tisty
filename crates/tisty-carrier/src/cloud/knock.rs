use std::io::Write;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tisty_share_proto::cloud::{CloudKey, Mark};
use tisty_share_proto::handing::{Ask, Giver, Handed};

use super::budget::{Class, Metered};
use crate::{Expect, Hitch, Remote};

pub const KNOCK: &str = "knock";
const ASK: &str = ".ask";
const KEY: &str = ".key";
const GIVER: &str = ".giver";
const OPEN: &str = ".open";

#[derive(serde::Serialize, serde::Deserialize)]
struct Open {
    reveal: String,
}
const AT_MOST: u64 = 4 * 1024;

fn broke(e: impl ToString) -> Hitch {
    Hitch::Broke(e.to_string())
}

// Anything else would let a name climb out of the box or collide with another machine's.
fn plain_name(device: &str) -> bool {
    !device.is_empty()
        && device.len() <= 64
        && device
            .chars()
            .all(|one| one.is_ascii_alphanumeric() || one == '_' || one == '-')
}

fn named(device: &str, end: &str) -> Result<String, Hitch> {
    match plain_name(device) {
        true => Ok(format!("{KNOCK}/{device}{end}")),
        false => Err(Hitch::Barred(tisty_sync::Trouble::Refused(format!(
            "{device} cannot knock"
        )))),
    }
}

// Stops the download at the cap and remembers why, so a file too large reads as unreadable, not as a failed fetch.
#[derive(Default)]
struct Bounded {
    said: Vec<u8>,
    over: bool,
}

impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.said.len() as u64 + bytes.len() as u64 > AT_MOST {
            self.over = true;
            return Err(std::io::Error::other("too large to be a knock"));
        }
        self.said.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

enum Found<T> {
    Missing,
    Unread,
    Read(T),
}

fn read<T: DeserializeOwned>(remote: &dyn Remote, name: &str) -> Result<Found<T>, Hitch> {
    let mut bounded = Bounded::default();
    let fetched = remote.fetch(name, 0, &mut bounded);
    if bounded.over {
        return Ok(Found::Unread);
    }
    match fetched {
        Ok(_) => Ok(serde_json::from_slice(&bounded.said).map_or(Found::Unread, Found::Read)),
        Err(Hitch::Missing(_)) => Ok(Found::Missing),
        Err(other) => Err(other),
    }
}

fn written(
    remote: &dyn Remote,
    name: &str,
    said: &impl Serialize,
    expect: Expect,
) -> Result<(), Hitch> {
    let held = crate::temp::Temp::new();
    let mut file = held.create().map_err(broke)?;
    file.write_all(&serde_json::to_vec(said).map_err(broke)?)
        .map_err(broke)?;
    drop(file);
    remote.put(name, &held.0, expect).map(|_| ())
}

fn gone(remote: &dyn Remote, name: &str) -> Result<(), Hitch> {
    match remote.delete(name, None) {
        Ok(()) | Err(Hitch::Missing(_)) => Ok(()),
        Err(other) => Err(other),
    }
}

fn boxed<'a>(plain: &'a dyn Remote, home: &Path) -> Metered<'a> {
    Metered::over(plain, home, Class::Sealing)
}

pub fn knocked(plain: &dyn Remote, home: &Path, ask: &Ask) -> Result<(), Hitch> {
    let metered = boxed(plain, home);
    gone(&metered, &named(&ask.device, KEY)?)?;
    gone(&metered, &named(&ask.device, GIVER)?)?;
    gone(&metered, &named(&ask.device, OPEN)?)?;
    written(&metered, &named(&ask.device, ASK)?, ask, Expect::Any)
}

/// An ask already answered is not read again: what it says is needed only until it is answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Knock {
    pub device: String,
    pub ask: Option<Ask>,
    pub offered: bool,
}

pub fn knocks(plain: &dyn Remote, home: &Path) -> Result<Vec<Knock>, Hitch> {
    let metered = boxed(plain, home);
    let listed = metered.list(KNOCK)?;
    let there = |device: &str, end: &str| {
        listed
            .iter()
            .any(|one| named(device, end).is_ok_and(|name| name == one.name))
    };
    let mut found = Vec::new();
    for one in &listed {
        let Some(device) = one
            .name
            .strip_prefix(&format!("{KNOCK}/"))
            .and_then(|leaf| leaf.strip_suffix(ASK))
        else {
            continue;
        };
        if one.bytes > AT_MOST || !plain_name(device) {
            continue;
        }
        let offered = there(device, GIVER);
        if there(device, KEY) {
            found.push(Knock {
                device: device.to_string(),
                ask: None,
                offered,
            });
            continue;
        }
        // One ask that cannot be read now is passed over this time, never the rest with it.
        if let Ok(Found::Read(ask)) = read::<Ask>(&metered, &one.name)
            && ask.device == device
        {
            found.push(Knock {
                device: device.to_string(),
                ask: Some(ask),
                offered,
            });
        }
    }
    Ok(found)
}

/// Whoever answers first stands, so a second machine answering the same ask changes nothing.
pub fn answer(plain: &dyn Remote, home: &Path, device: &str, handed: &Handed) -> Result<(), Hitch> {
    let metered = boxed(plain, home);
    match written(&metered, &named(device, KEY)?, handed, Expect::Absent) {
        Ok(()) | Err(Hitch::Changed(_)) => Ok(()),
        Err(other) => Err(other),
    }
}

/// An answer that cannot be read goes at once, since every machine with the key takes it for answered.
pub fn answered(plain: &dyn Remote, home: &Path, device: &str) -> Result<Option<Handed>, Hitch> {
    let metered = boxed(plain, home);
    let name = named(device, KEY)?;
    match read(&metered, &name)? {
        Found::Read(handed) => Ok(Some(handed)),
        Found::Unread => gone(&metered, &name).map(|()| None),
        Found::Missing => Ok(None),
    }
}

pub fn ask_of(plain: &dyn Remote, home: &Path, device: &str) -> Result<Option<Ask>, Hitch> {
    match read(&boxed(plain, home), &named(device, ASK)?)? {
        Found::Read(ask) => Ok(Some(ask).filter(|ask: &Ask| ask.device == device)),
        Found::Unread | Found::Missing => Ok(None),
    }
}

/// The first machine to take an ask stands; false says another took it first.
pub fn offered(
    plain: &dyn Remote,
    home: &Path,
    device: &str,
    giver: &Giver,
) -> Result<bool, Hitch> {
    match written(
        &boxed(plain, home),
        &named(device, GIVER)?,
        giver,
        Expect::Absent,
    ) {
        Ok(()) => Ok(true),
        Err(Hitch::Changed(_)) => Ok(false),
        Err(other) => Err(other),
    }
}

pub fn giver_of(plain: &dyn Remote, home: &Path, device: &str) -> Result<Option<Giver>, Hitch> {
    match read(&boxed(plain, home), &named(device, GIVER)?)? {
        Found::Read(giver) => Ok(Some(giver)),
        Found::Unread | Found::Missing => Ok(None),
    }
}

// Put up only once a machine took the ask, so the reveal comes after the giver and never before it.
pub fn revealed(plain: &dyn Remote, home: &Path, device: &str, reveal: &str) -> Result<(), Hitch> {
    let open = Open {
        reveal: reveal.to_string(),
    };
    written(
        &boxed(plain, home),
        &named(device, OPEN)?,
        &open,
        Expect::Any,
    )
}

pub fn reveal_of(plain: &dyn Remote, home: &Path, device: &str) -> Result<Option<String>, Hitch> {
    match read::<Open>(&boxed(plain, home), &named(device, OPEN)?)? {
        Found::Read(open) => Ok(Some(open.reveal)),
        Found::Unread | Found::Missing => Ok(None),
    }
}

pub fn unanswered(plain: &dyn Remote, home: &Path, device: &str) -> Result<(), Hitch> {
    gone(&boxed(plain, home), &named(device, KEY)?)
}

pub fn withdrawn(plain: &dyn Remote, home: &Path, device: &str) -> Result<(), Hitch> {
    gone(&boxed(plain, home), &named(device, ASK)?)?;
    gone(&boxed(plain, home), &named(device, GIVER)?)?;
    gone(&boxed(plain, home), &named(device, OPEN)?)?;
    unanswered(plain, home, device)
}

/// Says only that the key is this cloud's; who handed it is the caller's to check, since the mark sits with the provider.
// A mark that could not be read says nothing, so a key is never judged wrong on a failed fetch.
pub fn opens(plain: &dyn Remote, home: &Path, key: &CloudKey, cloud: &str) -> Result<bool, Hitch> {
    let sealed = crate::Sealed::new(boxed(plain, home), key.clone(), cloud);
    Ok(matches!(sealed.marked()?, Some(Mark::Holds)))
}

#[cfg(test)]
#[path = "knock_test.rs"]
mod tests;
