use serde_json::Value;
use tisty_contracts::Editor;

use super::{holds, with_mine};

const VECTORS: &str = include_str!("../../../contracts/vectors/vectors.json");

fn said() -> Vec<(String, bool, Editor)> {
    let all: Value = serde_json::from_str(VECTORS).unwrap();
    all["editors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|one| {
            let entry = serde_json::from_value(one["entry"].clone()).ok()?;
            Some((
                one["says"].as_str()?.to_owned(),
                one["holds"].as_bool()?,
                entry,
            ))
        })
        .collect()
}

#[test]
fn an_editor_holds_only_as_it_was_signed() {
    for (says, held, entry) in said() {
        assert_eq!(holds(&entry), held, "{says}");
    }
}

#[test]
fn a_version_keeps_who_holds_in_their_order_and_drops_who_does_not() {
    let all = said();
    let arrived: Vec<Editor> = all.iter().map(|(_, _, one)| one.clone()).collect();
    let kept = with_mine(arrived, None);
    let expected: Vec<Editor> = all
        .into_iter()
        .filter(|(_, held, _)| *held)
        .map(|(_, _, one)| one)
        .collect();
    assert_eq!(kept, expected);
}

#[test]
fn whoever_signs_again_keeps_their_place_and_a_newcomer_goes_last() {
    let all = said();
    let mario = all[0].2.clone();
    let ana = all[1].2.clone();
    let renamed = Editor {
        name: "Ana Pérez".to_owned(),
        ..ana.clone()
    };
    assert_eq!(
        with_mine(vec![mario.clone(), ana.clone()], Some(renamed.clone())),
        vec![mario.clone(), renamed]
    );

    let in_the_original = all[2].2.clone();
    assert_eq!(
        with_mine(
            vec![mario.clone(), ana.clone()],
            Some(in_the_original.clone())
        ),
        vec![mario, ana, in_the_original]
    );
}
