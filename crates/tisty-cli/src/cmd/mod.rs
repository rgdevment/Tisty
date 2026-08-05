mod org;
mod task;
mod view;

use std::io::{IsTerminal, Read};
use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::{Task, TaskId};

use crate::app::App;
use crate::i18n::Lang;
use crate::select::{self, Resolved, Selection};
use crate::{Command, EXIT_NOT_FOUND};

pub fn dispatch(
    command: Command,
    app: &mut App,
    lang: Lang,
    today: Date,
) -> anyhow::Result<ExitCode> {
    match command {
        Command::Add(args) => task::add(app, args, today, lang),
        Command::Done { selector } => task::done(app, selector.as_deref(), today, lang),
        Command::Undone { selector } => task::undone(app, &selector, today, lang),
        Command::Drop { selector } => task::drop(app, &selector, today, lang),
        Command::Rm { selector, force } => task::rm(app, &selector, force, lang),
        Command::Set(args) => task::set(app, args, today, lang),
        Command::Mv {
            selector,
            list,
            inbox,
        } => task::mv(app, &selector, list.as_deref(), inbox, today, lang),
        Command::Desc {
            selector,
            text,
            clear,
        } => task::desc(app, &selector, text, clear, today, lang),
        Command::Log { selector, text } => task::log(app, &selector, text, today, lang),
        Command::Step { selector, action } => task::step(app, &selector, action, today, lang),

        Command::Ls { filter, json } => view::ls(app, filter.as_deref(), json, today, lang),
        Command::Show { selector, json } => view::show(app, &selector, json, today, lang),
        Command::Search {
            query,
            open,
            archive,
            json,
        } => view::search(app, &query.join(" "), open, archive, json, today, lang),

        Command::Undo => org::undo(app, today, lang),
        Command::Lists { json } => org::lists(app, json, lang),
        Command::List { action } => org::list(app, action, lang),
        Command::Tag { action } => org::tag(app, action, lang),
    }
}

pub enum Pick {
    Task(TaskId),
    NotFound,
    Cancelled,
}

/// Resolving prints nothing on success: every caller reports its own outcome.
pub fn pick(app: &App, selector: Option<&str>, pool: &[&Task], lang: Lang) -> anyhow::Result<Pick> {
    if pool.is_empty() {
        return Ok(Pick::NotFound);
    }

    let candidates = match selector {
        None => pool.to_vec(),
        Some(selector) => {
            let selection = Selection::load(&app.paths);
            match select::resolve(selector, &selection, pool) {
                Resolved::One(id) => return Ok(Pick::Task(id)),
                Resolved::None => {
                    report_missing(&selection, selector, lang);
                    return Ok(Pick::NotFound);
                }
                Resolved::Many(ids) => ids.iter().map(|id| &app.state.tasks[id]).collect(),
            }
        }
    };

    Ok(match select::prompt(&candidates, lang)? {
        Some(id) => Pick::Task(id),
        None => Pick::Cancelled,
    })
}

/// The shape every mutating command shares: resolve, act, show the result.
macro_rules! resolved {
    ($app:expr, $selector:expr, $pool:expr, $lang:expr, |$id:ident| $body:expr) => {{
        let $id = match crate::cmd::pick($app, $selector, &$pool, $lang)? {
            crate::cmd::Pick::Task(id) => id,
            crate::cmd::Pick::NotFound => {
                return Ok(std::process::ExitCode::from(crate::EXIT_NOT_FOUND));
            }
            crate::cmd::Pick::Cancelled => return Ok(std::process::ExitCode::SUCCESS),
        };
        $body
    }};
}
pub(crate) use resolved;

pub fn not_found(app: &App, selector: &str, lang: Lang) -> ExitCode {
    report_missing(&Selection::load(&app.paths), selector, lang);
    ExitCode::from(EXIT_NOT_FOUND)
}

/// A number that outlived its listing is the commonest miss by far, and saying
/// «no task matches 2» for it sends the reader looking for the wrong problem.
fn report_missing(selection: &Selection, selector: &str, lang: Lang) {
    let stale = selector
        .parse::<usize>()
        .is_ok_and(|n| selection.number(n).is_none());

    if stale {
        eprintln!(
            "{}",
            lang.fill(
                "out-of-listing",
                &[
                    ("selector", selector),
                    ("n", &lang.plural("lines", selection.len())),
                ]
            )
        );
    } else {
        eprintln!("{}", lang.fill("not-found", &[("selector", selector)]));
    }
}

/// Reads stdin when no words were given, so an editor or a pipe can supply the
/// body; asking for it interactively would hang a script.
pub fn text_or_stdin(words: Vec<String>, lang: Lang) -> anyhow::Result<String> {
    let joined = words.join(" ").trim().to_string();
    if !joined.is_empty() {
        return Ok(joined);
    }

    if std::io::stdin().is_terminal() {
        anyhow::bail!("{}", lang.get("needs-text"));
    }

    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    let buffer = buffer.trim_end().to_string();
    if buffer.trim().is_empty() {
        anyhow::bail!("{}", lang.get("needs-text"));
    }
    Ok(buffer)
}

/// Irreversible actions ask first, and a script that cannot answer is refused
/// rather than assumed to agree.
pub fn confirm(question: &str, force: bool, lang: Lang) -> anyhow::Result<bool> {
    if force {
        return Ok(true);
    }
    if !std::io::stdin().is_terminal() {
        anyhow::bail!("{}", lang.get("needs-force"));
    }
    Ok(dialoguer::Confirm::new()
        .with_prompt(question)
        .default(false)
        .interact()?)
}
