use std::process::ExitCode;

use jiff::civil::Date;

use crate::app::{App, Edited};
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
            let remote = match (later(app), in_the_cloud(app, lang)) {
                (true, _) => Some(lang.get("remote-later").to_string()),
                (_, Some(cloud)) => Some(cloud),
                _ => value(app, "remote")?,
            };
            show("remote", remote.as_deref());
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
                None if key == "remote" && later(app) => {
                    eprintln!("{}", lang.get("remote-later"));
                    Ok(ExitCode::from(EXIT_NOT_FOUND))
                }
                None if key == "remote"
                    && let Some(cloud) = in_the_cloud(app, lang) =>
                {
                    eprintln!("{cloud}");
                    Ok(ExitCode::from(EXIT_NOT_FOUND))
                }
                None => {
                    eprintln!("{}", lang.fill("unset-key", &[("key", &key)]));
                    Ok(ExitCode::from(EXIT_NOT_FOUND))
                }
            }
        }

        Some(ConfigAction::Set { key, value }) => {
            check(&key, lang)?;
            let leaving = leaving_now(app);
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
                stays_unless_stranded(app, &tisty_core::config::Sync::folder(&at), lang)?;
            }

            let edited = app.edit_config_if(|c| match key.as_str() {
                "locale" => {
                    c.locale = Some(value.clone());
                    true
                }
                "remote" => replacing_the_way(c, tisty_core::config::Sync::folder(&value)),
                _ => {
                    c.editor = Some(value.clone());
                    true
                }
            })?;
            refused(edited, leaving, lang)?;
            println!("  {} {key} = {value}", style::paint(style::GREEN, "✓"));
            Ok(ExitCode::SUCCESS)
        }

        Some(ConfigAction::Unset { key }) => {
            check(&key, lang)?;
            if key == "remote" {
                stays_unless_stranded(app, &tisty_core::config::Sync::alone(), lang)?;
            }
            let leaving = leaving_now(app);
            let edited = app.edit_config_if(|c| match key.as_str() {
                "locale" => {
                    c.locale = None;
                    true
                }
                "remote" => replacing_the_way(c, tisty_core::config::Sync::alone()),
                _ => {
                    c.editor = None;
                    true
                }
            })?;
            refused(edited, leaving, lang)?;
            println!("  {} {key}", style::dim("✕"));
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn replacing_the_way(config: &mut tisty_core::Config, to: tisty_core::config::Sync) -> bool {
    use tisty_core::config::Leaving;
    let held = config
        .sync
        .as_ref()
        .is_some_and(|was| *was != to && matches!(was.leaving(), Leaving::Later | Leaving::Cloud));
    if !held {
        config.sync = Some(to);
    }
    !held
}

fn stays_unless_stranded(
    app: &App,
    to: &tisty_core::config::Sync,
    lang: Lang,
) -> anyhow::Result<()> {
    let config = app.config();
    let Some(was) = config.sync.as_ref() else {
        return Ok(());
    };
    let here = tisty_carrier::Here::of(&app.paths, config);
    if tisty_carrier::strands_on_leaving(was, to, config.holds(), &here) {
        let at = was.place().map(|at| at.display().to_string());
        anyhow::bail!(
            "{}",
            lang.fill("shared-away-to-leave", &[("at", &at.unwrap_or_default())])
        );
    }
    Ok(())
}

fn leaving_now(app: &App) -> Option<tisty_core::config::Leaving> {
    app.config()
        .sync
        .as_ref()
        .map(tisty_core::config::Sync::leaving)
}

fn refused(
    edited: Edited,
    leaving: Option<tisty_core::config::Leaving>,
    lang: Lang,
) -> anyhow::Result<()> {
    match edited {
        Edited::Saved => Ok(()),
        Edited::Refused if leaving == Some(tisty_core::config::Leaving::Cloud) => {
            anyhow::bail!("{}", lang.get("cloud-to-leave"))
        }
        Edited::Refused => anyhow::bail!("{}", lang.get("sync-later-to-leave")),
        Edited::Unreadable => anyhow::bail!("{}", lang.get("settings-unreadable")),
    }
}

fn in_the_cloud(app: &App, lang: Lang) -> Option<String> {
    tisty_carrier::cloud_of(app.config().sync.as_ref())
        .map(|at| lang.fill("remote-cloud", &[("email", &at.email)]))
}

fn later(app: &App) -> bool {
    tisty_carrier::chosen(app.config().sync.as_ref()).chosen == tisty_carrier::Chosen::Later
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
        "remote" => {
            Ok(tisty_carrier::place_of(config.sync.as_ref()).map(|at| at.display().to_string()))
        }
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
        let body = tisty_core::speaking::turned(&crate::cmd::text_or_stdin(Vec::new(), lang)?);
        if let Err(eats) = tisty_core::docs::survives(&body) {
            anyhow::bail!("{}", lang.fill("doc-eaten", &[("what", eats)]));
        }
        let made = tisty_core::docs::create(&app.paths.docs(), app.device(), &body)?;
        let order = tisty_core::order::last_of(
            app.state
                .docs
                .values()
                .filter(|one| one.folder.is_none() && one.page_of.is_none())
                .map(|one| one.order.as_str()),
        );
        app.commit(tisty_core::Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: app.state.signed.alias.clone(),
                file: made.id.clone(),
                order,
                said: Some(tisty_core::event::Said::of(&body).by(app.state.signed.alias.clone())),
                folder: None,
                page_of: None,
                task_of: None,
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
        let said = |one: &tisty_core::model::Kept| {
            let mut marks = Vec::new();
            if app.state.held_away(one) {
                marks.push(crate::style::dim(lang.get(match one.archived {
                    true => "left-away",
                    false => "left-covered",
                })));
            }
            if one.flagged.is_some() && !app.state.held_away(one) {
                marks.push(crate::style::dim(lang.get("doc-flagged")));
            }
            match marks.is_empty() {
                true => String::new(),
                false => format!("  {}", marks.join(" ")),
            }
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
            println!(
                "  {}  {}{}",
                crate::style::dim(&one.file),
                titled(&one.file),
                said(one)
            );
            for page in app.state.pages_of(one.id) {
                println!(
                    "    {}  {}{}",
                    crate::style::dim(&page.file),
                    titled(&page.file),
                    said(page)
                );
            }
        }
        return Ok(ExitCode::SUCCESS);
    };

    let Some(held) = app.state.docs.values().find(|one| one.file == which) else {
        anyhow::bail!("{}", lang.fill("no-such-doc", &[("name", &which)]));
    };
    if app.state.held_away(held) {
        println!("  {}", crate::style::dim(lang.get("left-away")));
    } else if let Some(said) = &held.flagged {
        println!(
            "  {} {}",
            crate::style::dim(lang.get("doc-flagged")),
            crate::style::dim(&said.body)
        );
    }
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
