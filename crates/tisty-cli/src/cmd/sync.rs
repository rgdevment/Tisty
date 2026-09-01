use std::process::ExitCode;

use tisty_core::config::Sync;
use tisty_sync as carrier;

use crate::{EXIT_ERROR, app::App, i18n::Lang, style};

pub struct Asked {
    pub push: bool,
    pub pull: bool,
    pub again: bool,
    pub join: Option<std::path::PathBuf>,
    pub take_over: Option<std::path::PathBuf>,
    pub merge: Option<std::path::PathBuf>,
}

pub fn sync(app: &mut App, asked: Asked, lang: Lang) -> anyhow::Result<ExitCode> {
    let Asked {
        push,
        pull,
        again,
        join,
        take_over,
        merge,
    } = asked;
    let Some(Sync::Folder(dest)) = app.config().sync.clone() else {
        anyhow::bail!("{}", lang.get("no-remote"));
    };

    let data = app.paths.data().to_path_buf();
    let mut device = app.config().device_id.0.clone();

    let way = match (push, pull, again) {
        (true, _, _) => carrier::Way::Push,
        (_, true, _) => carrier::Way::Pull,
        (_, _, true) => carrier::Way::Again,
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
        device = app.config().device_id.0.clone();
    }

    if let Some(into) = take_over {
        if tisty_core::paths::profile().is_some() {
            anyhow::bail!("{}", lang.get("sandbox-cannot-join"));
        }
        let aside = app.paths.cache().to_path_buf();
        let ours = tisty_core::store::identity(app.paths.store())?;
        let made = tisty_core::backup::take_over(&dest, &ours, &into, &aside)?;
        println!(
            "  {}",
            style::dim(&lang.fill(
                "took-over",
                &[("at", &into.display().to_string()), ("id", &made.store_id)]
            ))
        );
    }

    if let Some(into) = merge {
        if tisty_core::paths::profile().is_some() {
            anyhow::bail!("{}", lang.get("sandbox-cannot-join"));
        }
        let aside = app.paths.cache().to_path_buf();
        tisty_core::backup::write(&data, &into, &aside)?;
        let done = match carrier::stitch(&data, &device, &dest) {
            Ok(done) => done,
            Err(trouble) => return Ok(said(&trouble, lang)),
        };
        *app = App::at(app.paths.clone())?;
        match done.stitch {
            Some(seam) => println!(
                "  {}",
                style::dim(&lang.fill(
                    "stitched",
                    &[("was", &seam.absorbed), ("now", &seam.survivor)]
                ))
            ),
            None => println!("  {}", style::dim(lang.get("same-lineage"))),
        }
    }

    let alive: Vec<String> = app
        .state
        .docs
        .values()
        .map(|one| one.file.clone())
        .collect();
    let aside = app.paths.cache().to_path_buf();
    let holds = app.config().holds();
    let moved =
        match carrier::carry_holding(&data, Some(&aside), &device, &dest, way, &alive, holds) {
            Ok(moved) => moved,
            Err(trouble) => return Ok(said(&trouble, lang)),
        };

    if moved.brought > 0 {
        *app = App::at(app.paths.clone())?;
        settle_what_arrived(app, &moved.arrived);
    }
    app.tidy_up(true);

    let who = app.config().device_id.clone();
    if !tisty_core::store::ledger(app.paths.store())?
        .allowed
        .contains(&who)
    {
        app.commit(tisty_core::Op::DeviceJoin {
            d: who,
            k: Some(tisty_core::DeviceKind::Machine),
        })?;
    }

    let heard = moved.brought > 0;
    app.edit_config(|c| {
        c.synced_at = Some(jiff::Timestamp::now());
        if heard {
            c.heard_at = c.synced_at;
        }
    })?;
    let told = match (moved.sent > 0, moved.brought > 0) {
        (true, true) => "synced-both",
        (true, false) if again => "synced-again",
        (true, false) => "synced-sent",
        (false, true) => "synced-new",
        (false, false) => "synced-same",
    };
    println!("\n  {} {}", style::paint(style::GREEN, "✓"), lang.get(told));

    for (many, word) in [
        (&moved.joined, "papers-joined"),
        (&moved.undecided_ids(), "papers-undecided"),
        (&moved.astray, "papers-astray"),
        (&moved.unreadable, "machines-unreadable"),
    ] {
        if many.is_empty() {
            continue;
        }
        println!(
            "  {}",
            style::dim(&lang.fill(word, &[("names", &many.join(", "))]))
        );
    }
    println!();
    Ok(ExitCode::SUCCESS)
}

fn said(trouble: &carrier::Trouble, lang: Lang) -> ExitCode {
    let text = match trouble {
        carrier::Trouble::NotThere(at) => lang.fill("no-meeting-place", &[("at", at)]),
        carrier::Trouble::OtherStore { theirs } => lang.fill("would-reset", &[("id", theirs)]),
        carrier::Trouble::Newer(who) => lang.fill("sync-newer", &[("id", who)]),
        carrier::Trouble::Unreadable(why) => lang.fill("sync-unreadable", &[("why", why)]),
        carrier::Trouble::Refused(why) => lang.fill("sync-refused", &[("why", why)]),
        carrier::Trouble::Broke(why) => lang.fill("sync-broke", &[("why", why)]),
        carrier::Trouble::WouldReset { theirs } => lang.fill("would-reset", &[("id", theirs)]),
        carrier::Trouble::NotAllowed(who) => lang.fill("not-allowed", &[("id", who)]),
        carrier::Trouble::SameName(who) => lang.fill("same-name", &[("id", who)]),
        carrier::Trouble::Emptied(at) => lang.fill("emptied-place", &[("at", at)]),
    };
    eprintln!("{text}");
    ExitCode::from(EXIT_ERROR)
}

fn settle_what_arrived(app: &mut App, files: &[String]) {
    let told = tisty_core::tidy::settling_what_arrived(&app.paths, &app.state, files);
    if !told.is_empty() {
        let _ = app.commit_all(told);
    }
}
