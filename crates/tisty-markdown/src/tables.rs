use crate::{Block, Kind};

const STEP: u32 = 10;
const FEWEST: u32 = 4;

/// Tisty keeps a column's width in its rule: four dashes or more, ten pixels each.
pub fn widen(block: &mut Block, text: &str) {
    let Kind::Table { columns } = &mut block.kind else {
        return;
    };
    let Some(rule) = text[block.at.clone()].lines().nth(1) else {
        return;
    };
    let cells = rule.trim().trim_start_matches('|').trim_end_matches('|');
    for (column, cell) in columns.iter_mut().zip(cells.split('|')) {
        let dashes = cell.chars().filter(|c| *c == '-').count();
        let dashes = u32::try_from(dashes).unwrap_or(u32::MAX);
        column.wide = (dashes >= FEWEST).then(|| dashes.saturating_mul(STEP));
    }
}
