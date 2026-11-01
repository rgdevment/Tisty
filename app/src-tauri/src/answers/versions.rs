use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tisty_core::Op;

use super::shares::{apart, doc_of, sharer, signing_name};
use crate::sharing::{self, Net};
use crate::{Answer, Refusal, Session, held};

/// Each shared document's saves, counted so only the last of a burst goes up, and when the last uploads went.
#[derive(Default)]
pub struct Following {
    saves: Mutex<HashMap<String, u64>>,
    sent: Mutex<VecDeque<Instant>>,
}

// Saved words go up once the person stops this long, so a burst of saves is one version within the server's allowance.
const QUIET: Duration = Duration::from_secs(120);

// Below the server's 60 uploads an hour from one network, whose excess shuts that network out for hours or days.
const UPLOADS_AN_HOUR: usize = 40;

impl Following {
    // Every document counts toward one allowance; what is not sent now stays owed for the next round.
    fn paced(&self) -> bool {
        let mut sent = self.sent.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        while sent
            .front()
            .is_some_and(|at| now.duration_since(*at) > Duration::from_secs(3600))
        {
            sent.pop_front();
        }
        if sent.len() >= UPLOADS_AN_HOUR {
            return false;
        }
        sent.push_back(now);
        true
    }
}

/// Whether this file is shared and follows what is saved, asked by whoever just saved it.
pub(crate) fn follows(session: &Session, file: &str) -> bool {
    doc_of(session, file)
        .ok()
        .and_then(|doc| session.state.shares.get(&doc))
        .is_some_and(|one| !one.still)
}

/// Puts a shared document's saved words up as a new version once its saves rest (decision 48).
pub(crate) fn follow(app: &tauri::AppHandle, file: &str) {
    use tauri::Manager;
    let mine = {
        let following = app.state::<Following>();
        let mut saves = following.saves.lock().unwrap_or_else(|e| e.into_inner());
        let count = saves.entry(file.to_owned()).or_default();
        *count += 1;
        *count
    };
    let (app, file) = (app.clone(), file.to_owned());
    std::thread::spawn(move || {
        std::thread::sleep(QUIET);
        let following = app.state::<Following>();
        let last = following
            .saves
            .lock()
            .ok()
            .and_then(|saves| saves.get(&file).copied());
        if last == Some(mine) {
            tauri::async_runtime::block_on(followed(&app, &file));
        }
    });
}

async fn followed(app: &tauri::AppHandle, file: &str) {
    use tauri::Manager;
    let session = app.state::<Mutex<Session>>();
    let gathered = {
        let session = held(&session);
        let doc = doc_of(&session, file).ok();
        let shared = doc.and_then(|doc| session.state.shares.get(&doc).cloned());
        let text = tisty_core::docs::read(&session.paths.docs(), file).ok();
        match (doc, shared.filter(|one| !one.still), text) {
            (Some(doc), Some(shared), Some(text)) => Some((
                doc,
                shared,
                text,
                sharer(&session),
                signing_name(&session.state.signed),
                session.paths.data().to_path_buf(),
            )),
            _ => None,
        }
    };
    let Some((doc, shared, text, voucher, name, data)) = gathered else {
        return;
    };
    let print = sharing::print_of(&text);
    // Words already up, from here or another of the person's machines, are not put up again.
    if print == shared.print {
        return;
    }
    let done = match voucher {
        None => Err(Refusal::of("shareNoKey")),
        Some(_) if !app.state::<Following>().paced() => Err(Refusal::of("shareSlow")),
        Some((voucher, _)) => {
            let asked = shared.clone();
            let up = apart(move || {
                let (text, pictures) =
                    sharing::pictured(&text, |target| sharing::attachment(target, &data));
                let words = sharing::Words {
                    text: &text,
                    pictures: &pictures,
                    voucher: &voucher,
                    name: name.as_deref(),
                };
                sharing::update(&Net::to(&asked.server)?, &asked, &words)
            })
            .await;
            match up {
                Ok(version) => held(&session)
                    .commit(Op::ShareVersion {
                        id: doc,
                        link: shared.link,
                        version,
                        print,
                    })
                    .map_err(Refusal::from),
                // Someone put a version up since: theirs comes beside these words, which go up after it.
                Err(refused) if sharing::behind(&refused) => {
                    let brought = brought(app, file).await;
                    follow(app, file);
                    brought.map(|_| ())
                }
                Err(refused) => Err(refused),
            }
        }
    };
    if let Err(refused) = done {
        tisty_core::witness::note(
            tisty_core::witness::channel::WINDOW,
            "a shared document's saved words did not go up; the next round tries again",
            &[(
                "why",
                tisty_core::witness::Fact::Why(refused.code.to_string()),
            )],
        );
    }
}

/// Brings the newest version someone else put up of a share this store holds, beside the person's own words (decision 9).
pub(crate) async fn brought(app: &tauri::AppHandle, file: &str) -> Answer<bool> {
    use tauri::Manager;
    let session = app.state::<Mutex<Session>>();
    let (doc, shared, mine, data, cache) = {
        let session = held(&session);
        let doc = doc_of(&session, file)?;
        let Some(shared) = session.state.shares.get(&doc).cloned() else {
            return Ok(false);
        };
        let me = session.state.persons.person(&session.config.device_id);
        (
            doc,
            shared,
            me.map(|one| one.fingerprint()),
            session.paths.data().to_path_buf(),
            session.paths.cache().to_path_buf(),
        )
    };
    let asked = shared.clone();
    let got = apart(move || {
        let wire = Net::to(&asked.server)?;
        let newer = sharing::latest(&wire, &asked)?.filter(|one| *one > asked.version);
        let Some(latest) = newer else {
            return Ok(None);
        };
        let said = sharing::version(&wire, &asked, latest)?;
        let got = crate::received::opened_from(said, &asked.key)?;
        // The number is the server's word and the version inside is the writer's: they must agree, or neither is believed.
        if got.version != latest {
            return Err(Refusal::of("shareOpenOlder"));
        }
        Ok(Some((latest, got)))
    })
    .await;
    let (latest, got) = match got {
        Ok(Some(got)) => got,
        Ok(None) => return Ok(false),
        // Someone else's share that went, or no longer takes this token, is forgotten here and stays theirs.
        Err(refused)
            if shared.taken && (sharing::already_gone(&refused) || forbidden(&refused)) =>
        {
            held(&session)
                .commit(Op::ShareStop {
                    id: doc,
                    link: shared.link,
                })
                .map_err(Refusal::from)?;
            return Ok(false);
        }
        Err(refused) => return Err(refused),
    };
    let mut session = held(&session);
    // What this person's own key wrote came from another of their machines, which the sync brings whole.
    if mine.as_deref() != Some(got.print.as_str()) {
        let (body, told) = crate::received::kept_pictures(&got, &data, &cache);
        if !told.is_empty() {
            session.commit_all(told).map_err(Refusal::from)?;
        }
        // A name anyone can choose, so the fingerprint that cannot be chosen goes beside it.
        let by = format!(
            "{} · {}",
            got.signer.as_deref().unwrap_or("Tisty Share"),
            got.print
        );
        super::papers::other_version_kept(&mut session, file, body, Some(&by))?;
    }
    session
        .commit(Op::ShareVersion {
            id: doc,
            link: shared.link,
            version: latest,
            print: String::new(),
        })
        .map_err(Refusal::from)?;
    Ok(true)
}

fn forbidden(refused: &Refusal) -> bool {
    refused.code == "shareRefused" && refused.name.as_deref() == Some("forbidden")
}

/// At the start and with each round: brings every share's versions put up by others, and puts up what is owed.
pub(crate) async fn brought_all(app: &tauri::AppHandle) {
    use tauri::Manager;
    let files: Vec<String> = {
        let session = app.state::<Mutex<Session>>();
        let session = held(&session);
        session
            .state
            .shares
            .keys()
            .filter_map(|doc| session.state.docs.get(doc).map(|one| one.file.clone()))
            .collect()
    };
    for file in files {
        if let Err(refused) = brought(app, &file).await {
            tisty_core::witness::note(
                tisty_core::witness::channel::WINDOW,
                "a share's newer version could not be brought; the next round asks again",
                &[(
                    "why",
                    tisty_core::witness::Fact::Why(refused.code.to_string()),
                )],
            );
        }
        // Saved and not up, because Tisty closed before the words rested or another way of writing saved them.
        followed(app, &file).await;
    }
}
