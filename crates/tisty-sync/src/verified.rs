use std::path::Path;

use tisty_core::answering::Reached;

const KEPT: &str = ".verified-to";
const KEPT_FOR: usize = 64;

pub(crate) fn of(data: &Path, dest: &Path, device: &str) -> Reached {
    let looking = named(dest, device);
    lines(data)
        .into_iter()
        .find_map(|line| {
            let (whose, said) = line.split_once('\t')?;
            (whose == looking).then(|| Reached::read(said))?
        })
        .unwrap_or_default()
}

pub(crate) fn keep(data: &Path, dest: &Path, device: &str, held: Reached) {
    if of(data, dest, device) == held {
        return;
    }
    let looking = named(dest, device);
    let mut kept = vec![format!("{looking}\t{}", held.said())];
    kept.extend(
        lines(data)
            .into_iter()
            .filter(|line| {
                line.split_once('\t')
                    .is_some_and(|(whose, _)| whose != looking)
            })
            .take(KEPT_FOR),
    );
    let _ = tisty_core::store::write_atomic(&data.join(KEPT), kept.join("\n").as_bytes());
}

/// Against the folder it was read in as well as the machine: what one folder answered for says
/// nothing about another, and a memo that travelled between them would pass over the difference.
fn named(dest: &Path, device: &str) -> String {
    let at = std::fs::canonicalize(dest)
        .unwrap_or_else(|_| dest.to_path_buf())
        .display()
        .to_string();
    format!("{device} {}", at.replace(['\n', '\t'], " "))
}

fn lines(data: &Path) -> Vec<String> {
    std::fs::read_to_string(data.join(KEPT))
        .map(|said| said.lines().map(str::to_string).collect())
        .unwrap_or_default()
}
