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
    "show", "undo", "redo", "sync", "doctor", "demo", "lists", "list", "tag", "config", "export",
    "help",
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
    #[arg(long, alias = "fecha")]
    pub date: Option<String>,
    #[arg(long, alias = "limite")]
    pub deadline: Option<String>,
    #[arg(long, alias = "prioridad", value_name = "QUADRANT")]
    pub priority: Option<String>,
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
    #[arg(long, alias = "prioridad", value_name = "QUADRANT")]
    pub priority: Option<String>,
    #[arg(long, alias = "etiqueta")]
    pub tag: Vec<String>,
    #[arg(long)]
    pub untag: Vec<String>,
    #[arg(long, conflicts_with = "date")]
    pub no_date: bool,
    #[arg(long, conflicts_with = "deadline")]
    pub no_deadline: bool,
    #[arg(long, alias = "repetir")]
    pub repeat: Option<String>,
    #[arg(long, conflicts_with = "repeat")]
    pub no_repeat: bool,
}

#[derive(Subcommand)]
pub enum Command {
    Add(AddArgs),
    Ls {
        filter: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    Done {
        selector: Option<String>,
    },
    Undone {
        selector: String,
    },
    Drop {
        selector: String,
    },
    Rm {
        selector: String,
        #[arg(long, short)]
        force: bool,
    },
    Set(SetArgs),
    Mv {
        selector: String,
        list: Option<String>,
        #[arg(long, conflicts_with = "list")]
        inbox: bool,
    },
    Desc {
        selector: String,
        text: Vec<String>,
        #[arg(long, conflicts_with = "text")]
        clear: bool,
    },
    Log {
        selector: String,
        text: Vec<String>,
    },
    Step {
        selector: String,
        #[command(subcommand)]
        action: StepAction,
    },
    Search {
        query: Vec<String>,
        #[arg(long)]
        open: bool,
        #[arg(long, conflicts_with = "open")]
        archive: bool,
        #[arg(long)]
        json: bool,
    },
    Show {
        selector: String,
        #[arg(long)]
        json: bool,
    },
    Undo,
    Redo,
    Demo {
        #[arg(long)]
        force: bool,
    },
    Sync {
        #[arg(long)]
        push: bool,
        #[arg(long, conflicts_with = "push")]
        pull: bool,
        #[arg(long, conflicts_with_all = ["push", "pull"])]
        again: bool,
        #[arg(long, value_name = "BACKUP", conflicts_with_all = ["push", "pull", "again"])]
        join: Option<std::path::PathBuf>,
        #[arg(long, value_name = "BACKUP", conflicts_with_all = ["push", "pull", "again", "join"])]
        take_over: Option<std::path::PathBuf>,
        #[arg(long, value_name = "BACKUP", conflicts_with_all = ["push", "pull", "again", "join", "take_over"])]
        merge: Option<std::path::PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repair: bool,
    },
    Lists {
        #[arg(long)]
        json: bool,
    },
    List {
        #[command(subcommand)]
        action: Option<ListAction>,
    },
    Tag {
        #[command(subcommand)]
        action: Option<TagAction>,
    },
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    Export {
        filter: Vec<String>,
        #[arg(long)]
        markdown: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
    Path,
}

#[derive(Subcommand)]
pub enum StepAction {
    Add { text: Vec<String> },
    Done { number: usize },
    Undone { number: usize },
    Text { number: usize, text: Vec<String> },
    Rm { number: usize },
}

#[derive(Subcommand)]
pub enum ListAction {
    Ls {
        #[arg(long)]
        json: bool,
    },
    Add {
        name: Vec<String>,
    },
    Rename {
        selector: String,
        name: Vec<String>,
    },
    Archive {
        selector: String,
    },
    Unarchive {
        selector: String,
    },
    Rm {
        selector: String,
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    Ls {
        #[arg(long)]
        json: bool,
    },
    Rename {
        old: String,
        new: String,
    },
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
            tisty_core::witness::warn(
                tisty_core::witness::channel::TERMINAL,
                "a command ended in an error",
                &blamed(asked(), &e),
            );
            eprintln!(
                "{}: {e}",
                style::paint(style::RED, Lang::detect(None).get("error"))
            );
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn blamed(
    command: &'static str,
    e: &anyhow::Error,
) -> Vec<(&'static str, tisty_core::witness::Fact)> {
    let mut facts = vec![("command", tisty_core::witness::Fact::Code(command))];
    if let Some(ours) = e.downcast_ref::<tisty_core::Error>() {
        facts.extend(ours.told());
    }
    facts
}

fn asked() -> &'static str {
    named(std::env::args().nth(1).as_deref())
}

fn named(first: Option<&str>) -> &'static str {
    let Some(first) = first else {
        return "none";
    };
    SUBCOMMANDS
        .iter()
        .copied()
        .find(|known| *known == first)
        .unwrap_or("add")
}

fn run() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse_from(normalise(std::env::args()));
    let paths = tisty_core::Paths::resolve()?;
    tisty_core::witness::keeps(
        tisty_core::witness::file(&paths),
        tisty_core::witness::wants_all(),
    );
    tisty_core::witness::catches(tisty_core::witness::channel::TERMINAL);
    tisty_core::witness::note(
        tisty_core::witness::channel::TERMINAL,
        "a command ran",
        &[(
            "version",
            tisty_core::witness::Fact::Id(env!("CARGO_PKG_VERSION").to_string()),
        )],
    );
    let mut app = match cli.command {
        Command::Config { .. } => App::without_store(paths)?,
        Command::Ls { .. } | Command::Lists { .. } => App::listing(paths)?,
        _ => App::at(paths)?,
    };
    let lang = Lang::detect(app.locale());
    let today = render::today();

    cmd::dispatch(cli.command, &mut app, lang, today)
}

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

    fn keys(facts: &[(&'static str, tisty_core::witness::Fact)]) -> Vec<&'static str> {
        facts.iter().map(|(name, _)| *name).collect()
    }

    #[test]
    fn a_failure_from_the_core_carries_the_facts_the_core_deems_safe() {
        let broke = anyhow::Error::from(tisty_core::Error::MissingSegment {
            number: 7,
            device: "dev_a3f1".into(),
        });

        let facts = blamed("sync", &broke);

        assert_eq!(keys(&facts), ["command", "code", "number", "device"]);
    }

    #[test]
    fn a_refusal_meant_for_the_screen_leaves_its_words_on_the_screen() {
        let said = "no list matches «la clínica de Juan»";
        let refused = anyhow::anyhow!("{said}");

        let facts = blamed("mv", &refused);

        assert_eq!(keys(&facts), ["command"]);
        assert!(!format!("{facts:?}").contains("Juan"), "{facts:?}");
    }

    #[test]
    fn a_failed_command_is_written_down_by_the_name_of_its_subcommand() {
        assert_eq!(named(Some("sync")), "sync");
        assert_eq!(named(Some("doctor")), "doctor");
        assert_eq!(named(None), "none");
    }

    #[test]
    fn a_bare_capture_is_written_down_as_the_capture_it_is() {
        assert_eq!(named(Some("comprar pan")), "add");
        assert_eq!(named(Some("--version")), "add");
    }

    #[test]
    fn nothing_a_person_typed_can_reach_the_file() {
        let secret = "llamar a la clínica de Juan";

        let written = named(Some(secret));

        assert!(SUBCOMMANDS.contains(&written), "{written}");
        assert!(!written.contains("Juan"));
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
