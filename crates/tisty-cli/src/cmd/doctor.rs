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
        if adrift.files() > 0 {
            line(
                lang.get("loose-files"),
                &style::dim(&lang.fill(
                    "loose-files-are",
                    &[
                        ("count", &adrift.files().to_string()),
                        ("size", &weighed(adrift.bytes)),
                    ],
                )),
            );
        }

        let alive: Vec<String> = truth.docs.values().map(|one| one.file.clone()).collect();
        let stranded = tisty_core::docs::loose(&app.paths.docs(), &alive);
        if !stranded.is_empty() {
            line(
                lang.get("loose-papers"),
                &style::dim(&lang.fill(
                    "loose-papers-are",
                    &[("count", &stranded.len().to_string())],
                )),
            );
        }

        let gone = tisty_core::docs::missing(&app.paths.docs(), &alive);
        if !gone.is_empty() {
            line(
                lang.get("gone-papers"),
                &style::dim(&lang.fill("gone-papers-are", &[("count", &gone.len().to_string())])),
            );
        }
    }

    let private = app.paths.private();
    let aside = tisty_core::store::displaced(&app.paths);
    if !aside.is_empty() {
        line(
            lang.get("keys-aside"),
            &style::dim(&lang.fill(
                "keys-aside-are",
                &[
                    ("count", &aside.len().to_string()),
                    ("at", &private.display().to_string()),
                ],
            )),
        );
    }

    syncing(app, lang);

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

// Read off this disk alone: the command line never reaches the network nor the keychain.
fn syncing(app: &App, lang: Lang) {
    let config = app.config();
    let data = app.paths.data();
    let place = config
        .sync
        .as_ref()
        .map(tisty_core::moved::place_of)
        .unwrap_or_default();
    if let Some(at) = tisty_carrier::cloud_of(config.sync.as_ref()) {
        let home = app.paths.cloud_home(&at.provider, &at.account);
        cloud(&tisty_carrier::standing(&home), &at.email, lang);
        moved_word(
            std::fs::read_to_string(
                tisty_carrier::mirrored_at(&home).join(tisty_core::moved::NAMED),
            )
            .ok(),
            data,
            &place,
            &config.device_id,
            lang,
        );
    } else if let Some(carrier) = tisty_carrier::chosen(config.sync.as_ref()).carrier {
        moved_word(carrier.word_left(), data, &place, &config.device_id, lang);
    }
    let owed = [app.paths.private(), app.paths.cache().to_path_buf()];
    if owed
        .iter()
        .any(|at| at.join(tisty_core::moved::OWED).is_file())
    {
        line(lang.get("word-owed"), &style::dim(lang.get("word-owed-is")));
    }
}

fn cloud(told: &tisty_carrier::Standing, email: &str, lang: Lang) {
    line(
        lang.get("cloud"),
        &lang.fill("cloud-is", &[("email", email)]),
    );
    line(
        lang.get("cloud-read"),
        &match told.listed {
            true => lang.fill(
                "cloud-read-is",
                &[
                    ("files", &told.mirrored.to_string()),
                    ("attachments", &told.on_the_shelf.to_string()),
                ],
            ),
            false => lang.get("cloud-never-read").to_string(),
        },
    );
    if told.listed {
        line(
            lang.get("cloud-account"),
            &match told.confirmed {
                true => lang.get("cloud-account-confirmed").to_string(),
                false => style::dim(lang.get("cloud-account-unconfirmed")),
            },
        );
    }
    if told.owed + told.forgotten + told.owed_back > 0 {
        line(
            lang.get("cloud-waits"),
            &style::dim(&lang.fill(
                "cloud-waits-are",
                &[
                    ("owed", &told.owed.to_string()),
                    ("forgotten", &told.forgotten.to_string()),
                    ("back", &told.owed_back.to_string()),
                ],
            )),
        );
    }
    line(
        lang.get("cloud-today"),
        &lang.fill(
            "cloud-today-is",
            &[
                ("units", &told.units_spent.to_string()),
                ("units_most", &told.units_a_day.to_string()),
                ("bytes", &weighed(told.bytes_spent)),
                ("bytes_most", &weighed(told.bytes_a_day)),
            ],
        ),
    );
    if let Some(last) = &told.last {
        line(lang.get("cloud-last"), &ended(last, lang));
        if let Some(lasted) = lasting(last, lang) {
            line("", &style::dim(&lasted));
        }
    }
    if told.paused_here {
        line(
            lang.get("cloud-paused"),
            &style::dim(lang.get("cloud-paused-here")),
        );
    }
    if let Some(until) = told.halted_until {
        let when = stamped(until);
        line(
            lang.get("cloud-paused"),
            &style::paint(
                style::RED,
                &lang.fill("cloud-halted-until", &[("when", &when)]),
            ),
        );
    }
    line(
        lang.get("cloud-word"),
        &match told.serial {
            Some(serial) => lang.fill("cloud-word-is", &[("serial", &serial.to_string())]),
            None => lang.get("cloud-word-none").to_string(),
        },
    );
}

fn ended(last: &tisty_carrier::LastRound, lang: Lang) -> String {
    let when = stamped(last.at);
    match last.code.as_str() {
        "ok" => lang.fill("cloud-last-ok", &[("when", &when)]),
        code => match code.strip_prefix("deferred-") {
            Some(
                reason @ ("budget" | "limit" | "offline" | "busy" | "paused" | "changed"
                | "troubled" | "unsent"),
            ) => {
                let left = match last.left {
                    0 => String::new(),
                    left => format!(
                        " {}",
                        lang.fill("deferred-left", &[("left", &left.to_string())])
                    ),
                };
                style::dim(&lang.fill(
                    "cloud-last-waits",
                    &[
                        ("when", &when),
                        ("reason", lang.get(&format!("deferred-{reason}"))),
                        ("left", &left),
                    ],
                ))
            }
            _ => style::paint(
                style::RED,
                &lang.fill("cloud-last-stopped", &[("when", &when), ("code", code)]),
            ),
        },
    }
}

fn lasting(last: &tisty_carrier::LastRound, lang: Lang) -> Option<String> {
    if last.code == "ok" {
        return None;
    }
    let cause = last
        .cause
        .as_deref()
        .filter(|cause| matches!(*cause, "blocked" | "filtered" | "slow"))
        .map(|cause| lang.get(&format!("cause-{cause}")));
    let streak = (last.rounds > 1).then(|| {
        lang.fill(
            "cloud-last-streak",
            &[
                ("rounds", &last.rounds.to_string()),
                ("since", &stamped(last.since)),
            ],
        )
    });
    let told: Vec<String> = [cause.map(str::to_string), streak]
        .into_iter()
        .flatten()
        .collect();
    (!told.is_empty()).then(|| told.join("; "))
}

fn stamped(at: u64) -> String {
    jiff::Timestamp::from_second(i64::try_from(at).unwrap_or(i64::MAX))
        .map(|at| at.strftime("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_default()
}

fn moved_word(
    said: Option<String>,
    data: &std::path::Path,
    place: &str,
    me: &tisty_core::DeviceId,
    lang: Lang,
) {
    if let Some(word) = tisty_core::moved::heeded(said.as_deref(), data, me, place) {
        let said = match word.to.split_once(':') {
            Some(("cloud", email)) => lang.fill(
                "moved-away-to",
                &[("to", &format!("Google Drive · {email}"))],
            ),
            Some(("folder", named)) => lang.fill("moved-away-to", &[("to", named)]),
            _ => lang.get("moved-alone").to_string(),
        };
        line(lang.get("moved-away"), &style::paint(style::RED, &said));
    }
}

fn line(label: &str, value: &str) {
    let pad = " ".repeat(14usize.saturating_sub(label.chars().count()).max(1));
    println!("    {label}{pad}{value}");
}

fn weighed(bytes: u64) -> String {
    match bytes {
        0..=1023 => format!("{bytes} B"),
        1024..=1_048_575 => format!("{} kB", bytes / 1024),
        1_048_576..=1_073_741_823 => format!("{} MB", bytes / 1_048_576),
        _ => format!("{} GB", bytes / 1_073_741_824),
    }
}

#[cfg(test)]
#[path = "doctor_test.rs"]
mod tests;
