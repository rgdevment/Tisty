use crate::closing::{THIS_TIME, said_in, summed, this_time};
use crate::event::{LogAdd, Op};
use crate::model::TaskId;
use crate::{Paths, State, Store};

pub struct Closing {
    pub ops: Vec<Op>,
    pub said: Option<String>,
}

fn logged(id: TaskId, words: String) -> Op {
    let zone = jiff::tz::TimeZone::system().iana_name().map(str::to_string);
    let d = LogAdd::new(ulid::Ulid::generate(), words).in_zone(zone);
    Op::TaskLog { id, d }
}

fn page(paths: &Paths, state: &State, id: TaskId) -> Option<(crate::model::DocId, String)> {
    let kept = crate::task_docs::of(state, crate::task_docs::owner(state, id))?;
    let body = crate::docs::read(&paths.docs(), &kept.file).ok()?;
    Some((kept.id, body))
}

/// Kept apart from the closing so that undoing the close never takes these words with it.
pub fn closing_words(
    paths: &Paths,
    store: &mut Store,
    state: &State,
    id: TaskId,
    summary: Option<&str>,
) -> Closing {
    let summary = summary.map(str::trim).filter(|one| !one.is_empty());
    let nothing = Closing {
        ops: Vec::new(),
        said: None,
    };
    let Some(task) = state.tasks.get(&id).filter(|one| one.is_open()) else {
        return nothing;
    };
    if task.repeat.is_none() && task.after.is_none() {
        let Some(summary) = summary else {
            return nothing;
        };
        let onto_page = page(paths, state, id).is_some()
            && crate::task_docs::wrote(paths, store, id, |was, _| Some(summed(was, summary)))
                .is_ok();
        return Closing {
            ops: if onto_page {
                Vec::new()
            } else {
                vec![logged(id, summary.to_string())]
            },
            said: None,
        };
    }
    let said = page(paths, state, id)
        .map(|(_, body)| said_in(&body, &THIS_TIME))
        .filter(|one| !one.is_empty());
    let words: Vec<&str> = [summary, said.as_deref()].into_iter().flatten().collect();
    if words.is_empty() {
        return nothing;
    }
    Closing {
        ops: vec![logged(id, words.join("\n\n"))],
        said,
    }
}

/// The page lets a round's words go only while they still read as what its log kept.
pub fn round_cleared(
    paths: &Paths,
    store: &mut Store,
    state: &State,
    id: TaskId,
    said: &str,
) -> crate::Result<()> {
    let Some((doc, _)) = page(paths, state, id) else {
        return Ok(());
    };
    if state.stowed(doc) || state.shut(doc) {
        return Ok(());
    }
    crate::task_docs::wrote(paths, store, id, |was, _| {
        this_time(was)
            .filter(|(now, _)| now == said)
            .map(|(_, cleared)| cleared)
    })
}
