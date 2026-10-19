use crate::closing::{THIS_TIME, held, said_in, summed, this_time};
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

fn page(paths: &Paths, state: &State, id: TaskId) -> Option<String> {
    let kept = crate::task_docs::of(state, crate::task_docs::owner(state, id))?;
    crate::docs::read(&paths.docs(), &kept.file).ok()
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
        let open = crate::task_docs::of(state, crate::task_docs::owner(state, id))
            .is_some_and(|kept| !state.written_shut(kept.id));
        let onto_page = open && written(paths, store, id, |was| Some(summed(was, summary)));
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
        .map(|body| said_in(&body, &THIS_TIME))
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
pub fn round_cleared(paths: &Paths, store: &mut Store, state: &State, id: TaskId, said: &str) {
    let Some(kept) = crate::task_docs::of(state, crate::task_docs::owner(state, id)) else {
        return;
    };
    if state.written_shut(kept.id) {
        return;
    }
    let mut once = Vec::new();
    written(paths, store, id, |was| {
        let (_, cleared) = this_time(was).filter(|(now, _)| now == said)?;
        if let (Some(task), Some((_, words))) = (state.tasks.get(&id), held(was, &THIS_TIME)) {
            once = crate::stages::boxed_in(task, was, words);
        }
        Some(cleared)
    });
    let ops = left_behind(state, id, &once);
    if !ops.is_empty()
        && let Err(e) = store.append_batch(ops)
    {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "a round's own steps could not be left behind",
            &[("why", crate::witness::Fact::Why(e.to_string()))],
        );
    }
}

// Steps whose boxes sat under This time were for that round alone, so the next one goes without them.
fn left_behind(state: &State, id: TaskId, once: &[crate::model::StepId]) -> Vec<Op> {
    let Some(closed) = state.tasks.get(&id) else {
        return Vec::new();
    };
    let Some(next) = next_round(state, id) else {
        return Vec::new();
    };
    let mut taken = Vec::new();
    for step in once.iter().filter_map(|one| closed.step(*one)) {
        let twin = next.steps.iter().find(|one| {
            one.text == step.text && one.order == step.order && !taken.contains(&one.id)
        });
        if let Some(twin) = twin {
            taken.push(twin.id);
        }
    }
    taken
        .into_iter()
        .map(|step| Op::StepRemove {
            id: next.id,
            d: crate::event::StepRef { step },
        })
        .collect()
}

fn next_round(state: &State, id: TaskId) -> Option<&crate::model::Task> {
    let mut at = id;
    for _ in 0..state.tasks.len() {
        let next = state.tasks.values().find(|one| one.after == Some(at))?;
        if next.is_open() {
            return Some(next);
        }
        at = next.id;
    }
    None
}

fn written(
    paths: &Paths,
    store: &mut Store,
    id: TaskId,
    edit: impl FnOnce(&str) -> Option<String>,
) -> bool {
    let done = crate::task_docs::wrote(paths, store, id, |was, _| edit(was));
    if let Err(e) = &done {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "a task's page could not take what its closing said",
            &[("why", crate::witness::Fact::Why(e.to_string()))],
        );
    }
    done.is_ok()
}

#[cfg(test)]
#[path = "round_test.rs"]
mod tests;
