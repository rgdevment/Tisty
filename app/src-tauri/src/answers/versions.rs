use std::sync::Mutex;

use tisty_core::Op;

use super::shares::{apart, doc_of, sharer, signing_name};
use crate::sharing::{self, Net};
use crate::{Answer, Refusal, Session, held};

/// Each shared document's saves, counted, so only the last of a burst goes up.
#[derive(Default)]
pub struct Following(std::sync::Mutex<std::collections::HashMap<String, u64>>);

// Saved words go up once the person stops this long, so a burst of saves is one version within the server's allowance.
const QUIET: std::time::Duration = std::time::Duration::from_secs(120);

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
        let mut counts = following.0.lock().unwrap_or_else(|e| e.into_inner());
        let count = counts.entry(file.to_owned()).or_default();
        *count += 1;
        *count
    };
    let (app, file) = (app.clone(), file.to_owned());
    std::thread::spawn(move || {
        std::thread::sleep(QUIET);
        let following = app.state::<Following>();
        let last = following
            .0
            .lock()
            .ok()
            .and_then(|counts| counts.get(&file).copied());
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
        match (doc, shared.filter(|one| !one.still), text, sharer(&session)) {
            (Some(doc), Some(shared), Some(text), Some((voucher, _))) => Some((
                doc,
                shared,
                text,
                voucher,
                signing_name(&session.state.signed),
                session.paths.data().to_path_buf(),
            )),
            _ => None,
        }
    };
    let Some((doc, shared, text, voucher, name, data)) = gathered else {
        return;
    };
    let link = shared.link.clone();
    let up = apart(move || {
        let (text, pictures) =
            sharing::pictured(&text, |target| sharing::attachment(target, &data));
        let words = sharing::Words {
            text: &text,
            pictures: &pictures,
            voucher: &voucher,
            name: name.as_deref(),
        };
        sharing::update(&Net::to(&shared.server)?, &shared, &words)
    })
    .await;
    let done = match up {
        Ok(version) => held(&session)
            .commit(Op::ShareVersion {
                id: doc,
                link,
                version,
            })
            .map_err(Refusal::from),
        // Someone put a version up since: theirs comes beside these words, and the next save goes up after it.
        Err(refused) if sharing::behind(&refused) => brought(app, file).await.map(|_| ()),
        Err(refused) => Err(refused),
    };
    if let Err(refused) = done {
        tisty_core::witness::note(
            tisty_core::witness::channel::WINDOW,
            "a shared document's saved words did not go up; the next save tries again",
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
        Ok(Some((
            latest,
            crate::received::opened_from(said, &asked.key)?,
        )))
    })
    .await?;
    let Some((latest, got)) = got else {
        return Ok(false);
    };
    let mut session = held(&session);
    // What this person's own key wrote came from another of their machines, which the sync brings whole.
    if mine.as_deref() != Some(got.print.as_str()) {
        let (body, told) = crate::received::kept_pictures(&got, &data, &cache);
        if !told.is_empty() {
            session.commit_all(told).map_err(Refusal::from)?;
        }
        let by = got.signer.as_deref().unwrap_or("Tisty Share");
        super::papers::other_version_kept(&mut session, file, body, Some(by))?;
    }
    session
        .commit(Op::ShareVersion {
            id: doc,
            link: shared.link,
            version: latest,
        })
        .map_err(Refusal::from)?;
    Ok(true)
}

/// Asks after every share this store holds for versions someone else put up, at the start and with each round.
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
    }
}
