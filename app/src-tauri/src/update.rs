//! Whether a newer Tisty exists, and how this copy would get it.
//!
//! One read-only request for a small file, nothing sent but the headers a
//! request needs. Silent when it fails: a check that interrupts is worse than
//! one that never happens.
//!
//! The manifest says only which versions exist. Every address this program can
//! open is a constant compiled in, so nothing downloaded can point anyone
//! anywhere — which is what a signature would otherwise be for.

use tisty_core::witness::{self, Fact, channel};

const MANIFEST: &str =
    "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/release-manifest.json";
pub const RELEASES: &str = "https://github.com/rgdevment/Tisty/releases/latest";
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(5);
const APART: jiff::SignedDuration = jiff::SignedDuration::from_hours(24);

/// How this copy is kept up to date, which depends on where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Route {
    Store,
    Brew,
    BrewCli,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    pub route: Route,
    pub package: Option<&'static str>,
}

impl Kept {
    const fn plain(route: Route) -> Self {
        Self {
            route,
            package: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub version: String,
    pub route: Route,
    pub url: &'static str,
    pub package: Option<&'static str>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    latest: String,
    #[serde(default)]
    latest_prerelease: Option<String>,
}

/// A candidate is only offered to somebody already running one, and then only
/// if it beats the stable release too: whatever supersedes an `rc` is what they
/// want, stable or not.
pub fn newer(now: &str, manifest: &str, kept: Kept) -> Option<Ready> {
    let here: semver::Version = now.parse().ok()?;
    let read: Manifest = serde_json::from_str(manifest).ok()?;

    let mut best: semver::Version = read.latest.parse().ok()?;
    if !here.pre.is_empty()
        && let Some(said) = read.latest_prerelease.as_deref()
        && let Ok(candidate) = said.parse::<semver::Version>()
        && candidate > best
    {
        best = candidate;
    }

    (best > here).then(|| Ready {
        version: best.to_string(),
        route: kept.route,
        url: RELEASES,
        package: kept.package,
    })
}

pub fn due(last: Option<jiff::Timestamp>, now: jiff::Timestamp) -> bool {
    last.is_none_or(|at| now.duration_since(at) >= APART)
}

/// Asked of the running program, never of a setting: whoever installed it is
/// not always whoever is using it, and a wrong instruction is worse than none.
pub fn route() -> Kept {
    chosen(std::env::current_exe().ok().as_deref(), |at| at.is_dir())
}

const PREFIXES: [&str; 2] = ["/opt/homebrew", "/usr/local"];
const CASKS: [&str; 2] = ["tisty", "tisty-beta"];
const FORMULAE: [&str; 2] = ["tisty-cli", "tisty-cli-beta"];

fn chosen(running: Option<&std::path::Path>, there: impl Fn(&std::path::Path) -> bool) -> Kept {
    // Read as text, not as a path: `components` splits on a backslash only on
    // Windows, so the question would answer itself wrong anywhere else.
    let packaged = running.is_some_and(|at| {
        at.to_string_lossy()
            .split(['/', '\\'])
            .any(|part| part.eq_ignore_ascii_case("WindowsApps"))
    });
    if packaged {
        return Kept::plain(Route::Store);
    }

    for (shelf, names, route) in [
        ("Caskroom", CASKS, Route::Brew),
        ("Cellar", FORMULAE, Route::BrewCli),
    ] {
        for package in names {
            let brewed = PREFIXES
                .iter()
                .any(|root| there(std::path::Path::new(&format!("{root}/{shelf}/{package}"))));
            if brewed {
                return Kept {
                    route,
                    package: Some(package),
                };
            }
        }
    }
    Kept::plain(Route::Download)
}

pub fn fetch() -> Option<String> {
    let asked = reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        // GitHub refuses a request without one. It names the program and its
        // version, which the download itself already reveals; nothing else.
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?
        .get(MANIFEST)
        .send()
        .ok()?;

    if !asked.status().is_success() {
        witness::warn(
            channel::WINDOW,
            "the release manifest answered with a refusal",
            &[("code", Fact::Count(asked.status().as_u16() as usize))],
        );
        return None;
    }
    asked.text().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEED: &str = r#"{"schema":1,"latest":"0.3.0","latestPrerelease":"0.4.0-rc1"}"#;

    #[test]
    fn a_stable_copy_is_never_pointed_at_a_candidate() {
        let found = newer("0.2.0", FEED, Kept::plain(Route::Download)).expect("0.3.0 is newer");

        assert_eq!(found.version, "0.3.0");
    }

    #[test]
    fn a_candidate_is_offered_the_newest_of_either() {
        assert_eq!(
            newer("0.3.0-rc1", FEED, Kept::plain(Route::Download))
                .unwrap()
                .version,
            "0.4.0-rc1"
        );
    }

    /// A candidate older than the stable release is superseded by it.
    #[test]
    fn a_candidate_takes_the_stable_one_when_it_is_ahead() {
        let feed = r#"{"latest":"0.5.0","latestPrerelease":"0.4.0-rc1"}"#;

        assert_eq!(
            newer("0.4.0-rc1", feed, Kept::plain(Route::Download))
                .unwrap()
                .version,
            "0.5.0"
        );
    }

    #[test]
    fn the_same_version_is_not_an_update() {
        assert!(newer("0.3.0", FEED, Kept::plain(Route::Download)).is_none());
        assert!(newer("0.4.0-rc1", FEED, Kept::plain(Route::Download)).is_none());
    }

    #[test]
    fn a_manifest_that_makes_no_sense_says_nothing() {
        assert!(newer("0.1.0", "not json", Kept::plain(Route::Download)).is_none());
        assert!(
            newer(
                "0.1.0",
                r#"{"latest":"tomorrow"}"#,
                Kept::plain(Route::Download)
            )
            .is_none()
        );
    }

    #[test]
    fn a_manifest_without_a_candidate_still_reads() {
        let feed = r#"{"latest":"0.3.0"}"#;

        assert_eq!(
            newer("0.2.0-rc1", feed, Kept::plain(Route::Download))
                .unwrap()
                .version,
            "0.3.0"
        );
    }

    #[test]
    fn a_project_with_no_stable_release_yet_still_works() {
        let feed = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"0.2.0-rc6"}"#;

        assert_eq!(
            newer("0.2.0-rc5", feed, Kept::plain(Route::Download))
                .unwrap()
                .version,
            "0.2.0-rc6"
        );
        assert!(newer("0.1.0", feed, Kept::plain(Route::Download)).is_none());
    }

    fn nowhere(_: &std::path::Path) -> bool {
        false
    }

    fn at(said: &str) -> Option<&std::path::Path> {
        Some(std::path::Path::new(said))
    }

    fn only(named: &'static str) -> impl Fn(&std::path::Path) -> bool {
        move |what| what == std::path::Path::new(named)
    }

    const APP: &str = "/Applications/Tisty.app/Contents/MacOS/tisty";

    const MSIX: &str =
        r"C:\Program Files\WindowsApps\rgdevment.Tisty_0.2.0.0_x64__8wekyb3d8bbwe\tisty.exe";

    #[test]
    fn a_copy_under_windowsapps_is_kept_by_the_store() {
        assert_eq!(chosen(at(MSIX), nowhere), Kept::plain(Route::Store));
    }

    #[test]
    fn the_separator_is_read_the_same_on_every_system() {
        assert_eq!(
            chosen(at(&MSIX.replace('\\', "/")), nowhere),
            Kept::plain(Route::Store)
        );
    }

    #[test]
    fn a_folder_is_named_windowsapps_or_it_is_not() {
        let alike = r"C:\Program Files\WindowsAppsBackup\Tisty\tisty.exe";

        assert_eq!(chosen(at(alike), nowhere), Kept::plain(Route::Download));
        assert_eq!(
            chosen(at(&MSIX.to_lowercase()), nowhere),
            Kept::plain(Route::Store),
            "Windows does not distinguish the case of a folder"
        );
    }

    #[test]
    fn a_cask_answers_with_its_own_command() {
        assert_eq!(
            chosen(at(APP), only("/opt/homebrew/Caskroom/tisty")),
            Kept {
                route: Route::Brew,
                package: Some("tisty")
            }
        );
    }

    #[test]
    fn the_command_line_is_a_formula_and_lives_where_formulas_do() {
        assert_eq!(
            chosen(
                at("/opt/homebrew/bin/tisty"),
                only("/opt/homebrew/Cellar/tisty-cli")
            ),
            Kept {
                route: Route::BrewCli,
                package: Some("tisty-cli")
            }
        );
        assert_eq!(
            chosen(at(APP), only("/opt/homebrew/Caskroom/tisty-cli")),
            Kept::plain(Route::Download),
            "no formula is ever kept under Caskroom"
        );
    }

    #[test]
    fn a_candidate_is_upgraded_by_the_name_it_was_installed_under() {
        assert_eq!(
            chosen(at(APP), only("/opt/homebrew/Caskroom/tisty-beta")),
            Kept {
                route: Route::Brew,
                package: Some("tisty-beta")
            }
        );
        assert_eq!(
            chosen(
                at("/opt/homebrew/bin/tisty"),
                only("/opt/homebrew/Cellar/tisty-cli-beta")
            ),
            Kept {
                route: Route::BrewCli,
                package: Some("tisty-cli-beta")
            }
        );
    }

    #[test]
    fn the_older_homebrew_root_answers_too() {
        assert_eq!(
            chosen(at(APP), only("/usr/local/Caskroom/tisty")),
            Kept {
                route: Route::Brew,
                package: Some("tisty")
            }
        );
        assert_eq!(
            chosen(
                at("/usr/local/bin/tisty"),
                only("/usr/local/Cellar/tisty-cli")
            ),
            Kept {
                route: Route::BrewCli,
                package: Some("tisty-cli")
            }
        );
    }

    #[test]
    fn the_window_is_the_window_even_beside_its_own_command_line() {
        assert_eq!(
            chosen(at(APP), |_| true),
            Kept {
                route: Route::Brew,
                package: Some("tisty")
            }
        );
    }

    #[test]
    fn what_is_running_wins_over_what_is_merely_installed() {
        assert_eq!(chosen(at(MSIX), |_| true), Kept::plain(Route::Store));
    }

    #[test]
    fn everything_else_gets_the_page() {
        assert_eq!(
            chosen(at(r"C:\Program Files\Tisty\tisty.exe"), nowhere),
            Kept::plain(Route::Download)
        );
        assert_eq!(chosen(at(APP), nowhere), Kept::plain(Route::Download));
        assert_eq!(chosen(None, nowhere), Kept::plain(Route::Download));
    }

    #[test]
    fn nothing_is_owed_before_the_interval_is_up() {
        let now: jiff::Timestamp = "2026-08-12T12:00:00Z".parse().unwrap();
        let recent: jiff::Timestamp = "2026-08-12T09:00:00Z".parse().unwrap();
        let old: jiff::Timestamp = "2026-08-10T09:00:00Z".parse().unwrap();

        assert!(due(None, now), "a copy that never looked should look");
        assert!(!due(Some(recent), now));
        assert!(due(Some(old), now));
    }
}
