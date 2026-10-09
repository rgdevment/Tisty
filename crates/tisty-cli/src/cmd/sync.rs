use std::process::ExitCode;

use tisty_carrier::{self as carrier, Round};

use crate::{EXIT_ERROR, app::App, i18n::Lang, style};

pub struct Asked {
    pub push: bool,
    pub pull: bool,
    pub again: bool,
    pub join: Option<std::path::PathBuf>,
    pub take_over: Option<std::path::PathBuf>,
    pub merge: Option<std::path::PathBuf>,
    pub confirm: Option<String>,
    pub force: bool,
}

pub fn sync(app: &mut App, asked: Asked, lang: Lang) -> anyhow::Result<ExitCode> {
    let Asked {
        push,
        pull,
        again,
        join,
        take_over,
        merge,
        confirm,
        force,
    } = asked;
    let keeping = carrier::chosen(app.config().sync.as_ref());
    let Some(via) = keeping.carrier else {
        anyhow::bail!(
            "{}",
            lang.get(match keeping.chosen {
                carrier::Chosen::Later => "sync-later",
                _ => "no-remote",
            })
        );
    };
    let Some(dest) = via.place().map(std::path::Path::to_path_buf) else {
        anyhow::bail!("{}", lang.get("no-remote"));
    };

    let data = app.paths.data().to_path_buf();

    if let Some(whose) = confirm {
        return answered_for(app, &dest, &whose, force, lang);
    }

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
        let made = tisty_core::backup::reset(&app.paths, &into, &aside, None)?;
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
        tisty_core::backup::write(&data, &into, &aside, Some(&dest))?;
        let done = match via.stitch(&app.here(), app.signs()) {
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
    let holds = app.config().holds();
    let mut quiet = |_: carrier::Reached| {};
    let round = Round {
        way,
        alive: &alive,
        holds,
        saying: &mut quiet,
    };
    let moved = match via.carry(&app.here(), round) {
        Ok(moved) => moved,
        Err(trouble) => return Ok(said(&trouble, lang)),
    };
    for at in &moved.let_go {
        app.commit(tisty_core::Op::AttachLetGo { d: at.clone() })?;
    }
    for (at, sha256, bytes) in &moved.took_in {
        app.commit(tisty_core::Op::AttachKept {
            d: tisty_core::event::Held {
                at: at.clone(),
                sha256: sha256.clone(),
                bytes: *bytes,
            },
        })?;
    }

    if moved.brought > 0 {
        *app = App::at(app.paths.clone())?;
        settle_what_arrived(app, &moved.arrived);
    }
    let docs = app.paths.docs();
    let wrote = moved
        .to_answer()
        .iter()
        .filter(|file| {
            tisty_core::docs::read(&docs, file).is_ok_and(|body| app.retell(file, &body, None))
        })
        .count();
    let mut quiet = |_: carrier::Reached| {};
    let handed_on = Round {
        way: carrier::Way::Push,
        alive: &alive,
        holds,
        saying: &mut quiet,
    };
    if wrote > 0 && via.carry(&app.here(), handed_on).is_err() {
        tisty_core::witness::warn(
            tisty_core::witness::channel::SYNC,
            "a document was written down but not handed on yet",
            &[],
        );
    }
    app.tidy_up(true);

    let who = app.config().device_id.clone();
    let shown = tisty_core::signing::mine(&app.paths, &who)
        .as_ref()
        .map(tisty_core::signing::shown);
    if let Some(shown) = &shown {
        tisty_core::vouched::confirm(app.paths.data(), &who, shown);
        if tisty_core::vouched::confirmed(app.paths.data(), &who)
            .is_some_and(|stood| &stood.key != shown)
        {
            tisty_core::witness::warn(
                tisty_core::witness::channel::STORE,
                "this machine signs with a key other than the one it answered for, so the others will turn its history away",
                &[("at", tisty_core::witness::Fact::Id(who.0.clone()))],
            );
        }
    }
    let starting = shown.is_some() && !app.state.keys.contains_key(&who);
    if starting {
        tisty_core::vouched::confirm_each(app.paths.data(), &app.state.keys);
    }
    if tisty_core::store::ledger(app.paths.store())?
        .allowed
        .contains(&who)
    {
        if let Some(shown) = shown.filter(|_| starting) {
            app.commit(tisty_core::Op::DeviceKey { d: who, p: shown })?;
        }
    } else {
        app.commit(tisty_core::Op::DeviceJoin {
            d: who,
            k: Some(tisty_core::DeviceKind::Machine),
            p: shown,
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

    if moved.unprojected {
        println!("  {}", style::dim(lang.get("own-log-unreadable")));
    }
    for (many, word) in [
        (&moved.joined, "papers-joined"),
        (&moved.undecided_ids(), "papers-undecided"),
        (&moved.waiting, "papers-waiting"),
        (&moved.astray, "papers-astray"),
        (&moved.unreadable, "machines-unreadable"),
        (&moved.disowned, "machines-disowned"),
        (&moved.unconfirmed, "machines-unconfirmed"),
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

fn answered_for(
    app: &App,
    dest: &std::path::Path,
    whose: &str,
    force: bool,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let who = tisty_core::DeviceId(whose.to_string());
    if !tisty_core::store::is_device_name(whose) {
        eprintln!("{}", lang.fill("not-a-machine", &[("id", whose)]));
        return Ok(ExitCode::from(EXIT_ERROR));
    }
    let says =
        app.state.keys.get(&who).cloned().or_else(|| {
            tisty_core::store::key_said_in(&dest.join(carrier::STORE).join(whose), &who)
        });
    let Some(says) = says else {
        eprintln!(
            "{}",
            lang.fill("machine-signs-with-nothing", &[("id", whose)])
        );
        return Ok(ExitCode::from(EXIT_ERROR));
    };
    println!(
        "\n  {}",
        style::dim(&lang.fill("machine-signs-with", &[("id", whose)]))
    );
    println!("  {says}");
    if let Some(code) = tisty_core::signing::spoken(&says) {
        println!(
            "  {}",
            style::dim(&lang.fill("machine-code", &[("code", &code)]))
        );
    }
    println!();
    if !crate::cmd::confirm(&lang.fill("confirm-key", &[("id", whose)]), force, lang)? {
        return Ok(ExitCode::SUCCESS);
    }
    if !tisty_core::vouched::confirm(app.paths.data(), &who, &says) {
        eprintln!("{}", lang.fill("key-not-answered-for", &[("id", whose)]));
        return Ok(ExitCode::from(EXIT_ERROR));
    }
    carrier::turned::let_through(app.paths.data(), whose);
    println!(
        "  {} {}",
        style::paint(style::GREEN, "✓"),
        lang.fill("key-answered-for", &[("id", whose), ("key", &says)])
    );
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
        carrier::Trouble::Shape(at) => lang.fill("sync-shape", &[("at", at)]),
        carrier::Trouble::Unshaped(at) => lang.fill("sync-unshaped", &[("at", at)]),
    };
    eprintln!("{text}");
    ExitCode::from(EXIT_ERROR)
}

fn settle_what_arrived(app: &mut App, files: &[String]) {
    let told = tisty_core::tidy::settling_what_arrived(&app.paths, &app.state, files);
    if !told.is_empty()
        && let Err(e) = app.commit_all(told)
    {
        tisty_core::witness::warn(
            tisty_core::witness::channel::SYNC,
            "where a document's pages sit could not be settled",
            &[("why", tisty_core::witness::Fact::Why(e.to_string()))],
        );
    }
}
