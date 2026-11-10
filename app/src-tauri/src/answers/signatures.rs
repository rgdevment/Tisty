use std::sync::Mutex;

use tisty_core::Op;
use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signed {
    alias: Option<String>,
    name: Option<String>,
    before: Vec<String>,
    mine: usize,
    alias_imitates: bool,
    name_imitates: bool,
}

// Whoever reads what this Tisty shares sees such a name flagged in red, so the person hears it first (167).
fn imitating(session: &Session) -> impl Fn(Option<&String>) -> bool {
    let paths = &session.paths;
    let maker = tisty_carrier::folio_limits::maker(&paths.private()).filter(|_| {
        super::sharing::the_maker_here(paths, &session.state, &session.config.device_id).is_none()
    });
    move |said| {
        said.zip(maker.as_ref())
            .is_some_and(|(said, maker)| super::folio_standing::imitates_maker(said, maker))
    }
}

#[tauri::command]
pub fn signed(session: tauri::State<'_, Mutex<Session>>) -> Answer<Signed> {
    let session = held(&session);
    Ok(as_signed(&session))
}

fn as_signed(session: &Session) -> Signed {
    let imitates = imitating(session);
    Signed {
        alias_imitates: imitates(session.state.signed.alias.as_ref()),
        name_imitates: imitates(session.state.signed.name.as_ref()),
        alias: session.state.signed.alias.clone(),
        name: session.state.signed.name.clone(),
        before: {
            let mut seen: Vec<String> = Vec::new();
            for was in session.state.signed_before.iter().rev() {
                if !seen
                    .iter()
                    .any(|one| tisty_core::state::same_name(one, was))
                {
                    seen.push(was.clone());
                }
            }
            seen
        },
        mine: match session.state.signed.alias.is_some() {
            true => session.state.mine_to_sign().len(),
            false => 0,
        },
    }
}

#[tauri::command]
pub fn sign_the_rest(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let mut session = held(&session);
    let Some(alias) = session.state.signed.alias.clone() else {
        return Ok(0);
    };
    let ops: Vec<Op> = session
        .state
        .mine_to_sign()
        .into_iter()
        .map(|id| Op::DocSigned {
            id,
            d: alias.clone(),
        })
        .collect();
    let many = ops.len();
    if many > 0 {
        session
            .commit_all(ops)
            .map_err(|e| blamed(channel::WINDOW, "the documents could not be signed", e))?;
    }
    Ok(many)
}

#[tauri::command]
pub fn sign(session: tauri::State<'_, Mutex<Session>>, alias: Option<String>) -> Answer<Signed> {
    let said = alias
        .map(|one| tisty_core::text::plainly(&one).trim().to_string())
        .filter(|one| !one.is_empty());
    if said
        .as_ref()
        .is_some_and(|one| one.chars().count() > tisty_core::event::ALIAS_AT_MOST)
    {
        return Err(Refusal::about(
            "aliasTooLong",
            tisty_core::event::ALIAS_AT_MOST.to_string(),
        ));
    }

    let mut session = held(&session);
    let same = match (said.as_deref(), session.state.signed.alias.as_deref()) {
        (Some(one), Some(was)) => tisty_core::state::same_name(one, was),
        (one, was) => one == was,
    };
    if same {
        return Ok(as_signed(&session));
    }
    let mut signature = session.state.signed.clone();
    signature.alias = said;
    session
        .commit(Op::Signed {
            d: signature.clone(),
        })
        .map_err(|e| blamed(channel::WINDOW, "the signature could not be written", e))?;
    Ok(as_signed(&session))
}

#[tauri::command]
pub fn sign_name(
    session: tauri::State<'_, Mutex<Session>>,
    name: Option<String>,
) -> Answer<Signed> {
    let said = name_to_sign(name)?;
    let mut session = held(&session);
    if said == session.state.signed.name {
        return Ok(as_signed(&session));
    }
    let mut signature = session.state.signed.clone();
    signature.name = said;
    session
        .commit(Op::Signed { d: signature })
        .map_err(|e| blamed(channel::WINDOW, "the signature could not be written", e))?;
    Ok(as_signed(&session))
}

fn name_to_sign(name: Option<String>) -> Result<Option<String>, Refusal> {
    let Some(said) = name
        .map(|one| tisty_core::text::plainly(&one).trim().to_string())
        .filter(|one| !one.is_empty())
    else {
        return Ok(None);
    };
    if said.chars().count() > tisty_core::event::NAME_AT_MOST {
        return Err(Refusal::about(
            "nameTooLong",
            tisty_core::event::NAME_AT_MOST.to_string(),
        ));
    }
    match tisty_share_proto::name::named_well(&said) {
        true => Ok(Some(said)),
        false => Err(Refusal::of("nameUnfit")),
    }
}

#[cfg(test)]
#[path = "signatures_test.rs"]
mod tests;
