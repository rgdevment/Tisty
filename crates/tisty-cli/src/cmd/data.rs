use std::process::ExitCode;

use jiff::civil::Date;

use crate::app::App;
use crate::filter::Filter;
use crate::i18n::Lang;
use crate::{ConfigAction, EXIT_NOT_FOUND, render, style};

const KEYS: &[&str] = &["locale", "editor", "remote"];
const READABLE: &[&str] = &["device_id", "locale", "editor", "remote"];

pub fn config(app: &mut App, action: Option<ConfigAction>, lang: Lang) -> anyhow::Result<ExitCode> {
    match action {
        None => {
            let config = app.config();
            println!("\n  {}\n", style::bold(lang.get("config")));
            show("device_id", Some(&config.device_id.0));
            show("locale", config.locale.as_deref());
            show("editor", config.editor.as_deref());
            show("remote", value(app, "remote")?.as_deref());
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

            if key == "remote" {
                let at = std::path::PathBuf::from(&value);
                let data = app.paths.data();
                if at.starts_with(data) || data.starts_with(&at) {
                    anyhow::bail!("{}", lang.fill("remote-inside", &[("at", &value)]));
                }
            }

            app.edit_config(|c| match key.as_str() {
                "locale" => c.locale = Some(value.clone()),
                "remote" => c.sync = Some(tisty_core::config::Sync::Folder(value.clone().into())),
                _ => c.editor = Some(value.clone()),
            })?;
            println!("  {} {key} = {value}", style::paint(style::GREEN, "✓"));
            Ok(ExitCode::SUCCESS)
        }

        Some(ConfigAction::Unset { key }) => {
            check(&key, lang)?;
            app.edit_config(|c| match key.as_str() {
                "locale" => c.locale = None,
                "remote" => c.sync = Some(tisty_core::config::Sync::Local),
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
        "remote" => Ok(match &config.sync {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at.display().to_string()),
            _ => None,
        }),
        "data_dir" => Ok(Some(app.paths.data().display().to_string())),
        _ => Ok(None),
    }
}

pub fn doc(
    app: &mut App,
    which: Option<String>,
    new: bool,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    if new {
        let body = crate::cmd::text_or_stdin(Vec::new(), lang)?;
        if let Err(eats) = tisty_core::docs::survives(&body) {
            anyhow::bail!("{}", lang.fill("doc-eaten", &[("what", eats)]));
        }
        let made = tisty_core::docs::create(&app.paths.docs(), app.device(), &body)?;
        let order = tisty_core::order::last_of(
            app.state
                .docs
                .values()
                .filter(|one| one.folder.is_none())
                .map(|one| one.order.as_str()),
        );
        app.commit(tisty_core::Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id.clone(),
                order,
                folder: None,
                page_of: None,
            },
        })?;
        println!("  {}  {}", crate::style::dim(&made.id), made.title);
        return Ok(ExitCode::SUCCESS);
    }

    let Some(which) = which else {
        let titled = |file: &str| {
            tisty_core::docs::resolve(&app.paths.docs(), file)
                .ok()
                .and_then(|at| std::fs::read_to_string(at).ok())
                .map(|body| tisty_core::docs::titled(&body))
                .unwrap_or_default()
        };
        let mut all: Vec<_> = app
            .state
            .docs
            .values()
            .filter(|one| one.page_of.is_none())
            .collect();
        all.sort_by(|a, b| a.order.cmp(&b.order));
        if all.is_empty() {
            println!("  {}", crate::style::dim(lang.get("no-docs")));
        }
        for one in all {
            println!("  {}  {}", crate::style::dim(&one.file), titled(&one.file));
            for page in app.state.pages_of(one.id) {
                println!(
                    "    {}  {}",
                    crate::style::dim(&page.file),
                    titled(&page.file)
                );
            }
        }
        return Ok(ExitCode::SUCCESS);
    };

    let Some(held) = app.state.docs.values().find(|one| one.file == which) else {
        anyhow::bail!("{}", lang.fill("no-such-doc", &[("name", &which)]));
    };
    print!("{}", tisty_core::docs::read(&app.paths.docs(), &held.file)?);
    Ok(ExitCode::SUCCESS)
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

pub fn export(
    app: &App,
    tokens: &[String],
    markdown: bool,
    today: Date,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let mut filter = Filter::parse(tokens, app, today, lang)?;

    if tokens.is_empty() {
        filter.inner.scope = tisty_core::view::Scope::Either;
        filter.inner.window = None;
    }
    let mut tasks = app.state.matching(&filter.inner, today);
    if !filter.inner.hidden {
        let mut folded = filter.inner.clone();
        folded.hidden = true;
        tasks.extend(app.state.matching(&folded, today));
    }

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
