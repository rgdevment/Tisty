use super::*;

fn said(all: &[Decision]) -> Vec<(usize, &str, bool)> {
    all.iter()
        .map(|one| (one.line, one.said.as_str(), one.replaced))
        .collect()
}

#[test]
fn a_decisions_section_names_each_item_and_a_struck_one_is_replaced() {
    let body = [
        "# Trips",
        "",
        "Intro line.",
        "",
        "## Decisiones",
        "",
        "- **2026-11-13 · The car stays home.** We walk.",
        "- ~~2026-11-12 · The car goes.~~ Changed the next day.",
        "  - a detail under it",
        "1. The vet on Saturdays. Always at ten.",
        "- [ ] a step, not a decision",
        "",
        "## Packing",
        "",
        "- Boots",
    ]
    .join("\n");
    let body = body.as_str();

    assert_eq!(
        said(&decisions(body)),
        vec![
            (7, "2026-11-13 · The car stays home.", false),
            (8, "2026-11-12 · The car goes.", true),
            (10, "The vet on Saturdays.", false),
        ]
    );
}

#[test]
fn a_document_titled_for_decisions_holds_them_in_every_section() {
    let body = "# Brainsty: decisions\n\n## The base\n\n- **Local only.** Never synced.\n\n\
                ## Revised\n\n- **~~Grafo descartado.~~** Fell later.\n";

    assert_eq!(
        said(&decisions(body)),
        vec![(5, "Local only.", false), (9, "Grafo descartado.", true)]
    );
}

#[test]
fn lo_decidido_and_decided_name_decisions_too() {
    let body = "# Plan\n\n### Lo decidido\n\n1. **Ask by default.** On.\n\n## What we decided\n\n- Keep it.\n";

    assert_eq!(
        said(&decisions(body)),
        vec![(5, "Ask by default.", false), (9, "Keep it.", false)]
    );
}

#[test]
fn items_in_code_or_outside_a_decisions_section_are_not_decisions() {
    let body = "# Notes\n\n- not one\n\n## Decisions\n\n```\n- in code\n```\n";

    assert!(decisions(body).is_empty());
}

#[test]
fn a_link_in_a_decision_is_read_as_its_words() {
    let body = "# Decisions\n\n- Ask [the bank](tisty:task/01M7NC27JDW2R8VEASK8NYP628) first, as ![Trips](tisty:doc/trips-0001) says.\n";

    assert_eq!(
        said(&decisions(body)),
        vec![(3, "Ask the bank first, as Trips says.", false)]
    );
}

#[test]
fn a_long_decision_is_cut() {
    let long = "word ".repeat(40);
    let body = format!("# Decisions\n\n- **{long}**\n");

    let found = decisions(&body);

    assert_eq!(found.len(), 1);
    assert!(found[0].said.chars().count() <= SAID_AT_MOST + 1);
    assert!(found[0].said.ends_with('…'));
}
