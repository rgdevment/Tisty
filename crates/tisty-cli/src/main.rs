mod app;
mod cmd;
mod filter;
mod i18n;
mod render;
mod select;
mod style;

use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use app::App;
use i18n::Lang;

pub const EXIT_ERROR: u8 = 1;
pub const EXIT_NOT_FOUND: u8 = 4;

const SUBCOMMANDS: &[&str] = &[
    "add", "ls", "done", "undone", "drop", "rm", "set", "mv", "desc", "log", "step", "search",
    "show", "undo", "redo", "doctor", "lists", "list", "tag", "config", "export", "help",
];

#[derive(Parser)]
#[command(name = "tisty", version, about = "A local, private task manager")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
#[command(
    after_help = "Plain language is read, not obeyed: check the confirmation before \
moving on. What it guessed it says so, and `tisty undo` takes the capture back. The window \
lets you correct the reading before saving; here you correct it after, with `tisty set`."
)]
pub struct AddArgs {
    pub text: Vec<String>,
    /// Date, as YYYY-MM-DD or plain language
    #[arg(long, alias = "fecha")]
    pub date: Option<String>,
    /// Deadline, as YYYY-MM-DD or plain language
    #[arg(long, alias = "limite")]
    pub deadline: Option<String>,
    /// Priority, 1 to 4
    #[arg(long, alias = "prioridad", value_parser = clap::value_parser!(u8).range(1..=4))]
    pub priority: Option<u8>,
    /// List to file it under
    #[arg(long, alias = "lista")]
    pub list: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct SetArgs {
    pub selector: String,
    #[arg(long, alias = "titulo")]
    pub title: Option<String>,
    #[arg(long, alias = "fecha")]
    pub date: Option<String>,
    #[arg(long, alias = "limite")]
    pub deadline: Option<String>,
    #[arg(long, alias = "prioridad", value_parser = clap::value_parser!(u8).range(1..=4))]
    pub priority: Option<u8>,
    /// Add a tag. Repeatable
    #[arg(long, alias = "etiqueta")]
    pub tag: Vec<String>,
    /// Remove a tag. Repeatable
    #[arg(long)]
    pub untag: Vec<String>,
    #[arg(long, conflicts_with = "date")]
    pub no_date: bool,
    #[arg(long, conflicts_with = "deadline")]
    pub no_deadline: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Capture a task
    Add(AddArgs),
    /// List open tasks. Filters combine: `ls week @backend !1`
    Ls {
        /// today · tomorrow · week · overdue · inbox · archive · all · @list · #tag · !1
        filter: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Complete a task. Without a selector, opens the picker
    Done { selector: Option<String> },
    /// Reopen a task that was completed or dropped
    Undone { selector: String },
    /// Drop a task without completing it
    Drop { selector: String },
    /// Erase a task for good
    Rm {
        selector: String,
        #[arg(long, short)]
        force: bool,
    },
    /// Change a task's fields
    Set(SetArgs),
    /// File a task under a list
    Mv {
        selector: String,
        list: Option<String>,
        /// Take it out of every list
        #[arg(long, conflicts_with = "list")]
        inbox: bool,
    },
    /// Write what has to be done. Reads stdin when no text is given
    Desc {
        selector: String,
        text: Vec<String>,
        /// Leave the task without a description
        #[arg(long, conflicts_with = "text")]
        clear: bool,
    },
    /// Record what happened. Reads stdin when no text is given
    Log { selector: String, text: Vec<String> },
    /// Work with a task's steps
    Step {
        selector: String,
        #[command(subcommand)]
        action: StepAction,
    },
    /// Search everywhere, archive included
    Search {
        query: Vec<String>,
        /// Only tasks still open
        #[arg(long)]
        open: bool,
        /// Only archived tasks
        #[arg(long, conflicts_with = "open")]
        archive: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a task in full
    Show {
        selector: String,
        #[arg(long)]
        json: bool,
    },
    /// Undo the last change made on this device
    Undo,
    /// Redo what the last undo took back
    Redo,
    /// Check the read cache against the log
    Doctor {
        /// Throw the cache away so the next read rebuilds it
        #[arg(long)]
        repair: bool,
    },
    /// Show the lists
    Lists {
        #[arg(long)]
        json: bool,
    },
    /// Work with lists
    List {
        #[command(subcommand)]
        action: Option<ListAction>,
    },
    /// Work with tags
    Tag {
        #[command(subcommand)]
        action: Option<TagAction>,
    },
    /// Read or change the settings
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Write the tasks out, filters and all
    Export {
        filter: Vec<String>,
        /// A document to read, instead of data to process
        #[arg(long)]
        markdown: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print one setting
    Get { key: String },
    /// Change one setting
    Set { key: String, value: String },
    /// Go back to the default
    Unset { key: String },
    /// Where the settings file lives
    Path,
}

#[derive(Subcommand)]
pub enum StepAction {
    /// Add a step
    Add { text: Vec<String> },
    /// Tick a step off, by its number
    Done { number: usize },
    /// Untick a step, by its number
    Undone { number: usize },
    /// Rewrite a step, by its number
    Text { number: usize, text: Vec<String> },
    /// Remove a step, by its number
    Rm { number: usize },
}

#[derive(Subcommand)]
pub enum ListAction {
    /// Show the lists
    Ls {
        #[arg(long)]
        json: bool,
    },
    /// Create a list
    Add { name: Vec<String> },
    /// Rename a list
    Rename { selector: String, name: Vec<String> },
    /// Put a list away without losing its tasks
    Archive { selector: String },
    /// Erase a list for good. Its tasks go back to the inbox
    Rm {
        selector: String,
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    /// Show every tag in use
    Ls {
        #[arg(long)]
        json: bool,
    },
    /// Rename a tag everywhere it appears
    Rename { old: String, new: String },
    /// Remove a tag from every task
    Rm {
        tag: String,
        #[arg(long, short)]
        force: bool,
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
    let paths = tisty_core::Paths::resolve()?;
    let mut app = match cli.command {
        Command::Config { .. } => App::without_store(paths)?,
        Command::Ls { .. } | Command::Lists { .. } => App::listing(paths)?,
        _ => App::at(paths)?,
    };
    let lang = Lang::detect(app.locale());
    let today = render::today();

    cmd::dispatch(cli.command, &mut app, lang, today)
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

pub fn date_flag(raw: Option<&str>, lang: Lang) -> anyhow::Result<Option<tisty_core::DateSpec>> {
    let Some(raw) = raw else { return Ok(None) };
    let now = jiff::Zoned::now();

    match tisty_nl::parse_date(raw, &now, lang.code()) {
        Some(spec) => Ok(Some(spec)),
        None => anyhow::bail!("{}", lang.fill("not-a-date", &[("value", raw)])),
    }
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

    #[test]
    fn every_subcommand_is_known_to_the_capture_guard() {
        use clap::CommandFactory;

        for sub in Cli::command().get_subcommands() {
            let name = sub.get_name();
            assert!(
                SUBCOMMANDS.contains(&name),
                "«{name}» would be swallowed as a capture"
            );
        }
    }
}
