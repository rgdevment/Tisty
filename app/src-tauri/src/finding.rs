use std::sync::Mutex;

use crate::{Answer, Refusal, Session, elsewhere, held, vouching};

/// Long enough for a file already on its way, short enough that nobody thinks the app hung.
const COMES_WITHIN: std::time::Duration = std::time::Duration::from_millis(1_500);

pub fn unreachable(found: Sought, reference: String) -> Refusal {
    match found {
        Sought::Coming => Refusal::about("comingDown", reference),
        Sought::Away => Refusal::of("sharedAway"),
        Sought::Torn => Refusal::about("attachmentTorn", reference),
        Sought::Held => Refusal::about("heldAway", reference),
        _ => Refusal::about("cannotRead", reference),
    }
}

pub enum Sought {
    At(std::path::PathBuf),
    Coming,
    Away,
    /// It is there, and it is not what its name says it is.
    Torn,
    Held,
    No,
}

/// A link or a junction under somebody else's folder can point anywhere; the store's tree is ours.
pub fn under_root(at: &std::path::Path, root: &std::path::Path) -> bool {
    match (at.canonicalize(), root.canonicalize()) {
        (Ok(at), Ok(root)) => at.starts_with(root),
        _ => false,
    }
}

pub struct Where {
    pub data: std::path::PathBuf,
    pub avowed: Option<String>,
    pub shared: Option<std::path::PathBuf>,
    pub reached: std::path::PathBuf,
    /// A cloud reached through its API, which brings an attachment down the first time it is opened.
    pub cloud: Option<(tisty_carrier::Shared, tisty_carrier::Here)>,
    /// What the log says it weighs, known only with a cloud of its own, where a copy here can be let go of.
    pub weight: Option<u64>,
}

/// The attachment a reference names, without the moment or the page a link may point at inside it.
pub fn plain(reference: &str) -> &str {
    reference.split(['#', '?']).next().unwrap_or(reference)
}

/// On disk first; then, with a cloud of its own, what the log says, so one let go of here is not lost.
pub fn weight_of(
    reference: &str,
    data: &std::path::Path,
    shared: Option<&std::path::Path>,
    away: Option<u64>,
) -> Option<u64> {
    where_it_lies(reference, data, shared)
        .and_then(|at| std::fs::metadata(at).ok())
        .filter(std::fs::Metadata::is_file)
        .map(|told| told.len())
        .or(away)
}

/// Taken and let go of at once: what follows can wait on a cloud, and holding the session while it
/// does would freeze the window.
pub fn where_to(session: &tauri::State<'_, Mutex<Session>>, reference: &str) -> Where {
    let session = held(session);
    let kept = session.state.kept.get(plain(reference));
    let cloud = session
        .carrier()
        .filter(|carrier| carrier.frees_space())
        .map(|carrier| (carrier, session.here()));
    Where {
        data: session.paths.data().to_path_buf(),
        avowed: kept.map(|(sha256, _)| sha256.clone()),
        weight: cloud.as_ref().and(kept).map(|(_, bytes)| *bytes),
        shared: session.shared_now(),
        reached: session.paths.data().join(tisty_core::lately::USED),
        cloud,
    }
}

/// Runs the work away from the window. What a cloud brought with no history behind it stays here
/// unvouched, never written into the log on the strength of its name alone.
pub async fn fetched<T: Send + 'static>(
    session: &tauri::State<'_, Mutex<Session>>,
    reference: String,
    work: impl FnOnce(String, Where) -> Answer<T> + Send + 'static,
) -> Answer<T> {
    let at = where_to(session, &reference);
    elsewhere(move || work(reference, at)).await?
}

/// The store first, then the shared folder, which is where a machine that let go of it kept it.
/// For a size or a count, where reading the whole of it to check would fetch what nobody asked to
/// open — and on a cloud folder, fetching is the one thing this setting exists to avoid.
pub fn where_it_lies(
    reference: &str,
    data: &std::path::Path,
    shared: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    for root in [Some(data), shared].into_iter().flatten() {
        let Ok(at) = tisty_core::attach::resolve(reference, root) else {
            continue;
        };
        if at.is_file() && (root == data || under_root(&at, root)) {
            return Some(at);
        }
    }
    None
}

pub fn handed_over(reference: &str, at: &Where) -> Answer<std::path::PathBuf> {
    match found_in(
        reference,
        &at.data,
        at.shared.as_deref(),
        at.avowed.as_deref(),
    ) {
        Sought::At(found) => Ok(found),
        Sought::No if let Some((carrier, here)) = &at.cloud => {
            brought(&**carrier, here, plain(reference))
        }
        other => Err(unreachable(other, reference.to_string())),
    }
}

fn brought(
    carrier: &dyn tisty_carrier::Carrier,
    here: &tisty_carrier::Here,
    reference: &str,
) -> Answer<std::path::PathBuf> {
    match carrier.brought(here, reference).map_err(crate::said)? {
        tisty_carrier::Brought::At(found) | tisty_carrier::Brought::Unvouched { at: found, .. } => {
            Ok(found)
        }
        tisty_carrier::Brought::Later => Err(Refusal::about("cloudLater", reference.to_string())),
        tisty_carrier::Brought::Absent => Err(Refusal::about("cannotRead", reference.to_string())),
    }
}

pub fn found_in(
    reference: &str,
    data: &std::path::Path,
    shared: Option<&std::path::Path>,
    avowed: Option<&str>,
) -> Sought {
    for root in [Some(data), shared].into_iter().flatten() {
        let Ok(at) = tisty_core::attach::resolve(reference, root) else {
            continue;
        };
        let ours = root == data;
        if !at.is_file() {
            let left = tisty_core::holes::left_in_place(&at);
            if !matches!(left, tisty_core::holes::Left::Sidecar(_)) {
                continue;
            }
            if !tisty_core::holes::can_ask(&left) {
                return Sought::Away;
            }
            if !tisty_core::holes::waited_for(&at, COMES_WITHIN) {
                return Sought::Coming;
            }
        }
        // What comes back from a cloud answers for its name like anything else that lives there.
        if !ours && !under_root(&at, root) {
            return Sought::No;
        }
        if !ours {
            match vouching::vouches(&at, reference, avowed) {
                Some(true) => {}
                Some(false) => return Sought::Torn,
                None => {
                    return match tisty_core::holes::a_hole(&at) {
                        true => Sought::Held,
                        false => Sought::Torn,
                    };
                }
            }
        }
        return Sought::At(at);
    }
    match shared {
        Some(dest) if !dest.is_dir() => Sought::Away,
        _ => Sought::No,
    }
}

#[cfg(test)]
#[path = "finding_test.rs"]
mod tests;
