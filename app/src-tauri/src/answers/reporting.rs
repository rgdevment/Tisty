use std::sync::Mutex;

use tisty_core::view::{Filter, Scope};
use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Bound, Refusal, Session, blamed, command, held, report, today};

#[tauri::command(async)]
pub fn rebuild(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    let mut session = held(&session);
    tisty_core::cache::discard(session.paths.cache())
        .map_err(|e| blamed(channel::CACHE, "the cache could not be thrown away", e))?;
    session.cache = tisty_core::cache::Cache::open(session.paths.cache())
        .map_err(|e| blamed(channel::CACHE, "the cache could not be opened", e))?;
    session
        .reproject()
        .map_err(|e| blamed(channel::CACHE, "the store would not project", e))?;
    Ok(())
}

#[tauri::command(async)]
pub fn twinned(
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<Vec<tisty_core::attach::Twins>> {
    let data = held(&session).paths.data().to_path_buf();
    Ok(tisty_core::attach::twins(&data))
}

#[tauri::command(async)]
pub fn checked(session: tauri::State<'_, Mutex<Session>>) -> Answer<Reviewed> {
    let mut session = held(&session);
    let _ = session.reload();
    let audit =
        tisty_core::cache::audit(&session.paths.store(), session.paths.cache()).map_err(|e| {
            witness::error(
                channel::CACHE,
                "the cache could not be audited",
                &[("why", Fact::Why(e.to_string()))],
            );
            Refusal::of("internal")
        })?;

    let mut held: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    held.extend(tisty_core::docs::referenced(&session.paths.docs()));
    let adrift = session.adrift(&held);

    let kept = report::attachments(session.paths.data());
    let told = tisty_core::store::read_all(session.paths.store()).unwrap_or_default();
    let alive: Vec<String> = session
        .state
        .docs
        .values()
        .map(|one| one.file.clone())
        .collect();

    Ok(Reviewed {
        tasks: session.state.tasks.len(),
        lists: session.state.lists.len(),
        agrees: matches!(audit, tisty_core::cache::Audit::Agrees { .. }),
        loose: adrift.files(),
        loose_bytes: adrift.bytes,
        astray: adrift.items,
        stranded: tisty_core::docs::strayed(&session.paths.docs(), &alive),
        missing: tisty_core::docs::missing(&session.paths.docs(), &alive)
            .into_iter()
            .filter_map(|file| {
                let kept = session.state.docs.values().find(|one| one.file == file)?;
                let title = tisty_core::docs::read_before(session.paths.data(), &file)
                    .map(|was| tisty_core::docs::titled(&was))
                    .unwrap_or_default();
                Some(Gone {
                    file,
                    title,
                    id: kept.id.to_string(),
                })
            })
            .collect(),
        events: told.len(),
        machines: report::machines(
            &told,
            session.config.device_id.0.as_str(),
            &session.state.dropped,
            &session.state.assistants,
        ),
        log_bytes: report::weighed(&session.paths.store()),
        docs_bytes: report::weighed(&session.paths.docs()),
        held_bytes: kept.bytes,
        held_files: kept.files,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gone {
    pub id: String,
    pub file: String,
    pub title: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reviewed {
    pub tasks: usize,
    pub lists: usize,
    pub agrees: bool,
    pub loose: usize,
    pub loose_bytes: u64,
    pub astray: Vec<tisty_core::attach::Astray>,
    pub stranded: Vec<tisty_core::docs::Stray>,
    pub missing: Vec<Gone>,
    pub events: usize,
    pub machines: Vec<report::Machine>,
    pub log_bytes: u64,
    pub docs_bytes: u64,
    pub held_bytes: u64,
    pub held_files: usize,
}

#[tauri::command(async)]
pub fn facts(
    session: tauri::State<'_, Mutex<Session>>,
    bound: tauri::State<'_, Bound>,
    names: bool,
    paths: bool,
) -> Answer<report::Facts> {
    let session = held(&session);
    let store = session.paths.store();
    let audit = tisty_core::cache::audit(&store, session.paths.cache());

    let mut referenced: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    referenced.extend(tisty_core::docs::referenced(&session.paths.docs()));
    let adrift = session.adrift(&referenced);
    let kept = report::attachments(session.paths.data());

    let shown = |raw: String| if paths { raw } else { report::hidden(&raw) };
    let lists = session.state.ordered_lists();
    let tags = session.state.tags();

    Ok(report::Facts {
        version: env!("CARGO_PKG_VERSION").to_string(),
        dev: cfg!(debug_assertions),
        sandbox: tisty_core::paths::profile(),
        locale: session
            .locale
            .clone()
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| "?".into()),
        zone: jiff::tz::TimeZone::system()
            .iana_name()
            .unwrap_or("?")
            .to_string(),
        os: report::os(),
        arch: std::env::consts::ARCH,
        webview: tauri::webview_version().ok(),
        store: shown(store.display().to_string()),
        devices: report::devices(&store),
        events: tisty_core::store::read_all(&store)
            .map(|all| all.len())
            .unwrap_or(0),
        open: session.state.matching(&Filter::default(), today()).len(),
        archived: session
            .state
            .matching(
                &Filter {
                    scope: Scope::Archived,
                    ..Default::default()
                },
                today(),
            )
            .len(),
        lists: lists.len(),
        tags: tags.len(),
        list_names: if names {
            lists.iter().map(|one| one.name.clone()).collect()
        } else {
            Vec::new()
        },
        tag_names: if names {
            tags.iter().map(|one| one.to_string()).collect()
        } else {
            Vec::new()
        },
        cache: match audit {
            Ok(tisty_core::cache::Audit::Agrees { .. }) => "agrees",
            Ok(tisty_core::cache::Audit::Stale { .. }) => "stale",
            Ok(tisty_core::cache::Audit::Diverged { .. }) => "diverged",
            _ => "none",
        },
        attachments: kept.files,
        attachment_bytes: kept.bytes,
        loose: adrift.files(),
        loose_bytes: adrift.bytes,
        weight: report::weighed(session.paths.data()),
        syncs: session.config.sync.is_some(),
        shared: !session.config.backs_up(),
        backed_up_at: session.config.backed_up_at.map(|at| at.to_string()),
        quiet: session.config.muted().to_vec(),
        attach_up_to: session.config.copies_up_to(),
        in_path: command::reach().within_reach,
        shortcut: bound.0.clone(),
    })
}

#[tauri::command(async)]
pub fn keep_report(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
    text: String,
    logs: bool,
) -> Answer<()> {
    let path = std::path::PathBuf::from(&at);
    if path.extension().is_none_or(|kind| kind != "zip") {
        return Err(Refusal::about("cannotWrite", at));
    }

    let carried: Vec<(String, Vec<u8>)> = if logs {
        let session = held(&session);
        let live = witness::file(&session.paths);
        [live.clone(), live.with_extension("log.1")]
            .into_iter()
            .filter_map(|one| {
                let named = one.file_name()?.to_string_lossy().into_owned();
                Some((named, std::fs::read(&one).ok()?))
            })
            .collect()
    } else {
        Vec::new()
    };

    bundled(&path, &text, &carried)
        .map_err(|e| blamed(channel::WINDOW, "the report would not be written", e))
}

fn bundled(
    at: &std::path::Path,
    text: &str,
    carried: &[(String, Vec<u8>)],
) -> tisty_core::Result<()> {
    use std::io::Write;
    let file = std::fs::File::create(at)?;
    let _ = tisty_core::paths::ours_alone(at);
    let mut zip = zip::ZipWriter::new(file);
    let plain = zip::write::SimpleFileOptions::default();

    let mut put = |named: &str, body: &[u8]| -> tisty_core::Result<()> {
        zip.start_file(named, plain)
            .map_err(|e| tisty_core::Error::Io(std::io::Error::other(e)))?;
        zip.write_all(body)?;
        Ok(())
    };

    put("report.txt", text.as_bytes())?;
    for (named, body) in carried {
        put(named, body)?;
    }
    zip.finish()
        .map_err(|e| tisty_core::Error::Io(std::io::Error::other(e)))?;
    Ok(())
}

#[cfg(test)]
#[path = "reporting_test.rs"]
mod tests;
