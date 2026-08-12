use std::process::ExitCode;

use tisty_core::{cache, config::Sync};
use tisty_sync as carrier;

use crate::{EXIT_ERROR, app::App, i18n::Lang, style};

pub fn sync(
    app: &mut App,
    push: bool,
    pull: bool,
    merge: bool,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let Some(Sync::Folder(dest)) = app.config().sync.clone() else {
        anyhow::bail!("{}", lang.get("no-remote"));
    };

    let data = app.paths.data().to_path_buf();
    let root = app.paths.store();
    let device = app.config().device_id.0.clone();
    let before = cache::fingerprint(&root);

    let way = match (push, pull) {
        (true, _) => carrier::Way::Push,
        (_, true) => carrier::Way::Pull,
        _ => carrier::Way::Both,
    };
    if merge && tisty_core::paths::profile().is_some() {
        anyhow::bail!("{}", lang.get("sandbox-cannot-merge"));
    }
    let join = if merge {
        carrier::Join::Agreed
    } else {
        carrier::Join::Ask
    };
    if let Err(trouble) = carrier::carry(&data, &device, &dest, way, join) {
        return Ok(said(&trouble, lang));
    }

    app.edit_config(|c| c.synced_at = Some(jiff::Timestamp::now()))?;
    let moved = cache::fingerprint(&root) != before;
    println!(
        "\n  {} {}\n",
        style::paint(style::GREEN, "✓"),
        lang.get(if moved { "synced-new" } else { "synced-same" })
    );
    Ok(ExitCode::SUCCESS)
}

fn said(trouble: &carrier::Trouble, lang: Lang) -> ExitCode {
    let text = match trouble {
        carrier::Trouble::NotThere(at) => lang.fill("no-meeting-place", &[("at", at)]),
        carrier::Trouble::OtherStore { theirs } => lang.fill("other-store", &[("id", theirs)]),
        carrier::Trouble::Unreadable(why) => lang.fill("sync-unreadable", &[("why", why)]),
        carrier::Trouble::Refused(why) => lang.fill("sync-refused", &[("why", why)]),
        carrier::Trouble::Broke(why) => lang.fill("sync-broke", &[("why", why)]),
        carrier::Trouble::WouldMerge { theirs } => lang.fill("would-merge", &[("id", theirs)]),
    };
    eprintln!("{text}");
    ExitCode::from(EXIT_ERROR)
}
