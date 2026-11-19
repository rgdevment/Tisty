use std::collections::HashSet;

use super::*;

fn caps() -> Caps {
    Caps {
        common: 1,
        common_at_least: 100,
        steps: u64::MAX,
        ..CAPS
    }
}

fn counted(words: &[(&str, u32)]) -> Vec<(String, String, u32)> {
    words
        .iter()
        .map(|(word, count)| (word.to_string(), word.to_string(), *count))
        .collect()
}

fn item(words: &[&str]) -> Item {
    spoken(None, words)
}

fn spoken(lang: Option<&str>, words: &[&str]) -> Item {
    Item {
        lang: lang.map(str::to_string),
        kin: Vec::new(),
        words: counted(&words.iter().map(|one| (*one, 1)).collect::<Vec<_>>()),
    }
}

fn with_fillers(mut items: Vec<Item>, how_many: usize) -> Vec<Item> {
    for n in 0..how_many {
        items.push(item(&[&format!("filler{n}")]));
    }
    items
}

fn drawn(items: &[Item], caps: &Caps) -> Vec<(usize, usize)> {
    let mut all: Vec<(usize, usize)> = alike(items, &HashSet::new(), caps, Rule::Words)
        .pairs
        .iter()
        .map(|pair| (pair.a, pair.b))
        .collect();
    all.sort_unstable();
    all
}

#[test]
fn two_weighty_words_in_common_are_needed_and_one_is_not_enough() {
    let items = with_fillers(
        vec![
            item(&["budget", "family"]),
            item(&["budget", "family"]),
            item(&["budget", "router"]),
        ],
        6,
    );
    assert_eq!(drawn(&items, &caps()), [(0, 1)]);
}

#[test]
fn a_word_too_many_nodes_carry_weighs_nothing() {
    let common = ["notes", "week"];
    let mut items = vec![item(&common), item(&common)];
    for _ in 0..4 {
        items.push(item(&common));
    }
    let crowded = with_fillers(items, 4);
    let real = CAPS;
    assert!(
        drawn(&crowded, &real).is_empty(),
        "only common words in common"
    );

    let mut items = vec![
        item(&["notes", "week", "passport", "visa"]),
        item(&["notes", "week", "passport", "visa"]),
    ];
    for _ in 0..4 {
        items.push(item(&common));
    }
    assert_eq!(drawn(&with_fillers(items, 4), &real), [(0, 1)]);
}

#[test]
fn a_node_keeps_its_closest_three_at_most() {
    let hub: Vec<String> = (1..=5)
        .flat_map(|n| [format!("a{n}"), format!("b{n}")])
        .collect();
    let hub: Vec<&str> = hub.iter().map(String::as_str).collect();
    let mut items = vec![item(&hub)];
    for n in 1..=5 {
        items.push(item(&[&format!("a{n}"), &format!("b{n}")]));
    }
    let items = with_fillers(items, 6);
    assert_eq!(drawn(&items, &caps()), [(0, 1), (0, 2), (0, 3)]);
}

#[test]
fn a_rank_counts_from_both_ends() {
    let items = with_fillers(
        vec![
            item(&["ab1", "ab2"]),
            item(&["ab1", "ab2", "be1", "be2", "be3", "be4"]),
            item(&["be1", "be2", "be3", "be4", "ee1", "ee2", "ee3"]),
            item(&["ee1", "ee2", "ee3"]),
        ],
        8,
    );
    let one = Caps {
        closest: 1,
        ..caps()
    };
    assert_eq!(
        drawn(&items, &one),
        [(2, 3)],
        "the first is the second's closest, but the second has a closer one of its own"
    );
}

#[test]
fn a_pair_below_what_the_base_reaches_by_chance_stays_apart() {
    let cluster = ["k1", "k2", "k3"];
    let items = vec![
        item(&cluster),
        item(&cluster),
        item(&cluster),
        item(&cluster),
        item(&["w1", "w2", "u1", "u2", "u3", "u4", "u5", "u6"]),
        item(&["w1", "w2", "v1", "v2", "v3", "v4", "v5", "v6"]),
    ];
    let one = Caps {
        closest: 1,
        ..caps()
    };
    assert_eq!(drawn(&items, &one), [(0, 1)]);
}

#[test]
fn a_small_base_keeps_room_for_each_node_s_closest() {
    let items = with_fillers(
        vec![
            item(&["oven", "bread"]),
            item(&["oven", "bread"]),
            item(&["router", "wifi", "cable", "port"]),
            item(&["router", "wifi", "attic", "roof"]),
        ],
        4,
    );
    assert_eq!(drawn(&items, &caps()), [(0, 1), (2, 3)]);
}

#[test]
fn another_language_has_a_slot_of_its_own_and_takes_it_first() {
    let mut hub = vec!["a1", "b1", "a2", "b2", "a3", "b3", "x1", "x2"];
    hub.sort_unstable();
    let mut abroad = vec!["x1", "x2"];
    let english: Vec<String> = (0..10).map(|n| format!("en{n}")).collect();
    abroad.extend(english.iter().map(String::as_str));
    let items = with_fillers(
        vec![
            spoken(Some("es"), &hub),
            spoken(Some("es"), &["a1", "b1"]),
            spoken(Some("es"), &["a2", "b2"]),
            spoken(Some("es"), &["a3", "b3"]),
            spoken(Some("en"), &abroad),
        ],
        4,
    );
    assert_eq!(drawn(&items, &caps()), [(0, 1), (0, 2), (0, 4)]);
}

#[test]
fn what_says_no_language_meets_what_says_one_like_any_two_of_one_language() {
    let items = with_fillers(
        vec![
            item(&["passport", "visa", "embassy"]),
            spoken(Some("es"), &["passport", "visa", "consulate"]),
        ],
        4,
    );
    assert_eq!(drawn(&items, &caps()), [(0, 1)]);
}

#[test]
fn what_is_tied_or_kin_already_stays_apart() {
    let mut items = with_fillers(
        vec![
            item(&["oven", "bread"]),
            item(&["oven", "bread"]),
            item(&["router", "wifi"]),
            item(&["router", "wifi"]),
            item(&["passport", "visa"]),
            item(&["passport", "visa"]),
        ],
        4,
    );
    items[2].kin = vec!["book:trips".into()];
    items[3].kin = vec!["book:trips".into()];
    let tied: HashSet<(usize, usize)> = [(0, 1)].into();
    let found: Vec<(usize, usize)> = alike(&items, &tied, &caps(), Rule::Words)
        .pairs
        .iter()
        .map(|pair| (pair.a, pair.b))
        .collect();
    assert_eq!(found, [(4, 5)]);
}

#[test]
fn the_budget_bounds_every_pair_the_pass_compares() {
    let items = with_fillers(vec![item(&["oven", "bread"]), item(&["oven", "bread"])], 4);
    let within = Caps { steps: 8, ..caps() };
    let found = alike(&items, &HashSet::new(), &within, Rule::Words);
    assert_eq!(found.pairs.len(), 1);
    assert!(found.steps <= within.steps, "{} steps", found.steps);

    let short = Caps { steps: 7, ..caps() };
    let found = alike(&items, &HashSet::new(), &short, Rule::Words);
    assert!(found.pairs.is_empty());
    assert_eq!(found.common_from, 2, "no word is left to weigh");
    assert_eq!(found.steps, 0);
}

#[test]
fn the_why_names_the_heaviest_words_as_first_written() {
    let words = [
        ("metodo", "método", 5),
        ("bread", "bread", 3),
        ("oven", "oven", 1),
    ];
    let one = Item {
        lang: None,
        kin: Vec::new(),
        words: words
            .iter()
            .map(|(key, written, count)| (key.to_string(), written.to_string(), *count))
            .collect(),
    };
    let other = Item {
        lang: None,
        kin: Vec::new(),
        words: one.words.clone(),
    };
    let items = with_fillers(vec![one, other], 4);
    let told = Caps { told: 2, ..caps() };
    let found = alike(&items, &HashSet::new(), &told, Rule::Words);
    assert_eq!(found.pairs[0].shared, ["método", "bread"]);
}

#[test]
fn a_node_with_no_words_breaks_nothing() {
    let items = with_fillers(
        vec![
            item(&[]),
            item(&["oven", "bread"]),
            item(&["oven", "bread"]),
        ],
        4,
    );
    assert_eq!(drawn(&items, &caps()), [(1, 2)]);
    assert!(
        alike(&[], &HashSet::new(), &caps(), Rule::Words)
            .pairs
            .is_empty()
    );
}

fn written(words: &[(&str, &str)]) -> Item {
    Item {
        lang: None,
        kin: Vec::new(),
        words: words
            .iter()
            .map(|(key, said)| (key.to_string(), said.to_string(), 1))
            .collect(),
    }
}

fn found(items: &[Item], caps: &Caps, rule: Rule) -> Vec<(usize, usize, Vec<String>)> {
    alike(items, &HashSet::new(), caps, rule)
        .pairs
        .into_iter()
        .map(|pair| (pair.a, pair.b, pair.shared))
        .collect()
}

#[test]
fn a_track_needs_each_side_to_write_one_key_its_own_way() {
    let items = with_fillers(
        vec![
            written(&[("pay", "pagar"), ("water", "agua")]),
            written(&[("pay", "pay"), ("water", "water")]),
            written(&[("oven", "oven"), ("bread", "bread")]),
            written(&[("oven", "oven"), ("bread", "bread")]),
        ],
        6,
    );
    assert_eq!(drawn(&items, &caps()), [(0, 1), (2, 3)]);
    assert_eq!(
        found(&items, &caps(), Rule::Equivalent { loose: false }),
        [(
            0,
            1,
            vec!["pagar≈pay".to_string(), "agua≈water".to_string()]
        )]
    );
}

#[test]
fn a_common_key_reached_by_two_words_tells_only_when_loose() {
    let mut items = vec![
        written(&[("card", "tarjeta"), ("bank", "banco")]),
        written(&[("card", "card"), ("bank", "bank")]),
    ];
    for n in 0..3 {
        items.push(written(&[
            ("card", "card"),
            (&format!("errand{n}"), "errand"),
        ]));
    }
    let items = with_fillers(items, 25);
    let real = Caps {
        steps: u64::MAX,
        ..CAPS
    };
    assert!(found(&items, &real, Rule::Words).is_empty());
    assert!(found(&items, &real, Rule::Equivalent { loose: false }).is_empty());
    assert_eq!(
        found(&items, &real, Rule::Equivalent { loose: true }),
        [(
            0,
            1,
            vec!["banco≈bank".to_string(), "tarjeta≈card".to_string()]
        )]
    );
}

#[test]
fn a_track_drops_a_pair_whose_told_words_show_no_equivalence() {
    let heavy = |said: &str| Item {
        lang: None,
        kin: Vec::new(),
        words: vec![
            ("oven".to_string(), "oven".to_string(), 9),
            ("bread".to_string(), said.to_string(), 1),
        ],
    };
    let items = with_fillers(vec![heavy("pan"), heavy("bread")], 6);
    let one = Caps { told: 1, ..caps() };
    assert!(found(&items, &one, Rule::Equivalent { loose: false }).is_empty());
    let two = Caps { told: 2, ..caps() };
    assert_eq!(
        found(&items, &two, Rule::Equivalent { loose: false }),
        [(0, 1, vec!["oven".to_string(), "pan≈bread".to_string()])]
    );
}

#[test]
fn loose_keys_stay_under_a_quarter_of_the_base_and_within_the_budget() {
    let caps = Caps { steps: 30, ..CAPS };
    let df = [5, 5, 9, 2, 3];
    let weighty = [false, false, false, true, false];
    let crossed = [true, true, true, true, false];
    assert_eq!(
        loose(&df, &weighty, &crossed, 30, &caps),
        [true, false, false, false, false]
    );
}
