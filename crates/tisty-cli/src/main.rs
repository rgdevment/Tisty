#![recursion_limit = "256"]

mod i18n;
mod mcp;

use std::process::ExitCode;

use i18n::Lang;

pub const EXIT_ERROR: u8 = 1;

fn main() -> ExitCode {
    let first = std::env::args().nth(1);
    match first.as_deref() {
        Some("mcp") => served(),
        Some("--version" | "-V") => {
            let said = spoken().fill("version", &[("version", env!("CARGO_PKG_VERSION"))]);
            println!("{said}");
            ExitCode::SUCCESS
        }
        None | Some("--help" | "-h" | "help") => {
            eprintln!("{}", spoken().get("retired"));
            ExitCode::SUCCESS
        }
        _ => retired(),
    }
}

fn retired() -> ExitCode {
    let lang = spoken();
    eprintln!("{}", lang.get("retired"));
    ExitCode::from(EXIT_ERROR)
}

fn served() -> ExitCode {
    match serve() {
        Ok(code) => code,
        Err(e) => {
            let mut facts = vec![("command", tisty_core::witness::Fact::Code("mcp"))];
            if let Some(ours) = e.downcast_ref::<tisty_core::Error>() {
                facts.extend(ours.told());
            }
            tisty_core::witness::warn(
                tisty_core::witness::channel::TERMINAL,
                "the assistant's door ended in an error",
                &facts,
            );
            let lang = spoken();
            let said = match e.downcast_ref::<tisty_core::Error>() {
                Some(tisty_core::Error::UnsupportedVersion { version, .. })
                    if *version == tisty_core::moving::FENCE_VERSION =>
                {
                    lang.get("store-moved").to_string()
                }
                Some(tisty_core::Error::UnsupportedVersion { device, .. }) => {
                    let said = lang.fill("store-newer", &[("device", device)]);
                    match tisty_core::paths::from_the_store() {
                        true => format!("{said}. {}", lang.get("store-newer-store")),
                        false => said,
                    }
                }
                _ => e.to_string(),
            };
            eprintln!("{}: {said}", lang.get("error"));
            ExitCode::from(EXIT_ERROR)
        }
    }
}

// Serving does not hold a store open: each call reads what is on disk, so a window writing beside it is never a stale read.
fn serve() -> anyhow::Result<ExitCode> {
    let paths = tisty_core::Paths::resolve()?;
    tisty_core::witness::keeps(
        tisty_core::witness::file(&paths),
        tisty_core::witness::wants_all(),
    );
    tisty_core::witness::catches(tisty_core::witness::channel::TERMINAL);
    tisty_core::store::brought_home(&paths);
    if paths.the_persons_own() {
        tisty_core::store::keys_guarded(&paths);
    }
    tisty_core::witness::note(
        tisty_core::witness::channel::TERMINAL,
        "the assistant's door opened",
        &[(
            "version",
            tisty_core::witness::Fact::Id(env!("CARGO_PKG_VERSION").to_string()),
        )],
    );
    mcp::serve(paths)
}

// Read straight off the file: loading the settings may settle and write them, which a command that only answers must not do.
fn spoken() -> Lang {
    let configured = tisty_core::Paths::resolve()
        .ok()
        .and_then(|paths| std::fs::read_to_string(paths.config_file()).ok())
        .and_then(|text| toml::from_str::<toml::Table>(&text).ok())
        .and_then(|table| table.get("locale")?.as_str().map(str::to_string));
    Lang::detect(configured.as_deref())
}
