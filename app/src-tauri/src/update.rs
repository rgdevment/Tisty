use std::io::Read;

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
/// Market and language are fixed, so the request says nothing of the person making it.
const CATALOG: &str =
    "https://displaycatalog.mp.microsoft.com/v7.0/products/9PGVWXD8X93N?market=US&languages=en-US";
/// Its answer carries the whole listing, about 42 KB today, far past what a feed file may weigh.
const CATALOG_AT_MOST: usize = 256 * 1024;
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(5);
const APART: jiff::SignedDuration = jiff::SignedDuration::from_hours(24);
/// Short of the window's six-hour timer, so that a tick always finds the look due.
const APART_AT_THE_SHOP: jiff::SignedDuration = jiff::SignedDuration::from_hours(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Route {
    Store,
    Brew,
    Download,
}

impl Route {
    pub const fn code(self) -> &'static str {
        match self {
            Route::Store => "store",
            Route::Brew => "brew",
            Route::Download => "download",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    pub route: Route,
}

impl Kept {
    const fn plain(route: Route) -> Self {
        Self { route }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub version: String,
    pub route: Route,
    pub installs: bool,
    pub coming: bool,
}

pub fn on_its_way(version: Option<&str>) -> Ready {
    Ready {
        version: version.unwrap_or_default().to_string(),
        route: Route::Store,
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
        installs: self_installs(kept.route) && !from_a_mount(),
        coming: false,
    }
}

/// `self_installs` says no for the Store because a copy kept there cannot replace itself from a
/// download; an offer the Store itself made is the one it can take without leaving the window.
/// Any number comes from elsewhere, since the Store's offer names the package installed here.
pub fn from_the_shop(version: Option<&str>) -> Ready {
    Ready {
        version: version.unwrap_or_default().to_string(),
        route: Route::Store,
        installs: true,
        coming: false,
    }
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
    passed(last, now, APART)
}

pub fn due_at_the_shop(last: Option<jiff::Timestamp>, now: jiff::Timestamp) -> bool {
    passed(last, now, APART_AT_THE_SHOP)
}

fn passed(
    last: Option<jiff::Timestamp>,
    now: jiff::Timestamp,
    apart: jiff::SignedDuration,
) -> bool {
    last.is_none_or(|at| at > now || now.duration_since(at) >= apart)
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

fn chosen(running: Option<&std::path::Path>, there: impl Fn(&std::path::Path) -> bool) -> Kept {
    if running.is_some_and(tisty_core::paths::under_windows_apps) {
        return Kept::plain(Route::Store);
    }

    let brewed = CASKS.iter().any(|package| {
        PREFIXES
            .iter()
            .any(|root| there(std::path::Path::new(&format!("{root}/Caskroom/{package}"))))
    });
    Kept::plain(if brewed { Route::Brew } else { Route::Download })
}

pub fn feed_key() -> Option<&'static str> {
    option_env!("TISTY_FEED_KEY").filter(|key| !key.is_empty())
}

pub fn trusted(said: &[u8], signature: &[u8], key: &str, name: &str) -> bool {
    std::str::from_utf8(signature)
        .is_ok_and(|signature| tisty_carrier::policy::signed_as(said, signature, key, name))
}

fn named(url: &str) -> &str {
    let file = url.rsplit('/').next().unwrap_or(url);
    file.split('?').next().unwrap_or(file)
}

pub enum Heard {
    Nothing,
    Unsigned,
    Said(String),
}

fn asked(client: &reqwest::blocking::Client, url: &str, at_most: usize) -> Option<Vec<u8>> {
    let answer = match client.get(url).send() {
        Ok(answer) => answer,
        Err(e) => {
            witness::standing(
                channel::UPDATE,
                "the release feed could not be reached",
                &[
                    ("file", Fact::Id(named(url).to_string())),
                    (
                        "why",
                        Fact::Why(tisty_carrier::said::told(&e.without_url())),
                    ),
                ],
            );
            return None;
        }
    };
    if !answer.status().is_success() {
        witness::warn(
            channel::UPDATE,
            "the release feed answered with a refusal",
            &[
                ("file", Fact::Id(named(url).to_string())),
                ("code", Fact::Count(answer.status().as_u16() as usize)),
            ],
        );
        return None;
    }
    let mut held = Vec::new();
    answer
        .take(at_most as u64 + 1)
        .read_to_end(&mut held)
        .ok()?;
    Some(held)
}

fn client() -> Option<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()
}

enum Signed {
    Nothing,
    Unsigned,
    Said(Vec<u8>),
}

fn pair(client: &reqwest::blocking::Client, url: &str, past: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let at_most = tisty_carrier::policy::READ_AT_MOST;
    let said = asked(client, &format!("{url}{past}"), at_most)?;
    let signature = asked(client, &format!("{url}.sig{past}"), at_most)?;
    Some((said, signature))
}

// The file and its signature are cached apart, so a pair that does not do is asked again past the cache.
fn read_signed(
    client: &reqwest::blocking::Client,
    url: &str,
    key: &str,
    wanted: impl Fn(&[u8]) -> bool,
) -> Signed {
    let fresh = format!("?at={}", jiff::Timestamp::now().as_second());
    for past in ["", fresh.as_str()] {
        let Some((said, signature)) = pair(client, url, past) else {
            return Signed::Nothing;
        };
        if trusted(&said, &signature, key, named(url)) && wanted(&said) {
            return Signed::Said(said);
        }
    }
    Signed::Unsigned
}

/// A look that never answered is Nothing; Unsigned is only a feed that answered and did not verify.
pub fn fetch() -> Heard {
    let Some(client) = client() else {
        return Heard::Nothing;
    };
    let text = |said: Vec<u8>| match String::from_utf8(said) {
        Ok(said) => Heard::Said(said),
        Err(_) => {
            set_aside(MANIFEST, "not-text");
            Heard::Nothing
        }
    };
    let Some(key) = feed_key() else {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            witness::note(
                channel::UPDATE,
                "this build has no feed key, so the release feed is read unsigned",
                &[],
            );
        });
        return asked(&client, MANIFEST, tisty_carrier::policy::READ_AT_MOST)
            .map_or(Heard::Nothing, text);
    };
    match read_signed(&client, MANIFEST, key, |_| true) {
        Signed::Said(said) => text(said),
        Signed::Nothing => Heard::Nothing,
        Signed::Unsigned => {
            set_aside(MANIFEST, "signature");
            Heard::Unsigned
        }
    }
}

/// What the Store has published, which a Store copy may still not have been handed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Catalog {
    Unheard,
    NothingNewer,
    Lists(String),
}

/// A full name reads `name_version_arch_resource_publisher`, and the family is its two ends.
fn ours_in_the_catalog(full: &str) -> Option<semver::Version> {
    let mut parts = full.split('_');
    let (name, version, publisher) = (parts.next()?, parts.next()?, parts.next_back()?);
    if format!("{name}_{publisher}") != tisty_core::paths::STORE_PACKAGE {
        return None;
    }
    let mut numbers = version.split('.');
    format!(
        "{}.{}.{}",
        numbers.next()?,
        numbers.next()?,
        numbers.next()?
    )
    .parse()
    .ok()
}

/// Microsoft does not document this answer, so one with no package of ours in it says nothing.
pub fn listed(now: &str, said: &str) -> Catalog {
    let Ok(here) = now.parse::<semver::Version>() else {
        return Catalog::Unheard;
    };
    let Ok(read) = serde_json::from_str::<serde_json::Value>(said) else {
        return Catalog::Unheard;
    };
    // It can name older packages beside the newest.
    let best = read
        .pointer("/Product/DisplaySkuAvailabilities")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|one| one.pointer("/Sku/Properties/Packages")?.as_array())
        .flatten()
        .filter_map(|package| package.get("PackageFullName")?.as_str())
        .filter_map(ours_in_the_catalog)
        .max();
    match best {
        None => Catalog::Unheard,
        Some(best) if best > here => Catalog::Lists(best.to_string()),
        Some(_) => Catalog::NothingNewer,
    }
}

pub fn catalog(now: &str) -> Catalog {
    let Some(said) = client().and_then(|client| asked(&client, CATALOG, CATALOG_AT_MOST)) else {
        return Catalog::Unheard;
    };
    let answer = listed(now, &String::from_utf8_lossy(&said));
    match &answer {
        Catalog::Unheard => set_aside(CATALOG, "no-package-of-ours"),
        Catalog::NothingNewer => witness::standing(
            channel::UPDATE,
            "the Store's catalog lists nothing newer than this copy",
            &[],
        ),
        Catalog::Lists(version) => witness::standing(
            channel::UPDATE,
            "the Store's catalog lists a newer version",
            &[("version", Fact::Id(version.clone()))],
        ),
    }
    answer
}

fn set_aside(url: &str, why: &'static str) {
    witness::warn(
        channel::UPDATE,
        "a file from the release feed was set aside",
        &[
            ("file", Fact::Id(named(url).to_string())),
            ("why", Fact::Code(why)),
        ],
    );
}

pub fn same_channel(said: &[u8], read: &serde_json::Value) -> bool {
    serde_json::from_slice::<serde_json::Value>(said).is_ok_and(|one| one == *read)
}

// The plugin reads the channel itself and cannot be handed one, so what it read must be the very JSON the feed's key signed.
pub fn channel_vouched(read: &serde_json::Value, version: &str) -> bool {
    let Some(key) = feed_key() else {
        return true;
    };
    let Some(client) = client() else {
        return false;
    };
    let feeds = feeds_for(version);
    let vouched = feeds.iter().any(|url| {
        matches!(
            read_signed(&client, url, key, |said| same_channel(said, read)),
            Signed::Said(_)
        )
    });
    if !vouched {
        let files: Vec<&str> = feeds.iter().map(|url| named(url)).collect();
        witness::warn(
            channel::UPDATE,
            "a file from the release feed was set aside",
            &[
                ("file", Fact::Id(files.join(","))),
                ("why", Fact::Code("channel-differs")),
            ],
        );
    }
    vouched
}

pub fn later(now: &str, version: &str) -> bool {
    match (semver::Version::parse(now), semver::Version::parse(version)) {
        (Ok(now), Ok(version)) => version > now,
        _ => false,
    }
}

#[cfg(test)]
#[path = "update_test.rs"]
mod tests;
