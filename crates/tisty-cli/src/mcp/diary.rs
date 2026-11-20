use serde_json::{Value, json};
use tisty_core::diary::Opening;
use tisty_core::model::Kept;
use tisty_core::{Paths, State, Store, diary};

use super::asked::text;
use super::jsonrpc::told;
use super::{Refused, UNSETTLED, doc_named, hitch, retold};

fn says_diary(body: &str) -> bool {
    tisty_markdown::front::diary(body)
}

pub(super) fn may_start(body: &str) -> Result<(), Refused> {
    match says_diary(body) {
        true => Err(started_by_the_person()),
        false => Ok(()),
    }
}

pub(super) fn body_kept(before: &str, after: &str) -> Result<(), Refused> {
    match !says_diary(before) && says_diary(after) {
        true => Err(started_by_the_person()),
        false => Ok(()),
    }
}

fn started_by_the_person() -> Refused {
    Refused::Tool(
        "`diary: on` makes the person's diary, and only the person starts one, from the window. \
         Leave that line out. Nothing was written."
            .into(),
    )
}

const LEFT_ALONE: [&str; 12] = [
    "write_doc",
    "import_doc",
    "edit_doc",
    "edit_table",
    "restore_doc",
    "archive_doc",
    "flag_doc",
    "file_doc",
    "page_doc",
    "tongue_doc",
    "attach",
    "sum_up",
];

// One door for every change but adding to today, so no tool can rewrite what the person wrote.
pub(super) fn left_alone(paths: &Paths, name: &str, args: &Value) -> Result<(), Refused> {
    let only_shown = name == "edit_table" && text(args, "do").is_none_or(|asked| asked == "show");
    if !LEFT_ALONE.contains(&name) || only_shown {
        return Ok(());
    }
    let named = every_name(args);
    if named.is_empty() {
        return Ok(());
    }
    let state = tisty_core::cache::summarised(&paths.store(), paths.cache()).map_err(hitch)?;
    if !state.diary_open {
        return Ok(());
    }
    match named.iter().find(|file| in_the_diary(paths, &state, file)) {
        Some(file) => Err(Refused::Tool(format!(
            "{} is the person's diary, and its words are theirs: nothing in it is rewritten, \
             moved, marked, summed up or put away. Add to today with `append_doc`, and only when they ask. \
             Nothing was written.",
            doc_named(&state, file)
        ))),
        None => Ok(()),
    }
}

// A tool names a document under more than one key, alone or in a list, so the door reads them all.
fn every_name(args: &Value) -> Vec<&str> {
    let Some(given) = args.as_object() else {
        return Vec::new();
    };
    given
        .values()
        .flat_map(|one| match one {
            Value::Array(many) => many.iter().filter_map(Value::as_str).collect(),
            other => other.as_str().into_iter().collect::<Vec<_>>(),
        })
        .map(str::trim)
        .filter(|said| !said.is_empty())
        .collect()
}

pub(super) fn in_the_diary(paths: &Paths, state: &State, file: &str) -> bool {
    state
        .docs
        .values()
        .find(|one| one.file == file)
        .is_some_and(|kept| diary::holds(&paths.docs(), state, kept))
}

pub(super) fn written_today(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    kept: &Kept,
    said: &str,
    args: &Value,
) -> Result<Value, Refused> {
    let named = doc_named(state, &kept.file);
    if text(args, "under").is_some() {
        return Err(Refused::Tool(format!(
            "{named} is the person's diary, where you only ever add at the end of today, signed. \
             Send it without `under`. Nothing was written."
        )));
    }
    let said = said.trim();
    if said.is_empty() {
        return Err(Refused::Tool("adding needs words in `body`.".into()));
    }
    super::papers::writing::keeps(said)?;
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let words = diary::words_for(&tisty_core::model::spoken(config.locale.as_deref()));
    let now = jiff::Zoned::now();
    let today = now.date();
    let who = super::task_notes::assistant();
    let signed = tisty_core::assistant_notes::signed_by(&who, &now, said);

    let root = paths.docs();
    let device = store.device().clone();
    let writer = super::writing(paths);
    let maker = diary::Maker {
        root: &root,
        device: &device,
        by: None,
        writer: &writer,
    };
    let opening = match diary::opened(&maker, state, today, words).map_err(hitch)? {
        diary::Opened::Ready(opening) if !state.bolted(&opening.page) => opening,
        diary::Opened::Ready(Opening { book, .. }) | diary::Opened::Locked { book } => {
            return Err(Refused::Tool(format!(
                "{} is locked or put away: the person shut it so nothing writes in it. Nothing \
                 was written.",
                doc_named(state, &book)
            )));
        }
        diary::Opened::Waiting { .. } => {
            return Err(Refused::Tool(format!(
                "The person's diary waits until every one of their computers runs Tisty {} or \
                 later. Nothing was written.",
                diary::SINCE
            )));
        }
    };
    if !opening.ops.is_empty() {
        store.append_batch(opening.ops.clone()).map_err(hitch)?;
    }
    let state = State::replay(&store.read_all().map_err(hitch)?);
    let mut settled = true;
    if let Some(whole) =
        diary::named(&root, &crate::mcp::writing(paths), &opening).map_err(hitch)?
    {
        settled &= retold(&state, store, &opening.book, &whole).is_ok();
    }
    let whole = tisty_core::docs::amend(&root, &crate::mcp::writing(paths), &opening.page, |was| {
        Some(diary::written_on(was, today, words, &signed))
    })
    .map_err(hitch)?
    .unwrap_or_default();
    settled &= retold(&state, store, &opening.page, &whole).is_ok();
    let day = diary::day_title(today, words);
    Ok(told(
        format!(
            "Added to the end of «{day}» in {}, signed as {who}. Nothing the person wrote \
             changed.{}",
            doc_named(&state, &opening.page),
            if settled { "" } else { UNSETTLED }
        ),
        json!({
            "doc": opening.page,
            "day": day,
            "print": tisty_core::attach::printed(whole.as_bytes()),
        }),
    ))
}
