use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::Task;

use super::not_found;
use crate::app::App;
use crate::filter::Filter;
use crate::i18n::Lang;
use crate::render;
use crate::select::{Resolved, Selection, resolve};

pub fn ls(
    app: &App,
    tokens: &[String],
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let filter = Filter::parse(tokens, app, today, lang)?;
    let tasks = app.state.matching(&filter.inner, today);

    if !json && tasks.is_empty() && tokens.is_empty() {
        let ahead = ahead_of_today(app, today);
        if ahead > 0 {
            println!(
                "\n  {}\n",
                crate::style::dim(
                    &lang.fill("nothing-ahead", &[("n", &lang.plural("tasks", ahead))],)
                )
            );
            Selection::save(&app.paths, &tasks)?;
            return Ok(ExitCode::SUCCESS);
        }
    }

    show_many(app, &tasks, filter.heading(), json, today, lang)
}

fn ahead_of_today(app: &App, today: Date) -> usize {
    app.state
        .matching(
            &tisty_core::view::Filter {
                window: Some(tisty_core::view::Window::After(today)),
                ..Default::default()
            },
            today,
        )
        .len()
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

pub fn story(
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

    let told = tisty_core::story::story(&tisty_core::store::read_all(app.paths.store())?, id);
    if json {
        println!("{}", serde_json::to_string(&told)?);
    } else {
        print!(
            "{}",
            render::trail(&app.state.tasks[&id], &told, &app.state, today, lang)
        );
    }
    Ok(ExitCode::SUCCESS)
}

pub fn series(
    app: &App,
    selector: Option<&str>,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let Some(selector) = selector else {
        let all = tisty_core::series::routines(&app.state);
        if json {
            println!("{}", serde_json::to_string(&all)?);
        } else {
            print!("{}", render::shelf(&all, &app.state, lang));
        }
        return Ok(ExitCode::SUCCESS);
    };

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

    let Some(told) = tisty_core::series::series(&app.state, id) else {
        println!("{}", lang.get("series-not-one"));
        return Ok(crate::EXIT_NOT_FOUND.into());
    };
    if json {
        println!("{}", serde_json::to_string(&told)?);
    } else {
        print!("{}", render::series(&told, today, lang));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn search(
    app: &App,
    query: &str,
    open_only: bool,
    archive_only: bool,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    if query.trim().is_empty() {
        anyhow::bail!("{}", lang.get("needs-query"));
    }
    let query = query.trim().to_lowercase();

    let hits = app.state.search(
        &query,
        match (open_only, archive_only) {
            (true, _) => tisty_core::view::Scope::Open,
            (_, true) => tisty_core::view::Scope::Archived,
            _ => tisty_core::view::Scope::Either,
        },
    );

    let heading = lang.fill("results-for", &[("query", &query)]);
    show_many(app, &hits, &heading, json, today, lang)
}

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
