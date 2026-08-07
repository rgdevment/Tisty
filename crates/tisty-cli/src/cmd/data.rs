use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::Task;

use crate::app::App;
use crate::filter::Filter;
use crate::i18n::Lang;
use crate::{ConfigAction, EXIT_NOT_FOUND, render, style};

/// `device_id` is absent on purpose: editing it orphans what this machine wrote.
const KEYS: &[&str] = &["locale", "editor"];
const READABLE: &[&str] = &["device_id", "locale", "editor"];

pub fn config(app: &mut App, action: Option<ConfigAction>, lang: Lang) -> anyhow::Result<ExitCode> {
    match action {
        None => {
            let config = app.config();
            println!("\n  {}\n", style::bold(lang.get("config")));
            show("device_id", Some(&config.device_id.0));
            show("locale", config.locale.as_deref());
            show("editor", config.editor.as_deref());
            show("data_dir", Some(&app.paths.data().display().to_string()));
            println!();
            Ok(ExitCode::SUCCESS)
        }

        Some(ConfigAction::Path) => {
            println!("{}", app.paths.config_file().display());
            Ok(ExitCode::SUCCESS)
        }

        Some(ConfigAction::Get { key }) => {
            known(&key, READABLE, lang)?;
            match value(app, &key)? {
                Some(value) => {
                    println!("{value}");
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    eprintln!("{}", lang.fill("unset-key", &[("key", &key)]));
                    Ok(ExitCode::from(EXIT_NOT_FOUND))
                }
            }
        }

        Some(ConfigAction::Set { key, value }) => {
            check(&key, lang)?;
            if key == "locale" && Lang::known(&value).is_none() {
                anyhow::bail!(
                    "{}",
                    lang.fill(
                        "unknown-locale",
                        &[("value", &value), ("known", &Lang::available())]
                    )
                );
            }

            app.edit_config(|c| match key.as_str() {
                "locale" => c.locale = Some(value.clone()),
                _ => c.editor = Some(value.clone()),
            })?;
            println!("  {} {key} = {value}", style::paint(style::GREEN, "✓"));
            Ok(ExitCode::SUCCESS)
        }

        Some(ConfigAction::Unset { key }) => {
            check(&key, lang)?;
            app.edit_config(|c| match key.as_str() {
                "locale" => c.locale = None,
                _ => c.editor = None,
            })?;
            println!("  {} {key}", style::dim("✕"));
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn show(key: &str, value: Option<&str>) {
    match value {
        Some(v) => println!("    {key:<12}{v}"),
        None => println!("    {key:<12}{}", style::dim("—")),
    }
}

fn value(app: &App, key: &str) -> anyhow::Result<Option<String>> {
    let config = app.config();
    match key {
        "device_id" => Ok(Some(config.device_id.0.clone())),
        "locale" => Ok(config.locale.clone()),
        "editor" => Ok(config.editor.clone()),
        "data_dir" => Ok(Some(app.paths.data().display().to_string())),
        _ => Ok(None),
    }
}

fn check(key: &str, lang: Lang) -> anyhow::Result<()> {
    known(key, KEYS, lang)
}

fn known(key: &str, allowed: &[&str], lang: Lang) -> anyhow::Result<()> {
    if !allowed.contains(&key) {
        anyhow::bail!(
            "{}",
            lang.fill(
                "unknown-key",
                &[("key", key), ("known", &allowed.join(" · "))]
            )
        );
    }
    Ok(())
}

/// The way out of the format, which is what makes the data worth trusting.
pub fn export(
    app: &App,
    tokens: &[String],
    markdown: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let filter = Filter::parse(tokens, app, today, lang)?;

    let pool: Vec<&Task> = if filter.archive {
        let mut done: Vec<&Task> = app.state.archived_tasks().collect();
        done.sort_by_key(|t| (std::cmp::Reverse(t.completed_at), std::cmp::Reverse(t.id)));
        done
    } else {
        app.state.ordered_open()
    };
    let tasks: Vec<&Task> = pool
        .into_iter()
        .filter(|t| filter.matches(t, today))
        .collect();

    if markdown {
        print!(
            "{}",
            render::markdown(&tasks, &app.state, filter.heading(), lang)
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&tasks)?);
    }
    Ok(ExitCode::SUCCESS)
}
