use std::sync::Mutex;

use crate::{Refusal, Session, held, vouching};

/// Long enough for a file already on its way, short enough that nobody thinks the app hung.
const COMES_WITHIN: std::time::Duration = std::time::Duration::from_millis(1_500);

pub fn unreachable(found: Sought, reference: String) -> Refusal {
    match found {
        Sought::Coming => Refusal::about("comingDown", reference),
        Sought::Away => Refusal::of("sharedAway"),
        Sought::Torn => Refusal::about("attachmentTorn", reference),
        _ => Refusal::about("cannotRead", reference),
    }
}

pub enum Sought {
    At(std::path::PathBuf),
    Coming,
    Away,
    /// It is there, and it is not what its name says it is.
    Torn,
    No,
}

/// A link or a junction under somebody else's folder can point anywhere; the store's tree is ours.
pub fn under_root(at: &std::path::Path, root: &std::path::Path) -> bool {
    match (at.canonicalize(), root.canonicalize()) {
        (Ok(at), Ok(root)) => at.starts_with(root),
        _ => false,
    }
}

/// Where to look, taken and let go of at once: what follows can wait on iCloud, and holding the
/// session while it does would freeze the window.
pub fn where_to(
    session: &tauri::State<'_, Mutex<Session>>,
) -> (std::path::PathBuf, Option<std::path::PathBuf>) {
    let session = held(session);
    (session.paths.data().to_path_buf(), session.shared_now())
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

pub fn found_in(
    reference: &str,
    data: &std::path::Path,
    shared: Option<&std::path::Path>,
) -> Sought {
    for root in [Some(data), shared].into_iter().flatten() {
        let Ok(at) = tisty_core::attach::resolve(reference, root) else {
            continue;
        };
        let ours = root == data;
        if at.is_file() {
            if !ours && !under_root(&at, root) {
                return Sought::No;
            }
            if !ours && !vouching::vouches(&at, reference) {
                return Sought::Torn;
            }
            return Sought::At(at);
        }
        if tisty_core::icloud::shed(&at).is_some() {
            if !tisty_core::icloud::can_ask() {
                return Sought::Away;
            }
            if !tisty_core::icloud::waited_for(&at, COMES_WITHIN) {
                return Sought::Coming;
            }
            // What comes back from a cloud answers for its name like anything else that lives there.
            return match ours || (under_root(&at, root) && vouching::vouches(&at, reference)) {
                true => Sought::At(at),
                false => Sought::Torn,
            };
        }
    }
    match shared {
        Some(dest) if !dest.is_dir() => Sought::Away,
        _ => Sought::No,
    }
}
