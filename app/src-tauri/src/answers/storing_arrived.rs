use tisty_core::witness::{self, Fact, channel};

use crate::Session;

/// A file's length and last write, taken before it is read: when they still match, nobody wrote it since.
type Stamp = Option<(u64, std::time::SystemTime)>;

const SETTLED_AFTER: std::time::Duration = std::time::Duration::from_millis(100);

pub(super) struct Stamped {
    file: String,
    pub(super) stamp: Stamp,
    body: Option<(String, tisty_core::event::Said)>,
}

fn stamp_of(docs: &std::path::Path, file: &str) -> Stamp {
    let meta = std::fs::metadata(tisty_core::docs::resolve(docs, file).ok()?).ok()?;
    let written = meta.modified().ok()?;
    // A write within the same clock tick keeps the time, and maybe the length: a fresh time proves nothing.
    let settled = std::time::SystemTime::now()
        .duration_since(written)
        .is_ok_and(|age| age > SETTLED_AFTER);
    settled.then_some((meta.len(), written))
}

fn said_of(docs: &std::path::Path, file: &str) -> Option<(String, tisty_core::event::Said)> {
    let body = tisty_core::docs::read(docs, file).ok()?;
    let said = tisty_core::event::Said::of(&body);
    Some((body, said))
}

/// Read and parsed without the session, so a save or a lock from the window never waits on a long round.
pub(super) fn read_stamped(docs: &std::path::Path, files: &[String]) -> Vec<Stamped> {
    files
        .iter()
        .map(|file| {
            let stamp = stamp_of(docs, file);
            Stamped {
                file: file.clone(),
                stamp,
                body: said_of(docs, file),
            }
        })
        .collect()
}

/// One commit for the round: a commit for each document held the session for seconds.
pub(super) fn retold(session: &mut Session, read: Vec<Stamped>) -> usize {
    let docs = session.paths.docs();
    let mut told = Vec::new();
    let mut news = 0;
    for one in read {
        // Written since it was read: the newest words are the ones told.
        let body = match one.stamp.is_some() && stamp_of(&docs, &one.file) == one.stamp {
            true => one.body,
            false => said_of(&docs, &one.file),
        };
        let Some((body, said)) = body else {
            continue;
        };
        let ops = news_of(&session.state, &one.file, &body, said);
        news += usize::from(!ops.is_empty());
        told.extend(ops);
    }
    if told.is_empty() {
        return 0;
    }
    if let Err(e) = session.commit_all(told) {
        witness::warn(
            channel::SYNC,
            "what the documents that arrived say could not be written down",
            &[
                ("documents", Fact::Count(news)),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        return 0;
    }
    news
}

fn news_of(
    state: &tisty_core::State,
    file: &str,
    body: &str,
    said: tisty_core::event::Said,
) -> Vec<tisty_core::Op> {
    let mut told = state.settling(file, body);
    if let Some(kept) = state.docs.values().find(|one| one.file == file)
        && said.news_for(kept)
    {
        told.push(tisty_core::Op::DocSaid {
            id: kept.id,
            d: said,
        });
    }
    told
}
