use crate::State;
use crate::event::{DeviceId, Event, Op};
use crate::model::DocId;

use super::{Version, cuts};

/// A line of the log that wrote a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrote {
    pub at: jiff::Timestamp,
    pub via: Option<String>,
    pub by: DeviceId,
    pub print: String,
    pub bytes: Option<u64>,
}

/// Oldest first, as the log is read; a line that tells the same body again is no hand writing it.
pub fn wrote(log: &[Event], id: DocId) -> Vec<Wrote> {
    let mut all: Vec<Wrote> = Vec::new();
    for event in log {
        let said = match &event.op {
            Op::DocAdd { id: which, d } if *which == id => d.said.as_ref(),
            Op::DocSaid { id: which, d } if *which == id => Some(d),
            _ => None,
        };
        let Some(print) = said.and_then(|one| one.print.clone()) else {
            continue;
        };
        if all.last().is_some_and(|last| last.print == print) {
            continue;
        }
        all.push(Wrote {
            at: event.timestamp,
            via: event.via.clone(),
            by: event.device.clone(),
            print,
            bytes: said.and_then(|one| one.bytes),
        });
    }
    all
}

/// The machine a device writes from: an assistant's is its host's, and one nobody hosts is unknown.
pub fn machine(state: &State, device: &DeviceId) -> Option<DeviceId> {
    match state.hosts.get(device) {
        Some(host) => Some(host.clone()),
        None => (!state.assistants.contains(device)).then(|| device.clone()),
    }
}

/// One past version, as a list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub print: String,
    /// The line of the log that wrote this text, when one answers for it.
    pub wrote: Option<Wrote>,
    pub replaced: jiff::Timestamp,
    pub bytes: Option<u64>,
    /// The machine that wrote over it and keeps it, when its text is not on this one.
    pub elsewhere: Option<DeviceId>,
}

/// What decides whether a version the log names is on another machine.
pub struct Here<'a> {
    /// This machine, as the log names it.
    pub machine: &'a DeviceId,
    /// The print of the body this disk holds now, which is here whatever the log says came after.
    pub says: Option<&'a str>,
    /// What another machine kept before this moment is past any limit, and is left out.
    pub since: jiff::Timestamp,
}

/// Newest first: what this machine keeps, then what the same cut kept only on another machine.
/// A version the log names that this machine wrote over and no longer keeps is gone, and is left out.
pub fn shown(
    kept: &[Version],
    wrote: &[Wrote],
    machine: impl Fn(&DeviceId) -> Option<DeviceId>,
    here: &Here,
) -> Vec<Shown> {
    let written = |one: &Version| {
        let lines = || wrote.iter().rev().filter(|line| line.print == one.print);
        lines()
            .find(|line| line.at <= one.replaced)
            .or_else(|| lines().next())
            .cloned()
    };
    let mut all: Vec<Shown> = kept
        .iter()
        .filter(|one| !one.spare)
        .map(|one| Shown {
            print: one.print.clone(),
            wrote: written(one),
            replaced: one.replaced,
            bytes: Some(one.bytes),
            elsewhere: None,
        })
        .collect();

    let told_last = wrote.last().map(|one| one.print.as_str());
    for pair in wrote.windows(2).rev() {
        let (was, next) = (&pair[0], &pair[1]);
        if next.at < here.since
            || !cuts(
                Some((was.at, was.via.as_deref())),
                next.at,
                next.via.as_deref(),
            )
        {
            continue;
        }
        let said = Some(was.print.as_str());
        let here_already = said == told_last
            || said == here.says
            || kept.iter().any(|one| one.print == was.print)
            || all.iter().any(|one| one.print == was.print);
        if here_already {
            continue;
        }
        let Some(there) = machine(&next.by).filter(|there| there != here.machine) else {
            continue;
        };
        all.push(Shown {
            print: was.print.clone(),
            wrote: Some(was.clone()),
            replaced: next.at,
            bytes: was.bytes,
            elsewhere: Some(there),
        });
    }
    all.sort_by(|a, b| b.replaced.cmp(&a.replaced).then(a.print.cmp(&b.print)));
    all
}

#[cfg(test)]
#[path = "shown_test.rs"]
mod tests;
