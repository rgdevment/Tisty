use std::collections::BTreeMap;
use std::path::Path;

use crate::event::{DeviceId, Event, Op};
use crate::model::DocId;

use super::{cuts, versions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logged {
    pub print: String,
    pub wrote: jiff::Timestamp,
    pub via: Option<String>,
    pub device: DeviceId,
    pub bytes: Option<u64>,
    pub replaced: Option<jiff::Timestamp>,
    /// The write that replaced it is where the cut ran, so that machine is the one that keeps it.
    pub replaced_by: Option<DeviceId>,
    pub cut: bool,
}

/// A note that reads the same body again is nobody writing it, so it neither replaces nor cuts.
pub fn logged(events: &[Event], id: DocId) -> Vec<Logged> {
    let mut all: Vec<Logged> = Vec::new();
    for event in events {
        let said = match &event.op {
            Op::DocAdd { id: one, d } if *one == id => d.said.as_ref(),
            Op::DocSaid { id: one, d } if *one == id => Some(d),
            _ => None,
        };
        let Some((said, print)) = said.and_then(|d| d.print.clone().map(|print| (d, print))) else {
            continue;
        };
        if let Some(last) = all.last_mut() {
            if last.print == print {
                continue;
            }
            last.replaced = Some(event.timestamp);
            last.replaced_by = Some(event.device.clone());
            last.cut = cuts(
                Some((last.wrote, last.via.as_deref())),
                event.timestamp,
                event.via.as_deref(),
            );
        }
        all.push(Logged {
            print,
            wrote: event.timestamp,
            via: event.via.clone(),
            device: event.device.clone(),
            bytes: said.bytes,
            replaced: None,
            replaced_by: None,
            cut: false,
        });
    }
    all
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub print: String,
    pub wrote: Option<jiff::Timestamp>,
    pub replaced: jiff::Timestamp,
    pub via: Option<String>,
    pub device: Option<DeviceId>,
    pub bytes: u64,
    pub here: bool,
    /// For a version not here: the device whose write replaced it, on whose machine it is kept.
    pub kept_by: Option<DeviceId>,
}

/// Newest first: what this machine kept and what the same cut and limit give another, never what it says now.
pub fn rows(
    data: &Path,
    file: &str,
    said: &[Logged],
    days: u32,
    now: jiff::Timestamp,
    elsewhere: impl Fn(&DeviceId) -> bool,
) -> Vec<Row> {
    let named: BTreeMap<&str, &Logged> = said
        .iter()
        .filter(|one| one.replaced.is_some())
        .map(|one| (one.print.as_str(), one))
        .collect();
    let says = crate::docs::resolve(&data.join("docs"), file)
        .ok()
        .and_then(|at| crate::docs::print_of(&at).ok().flatten());
    let earlier = |print: &str| says.as_deref() != Some(print);
    let mut all: Vec<Row> = versions(data, file)
        .into_iter()
        .filter(|one| !one.spare && earlier(&one.print))
        .map(|kept| {
            let told = named.get(kept.print.as_str());
            Row {
                wrote: told.map(|one| one.wrote),
                replaced: kept.replaced,
                via: told.and_then(|one| one.via.clone()),
                device: told.map(|one| one.device.clone()),
                bytes: kept.bytes,
                here: true,
                kept_by: None,
                print: kept.print,
            }
        })
        .collect();
    let limit = jiff::SignedDuration::from_hours(24 * i64::from(days));
    let away: Vec<Row> = named
        .into_values()
        .filter(|one| one.cut && one.replaced_by.as_ref().is_some_and(&elsewhere))
        .filter(|one| earlier(&one.print))
        .filter(|one| all.iter().all(|kept| kept.print != one.print))
        .filter_map(|one| {
            let replaced = one.replaced.filter(|at| now.duration_since(*at) <= limit)?;
            Some(Row {
                print: one.print.clone(),
                wrote: Some(one.wrote),
                replaced,
                via: one.via.clone(),
                device: Some(one.device.clone()),
                bytes: one.bytes.unwrap_or_default(),
                here: false,
                kept_by: one.replaced_by.clone(),
            })
        })
        .collect();
    all.extend(away);
    // A body that arrived is replaced here later than the log says, so the order is the one they were written in.
    let written = |one: &Row| one.wrote.unwrap_or(one.replaced);
    all.sort_by(|a, b| written(b).cmp(&written(a)).then(a.print.cmp(&b.print)));
    all
}

#[cfg(test)]
#[path = "logged_test.rs"]
mod tests;
