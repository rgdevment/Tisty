use tisty_core::witness::{self, Fact, channel};

/// A branch of its own, holding these three files and nothing else. The address has to stay the
/// same across every version, and no release can answer for that: one is named after the version
/// in it, and the one GitHub calls the latest follows the date it was published rather than the
/// version in it, and is never a candidate.
const MANIFEST: &str =
    "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/release-manifest.json";
/// One holds a stable version and the other a candidate, never both, so the track a copy belongs
/// to can be read off the version it is being offered.
pub const LATEST: &str = "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/latest.json";
pub const CANDIDATE: &str =
    "https://raw.githubusercontent.com/rgdevment/Tisty/manifest/candidate.json";
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(5);
const APART: jiff::SignedDuration = jiff::SignedDuration::from_hours(24);

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
    pub package: Option<&'static str>,
    pub installs: bool,
    pub coming: bool,
}

pub fn on_its_way(version: Option<&str>) -> Ready {
    Ready {
        version: version.unwrap_or_default().to_string(),
        route: Route::Store,
        package: None,
        installs: false,
        coming: true,
    }
}

/// The number the manifest publishes, read by a copy the Store keeps. It says a newer one exists,
/// which is a different question from whether the Store will hand it over today.
pub fn published(now: &str, manifest: &str, wants: Option<bool>) -> Option<String> {
    let here: semver::Version = now.parse().ok()?;
    let read: Manifest = serde_json::from_str(manifest).ok()?;
    let best: semver::Version = read.latest.parse().ok()?;
    if !tracking(now, wants) && !best.pre.is_empty() {
        return None;
    }
    (best > here).then(|| best.to_string())
}

pub const fn self_installs(route: Route) -> bool {
    matches!(route, Route::Brew | Route::Download)
}

/// Where asking for candidates can lead anywhere. The Store is sent none, and the command line's
/// candidates live in a formula of another name, so the command that route prints could never
/// hand over the version it had just announced.
pub const fn takes_candidates(route: Route) -> bool {
    matches!(route, Route::Brew | Route::Download)
}

/// Where a release of ours can possibly come from. The plugin fetches whatever address the feed
/// names, so a feed that was tampered with could otherwise send the download anywhere.
const FROM: [&str; 2] = ["github.com", "objects.githubusercontent.com"];

pub fn ours(url: &str) -> bool {
    url.parse::<url::Url>().is_ok_and(|at| {
        at.scheme() == "https" && at.host_str().is_some_and(|host| FROM.contains(&host))
    })
}

/// Where to look for the version being offered, in the order to try. A candidate is looked for
/// among the candidates, and then among the stable releases — because the release that retires a
/// candidate takes its channel away with it, and a copy still holding that offer would otherwise
/// meet a refusal from the network rather than an answer. The stable channel does not have the
/// version it was promised either, so what it is told is that the offer is gone.
pub fn feeds_for(version: &str) -> Vec<&'static str> {
    match version.parse::<semver::Version>() {
        Ok(said) if !said.pre.is_empty() => vec![CANDIDATE, LATEST],
        _ => vec![LATEST],
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    latest: String,
    #[serde(default)]
    latest_prerelease: Option<String>,
}

/// Which track a copy is on. Nobody having said is not the same as somebody having said no: a
/// candidate that was installed by hand goes on being offered candidates until its owner says
/// otherwise, and saying otherwise is what walks it back to the stable track.
pub fn tracking(now: &str, wants: Option<bool>) -> bool {
    wants.unwrap_or_else(|| {
        now.parse::<semver::Version>()
            .is_ok_and(|here| !here.pre.is_empty())
    })
}

pub fn newer(now: &str, manifest: &str, kept: Kept, wants: Option<bool>) -> Option<Ready> {
    if kept.route == Route::Store {
        return None;
    }
    let here: semver::Version = now.parse().ok()?;
    let read: Manifest = serde_json::from_str(manifest).ok()?;

    let mut best: semver::Version = read.latest.parse().ok()?;
    // A stable copy stays on the stable track whatever the manifest says, so a hostile one cannot
    // walk it onto a less-tested build. Asking for the candidates is the one way across, and it is
    // asked for here, not out there.
    let tracking = tracking(now, wants);
    if !tracking && !best.pre.is_empty() {
        return None;
    }
    if tracking
        && let Some(said) = read.latest_prerelease.as_deref()
        && let Ok(candidate) = said.parse::<semver::Version>()
        && candidate > best
    {
        best = candidate;
    }

    (best > here).then(|| offered(best.to_string(), kept))
}

fn offered(version: String, kept: Kept) -> Ready {
    Ready {
        version,
        route: kept.route,
        package: kept.package,
        installs: self_installs(kept.route) && !from_a_mount(),
        coming: false,
    }
}

/// `self_installs` says no for the Store because a copy kept there cannot replace itself from a
/// download; an offer the Store itself made is the one it can take without leaving the window.
pub fn from_the_shop(version: &str, now: &str) -> Option<Ready> {
    let here: semver::Version = now.parse().ok()?;
    let said: semver::Version = version.parse().ok()?;

    (said > here).then(|| Ready {
        version: said.to_string(),
        route: Route::Store,
        package: None,
        installs: true,
        coming: false,
    })
}

/// What the last look found, so closing the window does not take the offer away with it. The copy
/// may have moved between a download and a cask since, so where it stands is read again — and so
/// may the track it is on, which is why the rule is applied here too rather than trusted to
/// whatever was true when the answer was written down.
pub fn remembered(now: &str, said: Option<&str>, kept: Kept, wants: Option<bool>) -> Option<Ready> {
    if kept.route == Route::Store {
        return None;
    }
    let here: semver::Version = now.parse().ok()?;
    let kept_version: semver::Version = said?.parse().ok()?;

    if !tracking(now, wants) && !kept_version.pre.is_empty() {
        return None;
    }

    (kept_version > here).then(|| offered(kept_version.to_string(), kept))
}

/// A clock put back leaves the last look in the future, and a copy that only counts forward from
/// it would wait out the difference before ever looking again.
pub fn due(last: Option<jiff::Timestamp>, now: jiff::Timestamp) -> bool {
    last.is_none_or(|at| at > now || now.duration_since(at) >= APART)
}

pub fn route() -> Kept {
    chosen(std::env::current_exe().ok().as_deref(), |at| at.is_dir())
}

/// A copy running from the mounted disk image cannot replace itself: the volume is read only, and
/// the plugin only finds that out after the whole download.
pub fn mounted(running: Option<&std::path::Path>) -> bool {
    cfg!(target_os = "macos")
        && running.is_some_and(|at| at.starts_with("/Volumes/") || at.starts_with("/private/tmp/"))
}

pub fn from_a_mount() -> bool {
    mounted(std::env::current_exe().ok().as_deref())
}

/// The plugin names the platform after the architecture the binary was built for, and a copy
/// running under Rosetta would ask for Intel on a machine that is not. Only an Apple Silicon Mac
/// can be translating, so a translated copy asks for the native build and comes out of the update
/// as it.
pub fn platform(translated: bool) -> Option<&'static str> {
    translated.then_some("darwin-aarch64")
}

const PREFIXES: [&str; 2] = ["/opt/homebrew", "/usr/local"];
const CASKS: [&str; 2] = ["tisty", "tisty-beta"];
const FORMULAE: [&str; 2] = ["tisty-cli", "tisty-cli-beta"];

fn chosen(running: Option<&std::path::Path>, there: impl Fn(&std::path::Path) -> bool) -> Kept {
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
#[path = "update_test.rs"]
mod tests;
