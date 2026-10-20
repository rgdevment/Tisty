use std::sync::Mutex;

use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Torn {
    rifts: Vec<tisty_core::merge::Rift>,
    print: String,
}

fn three_bodies(session: &Session, id: &str) -> Answer<Option<(String, String, String)>> {
    let carrier = session.carrying()?;
    let Some(base) = tisty_core::docs::read_carried(session.paths.data(), id) else {
        return Ok(None);
    };
    match carrier.both_papers(&session.here(), id) {
        Ok((mine, theirs)) => Ok(Some((base, mine, theirs))),
        Err(_) => Ok(None),
    }
}

fn print_of_three(base: &str, mine: &str, theirs: &str) -> String {
    tisty_core::attach::printed(format!("{base}\u{0}{mine}\u{0}{theirs}").as_bytes())
}

#[tauri::command(async)]
pub fn paper_rifts(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Torn> {
    let session = held(&session);
    if session.state.shut_tight(&id) {
        return Err(Refusal::of(match session.state.away(&id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    let Some((base, mine, theirs)) = three_bodies(&session, &id)? else {
        return Ok(Torn {
            rifts: Vec::new(),
            print: String::new(),
        });
    };
    Ok(Torn {
        print: print_of_three(&base, &mine, &theirs),
        rifts: tisty_core::merge::rifts(&base, &mine, &theirs),
    })
}

// Only both sides holding the same words means settled elsewhere; a side missing or unread may still be at odds.
#[tauri::command(async)]
pub fn still_at_odds(session: tauri::State<'_, Mutex<Session>>, ids: Vec<String>) -> Vec<String> {
    let (carrier, docs) = {
        let session = held(&session);
        let Ok(carrier) = session.carrying() else {
            return ids;
        };
        (carrier, session.paths.docs())
    };
    ids.into_iter()
        .filter(|id| !settled(&carrier, &docs, id))
        .collect()
}

fn settled(carrier: &tisty_carrier::Shared, docs: &std::path::Path, id: &str) -> bool {
    let Ok(at) = tisty_core::docs::resolve(docs, id) else {
        return false;
    };
    let Ok(Some(mine)) = tisty_core::docs::print_of(&at) else {
        return false;
    };
    carrier.paper_print(id).is_some_and(|theirs| theirs == mine)
}

#[tauri::command(async)]
pub fn weave_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    picks: Vec<String>,
    print: String,
) -> Answer<()> {
    let mut session = held(&session);
    // Settling what two machines already wrote is not writing into it, so being archived is no
    // reason to leave the rift with no way out. Only a lock guards the text itself.
    if session.state.shut_tight(&id) {
        return Err(Refusal::of("documentLocked"));
    }
    let Some((base, mine, theirs)) = three_bodies(&session, &id)? else {
        return Err(Refusal::of("noBase"));
    };
    if print_of_three(&base, &mine, &theirs) != print {
        return Err(Refusal::of("movedUnderfoot"));
    }
    let picked: Vec<tisty_core::merge::Pick> = picks
        .iter()
        .map(|one| match one.as_str() {
            "mine" => tisty_core::merge::Pick::Mine,
            "theirs" => tisty_core::merge::Pick::Theirs,
            _ => tisty_core::merge::Pick::Both,
        })
        .collect();
    let whole = tisty_core::merge::woven_with(&base, &mine, &theirs, &picked)
        .ok_or_else(|| Refusal::of("cannotWeave"))?;

    let papers = session.paths.docs();
    tisty_core::docs::kept_before(session.paths.data(), &id, &mine, &whole)
        .map_err(|e| blamed(channel::SYNC, "what it was could not be kept", e))?;
    tisty_core::docs::write(&papers, &id, &whole).map_err(|e| match e {
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        e => blamed(channel::SYNC, "the woven body could not be written", e),
    })?;
    session.mind(&id);
    let hand = crate::signing(&session.state);
    session.retell(&id, &whole, hand);
    Ok(())
}
