use std::path::Path;

use crate::Named;
use crate::event::{DeviceId, Event, Op};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Introduced {
    pub key: Option<String>,
    pub named: Option<Named>,
    pub since: Option<jiff::Timestamp>,
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
                name: crate::text::plainly(name),
                os: os.clone(),
            }),
            _ => None,
        }),
        since: events.first().map(|one| one.timestamp),
    }
}
