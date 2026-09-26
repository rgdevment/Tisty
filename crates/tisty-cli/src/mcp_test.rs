use super::*;

fn four_ids() -> Vec<String> {
    (1..=4).map(|n| format!("wwwwwwww-000{n}")).collect()
}

fn a_shape() -> impl proptest::strategy::Strategy<Value = (Vec<usize>, Vec<String>)> {
    (
        proptest::sample::subsequence(vec![0usize, 1, 2, 3], 2..=4),
        proptest::collection::vec("[a-z][a-z ]{0,18}", 0..5),
    )
}

fn built(cards: &[usize], prose: &[String], ids: &[String]) -> String {
    let mut out = String::from("# Titulo\n");
    for (at, one) in cards.iter().enumerate() {
        if let Some(said) = prose.get(at) {
            out.push('\n');
            out.push_str(said.trim());
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&tisty_core::refs::card(&ids[*one], "Pagina"));
        out.push('\n');
    }
    for said in prose.iter().skip(cards.len()) {
        out.push('\n');
        out.push_str(said.trim());
        out.push('\n');
    }
    out
}

fn named_times(body: &str, id: &str) -> usize {
    body.lines()
        .filter(|one| tisty_core::refs::papers(one).iter().any(|said| said == id))
        .count()
}

proptest::proptest! {
    #[test]
    fn moving_a_page_keeps_every_word_that_was_not_its_line(
        (cards, prose) in a_shape(),
        pick in 0usize..4,
        mover in 0usize..4,
        before in proptest::bool::ANY,
    ) {
        let ids = four_ids();
        let anchor = cards[pick % cards.len()];
        proptest::prop_assume!(mover != anchor);
        let body = built(&cards, &prose, &ids);

        let lines: Vec<String> = body.lines().map(str::to_string).collect();
        let sits = line_of(&lines, &ids[anchor]).unwrap();
        let spot = match before && sits > 0 {
            true => Spot::Before(&ids[anchor]),
            false => Spot::After(&ids[anchor]),
        };
        let out = card_moved(&body, &ids[mover], "Pagina", spot).unwrap();

        for one in body.lines() {
            if tisty_core::refs::papers(one).is_empty() && !one.trim().is_empty() {
                proptest::prop_assert!(
                    out.lines().any(|now| now == one),
                    "a line of prose was lost: {one:?}\nfrom:\n{body}\nto:\n{out}"
                );
            }
        }
        for (at, id) in ids.iter().enumerate() {
            let want = match at == mover {
                true => 1,
                false => named_times(&body, id),
            };
            proptest::prop_assert_eq!(
                named_times(&out, id), want,
                "{} is named the wrong number of times\nfrom:\n{}\nto:\n{}",
                id, body, out
            );
        }
    }

    #[test]
    fn a_move_never_turns_one_kind_of_line_ending_into_another(
        (cards, prose) in a_shape(),
        pick in 0usize..4,
        mover in 0usize..4,
    ) {
        let ids = four_ids();
        let anchor = cards[pick % cards.len()];
        proptest::prop_assume!(mover != anchor);
        let body = built(&cards, &prose, &ids).replace('\n', "\r\n");

        let out = card_moved(&body, &ids[mover], "Pagina", Spot::After(&ids[anchor])).unwrap();

        proptest::prop_assert!(
            !out.lines().any(|one| one.ends_with('\r')),
            "a stray carriage return was left inside a line: {out:?}"
        );
        proptest::prop_assert_eq!(
            out.matches("\r\n").count(),
            out.lines().count(),
            "the file changed how its lines end:\n{:?}",
            out
        );
    }
}

#[test]
fn a_line_that_says_anything_besides_one_page_is_not_a_card_on_its_own() {
    let id = "wwwwwwww-0001";
    let card = tisty_core::refs::card(id, "Uno");
    assert!(card_alone(&card, id));
    assert!(card_alone(&format!("  {card}  "), id));
    assert!(!card_alone(&format!("La puerta esta en {card}"), id));
    assert!(!card_alone(&format!("{card} y mas"), id));
    assert!(!card_alone(
        &format!("{card} {}", tisty_core::refs::card("wwwwwwww-0002", "Dos")),
        id
    ));
    assert!(!card_alone(&format!("- {card}"), id));
    assert!(!card_alone(&format!("| {card} | ok |"), id));
    assert!(!card_alone(&card, "wwwwwwww-0002"));
}

#[test]
fn a_page_is_found_however_markdown_spells_the_link() {
    let id = "wwwwwwww-0001";
    for said in [
        format!("![Uno](tisty:doc/{id})"),
        format!("![Uno](<tisty:doc/{id}>)"),
        format!("![Uno](tisty:doc/{id} \"Uno\")"),
    ] {
        let lines = vec![String::from("# T"), String::new(), said.clone()];
        assert_eq!(line_of(&lines, id), Some(2), "not found: {said}");
    }
    let inside = vec![String::from("el id es `tisty:doc/wwwwwwww-0001)`")];
    assert_eq!(
        line_of(&inside, id),
        None,
        "a mention inside code is not a line"
    );
}

#[test]
fn every_tool_says_what_it_takes_and_asks_only_for_what_it_declares() {
    let all = tools();
    let all = all.as_array().expect("the tools come back as a list");
    assert!(!all.is_empty());
    for one in all {
        let name = one["name"].as_str().expect("a tool has a name");
        for key in ["title", "description"] {
            let said = one[key].as_str().unwrap_or_default();
            assert!(!said.trim().is_empty(), "{name} has no {key}");
        }
        let shape = &one["inputSchema"];
        assert_eq!(shape["type"], json!("object"), "{name} takes an object");
        let none = serde_json::Map::new();
        let fields = shape["properties"].as_object().unwrap_or(&none);
        for (field, said) in fields {
            assert!(
                said["description"]
                    .as_str()
                    .is_some_and(|one| !one.trim().is_empty()),
                "{name}.{field} has no description"
            );
            if said.get("enum").is_some() {
                assert!(
                    said.get("type").is_some(),
                    "{name}.{field} lists what it takes but not its type"
                );
            }
        }
        let asked: Vec<&Value> = shape
            .get("required")
            .into_iter()
            .chain(
                shape
                    .get("anyOf")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|one| one.get("required")),
            )
            .collect();
        for group in asked {
            for want in group.as_array().unwrap_or(&Vec::new()) {
                let want = want.as_str().unwrap_or_default();
                assert!(
                    fields.contains_key(want),
                    "{name} asks for `{want}`, which it does not take"
                );
            }
        }
    }
}

#[test]
fn an_assistant_is_let_in_only_by_somebody_at_a_terminal() {
    assert_eq!(let_in(true, true), Door::Asks);
    assert_eq!(let_in(true, false), Door::NoTerminal);
}

#[test]
fn a_store_the_person_did_not_choose_asks_nobody() {
    assert_eq!(let_in(false, false), Door::Open);
    assert_eq!(let_in(false, true), Door::Open);
}

#[test]
fn a_notice_goes_after_the_heading_and_otherwise_after_the_block_that_opens_the_body() {
    assert_eq!(room_for_a_notice("# Titulo\n\ncuerpo\n"), 9);
    assert_eq!(room_for_a_notice("# Titulo\nprosa\n"), 9);
    assert_eq!(room_for_a_notice("\n\n# Titulo\n"), 11);
    assert_eq!(room_for_a_notice("prosa\notra\n\nmas\n"), 11);
}

#[test]
fn three_prose_lines_of_a_hand_wrapped_width_are_seen_as_wrapped() {
    let line = "a".repeat(60);
    assert!(looks_wrapped(&format!("{line}\n{line}\n{line}\n")));
    assert!(!looks_wrapped(&format!("{line}\n{line}\n")));
}

#[test]
fn the_same_lines_inside_a_fence_are_code_and_not_wrapped_prose() {
    let line = "a".repeat(60);
    assert!(!looks_wrapped(&format!(
        "```\n{line}\n{line}\n{line}\n```\n"
    )));
}

#[test]
fn a_list_an_indent_and_a_number_are_not_prose_however_wide_the_line_is() {
    let tail = "a".repeat(58);
    assert!(!looks_wrapped(&format!("- {tail}\n- {tail}\n- {tail}\n")));
    assert!(!looks_wrapped(&format!("1 {tail}\n1 {tail}\n1 {tail}\n")));
    let wide = "a".repeat(60);
    assert!(!looks_wrapped(&format!(
        "    {wide}\n    {wide}\n    {wide}\n"
    )));
}

#[test]
fn a_page_stops_on_the_line_that_would_overrun_the_room_it_was_given() {
    let body = "aaa\nbbb\nccc\n";
    assert_eq!(as_far_as(body, 1, 7, 10), 1);
    assert_eq!(as_far_as(body, 1, 8, 10), 2);
    assert_eq!(as_far_as(body, 1, 9, 10), 2);
    assert_eq!(as_far_as(body, 1, 100, 10), 3);
    assert_eq!(as_far_as(body, 1, 100, 2), 2);
}

#[test]
fn a_path_is_found_by_its_drive_or_by_the_slash_that_opens_it() {
    assert_eq!(absolute("mira C:/Users/x"), Some(5));
    assert_eq!(absolute(r"mira C:\Users\x"), Some(5));
    assert_eq!(absolute("mira C:/"), Some(5));
    assert_eq!(absolute("lee /etc/hosts"), Some(4));
    assert_eq!(absolute("https://ejemplo.com"), None);
    assert_eq!(absolute("/etc/hosts"), None);
    assert_eq!(absolute("ver C:"), None);
}

fn kept(body: &str) -> String {
    retargeted(body, &mut |_, target, title| {
        Some((target.to_string(), title.to_string()))
    })
}

#[test]
fn a_link_is_rewritten_and_what_surrounds_it_is_left_alone() {
    let out = retargeted("ver [doc](tisty:doc/1) ahora", &mut |_, target, title| {
        Some((format!("nuevo:{target}"), title.to_string()))
    });
    assert_eq!(out, "ver [doc](<nuevo:tisty:doc/1>) ahora");
}

#[test]
fn a_body_with_nothing_to_point_at_comes_back_as_it_went_in() {
    let body = "nada que reescribir (ni esto) [ni esto";
    assert_eq!(kept(body), body);
    assert_eq!(kept("texto ](y) mas"), "texto ](y) mas");
}

#[test]
fn a_link_whose_parenthesis_never_closes_is_left_as_written() {
    let body = "esto [x](sin cerrar";
    assert_eq!(kept(body), body);
}

#[test]
fn a_target_carrying_its_own_parentheses_is_read_whole() {
    assert_eq!(kept("[x](a(b)c)"), "[x](<a(b)c>)");
}

#[test]
fn a_picture_keeps_its_mark_and_a_refused_link_keeps_only_its_words() {
    assert_eq!(kept("mira ![foto](a.png)"), "mira ![foto](<a.png>)");
    assert_eq!(retargeted("mira [doc](x)", &mut |_, _, _| None), "mira doc");
    assert_eq!(
        retargeted("mira ![foto](a.png)", &mut |_, _, _| None),
        "mira foto",
        "el signo de la imagen se quedo sin nada que marcar"
    );
    assert_eq!(kept("a[!x](y)"), "a[!x](<y>)");
}

#[test]
fn a_link_written_inside_a_fence_is_code_and_is_not_pointed_anywhere_else() {
    let body = "```\n[x](y)\n```\n";
    assert_eq!(kept(body), body);
}

#[test]
fn a_write_says_which_way_the_document_moved_and_by_how_much() {
    assert_eq!(by_how_much("abc", "abc"), "the same length");
    assert_eq!(by_how_much("abc", "abcde"), "2 characters longer");
    assert_eq!(by_how_much("abcde", "abc"), "2 characters shorter");
}

#[test]
fn the_nearest_line_is_the_one_that_shares_the_most_with_what_was_asked_for() {
    let body = "abcdXY\nabcdef\n";
    assert_eq!(nearest(body, "abcdef"), Some((2, "abcdef".to_string())));
    assert_eq!(nearest(body, "abcd"), Some((1, "abcdXY".to_string())));
    assert_eq!(nearest(body, "abc"), None);
    assert_eq!(nearest("uno\ndos\n", "abcdef"), None);
}

#[test]
fn an_escape_is_read_only_where_two_digits_follow_it() {
    assert_eq!(unescaped("%41BC"), "ABC");
    assert_eq!(unescaped("a%41"), "aA");
    assert_eq!(unescaped("a%4"), "a%4");
    assert_eq!(unescaped("a%zz"), "a%zz");
}

fn tramo(body: &str, args: Value) -> (usize, usize, Option<usize>) {
    match part_asked(body, &args) {
        Ok(Part::Held { from, to, next, .. }) => (from, to, next),
        _ => panic!("se esperaba un tramo de {args}"),
    }
}

#[test]
fn a_run_that_fits_says_nothing_about_carrying_on_and_one_that_does_not_says_where() {
    let short = "uno\ndos\ntres\n";
    assert_eq!(tramo(short, json!({"from": 1, "to": 3})), (1, 3, None));

    let long = "0123456789012345678901234567890123456789\n".repeat(400);
    let (from, to, next) = tramo(&long, json!({"from": 1, "to": 400}));
    assert_eq!(from, 1);
    assert!(to < 400, "el presupuesto no recorto nada");
    assert_eq!(next, Some(to + 1));
}

#[test]
fn a_run_named_by_one_end_alone_is_still_a_run() {
    let short = "uno\ndos\ntres\n";
    assert_eq!(tramo(short, json!({"from": 2})), (2, 3, None));
    assert_eq!(tramo(short, json!({"to": 2})), (1, 2, None));
}

#[test]
fn a_run_that_ends_before_it_starts_is_refused_and_one_line_alone_is_not() {
    let short = "uno\ndos\ntres\n";
    assert!(part_asked(short, &json!({"from": 3, "to": 2})).is_err());
    assert_eq!(tramo(short, json!({"from": 2, "to": 2})), (2, 2, None));
}

#[test]
fn a_budget_of_characters_says_where_to_carry_on_and_stops_saying_it_at_the_end() {
    let long = "0123456789\n".repeat(50);
    let (from, to, next) = tramo(&long, json!({"chars": 30}));
    assert_eq!(from, 1);
    assert!(to < 50);
    assert_eq!(next, Some(to + 1));
    assert_eq!(tramo(&long, json!({"chars": 100_000})), (1, 50, None));
}

#[test]
fn a_section_is_handed_over_whole_or_with_the_line_it_was_cut_at() {
    let short = "# Uno\ntexto\n## Dos\notro\n";
    assert_eq!(tramo(short, json!({"section": 0})), (1, 4, None));
    assert_eq!(tramo(short, json!({"section": 1})), (3, 4, None));
    assert!(part_asked(short, &json!({"section": 9})).is_err());

    let long = format!(
        "# Uno\n{}",
        "0123456789012345678901234567890123456789\n".repeat(400)
    );
    let (from, to, next) = tramo(&long, json!({"section": 0}));
    assert_eq!(from, 1);
    assert!(to < 401, "el presupuesto no recorto la seccion");
    assert_eq!(next, Some(to + 1));
}

#[test]
fn a_document_right_at_the_budget_is_still_handed_over_whole() {
    let body = "a".repeat(WHOLE_UP_TO);
    assert!(matches!(
        part_asked(&body, &json!({})),
        Ok(Part::Held { .. })
    ));
    let over = "a".repeat(WHOLE_UP_TO + 1);
    assert!(matches!(part_asked(&over, &json!({})), Ok(Part::Outline)));
}
