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

/// What a machine says about itself in the folder, for one waiting where nothing of it came in.
pub fn introduced_in(device_dir: &Path, who: &DeviceId) -> Introduced {
    let Ok(segments) = super::segments_in(device_dir) else {
        return Introduced::default();
    };
    let mut events: Vec<Event> = Vec::new();
    for segment in &segments {
        let _ = super::read_segment(segment, &mut events);
    }
    events.retain(|one| &one.device == who);
    events.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
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

#[cfg(test)]
#[path = "introduced_test.rs"]
mod tests;
