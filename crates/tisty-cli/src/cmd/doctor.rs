use std::process::ExitCode;

use tisty_core::cache::{self, Audit};

use crate::EXIT_ERROR;
use crate::app::App;
use crate::i18n::Lang;
use crate::style::{self, GREEN};

pub fn doctor(app: &App, repair: bool, lang: Lang) -> anyhow::Result<ExitCode> {
    let store = app.paths.store();
    let audit = cache::audit(&store, app.paths.cache())?;

    println!("\n  {}\n", style::bold(lang.get("doctor")));
    if let Some(truth) = audit.state() {
        line(lang.get("tasks-in-log"), &truth.tasks.len().to_string());
        line(lang.get("lists-in-log"), &truth.lists.len().to_string());

        let mut held: Vec<String> = truth
            .tasks
            .values()
            .flat_map(|task| task.references())
            .map(|one| one.target)
            .collect();
        held.extend(tisty_core::docs::referenced(&app.paths.docs()));
        let adrift = tisty_core::attach::loose(app.paths.data(), &held);
        if adrift.files > 0 {
            line(
                lang.get("loose-files"),
                &style::dim(&lang.fill(
                    "loose-files-are",
                    &[
                        ("count", &adrift.files.to_string()),
                        ("size", &weighed(adrift.bytes)),
                    ],
                )),
            );
        }
    }

    let verdict = match &audit {
        Audit::Unavailable => {
            line(lang.get("cache"), &style::dim(lang.get("no-cache")));
            ExitCode::SUCCESS
        }
        Audit::Agrees { .. } => {
            line(lang.get("cache"), &style::paint(GREEN, lang.get("agrees")));
            ExitCode::SUCCESS
        }
        Audit::Stale { .. } => {
            line(lang.get("cache"), &style::dim(lang.get("stale")));
            ExitCode::SUCCESS
        }
        Audit::Diverged { tasks, lists, .. } => {
            line(
                lang.get("cache"),
                &style::paint(style::RED, lang.get("diverged")),
            );
            println!();
            println!(
                "    {:<14}{} / {}",
                lang.get("tasks-in-log"),
                tasks.0,
                style::dim(&tasks.1.to_string())
            );
            println!(
                "    {:<14}{} / {}",
                lang.get("lists-in-log"),
                lists.0,
                style::dim(&lists.1.to_string())
            );
            ExitCode::from(EXIT_ERROR)
        }
    };

    if repair {
        cache::discard(app.paths.cache())?;
        println!(
            "\n  {} {}",
            style::paint(GREEN, "✓"),
            lang.get("cache-discarded")
        );
        println!();
        return Ok(ExitCode::SUCCESS);
    }

    if matches!(audit, Audit::Diverged { .. }) {
        println!("\n  {}", style::dim(lang.get("run-repair")));
    }
    println!();
    Ok(verdict)
}

fn line(label: &str, value: &str) {
    println!("    {label:<14}{value}");
}

fn weighed(bytes: u64) -> String {
    match bytes {
        0..=1023 => format!("{bytes} B"),
        1024..=1_048_575 => format!("{} kB", bytes / 1024),
        _ => format!("{} MB", bytes / 1_048_576),
    }
}
