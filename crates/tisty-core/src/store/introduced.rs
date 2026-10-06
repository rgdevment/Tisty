use std::path::Path;

use crate::Named;
use crate::event::{DeviceId, Event, Op};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Introduced {
    pub key: Option<String>,
    pub named: Option<Named>,
    pub since: Option<jiff::Timestamp>,
    /// The machine an assistant runs on, by its own word.
    pub host: Option<DeviceId>,
    /// Whether it joined as an agent, by its own word.
    pub agent: bool,
}

fn events_of(device_dir: &Path, who: &DeviceId, loud: bool) -> Vec<Event> {
    let Ok(segments) = super::segments_in(device_dir) else {
        return Vec::new();
    };
    let mut events: Vec<Event> = Vec::new();
    for segment in &segments {
        if let Err(why) = super::read_segment(segment, &mut events)
            && loud
        {
            crate::witness::warn(
                crate::witness::channel::SYNC,
                "a waiting history could not be read whole from the folder",
                &[
                    ("at", crate::witness::Fact::Id(who.0.clone())),
                    ("why", crate::witness::Fact::Why(why.to_string())),
                ],
            );
        }
    }
    events.retain(|one| &one.device == who);
    events.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    events
}

struct Snapshot {
    whole: std::path::PathBuf,
    at: std::path::PathBuf,
}

impl Snapshot {
    // Kept under the machine's own name, which is what its lines are read as.
    fn of(device_dir: &Path, who: &DeviceId) -> Option<Self> {
        let whole = std::env::temp_dir().join(format!("tisty-waiting-{}", ulid::Ulid::generate()));
        let taken = Self {
            at: whole.join(&who.0),
            whole,
        };
        std::fs::create_dir_all(&taken.at).ok()?;
        for entry in std::fs::read_dir(device_dir).ok()?.flatten() {
            if entry.file_type().ok()?.is_file() {
                std::fs::copy(entry.path(), taken.at.join(entry.file_name())).ok()?;
            }
        }
        Some(taken)
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.whole);
    }
}

/// What a machine says about itself in the folder, for one waiting where nothing of it came in.
pub fn introduced_in(device_dir: &Path, who: &DeviceId) -> Introduced {
    let events = events_of(device_dir, who, false);
    Introduced {
        key: key_of(&events, who),
        named: events.iter().rev().find_map(|one| match &one.op {
            Op::DeviceNamed { d, name, os } if d == who => Some(Named {
                name: crate::called::cleaned(name),
                os: os.as_deref().map(crate::called::cleaned),
            }),
            _ => None,
        }),
        since: events.first().map(|one| one.timestamp),
        host: events.iter().find_map(|one| match &one.op {
            Op::DeviceHost { d, of, .. } if d == who => Some(of.clone()),
            _ => None,
        }),
        agent: events.iter().any(|one| {
            matches!(&one.op, Op::DeviceJoin { d, k: Some(crate::event::DeviceKind::Agent), .. } if d == who)
        }),
    }
}

fn key_of(events: &[Event], who: &DeviceId) -> Option<String> {
    events.iter().find_map(|one| match &one.op {
        Op::DeviceKey { d, p } if d == who => Some(p.clone()),
        Op::DeviceJoin { d, p: Some(p), .. } if d == who => Some(p.clone()),
        _ => None,
    })
}

// Read from one copy, so the prints come from the very bytes whose signatures were checked.
pub fn prints_in(
    device_dir: &Path,
    who: &DeviceId,
    known: Option<&str>,
) -> Option<Vec<(crate::model::DocId, String)>> {
    if !super::is_device_name(&who.0) {
        return None;
    }
    let copy = Snapshot::of(device_dir, who)?;
    let events = events_of(&copy.at, who, true);
    let said = match known {
        Some(key) => key.to_string(),
        None => key_of(&events, who)?,
    };
    let by = crate::signing::read(&said)?;
    let reached =
        crate::answering::answers(&copy.at, who, &by, Default::default(), &|_| false).ok()?;
    if !reached.signing {
        return None;
    }
    let mut prints: std::collections::BTreeSet<(crate::model::DocId, String)> = Default::default();
    for one in events {
        match one.op {
            Op::DocSaid { id, d } => {
                if let Some(print) = d.print {
                    prints.insert((id, print));
                }
            }
            Op::DocAdd { id, d } => {
                if let Some(print) = d.said.and_then(|said| said.print) {
                    prints.insert((id, print));
                }
            }
            _ => {}
        }
    }
    Some(prints.into_iter().collect())
}

#[cfg(test)]
#[path = "introduced_test.rs"]
mod tests;
