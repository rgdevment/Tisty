mod app;
mod render;
mod select;
mod style;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use jiff::civil::Date;
use tisty_core::{Op, Task, event::TaskAdd};
use ulid::Ulid;

use app::App;
use select::{Resolved, Selection};

const EXIT_ERROR: u8 = 1;
const EXIT_NOT_FOUND: u8 = 4;

const SUBCOMMANDS: &[&str] = &["add", "ls", "done", "show", "lists", "help"];

#[derive(Parser)]
#[command(name = "tisty", version, about = "Gestor de tareas local y privado")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Captura una tarea
    Add {
        text: Vec<String>,
        /// Fecha en formato YYYY-MM-DD
        #[arg(long)]
        fecha: Option<Date>,
        /// Fecha límite en formato YYYY-MM-DD
        #[arg(long)]
        limite: Option<Date>,
        /// Prioridad de 1 a 4
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=4))]
        prioridad: Option<u8>,
        #[arg(long)]
        json: bool,
    },
    /// Lista las tareas abiertas
    Ls {
        /// hoy · todas · inbox · hechas
        filtro: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Completa una tarea
    Done { selector: String },
    /// Muestra el detalle de una tarea
    Show {
        selector: String,
        #[arg(long)]
        json: bool,
    },
    /// Lista las listas
    Lists {
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}: {e}", style::paint(style::RED, "error"));
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse_from(normalise(std::env::args()));
    let mut app = App::open()?;
    let today = render::today();

    match cli.command {
        Command::Add {
            text,
            fecha,
            limite,
            prioridad,
            json,
        } => add(&mut app, text, fecha, limite, prioridad, json, today),
        Command::Ls { filtro, json } => ls(&app, filtro.as_deref(), json, today),
        Command::Done { selector } => done(&mut app, &selector),
        Command::Show { selector, json } => show(&app, &selector, json, today),
        Command::Lists { json } => lists(&app, json),
    }
}

/// `tisty "texto"` captures; `tisty ls` runs a command. A first argument that
/// matches no known subcommand is treated as capture, which is where speed
/// matters most.
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

fn add(
    app: &mut App,
    text: Vec<String>,
    fecha: Option<Date>,
    limite: Option<Date>,
    prioridad: Option<u8>,
    json: bool,
    today: Date,
) -> anyhow::Result<ExitCode> {
    let title = text.join(" ").trim().to_string();
    if title.is_empty() {
        anyhow::bail!("hace falta un título");
    }

    let system = jiff::tz::TimeZone::system();
    let tz = system.iana_name().unwrap_or("UTC");
    let id = Ulid::generate();
    let d = TaskAdd {
        date: fecha.map(|f| tisty_core::DateSpec::all_day(f, tz)),
        deadline: limite.map(|f| tisty_core::DateSpec::all_day(f, tz)),
        priority: prioridad.map(|p| p.try_into()).transpose()?,
        ..TaskAdd::new(title, "a0")
    };

    app.commit(Op::TaskAdd { id, d })?;
    let task = &app.state.tasks[&id];

    if json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!("{}", render::captured(task, &app.state, today));
    }
    Ok(ExitCode::SUCCESS)
}

fn ls(app: &App, filtro: Option<&str>, json: bool, today: Date) -> anyhow::Result<ExitCode> {
    let open = app.ordered_open();

    let (heading, tasks): (&str, Vec<&Task>) = match filtro.unwrap_or("hoy") {
        "todas" => ("todas", open),
        "inbox" => (
            "bandeja de entrada",
            open.into_iter().filter(|t| t.list.is_none()).collect(),
        ),
        "hechas" | "archivo" => {
            let mut done: Vec<_> = app.state.archived_tasks().collect();
            done.sort_by_key(|t| std::cmp::Reverse(t.completed_at));
            ("archivo", done)
        }
        "hoy" => (
            "hoy",
            open.into_iter()
                .filter(|t| t.date.as_ref().is_none_or(|d| d.date() <= today))
                .collect(),
        ),
        other => anyhow::bail!("filtro desconocido: {other} (hoy · todas · inbox · hechas)"),
    };

    if json {
        println!("{}", serde_json::to_string(&tasks)?);
    } else {
        print!("{}", render::list(&tasks, &app.state, heading, today));
    }

    Selection::save(&app.paths, &tasks)?;
    Ok(ExitCode::SUCCESS)
}

fn done(app: &mut App, selector: &str) -> anyhow::Result<ExitCode> {
    let open = app.ordered_open();
    let selection = Selection::load(&app.paths);

    let id = match select::resolve(selector, &selection, &open) {
        Resolved::One(id) => id,
        Resolved::None => {
            eprintln!("no encontré ninguna tarea que coincida con «{selector}»");
            return Ok(ExitCode::from(EXIT_NOT_FOUND));
        }
        Resolved::Many(ids) => {
            eprintln!("«{selector}» coincide con {} tareas:", ids.len());
            for id in ids {
                let text = id.to_string();
                let short = text[text.len() - 6..].to_lowercase();
                eprintln!("  {short}  {}", app.state.tasks[&id].title);
            }
            return Ok(ExitCode::from(EXIT_NOT_FOUND));
        }
    };

    let title = app.state.tasks[&id].title.clone();
    app.commit(Op::TaskDone { id })?;
    println!("  {} {title}", style::paint(style::GREEN, "✓"));
    Ok(ExitCode::SUCCESS)
}

fn show(app: &App, selector: &str, json: bool, today: Date) -> anyhow::Result<ExitCode> {
    let all: Vec<&Task> = app.state.tasks.values().collect();
    let selection = Selection::load(&app.paths);

    let id = match select::resolve(selector, &selection, &all) {
        Resolved::One(id) => id,
        _ => {
            eprintln!("no encontré ninguna tarea que coincida con «{selector}»");
            return Ok(ExitCode::from(EXIT_NOT_FOUND));
        }
    };

    let task = &app.state.tasks[&id];
    if json {
        println!("{}", serde_json::to_string(task)?);
    } else {
        print!("{}", render::detail(task, &app.state, today));
    }
    Ok(ExitCode::SUCCESS)
}

fn lists(app: &App, json: bool) -> anyhow::Result<ExitCode> {
    if json {
        let all: Vec<_> = app.state.active_lists().collect();
        println!("{}", serde_json::to_string(&all)?);
    } else {
        print!("{}", render::lists(&app.state));
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
            normalised(&["tisty", "enviar SOBR a producción"]),
            ["tisty", "add", "enviar SOBR a producción"]
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
