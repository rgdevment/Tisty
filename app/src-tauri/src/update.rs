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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub version: String,
    pub route: Route,
    pub url: &'static str,
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
pub fn newer(now: &str, manifest: &str, route: Route) -> Option<Ready> {
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
        route,
        url: RELEASES,
    })
}

pub fn due(last: Option<jiff::Timestamp>, now: jiff::Timestamp) -> bool {
    last.is_none_or(|at| now.duration_since(at) >= APART)
}

/// Asked of the running program, never of a setting: whoever installed it is
/// not always whoever is using it, and a wrong instruction is worse than none.
pub fn route() -> Route {
    chosen(std::env::current_exe().ok().as_deref(), |at| at.is_dir())
}

fn chosen(running: Option<&std::path::Path>, there: impl Fn(&std::path::Path) -> bool) -> Route {
    let packaged = running.is_some_and(|at| {
        at.components().any(|part| {
            part.as_os_str()
                .to_string_lossy()
                .starts_with("WindowsApps")
        })
    });
    if packaged {
        return Route::Store;
    }

    for (cask, route) in [("tisty", Route::Brew), ("tisty-cli", Route::BrewCli)] {
        let brewed = ["/opt/homebrew/Caskroom/", "/usr/local/Caskroom/"]
            .iter()
            .any(|root| there(std::path::Path::new(&format!("{root}{cask}"))));
        if brewed {
            return route;
        }
    }
    Route::Download
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
        let found = newer("0.2.0", FEED, Route::Download).expect("0.3.0 is newer");

        assert_eq!(found.version, "0.3.0");
    }

    #[test]
    fn a_candidate_is_offered_the_newest_of_either() {
        assert_eq!(
            newer("0.3.0-rc1", FEED, Route::Download).unwrap().version,
            "0.4.0-rc1"
        );
    }

    /// A candidate older than the stable release is superseded by it.
    #[test]
    fn a_candidate_takes_the_stable_one_when_it_is_ahead() {
        let feed = r#"{"latest":"0.5.0","latestPrerelease":"0.4.0-rc1"}"#;

        assert_eq!(
            newer("0.4.0-rc1", feed, Route::Download).unwrap().version,
            "0.5.0"
        );
    }

    #[test]
    fn the_same_version_is_not_an_update() {
        assert!(newer("0.3.0", FEED, Route::Download).is_none());
        assert!(newer("0.4.0-rc1", FEED, Route::Download).is_none());
    }

    #[test]
    fn a_manifest_that_makes_no_sense_says_nothing() {
        assert!(newer("0.1.0", "not json", Route::Download).is_none());
        assert!(newer("0.1.0", r#"{"latest":"tomorrow"}"#, Route::Download).is_none());
    }

    #[test]
    fn a_manifest_without_a_candidate_still_reads() {
        let feed = r#"{"latest":"0.3.0"}"#;

        assert_eq!(
            newer("0.2.0-rc1", feed, Route::Download).unwrap().version,
            "0.3.0"
        );
    }

    #[test]
    fn a_project_with_no_stable_release_yet_still_works() {
        let feed = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"0.2.0-rc6"}"#;

        assert_eq!(
            newer("0.2.0-rc5", feed, Route::Download).unwrap().version,
            "0.2.0-rc6"
        );
        assert!(newer("0.1.0", feed, Route::Download).is_none());
    }

    fn nowhere(_: &std::path::Path) -> bool {
        false
    }

    #[test]
    fn a_copy_under_windowsapps_is_kept_by_the_store() {
        let at = std::path::Path::new(r"C:\Program Files\WindowsApps\Tisty\tisty.exe");

        assert_eq!(chosen(Some(at), nowhere), Route::Store);
    }

    #[test]
    fn a_cask_answers_with_its_own_command() {
        let plain = std::path::Path::new("/Applications/Tisty.app/Contents/MacOS/tisty");

        assert_eq!(
            chosen(Some(plain), |at| at.ends_with("Caskroom/tisty")),
            Route::Brew
        );
        assert_eq!(
            chosen(Some(plain), |at| at.ends_with("Caskroom/tisty-cli")),
            Route::BrewCli
        );
    }

    #[test]
    fn everything_else_gets_the_page() {
        let at = std::path::Path::new("C:/Program Files/Tisty/tisty.exe");

        assert_eq!(chosen(Some(at), nowhere), Route::Download);
        assert_eq!(chosen(None, nowhere), Route::Download);
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
