use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::unvouched::{self, Unvouched, Vouch};
use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Session, blamed, elsewhere, held};

#[derive(Debug, Default, serde::Serialize)]
pub struct Vouched {
    pub kept: usize,
    pub unlike: Vec<String>,
    pub gone: Vec<String>,
}

// Taken under the lock and read outside it: the folder may be a cloud drive that answers slowly.
fn asked(session: &tauri::State<'_, Mutex<Session>>) -> (Vec<String>, Vec<PathBuf>) {
    let session = held(session);
    let wanted = session
        .pointed_at()
        .into_iter()
        .filter(|at| !session.state.kept.contains_key(at))
        .collect();
    let mut places = vec![session.paths.data().to_path_buf()];
    if let Some(dest) = session.place() {
        places.push(dest);
    }
    (wanted, places)
}

fn roots(places: &[PathBuf]) -> Vec<&Path> {
    places.iter().map(PathBuf::as_path).collect()
}

#[tauri::command]
pub async fn unvouched_attachments(
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<Vec<Unvouched>> {
    let (wanted, places) = asked(&session);
    elsewhere(move || unvouched::unvouched(&tisty_core::State::default(), &wanted, &roots(&places)))
        .await
}

#[tauri::command]
pub async fn vouch_attachments(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vouched> {
    let (wanted, places) = asked(&session);
    let read = elsewhere(move || {
        let roots = roots(&places);
        unvouched::unvouched(&tisty_core::State::default(), &wanted, &roots)
            .into_iter()
            .map(|one| {
                let said = unvouched::vouch(&one.at, &roots);
                (one.at, said)
            })
            .collect::<Vec<_>>()
    })
    .await?;

    let mut told = Vouched::default();
    let mut ops = Vec::new();
    for (at, said) in read {
        match said {
            Ok(Vouch::Held { held: d, here }) => {
                let at = d.at.clone();
                ops.push(tisty_core::Op::AttachKept { d });
                // Its print is written down, but a copy left only in the folder is not this machine's to claim.
                if !here {
                    ops.push(tisty_core::Op::AttachLetGo { d: at });
                }
            }
            Ok(Vouch::Unlike) => told.unlike.push(at),
            Ok(Vouch::Gone) => told.gone.push(at),
            Err(e) => {
                witness::warn(
                    channel::ATTACH,
                    "an attachment could not be read to be answered for",
                    &[
                        ("at", Fact::Id(at.clone())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
                told.gone.push(at);
            }
        }
    }
    told.kept = ops
        .iter()
        .filter(|one| matches!(one, tisty_core::Op::AttachKept { .. }))
        .count();
    if !ops.is_empty() {
        held(&session).commit_all(ops).map_err(|e| {
            blamed(
                channel::STORE,
                "the attachments read were not written down",
                e,
            )
        })?;
    }
    witness::note(
        channel::ATTACH,
        "attachments no history answered for were read and written down",
        &[
            ("kept", Fact::Count(told.kept)),
            ("unlike", Fact::Count(told.unlike.len())),
            ("gone", Fact::Count(told.gone.len())),
        ],
    );
    Ok(told)
}
