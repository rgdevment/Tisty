use std::io::Write;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tisty_share_proto::cloud::{CloudKey, Mark};
use tisty_share_proto::handing::{Ask, Handed};

use super::budget::{Class, Metered};
use crate::{Expect, Hitch, Remote};

pub const KNOCK: &str = "knock";
const ASK: &str = ".ask";
const KEY: &str = ".key";
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

struct Bounded<'a>(&'a mut Vec<u8>);

impl Write for Bounded<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len() as u64 + bytes.len() as u64 > AT_MOST {
            return Err(std::io::Error::other("too large to be a knock"));
        }
        self.0.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn read<T: DeserializeOwned>(remote: &dyn Remote, name: &str) -> Result<Option<T>, Hitch> {
    let mut said = Vec::new();
    match remote.fetch(name, 0, &mut Bounded(&mut said)) {
        Ok(_) => Ok(serde_json::from_slice(&said).ok()),
        Err(Hitch::Missing(_)) => Ok(None),
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

/// A new ask stands in for an old one, and the answer to the old one goes with it.
pub fn knocked(plain: &dyn Remote, home: &Path, ask: &Ask) -> Result<(), Hitch> {
    let metered = Metered::over(plain, home, Class::Sealing);
    gone(&metered, &named(&ask.device, KEY)?)?;
    written(&metered, &named(&ask.device, ASK)?, ask, Expect::Any)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Knock {
    pub ask: Ask,
    pub answered: bool,
}

/// Only asks that read whole and name the machine their file is named for; the rest is noise to the reader.
pub fn knocks(plain: &dyn Remote, home: &Path) -> Result<Vec<Knock>, Hitch> {
    let metered = Metered::over(plain, home, Class::Sealing);
    let listed = metered.list(KNOCK)?;
    let answered = |device: &str| {
        listed
            .iter()
            .any(|one| named(device, KEY).is_ok_and(|name| name == one.name))
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
        if let Some(ask) = read::<Ask>(&metered, &one.name)?.filter(|ask| ask.device == device) {
            found.push(Knock {
                answered: answered(device),
                ask,
            });
        }
    }
    Ok(found)
}

/// Whoever answers first stands, so a second machine answering the same ask changes nothing.
pub fn answer(plain: &dyn Remote, home: &Path, device: &str, handed: &Handed) -> Result<(), Hitch> {
    let metered = Metered::over(plain, home, Class::Sealing);
    match written(&metered, &named(device, KEY)?, handed, Expect::Absent) {
        Ok(()) | Err(Hitch::Changed(_)) => Ok(()),
        Err(other) => Err(other),
    }
}

pub fn answered(plain: &dyn Remote, home: &Path, device: &str) -> Result<Option<Handed>, Hitch> {
    let metered = Metered::over(plain, home, Class::Sealing);
    read(&metered, &named(device, KEY)?)
}

pub fn withdrawn(plain: &dyn Remote, home: &Path, device: &str) -> Result<(), Hitch> {
    let metered = Metered::over(plain, home, Class::Sealing);
    gone(&metered, &named(device, ASK)?)?;
    gone(&metered, &named(device, KEY)?)
}

/// A provider can swap an answer, but cannot make a key that opens this cloud's mark.
pub fn opens(plain: &dyn Remote, home: &Path, key: &CloudKey, cloud: &str) -> bool {
    let sealed = crate::Sealed::new(
        Metered::over(plain, home, Class::Sealing),
        key.clone(),
        cloud,
    );
    matches!(sealed.marked(), Ok(Some(Mark::Holds)))
}

#[cfg(test)]
#[path = "knock_test.rs"]
mod tests;
