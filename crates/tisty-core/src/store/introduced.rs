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

fn events_of(device_dir: &Path, who: &DeviceId) -> Vec<Event> {
    let Ok(segments) = super::segments_in(device_dir) else {
        return Vec::new();
    };
    let mut events: Vec<Event> = Vec::new();
    for segment in &segments {
        if let Err(why) = super::read_segment(segment, &mut events) {
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
    events.retain(|one| one.device.0.eq_ignore_ascii_case(&who.0));
    events.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    events
}

/// What a machine says about itself in the folder, for one waiting where nothing of it came in.
pub fn introduced_in(device_dir: &Path, who: &DeviceId) -> Introduced {
    let events = events_of(device_dir, who);
    Introduced {
        key: events.iter().find_map(|one| match &one.op {
            Op::DeviceKey { d, p } if d == who => Some(p.clone()),
            Op::DeviceJoin { d, p: Some(p), .. } if d == who => Some(p.clone()),
            _ => None,
        }),
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

// Only a history that holds up whole under the key it says may vouch for a body, or anyone who can write the folder could silence the question.
pub fn prints_in(device_dir: &Path, who: &DeviceId) -> Option<Vec<(crate::model::DocId, String)>> {
    let said = super::key_said_in(device_dir, who)?;
    let by = crate::signing::read(&said)?;
    let reached =
        crate::answering::answers(device_dir, who, &by, Default::default(), &|_| false).ok()?;
    if !reached.signing {
        return None;
    }
    Some(
        events_of(device_dir, who)
            .into_iter()
            .filter_map(|one| match one.op {
                Op::DocSaid { id, d } => d.print.map(|print| (id, print)),
                Op::DocAdd { id, d } => d.said.and_then(|said| said.print).map(|print| (id, print)),
                _ => None,
            })
            .collect(),
    )
}

#[cfg(test)]
#[path = "introduced_test.rs"]
mod tests;
