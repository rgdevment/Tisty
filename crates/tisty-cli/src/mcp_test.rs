use super::unpathing::{absolute, unpathed, unpathed_body};
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
        for top in ["anyOf", "oneOf", "allOf", "not", "if", "then", "else"] {
            assert!(
                shape.get(top).is_none(),
                "{name} says `{top}` at the top of its schema, which a client that hands the schema straight to the model refuses"
            );
        }
        assert!(
            !names_a_key(shape, "oneOf"),
            "{name} uses `oneOf`, which some clients do not take; `anyOf` says the same"
        );
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
        if let Some(asked) = shape.get("required").and_then(Value::as_array) {
            for want in asked {
                let want = want.as_str().unwrap_or_default();
                assert!(
                    fields.contains_key(want),
                    "{name} asks for `{want}`, which it does not take"
                );
            }
        }
    }
}

fn names_a_key(shape: &Value, key: &str) -> bool {
    match shape {
        Value::Object(all) => all
            .iter()
            .any(|(named, inside)| named == key || names_a_key(inside, key)),
        Value::Array(all) => all.iter().any(|one| names_a_key(one, key)),
        _ => false,
    }
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
    assert_eq!(absolute("2 GB / 90 días"), None);
    assert_eq!(absolute("uno / otro"), None);
}

#[test]
fn only_the_path_is_hidden_and_the_rest_of_the_line_is_read() {
    assert_eq!(
        unpathed(r"De paso: C:\Users\Mario\OneDrive\Tisty guarda otro almacén viejo."),
        "De paso: [path] guarda otro almacén viejo."
    );
    assert_eq!(
        unpathed("kept at C:/Users/someone/Downloads/x.csv and opened twice"),
        "kept at [path] and opened twice"
    );
    assert_eq!(
        unpathed(r"from G:\Mi unidad\Tisty to /home/one/tisty, both"),
        "from [path] to [path], both"
    );
    assert_eq!(
        unpathed("arrancando en **2 GB / 90 días**. Eso implica más"),
        "arrancando en **2 GB / 90 días**. Eso implica más"
    );
    assert_eq!(unpathed("https://ejemplo.com/a"), "https://ejemplo.com/a");
}

#[test]
fn a_file_name_closes_the_path_and_what_follows_is_read() {
    assert_eq!(
        unpathed("kept at C:/a/b.csv and moved to x/y later"),
        "kept at [path] and moved to x/y later"
    );
    assert_eq!(
        unpathed(r"from C:\temp\a.txt and/or the other"),
        "from [path] and/or the other"
    );
}

#[test]
fn a_path_with_spaces_is_hidden_whole_without_quotes() {
    assert_eq!(
        unpathed(r"guardado desde C:\Users\Mario Hidalgo\OneDrive - Personal\contrato.pdf"),
        "guardado desde [path]"
    );
    assert_eq!(
        unpathed("kept from /Users/x/Library/Mobile Documents/com~apple/notes.pdf"),
        "kept from [path]"
    );
    assert_eq!(
        unpathed(r#"run "C:\Program Files\Tisty\tisty.exe" twice"#),
        r#"run "[path]" twice"#
    );
    assert_eq!(
        unpathed(r"seen in C:\temp\one dir\x.md, then **2 GB / 90 días** stayed"),
        "seen in [path], then **2 GB / 90 días** stayed"
    );
    assert_eq!(
        unpathed("kept at C:/a/b.csv and read at https://ejemplo.com/x later"),
        "kept at [path] and read at https://ejemplo.com/x later"
    );
}

#[test]
fn a_word_with_one_slash_is_a_command_or_a_switch_and_the_prose_after_it_stays() {
    assert_eq!(absolute("lee /code-review y sigue"), None);
    assert_eq!(absolute("con RMDir /r y /REBOOTOK"), None);
    let note =
        "Corrí /code-review tres veces: encontró huecos reales y los corrigió, sin tocar old/new";
    assert_eq!(unpathed(note), note);
    let switches = "RMDir /r, dos espacios, /REBOOTOK /r y guardar la ruta pasaban en verde";
    assert_eq!(unpathed(switches), switches);
}

#[test]
fn a_path_ends_where_its_word_ends_when_the_next_separator_is_a_sentence_away() {
    assert_eq!(
        unpathed(
            "pasa /dev/null y así no recibe las refs; las firmas van en rgdevment/cla-signatures"
        ),
        "pasa [path] y así no recibe las refs; las firmas van en rgdevment/cla-signatures"
    );
    assert_eq!(
        unpathed(r"en C:\temp pero mucho más tarde quedó otra\cosa"),
        r"en [path] pero mucho más tarde quedó otra\cosa"
    );
    assert_eq!(
        unpathed("git -C /d/code/github/Tisty push origin main y luego el tag"),
        "git -C [path] push origin main y luego el tag"
    );
    assert_eq!(
        unpathed("(desde C:/a/b, que es donde la persona guarda todo)"),
        "(desde [path], que es donde la persona guarda todo)"
    );
}

#[test]
fn what_sits_between_backticks_is_an_example_and_is_read_as_written() {
    let examples = [
        r"El prefijo `\\?\` delante de `C:\Users\x` es lo que se explica aquí.",
        "`pre-push` debe llamar a `rules.sh < /dev/null`; las firmas van en `rgdevment/cla-signatures`.",
        r#"Emite `RMDir /r "$INSTDIR"` en el desinstalador, y `git -C /d/code/x push origin *` después."#,
        "Dos tildes: ``C:/a/b `con una` dentro`` y nada más.",
    ];
    for line in examples {
        assert_eq!(unpathed(line), line);
    }
    assert_eq!(
        unpathed(r"`C:\ejemplo` aparte de C:\Users\x\a.txt"),
        r"`C:\ejemplo` aparte de [path]"
    );
    assert_eq!(unpathed("un ` suelto y C:/a/b.md"), "un ` suelto y [path]");
}

#[test]
fn a_fenced_block_is_read_as_written_and_the_lines_around_it_are_not() {
    let body = [
        r"visto en C:\temp\x.md",
        "```sh",
        "git -C /d/code/github/Tisty push",
        r"C:\Users\x",
        "```",
        "~~~~",
        "C:/a/b",
        "~~~",
        "C:/a/b",
        "~~~~",
        "y C:/c/d.txt",
    ]
    .join("\n");
    let kept: Vec<String> = body.lines().map(str::to_string).collect();
    let read = unpathed_body(&body);
    let read: Vec<&str> = read.lines().collect();
    assert_eq!(read[0], "visto en [path]");
    assert_eq!(read[1..10], kept[1..10]);
    assert_eq!(read[10], "y [path]");
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

#[test]
fn a_store_left_behind_by_a_move_asks_for_a_restart_not_an_update() {
    let Refused::Tool(said) = hitch(tisty_core::Error::UnsupportedVersion {
        version: tisty_core::moving::FENCE_VERSION,
        device: "dev_a".into(),
    }) else {
        panic!("not a tool refusal");
    };
    assert!(said.contains("restart"), "{said}");
    assert!(!said.contains("update Tisty"), "{said}");
}
