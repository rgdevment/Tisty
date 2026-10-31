use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, blamed, held};

#[tauri::command]
pub async fn remove_machine(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let who = tisty_core::event::DeviceId(id.clone());
    if who == held(&session).config.device_id {
        return Err(Refusal::of("notThisMachine"));
    }
    let (seed, mut session) = (
        crate::identity::for_removing(&session).await,
        held(&session),
    );

    let ops = crate::identity::removing(&session, who, seed.as_ref());
    let revoked = ops
        .iter()
        .filter_map(|op| match op {
            tisty_core::Op::ShareRevoked { r, .. } => Some(r.clone()),
            _ => None,
        })
        .collect();
    session
        .commit_all(ops)
        .map_err(|e| blamed(channel::STORE, "the machine could not be removed", e))?;
    session.reproject().map_err(|e| {
        blamed(
            channel::CACHE,
            "the store would not project after removing",
            e,
        )
    })?;

    crate::sharing::revoke_later(revoked);
    witness::note(
        channel::SYNC,
        "a machine was removed and what it wrote was left where everyone can still read it",
        &[("at", Fact::Id(id))],
    );
    Ok(())
}

/// The key travels back from the window so a claim that changed between the reading and the
/// answering is refused instead of confirmed.
#[tauri::command(async)]
pub fn confirm_machine_key(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    key: String,
) -> Answer<()> {
    if !tisty_core::store::is_device_name(&id) {
        return Err(Refusal::of("keyNotConfirmed"));
    }
    let who = tisty_core::event::DeviceId(id.clone());
    // Read out under the lock and let go of it: a waiting history may be long.
    let (known, histories, paths) = {
        let session = held(&session);
        (
            session.state.keys.get(&who).cloned(),
            session.histories(),
            session.paths.clone(),
        )
    };
    let says = known.or_else(|| {
        tisty_core::store::key_said_in(&histories?.join(tisty_carrier::STORE).join(&id), &who)
    });
    if says.as_deref() != Some(key.as_str()) {
        return Err(Refusal::of("keyMoved"));
    }
    // Written under the lock again, so two confirms or a round never rewrite the waiting list at once.
    let _held = held(&session);
    if !tisty_core::vouched::confirm(paths.data(), &who, &key) {
        return Err(Refusal::of("keyNotConfirmed"));
    }

    tisty_carrier::turned::let_through(paths.data(), &id);

    witness::note(
        channel::SYNC,
        "somebody answered for the key a machine signs with",
        &[("at", Fact::Id(id))],
    );
    Ok(())
}
