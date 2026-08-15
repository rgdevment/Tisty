use std::process::ExitCode;

use tisty_core::config::Sync;
use tisty_sync as carrier;

use crate::{EXIT_ERROR, app::App, i18n::Lang, style};

pub fn sync(
    app: &mut App,
    push: bool,
    pull: bool,
    join: Option<std::path::PathBuf>,
    take_over: Option<std::path::PathBuf>,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let Some(Sync::Folder(dest)) = app.config().sync.clone() else {
        anyhow::bail!("{}", lang.get("no-remote"));
    };

    let data = app.paths.data().to_path_buf();
    let device = app.config().device_id.0.clone();

    let way = match (push, pull) {
        (true, _) => carrier::Way::Push,
        (_, true) => carrier::Way::Pull,
        _ => carrier::Way::Both,
    };
    if let Some(into) = join {
        if tisty_core::paths::profile().is_some() {
            anyhow::bail!("{}", lang.get("sandbox-cannot-join"));
        }
        let aside = app.paths.cache().to_path_buf();
        let made = tisty_core::backup::reset(&app.paths, &into, &aside)?;
        println!(
            "  {}",
            style::dim(&lang.fill(
                "reset-kept",
                &[("at", &into.display().to_string()), ("id", &made.store_id)]
            ))
        );
        *app = App::at(app.paths.clone())?;
    }

    if let Some(into) = take_over {
        if tisty_core::paths::profile().is_some() {
            anyhow::bail!("{}", lang.get("sandbox-cannot-join"));
        }
        let aside = app.paths.cache().to_path_buf();
        let made = tisty_core::backup::take_over(&dest, &into, &aside)?;
        println!(
            "  {}",
            style::dim(&lang.fill(
                "took-over",
                &[("at", &into.display().to_string()), ("id", &made.store_id)]
            ))
        );
    }

    let alive: Vec<String> = app
        .state
        .docs
        .values()
        .map(|one| one.file.clone())
        .collect();
    let moved = match carrier::carry(&data, &device, &dest, way, &alive) {
        Ok(moved) => moved,
        Err(trouble) => return Ok(said(&trouble, lang)),
    };

    let who = app.config().device_id.clone();
    if !tisty_core::store::ledger(app.paths.store())?
        .allowed
        .contains(&who)
    {
        app.commit(tisty_core::Op::DeviceJoin { d: who })?;
    }

    app.edit_config(|c| c.synced_at = Some(jiff::Timestamp::now()))?;
    let told = match (moved.sent > 0, moved.brought > 0) {
        (true, true) => "synced-both",
        (true, false) => "synced-sent",
        (false, true) => "synced-new",
        (false, false) => "synced-same",
    };
    println!(
        "\n  {} {}\n",
        style::paint(style::GREEN, "✓"),
        lang.get(told)
    );
    Ok(ExitCode::SUCCESS)
}

fn said(trouble: &carrier::Trouble, lang: Lang) -> ExitCode {
    let text = match trouble {
        carrier::Trouble::NotThere(at) => lang.fill("no-meeting-place", &[("at", at)]),
        carrier::Trouble::OtherStore { theirs } => lang.fill("would-reset", &[("id", theirs)]),
        carrier::Trouble::Unreadable(why) => lang.fill("sync-unreadable", &[("why", why)]),
        carrier::Trouble::Refused(why) => lang.fill("sync-refused", &[("why", why)]),
        carrier::Trouble::Broke(why) => lang.fill("sync-broke", &[("why", why)]),
        carrier::Trouble::WouldReset { theirs } => lang.fill("would-reset", &[("id", theirs)]),
        carrier::Trouble::NotAllowed(who) => lang.fill("not-allowed", &[("id", who)]),
    };
    eprintln!("{text}");
    ExitCode::from(EXIT_ERROR)
}
