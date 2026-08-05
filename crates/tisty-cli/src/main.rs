mod app;
mod i18n;
mod render;
mod select;
mod style;

use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use jiff::civil::Date;
use tisty_core::{Op, Task, event::TaskAdd};
use ulid::Ulid;

use app::App;
use i18n::Lang;
use select::{Resolved, Selection};

const EXIT_ERROR: u8 = 1;
const EXIT_NOT_FOUND: u8 = 4;

const SUBCOMMANDS: &[&str] = &["add", "ls", "done", "show", "lists", "help"];

#[derive(Parser)]
#[command(name = "tisty", version, about = "A local, private task manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
struct AddArgs {
    text: Vec<String>,
    /// Date, as YYYY-MM-DD
    #[arg(long, alias = "fecha")]
    date: Option<Date>,
    /// Deadline, as YYYY-MM-DD
    #[arg(long, alias = "limite")]
    deadline: Option<Date>,
    /// Priority, 1 to 4
    #[arg(long, alias = "prioridad", value_parser = clap::value_parser!(u8).range(1..=4))]
    priority: Option<u8>,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Capture a task
    Add(AddArgs),
    /// List open tasks
    Ls {
        /// today · all · inbox · done
        filter: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Complete a task. Without a selector, opens the interactive picker
    Done { selector: Option<String> },
    /// Show a task in full
    Show {
        selector: String,
        #[arg(long)]
        json: bool,
    },
    /// Show the lists
    Lists {
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!(
                "{}: {e}",
                style::paint(style::RED, Lang::detect(None).get("error"))
            );
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse_from(normalise(std::env::args()));
    let mut app = App::open()?;
    let lang = Lang::detect(app.locale());
    let today = render::today();

    match cli.command {
        Command::Add(args) => add(&mut app, args, today, lang),
        Command::Ls { filter, json } => ls(&app, filter.as_deref(), json, today, lang),
        Command::Done { selector } => done(&mut app, selector.as_deref(), lang),
        Command::Show { selector, json } => show(&app, &selector, json, today, lang),
        Command::Lists { json } => lists(&app, json, lang),
    }
}

/// An unrecognised first argument is a capture, not a typo'd subcommand.
fn normalise(args: impl Iterator<Item = String>) -> Vec<String> {
    let args: Vec<String> = args.collect();
    let is_command = args
        .get(1)
        .is_none_or(|a| SUBCOMMANDS.contains(&a.as_str()) || a.starts_with('-'));

    if is_command {
        return args;
    }

    let mut out = vec![args[0].clone(), "add".into()];
    out.extend(args.into_iter().skip(1));
    out
}

fn add(app: &mut App, args: AddArgs, today: Date, lang: Lang) -> anyhow::Result<ExitCode> {
    let title = args.text.join(" ").trim().to_string();
    if title.is_empty() {
        anyhow::bail!("{}", lang.get("needs-title"));
    }

    let system = jiff::tz::TimeZone::system();
    let tz = system.iana_name().unwrap_or("UTC");
    let id = Ulid::generate();
    let d = TaskAdd {
        date: args.date.map(|d| tisty_core::DateSpec::all_day(d, tz)),
        deadline: args.deadline.map(|d| tisty_core::DateSpec::all_day(d, tz)),
        priority: args.priority.map(|p| p.try_into()).transpose()?,
        ..TaskAdd::new(title, "a0")
    };

    app.commit(Op::TaskAdd { id, d })?;
    let task = &app.state.tasks[&id];

    if args.json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!("{}", render::captured(task, &app.state, today, lang));
    }
    Ok(ExitCode::SUCCESS)
}

fn ls(
    app: &App,
    filter: Option<&str>,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let open = app.ordered_open();

    let raw = filter.unwrap_or("today");
    let Some(canonical) = i18n::canonical_filter(raw) else {
        anyhow::bail!(
            "{}",
            lang.fill(
                "unknown-filter",
                &[("filter", raw), ("known", "today · all · inbox · done")]
            )
        );
    };

    let (heading, tasks): (&str, Vec<&Task>) = match canonical {
        "all" => (lang.get("all"), open),
        "inbox" => (
            lang.get("inbox"),
            open.into_iter().filter(|t| t.list.is_none()).collect(),
        ),
        "archive" => {
            let mut done: Vec<_> = app.state.archived_tasks().collect();
            done.sort_by_key(|t| std::cmp::Reverse(t.completed_at));
            (lang.get("archive"), done)
        }
        _ => (
            lang.get("today"),
            open.into_iter()
                .filter(|t| t.date.as_ref().is_none_or(|d| d.date() <= today))
                .collect(),
        ),
    };

    if json {
        println!("{}", serde_json::to_string(&tasks)?);
    } else {
        print!("{}", render::list(&tasks, &app.state, heading, today, lang));
    }

    Selection::save(&app.paths, &tasks)?;
    Ok(ExitCode::SUCCESS)
}

fn done(app: &mut App, selector: Option<&str>, lang: Lang) -> anyhow::Result<ExitCode> {
    let open = app.ordered_open();

    let id = match selector {
        None => {
            if open.is_empty() {
                println!("  {}", style::dim(lang.get("nothing-open")));
                return Ok(ExitCode::SUCCESS);
            }
            match select::prompt(&open, lang)? {
                Some(id) => id,
                None => return Ok(ExitCode::SUCCESS),
            }
        }
        Some(selector) => {
            let selection = Selection::load(&app.paths);
            match select::resolve(selector, &selection, &open) {
                Resolved::One(id) => id,
                Resolved::None => {
                    eprintln!("{}", lang.fill("not-found", &[("selector", selector)]));
                    return Ok(ExitCode::from(EXIT_NOT_FOUND));
                }
                Resolved::Many(ids) => {
                    let candidates: Vec<&Task> =
                        ids.iter().map(|id| &app.state.tasks[id]).collect();
                    match select::prompt(&candidates, lang)? {
                        Some(id) => id,
                        None => return Ok(ExitCode::SUCCESS),
                    }
                }
            }
        }
    };

    let title = app.state.tasks[&id].title.clone();
    app.commit(Op::TaskDone { id })?;
    println!("  {} {title}", style::paint(style::GREEN, "✓"));
    Ok(ExitCode::SUCCESS)
}

fn show(
    app: &App,
    selector: &str,
    json: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let all: Vec<&Task> = app.state.tasks.values().collect();
    let selection = Selection::load(&app.paths);

    let id = match select::resolve(selector, &selection, &all) {
        Resolved::One(id) => id,
        _ => {
            eprintln!("{}", lang.fill("not-found", &[("selector", selector)]));
            return Ok(ExitCode::from(EXIT_NOT_FOUND));
        }
    };

    let task = &app.state.tasks[&id];
    if json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!("{}", render::detail(task, &app.state, today, lang));
    }
    Ok(ExitCode::SUCCESS)
}

fn lists(app: &App, json: bool, lang: Lang) -> anyhow::Result<ExitCode> {
    if json {
        let all: Vec<_> = app.state.active_lists().collect();
        println!("{}", serde_json::to_string(&all)?);
    } else {
        print!("{}", render::lists(&app.state, lang));
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalised(args: &[&str]) -> Vec<String> {
        normalise(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn a_known_subcommand_stays_a_subcommand() {
        assert_eq!(normalised(&["tisty", "ls"]), ["tisty", "ls"]);
        assert_eq!(normalised(&["tisty", "done", "2"]), ["tisty", "done", "2"]);
    }

    #[test]
    fn free_text_becomes_a_capture() {
        assert_eq!(
            normalised(&["tisty", "deploy the release"]),
            ["tisty", "add", "deploy the release"]
        );
    }

    #[test]
    fn flags_are_left_to_clap() {
        assert_eq!(normalised(&["tisty", "--version"]), ["tisty", "--version"]);
        assert_eq!(normalised(&["tisty", "--help"]), ["tisty", "--help"]);
    }

    #[test]
    fn bare_invocation_is_untouched() {
        assert_eq!(normalised(&["tisty"]), ["tisty"]);
    }
}
