use std::collections::HashMap;

use super::{Drawing, draw, placed_near};
use crate::read;
use crate::rereading::tests::{PIECES, SAMPLES, Steps, boundary, plain};

fn alone(block: &crate::Block, text: &str) -> String {
    let drawing = Drawing {
        anchors: HashMap::new(),
        aligned: false,
        placed: true,
    };
    let mut out = String::new();
    draw(block, text, &drawing, &mut out);
    out
}

fn moved(html: &str, by: isize) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = ["data-at=\"", "data-src=\"", "data-placed=\""]
        .iter()
        .filter_map(|name| rest.find(name).map(|at| (at, name.len())))
        .min()
    {
        let (start, size) = at;
        out.push_str(&rest[..start + size]);
        rest = &rest[start + size..];
        let end = rest.find('"').unwrap_or(rest.len());
        let said = &rest[..end];
        match said
            .split_once('-')
            .and_then(|(a, b)| Some((a.parse::<isize>().ok()?, b.parse::<isize>().ok()?)))
        {
            Some((a, b)) => out.push_str(&format!("{}-{}", a + by, b + by)),
            None => out.push_str(said),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

// The page keeps a block the core draws beside a key when its place held and the key never reached it.
#[test]
fn a_block_beside_an_edit_whose_place_holds_is_drawn_as_before() {
    let texts = SAMPLES
        .iter()
        .flat_map(|one| [plain(one), (*one).to_string()]);
    for (seed, mut text) in texts.enumerate() {
        let mut steps = Steps(0x5EED_0000 + seed as u64);
        let mut checked = 0;
        for _ in 0..400 {
            let from = boundary(&text, steps.next(text.len() + 1));
            let (to, insert) = if steps.next(3) == 0 {
                (boundary(&text, (from + steps.next(12)).min(text.len())), "")
            } else {
                (from, PIECES[steps.next(PIECES.len())])
            };
            let next = format!("{}{insert}{}", &text[..from], &text[to..]);
            let was = read(&text);
            let now = read(&next);
            if was.labels() == now.labels() {
                let by = insert.len() as isize - (to - from) as isize;
                let near = placed_near(&now, from, from + insert.len(), None);
                for block in &now.blocks()[near.drawn.clone()] {
                    let at = &block.at;
                    let before = was
                        .blocks()
                        .iter()
                        .find(|one| one.at == *at && one.at.end < from);
                    let after = was.blocks().iter().find(|one| {
                        one.at.start > to
                            && one.at.start as isize + by == at.start as isize
                            && one.at.end as isize + by == at.end as isize
                    });
                    let (old, shift) = match (before, after) {
                        (Some(one), _) => (one, 0),
                        (None, Some(one)) => (one, by),
                        (None, None) => continue,
                    };
                    checked += 1;
                    assert_eq!(
                        alone(block, &next),
                        moved(&alone(old, &text), shift),
                        "after {from}..{to} became {insert:?} in sample {seed}"
                    );
                }
            }
            text = next;
        }
        assert!(
            checked > 100,
            "only {checked} blocks beside an edit were compared in sample {seed}"
        );
    }
}
