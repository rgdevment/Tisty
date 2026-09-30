use std::path::Path;

use tisty_core::answering::Reached;

const KEPT: &str = ".verified-to";
const KEPT_FOR: usize = 64;

pub(crate) fn of(data: &Path, device: &str) -> Reached {
    lines(data)
        .into_iter()
        .find_map(|line| {
            let (whose, said) = line.split_once('\t')?;
            (whose == device).then(|| Reached::read(said))?
        })
        .unwrap_or_default()
}

pub(crate) fn keep(data: &Path, device: &str, held: Reached) {
    if of(data, device) == held {
        return;
    }
    let mut kept = vec![format!("{device}\t{}", held.said())];
    kept.extend(
        lines(data)
            .into_iter()
            .filter(|line| {
                line.split_once('\t')
                    .is_some_and(|(whose, _)| whose != device)
            })
            .take(KEPT_FOR),
    );
    let _ = tisty_core::store::write_atomic(&data.join(KEPT), kept.join("\n").as_bytes());
}

fn lines(data: &Path) -> Vec<String> {
    std::fs::read_to_string(data.join(KEPT))
        .map(|said| said.lines().map(str::to_string).collect())
        .unwrap_or_default()
}
