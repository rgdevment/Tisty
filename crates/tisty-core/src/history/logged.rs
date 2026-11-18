use std::collections::{BTreeMap, BTreeSet};
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
    /// The name a body that came in shared was signed with, which is not the person's here.
    pub signed: Option<String>,
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
            signed: said.by.clone(),
            bytes: said.bytes,
            replaced: None,
            replaced_by: None,
            cut: false,
        });
    }
    all
}

/// Which machine each device writes from: this one, another the person has, or one they removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machines {
    machine: DeviceId,
    agent: Option<DeviceId>,
    hosts: BTreeMap<DeviceId, DeviceId>,
    assistants: BTreeSet<DeviceId>,
    removed: BTreeSet<DeviceId>,
}

impl Machines {
    pub fn of(state: &crate::State, machine: &DeviceId, agent: Option<&DeviceId>) -> Self {
        Self {
            machine: machine.clone(),
            agent: agent.cloned(),
            hosts: state.hosts.clone(),
            assistants: state.assistants.clone(),
            removed: state
                .dropped
                .iter()
                .filter(|one| !state.assistants.contains(*one))
                .cloned()
                .collect(),
        }
    }

    /// A retired assistant loses its host from the state, but the log still says where it lived last.
    pub fn told_by(mut self, log: &[Event]) -> Self {
        let mut said = BTreeMap::new();
        for event in log {
            if let Op::DeviceHost { d, of, .. } = &event.op
                && (event.device == *d || event.device == *of)
                && self.assistants.contains(d)
                && !self.assistants.contains(of)
            {
                said.insert(d.clone(), of.clone());
            }
        }
        for (agent, host) in said {
            self.hosts.entry(agent).or_insert(host);
        }
        self
    }

    /// The machine a device writes from; an assistant whose host nobody declared names none.
    pub fn machine_of(&self, device: &DeviceId) -> Option<DeviceId> {
        match self.hosts.get(device) {
            Some(host) => Some(host.clone()),
            None => (!self.assistants.contains(device)).then(|| device.clone()),
        }
    }

    /// Whether what this device wrote over may be kept on another machine: not this one, nor one removed.
    pub fn elsewhere(&self, device: &DeviceId) -> bool {
        if *device == self.machine || self.agent.as_ref() == Some(device) {
            return false;
        }
        match self.machine_of(device) {
            Some(on) => on != self.machine && !self.removed.contains(&on),
            None => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub print: String,
    pub wrote: Option<jiff::Timestamp>,
    pub replaced: jiff::Timestamp,
    pub via: Option<String>,
    pub device: Option<DeviceId>,
    pub signed: Option<String>,
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
    let mut named: BTreeMap<&str, Vec<&Logged>> = BTreeMap::new();
    for one in said.iter().filter(|one| one.replaced.is_some()) {
        named.entry(one.print.as_str()).or_default().push(one);
    }
    // A body written more than once is told by the writing that stood when it was set aside.
    let told_of = |kept: &super::Version| {
        let all = named.get(kept.print.as_str())?;
        all.iter()
            .rev()
            .find(|one| one.wrote <= kept.replaced)
            .or_else(|| all.last())
            .copied()
    };
    let says = crate::docs::resolve(&data.join("docs"), file)
        .ok()
        .and_then(|at| crate::docs::print_of(&at).ok().flatten());
    let earlier = |print: &str| says.as_deref() != Some(print);
    let mut all: Vec<Row> = versions(data, file)
        .into_iter()
        .filter(|one| !one.spare && earlier(&one.print))
        .map(|kept| {
            let told = told_of(&kept);
            Row {
                wrote: told.map(|one| one.wrote),
                replaced: kept.replaced,
                via: told.and_then(|one| one.via.clone()),
                device: told.map(|one| one.device.clone()),
                signed: told.and_then(|one| one.signed.clone()),
                bytes: kept.bytes,
                here: true,
                kept_by: None,
                print: kept.print,
            }
        })
        .collect();
    let limit = jiff::SignedDuration::from_hours(24 * i64::from(days));
    let mut listed: std::collections::BTreeSet<&str> =
        all.iter().map(|kept| kept.print.as_str()).collect();
    // Every time a body was replaced counts: another machine may keep it from an earlier one.
    let away: Vec<Row> = said
        .iter()
        .rev()
        .filter(|one| one.cut && one.replaced_by.as_ref().is_some_and(&elsewhere))
        .filter(|one| earlier(&one.print))
        .filter_map(|one| {
            let replaced = one.replaced.filter(|at| now.duration_since(*at) <= limit)?;
            if !listed.insert(one.print.as_str()) {
                return None;
            }
            Some(Row {
                print: one.print.clone(),
                wrote: Some(one.wrote),
                replaced,
                via: one.via.clone(),
                device: Some(one.device.clone()),
                signed: one.signed.clone(),
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
