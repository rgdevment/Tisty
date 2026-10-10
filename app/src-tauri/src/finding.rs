use std::sync::Mutex;

use crate::{Answer, Refusal, Session, held, vouching};

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
}

/// Taken and let go of at once: what follows can wait on a cloud, and holding the session while it
/// does would freeze the window.
pub fn where_to(session: &tauri::State<'_, Mutex<Session>>, reference: &str) -> Where {
    let session = held(session);
    Where {
        data: session.paths.data().to_path_buf(),
        avowed: session
            .state
            .kept
            .get(reference)
            .map(|(sha256, _)| sha256.clone()),
        shared: session.shared_now(),
        reached: session.paths.cache().join(tisty_core::lately::USED),
    }
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
        other => Err(unreachable(other, reference.to_string())),
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
