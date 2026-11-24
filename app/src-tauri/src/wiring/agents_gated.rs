use std::collections::BTreeMap;

use tisty_core::State;
use tisty_core::agents::{Asked, Gate, Ledger, gate};
use tisty_core::event::{DeviceId, DocAdd, Event, FolderAdd, Op, Said};

pub(crate) const ME: &str = "dev_here";

pub(crate) struct Gated {
    _docs: tempfile::TempDir,
    pub(crate) gate: Gate,
    names: BTreeMap<String, String>,
}

impl Gated {
    pub(crate) fn id(&self, name: &str) -> &str {
        self.names
            .iter()
            .find(|(_, one)| *one == name)
            .map(|(id, _)| id.as_str())
            .expect("a document by that name")
    }

    pub(crate) fn named(&self, ids: &[String]) -> Vec<&str> {
        ids.iter().map(|id| self.names[id].as_str()).collect()
    }
}

// The same name gives the same id in every call, in the same order as the names.
pub(crate) fn id_of(name: &str) -> String {
    let mut first = [0u8; 4];
    for (at, byte) in name.bytes().take(4).enumerate() {
        first[at] = byte;
    }
    format!("{ME}-{:010}", u32::from_be_bytes(first))
}

pub(crate) fn consented() -> Ledger {
    let mut ledger = Ledger::fresh("machine-here");
    ledger.consent(true);
    ledger
}

pub(crate) fn gated(ledger: &Ledger, docs: &[(String, String)]) -> Gated {
    gated_in(ledger, docs, &[])
}

/// `rooms` names the documents that sit in a computer's folder under AGENTS, by the computer.
pub(crate) fn gated_in(
    ledger: &Ledger,
    docs: &[(String, String)],
    rooms: &[(&str, &str)],
) -> Gated {
    let dir = tempfile::tempdir().unwrap();
    let me = DeviceId(ME.into());
    let mut state = State::default();
    let mut ms = 0;
    let mut apply_by = |state: &mut State, who: &DeviceId, op: Op| {
        ms += 1;
        let at = jiff::Timestamp::from_millisecond(ms).unwrap();
        state.apply(&Event::new(who.clone(), at, op));
    };
    let root = ulid::Ulid::generate();
    apply_by(
        &mut state,
        &me,
        Op::FolderAdd {
            id: root,
            d: FolderAdd {
                name: "AGENTS".into(),
                order: "a0".into(),
                parent: None,
                icon: Some(tisty_core::agents::ICON.into()),
                color: None,
                computer: None,
            },
        },
    );
    let mut computers: BTreeMap<&str, ulid::Ulid> = BTreeMap::new();
    for (_, device) in rooms {
        computers.entry(device).or_insert_with(|| {
            let id = ulid::Ulid::generate();
            let owner = DeviceId((*device).into());
            apply_by(
                &mut state,
                &owner,
                Op::FolderAdd {
                    id,
                    d: FolderAdd {
                        name: format!("computer {device}"),
                        order: "a1".into(),
                        parent: Some(root),
                        icon: None,
                        color: None,
                        computer: Some(owner.clone()),
                    },
                },
            );
            id
        });
    }
    let room_of = |name: &str| {
        let device = rooms.iter().find(|(doc, _)| *doc == name)?.1;
        computers.get(device).copied()
    };
    let mut sorted: Vec<&(String, String)> = docs.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut names = BTreeMap::new();
    for (name, body) in sorted {
        let file = id_of(name);
        tisty_core::docs::create_named(dir.path(), &file, body).unwrap();
        apply_by(
            &mut state,
            &me,
            Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: DocAdd {
                    file: file.clone(),
                    order: "a0".into(),
                    said: Some(Said::of(body).agreed()),
                    folder: Some(room_of(name).unwrap_or(root)),
                    ..Default::default()
                },
            },
        );
        names.insert(file, name.clone());
    }
    let gate = gate(&Asked {
        state: &state,
        docs: dir.path(),
        root: Some(root),
        ledger,
        me: &me,
    });
    Gated {
        _docs: dir,
        gate,
        names,
    }
}
