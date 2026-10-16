use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Refusal, Session, elsewhere, held};

// Before the first carry, so a document another machine already turned arrives as these same bytes.
pub(crate) async fn spoken(
    session: &tauri::State<'_, Mutex<Session>>,
    paths: &tisty_core::paths::Paths,
) -> Result<(), Refusal> {
    let (data, docs) = (paths.data().to_path_buf(), paths.docs());
    if tisty_core::speaking::spoken(&data) {
        return Ok(());
    }
    let at = paths.clone();
    let kept =
        elsewhere(move || tisty_core::backup::before_a_change(&at, jiff::Timestamp::now())).await?;
    if let Err(e) = kept {
        witness::warn(
            channel::BACKUP,
            "the store was left in its older words until a copy can be kept first",
            &[("why", Fact::Why(e.to_string()))],
        );
        return Ok(());
    }
    let spoke = elsewhere(move || tisty_core::speaking::speak(&data, &docs)).await?;
    witness::note(
        channel::STORE,
        "the store now speaks only Tisty's own words",
        &[
            ("docs", Fact::Count(spoke.docs.len())),
            ("moved", Fact::Count(spoke.moved)),
            ("left", Fact::Count(spoke.left)),
            ("missed", Fact::Count(spoke.missed)),
        ],
    );
    let mut session = held(session);
    for (file, _) in &spoke.docs {
        let Ok(body) = tisty_core::docs::read(&paths.docs(), file) else {
            continue;
        };
        session.mind_body(file, &body);
        session.retell(file, &body, None);
    }
    Ok(())
}
