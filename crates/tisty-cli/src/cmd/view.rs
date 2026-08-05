use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::Task;

use super::not_found;
use crate::app::App;
use crate::i18n::{self, Lang};
use crate::render;
use crate::select::{Resolved, Selection, resolve};

pub fn ls(
    app: &App,
    filter: Option<&str>,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let raw = filter.unwrap_or("today");
    let Some(canonical) = i18n::canonical_filter(raw) else {
        anyhow::bail!(
            "{}",
            lang.fill(
                "unknown-filter",
                &[("filter", raw), ("known", "today · all · inbox · archive")]
            )
        );
    };

    let open = app.ordered_open();
    let (heading, tasks): (&str, Vec<&Task>) = match canonical {
        "all" => (lang.get("all"), open),
        "inbox" => (
            lang.get("inbox"),
            open.into_iter().filter(|t| t.list.is_none()).collect(),
        ),
        "archive" => (lang.get("archive"), newest_first(app)),
        _ => (
            lang.get("today"),
            open.into_iter()
                .filter(|t| t.date.as_ref().is_none_or(|d| d.date() <= today))
                .collect(),
        ),
    };

    show_many(app, &tasks, heading, json, today, lang)
}

pub fn show(
    app: &App,
    selector: &str,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let all: Vec<&Task> = app.state.tasks.values().collect();
    let selection = Selection::load(&app.paths);

    let id = match resolve(selector, &selection, &all) {
        Resolved::One(id) => id,
        Resolved::Many(ids) => match crate::select::prompt(
            &ids.iter()
                .map(|id| &app.state.tasks[id])
                .collect::<Vec<_>>(),
            lang,
        )? {
            Some(id) => id,
            None => return Ok(ExitCode::SUCCESS),
        },
        Resolved::None => return Ok(not_found(app, selector, lang)),
    };

    let task = &app.state.tasks[&id];
    if json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!("{}", render::detail(task, &app.state, today, lang));
    }
    Ok(ExitCode::SUCCESS)
}

/// The archive is the point of keeping everything, so search reaches into the
/// description, the journal and the steps, not just the title.
pub fn search(
    app: &App,
    query: &str,
    open_only: bool,
    archive_only: bool,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        anyhow::bail!("{}", lang.get("needs-query"));
    }

    let mut hits: Vec<&Task> = app
        .state
        .tasks
        .values()
        .filter(|t| match (open_only, archive_only) {
            (true, _) => t.is_open(),
            (_, true) => t.is_archived(),
            _ => true,
        })
        .filter(|t| matches(t, &query))
        .collect();

    // Open work first, then the archive newest first: what is still pending is
    // what the search was probably for.
    hits.sort_by_key(|t| {
        (
            t.is_archived(),
            std::cmp::Reverse(t.completed_at),
            std::cmp::Reverse(t.id),
        )
    });

    let heading = lang.fill("results-for", &[("query", &query)]);
    show_many(app, &hits, &heading, json, today, lang)
}

fn matches(task: &Task, query: &str) -> bool {
    let contains = |text: &str| text.to_lowercase().contains(query);

    contains(&task.title)
        || task.description.as_deref().is_some_and(contains)
        || task.log.iter().any(|e| contains(&e.body))
        || task.steps.iter().any(|s| contains(&s.text))
        || task.tags.iter().any(|t| contains(t.as_str()))
}

fn newest_first(app: &App) -> Vec<&Task> {
    let mut done: Vec<&Task> = app.state.archived_tasks().collect();
    done.sort_by_key(|t| (std::cmp::Reverse(t.completed_at), std::cmp::Reverse(t.id)));
    done
}

/// Every listing records what it showed, so `done 2` means the second line the
/// user is looking at right now.
fn show_many(
    app: &App,
    tasks: &[&Task],
    heading: &str,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    if json {
        println!("{}", serde_json::to_string(tasks)?);
    } else {
        print!("{}", render::list(tasks, &app.state, heading, today, lang));
    }

    Selection::save(&app.paths, tasks)?;
    Ok(ExitCode::SUCCESS)
}
