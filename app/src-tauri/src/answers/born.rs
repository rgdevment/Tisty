use tisty_core::capture::{Draft, Filing};
use tisty_core::model::TaskId;
use tisty_core::{Op, State};

use crate::Refusal;

#[derive(serde::Deserialize, Debug, Default)]
pub struct Born {
    doc: Option<String>,
    before: Option<String>,
    whole: Option<String>,
}

impl Born {
    pub(crate) fn rooted(
        &self,
        state: &State,
        draft: &mut Draft,
    ) -> Result<Option<TaskId>, Refusal> {
        if let Some(whole) = &self.whole {
            let whole = super::parts::task_id(whole)?;
            let holder = state
                .tasks
                .get(&whole)
                .ok_or_else(|| Refusal::of("notATaskId"))?;
            super::parts::takes_parts(holder).map_err(Refusal::of)?;
            // The core would let a repeating part go unhung instead of saying why.
            if draft.repeat.is_some() {
                return Err(Refusal::of("partRepeats"));
            }
            if draft.filing.is_none() {
                draft.filing = holder.list.map(Filing::Kept);
            }
            return Ok(Some(whole));
        }
        if let Some(doc) = self
            .doc
            .as_deref()
            .filter(|doc| state.docs.values().any(|kept| kept.file == *doc))
        {
            draft.source = Some(born_under(doc, self.before.as_deref().unwrap_or_default()));
        }
        Ok(None)
    }
}

pub(crate) fn hung(ops: Vec<Op>, born: TaskId, whole: Option<TaskId>) -> Vec<Op> {
    match whole {
        Some(whole) => super::parts::with_whole(ops, born, whole),
        None => ops,
    }
}

fn born_under(doc: &str, before: &str) -> String {
    let heads = tisty_markdown::outline(&tisty_markdown::read(before));
    let anchor = heads.last().map_or("top", |head| head.id.as_str());
    format!("{doc}#{anchor}")
}

#[cfg(test)]
#[path = "born_test.rs"]
mod tests;
