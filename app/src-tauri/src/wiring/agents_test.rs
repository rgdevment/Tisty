use super::gated::{ME, consented, gated, gated_in};
use super::*;
use tisty_core::agents::{doc_of, found};
use tisty_core::event::DeviceId;

fn put(at: &Path, said: &str) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, said).unwrap();
}

fn card(level: &str, kind: Kind, name: &str) -> Card {
    Card::of(&doc_of(level, kind, name, b"x\n").unwrap()).unwrap()
}

fn seeded() -> (tempfile::TempDir, Homes) {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    put(&home.join(".claude/CLAUDE.md"), "# Claude\n");
    put(&home.join(".claude/rules/CLAUDE.md"), "Not the main one.\n");
    put(&home.join(".claude/rules/style/tone.md"), "Short.\n");
    put(
        &home.join(".claude/skills/pr/SKILL.md"),
        "---\nname: pr\n---\n",
    );
    put(
        &home.join(".claude/projects/repo/memory/MEMORY.md"),
        "- a fact\n",
    );
    put(&home.join(".claude/projects/repo/1234.jsonl"), "{}\n");
    put(&home.join(".claude/settings.json"), "{}\n");
    put(&home.join(".codex/AGENTS.md"), "Tests first.\n");
    put(&home.join(".agents/skills/lint/SKILL.md"), "lint\n");
    put(
        &home.join(".copilot/copilot-instructions.md"),
        "Be brief.\n",
    );
    put(
        &home.join(".copilot/instructions/rust.instructions.md"),
        "Clippy.\n",
    );
    put(&home.join(".gemini/GEMINI.md"), "Remembered.\n");
    let homes = Homes::with(home, &[]);
    (tmp, homes)
}

#[test]
fn every_file_found_is_written_back_to_the_very_path_it_came_from() {
    let (_tmp, homes) = seeded();
    let spots: Vec<Spot> = homes
        .detected()
        .into_iter()
        .flat_map(|one| spots(&homes, one))
        .collect();
    let all = found(&spots);
    let names: Vec<String> = all
        .iter()
        .map(|one| format!("{}/{}", one.assistant, one.name))
        .collect();
    for wanted in [
        "claude/CLAUDE.md",
        "claude/rules/CLAUDE.md",
        "claude/rules/style/tone.md",
        "claude/pr/SKILL.md",
        "claude/projects/repo/memory/MEMORY.md",
        "codex/AGENTS.md",
        "codex/lint/SKILL.md",
        "copilot/copilot-instructions.md",
        "copilot/instructions/rust.instructions.md",
        "gemini/GEMINI.md",
    ] {
        assert!(names.contains(&wanted.to_string()), "{wanted} in {names:?}");
    }
    assert!(
        !names.iter().any(|one| one.ends_with(".jsonl")),
        "{names:?}"
    );
    assert!(!names.iter().any(|one| one.ends_with("settings.json")));
    for one in all.iter().filter(|one| one.skip.is_none()) {
        let card = card(&one.assistant, one.kind, &one.name);
        let (root, at) = target_of(&homes, &card).unwrap_or_else(|| panic!("{one:?}"));
        assert_eq!(at, one.at, "{}", one.name);
        assert!(at.starts_with(&root));
    }
}

#[test]
fn the_variables_that_move_an_assistant_are_honoured() {
    let tmp = tempfile::tempdir().unwrap();
    let elsewhere = tmp.path().join("elsewhere");
    let homes = Homes::with(
        &tmp.path().join("home"),
        &[
            ("CLAUDE_CONFIG_DIR", &elsewhere.join("claude")),
            ("CODEX_HOME", &elsewhere.join("codex")),
            ("COPILOT_HOME", &elsewhere.join("copilot")),
            ("GEMINI_CLI_HOME", &elsewhere.join("g")),
        ],
    );
    assert_eq!(homes.root("claude"), Some(elsewhere.join("claude")));
    assert_eq!(homes.root("codex"), Some(elsewhere.join("codex")));
    assert_eq!(homes.root("copilot"), Some(elsewhere.join("copilot")));
    assert_eq!(
        homes.root("gemini"),
        Some(elsewhere.join("g").join(".gemini"))
    );
    assert_eq!(
        homes.root("cursor"),
        Some(tmp.path().join("home").join(".cursor"))
    );
}

#[test]
fn only_assistants_with_a_folder_here_are_detected() {
    let (_tmp, homes) = seeded();
    let ids: Vec<&str> = homes.detected().iter().map(|one| one.id).collect();
    assert_eq!(ids, ["claude", "codex", "copilot", "gemini"]);
}

#[test]
fn a_rule_outside_what_the_assistant_reads_has_no_target() {
    let (_tmp, homes) = seeded();
    assert!(target_of(&homes, &card("claude", Kind::Rule, "notes.md")).is_none());
    assert!(target_of(&homes, &card("codex", Kind::Rule, "rules/a.md")).is_none());
    assert!(target_of(&homes, &card("claude", Kind::Memory, "MEMORY.md")).is_none());
    assert!(target_of(&homes, &card("nobody", Kind::Rule, "CLAUDE.md")).is_none());
}

fn doc(level: &str, kind: Kind, name: &str, said: &str) -> String {
    doc_of(level, kind, name, said.as_bytes()).unwrap()
}

#[test]
fn a_global_rule_lands_in_no_file_and_global_skills_land_once_where_they_are_shared() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        (
            "a".to_string(),
            doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
        ),
        (
            "b".to_string(),
            doc(GLOBAL, Kind::Rule, "style.md", "Spanish.\n"),
        ),
        (
            "c".to_string(),
            doc(GLOBAL, Kind::Skill, "pr/SKILL.md", "pr\n"),
        ),
        (
            "d".to_string(),
            doc("cursor", Kind::Skill, "x/SKILL.md", "x\n"),
        ),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let claude = placed
        .iter()
        .find(|one| one.target.at == homes.root("claude").unwrap().join("CLAUDE.md"))
        .unwrap();
    assert_eq!(gated.named(&claude.files), ["a"]);
    for (id, main) in [("codex", "AGENTS.md"), ("gemini", GEMINI_RULES)] {
        let at = homes.root(id).unwrap().join(main);
        assert!(
            !placed.iter().any(|one| one.target.at == at),
            "no Global rule is written in {at:?}"
        );
    }
    assert!(placed.iter().all(|one| one.lead_in.is_none()));
    assert!(
        !placed
            .iter()
            .any(|one| gated.named(&one.files).contains(&"b")),
        "a Global rule is written nowhere"
    );
    let skills: Vec<&Placed> = placed
        .iter()
        .filter(|one| one.target.at.ends_with(Path::new("pr").join("SKILL.md")))
        .collect();
    let mut owners: Vec<&str> = skills
        .iter()
        .map(|one| one.target.assistant.as_str())
        .collect();
    owners.sort_unstable();
    assert_eq!(owners, ["claude", "codex"], "{skills:?}");
    assert!(
        !placed.iter().any(|one| one.target.assistant == "cursor"),
        "Cursor is not here, so nothing is written for it"
    );
}

#[test]
fn a_document_without_a_card_is_never_placed() {
    let (_tmp, homes) = seeded();
    let docs = vec![("a".to_string(), "# Just notes\n".to_string())];
    let ledger = consented();
    assert!(targets(&homes, &gated(&ledger, &docs).gate, &ledger).is_empty());
}

#[test]
fn a_global_rule_is_not_written_in_the_codex_override_either() {
    let (_tmp, homes) = seeded();
    let override_md = homes.root("codex").unwrap().join(CODEX_OVERRIDE);
    put(&override_md, "Override.\n");
    let docs = vec![
        (
            "g".to_string(),
            doc(GLOBAL, Kind::Rule, "style.md", "Spanish.\n"),
        ),
        (
            "o".to_string(),
            doc("codex", Kind::Rule, CODEX_OVERRIDE, "Mine.\n"),
        ),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let codex: Vec<&Placed> = placed
        .iter()
        .filter(|one| one.target.assistant == "codex")
        .collect();
    let [only] = codex.as_slice() else {
        panic!("{codex:?}");
    };
    assert_eq!(only.target.at, override_md);
    assert_eq!(gated.named(&only.files), ["o"]);
}

fn twice() -> Vec<(String, String)> {
    let body = doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n");
    vec![("b".to_string(), body.clone()), ("a".to_string(), body)]
}

fn claude_main(homes: &Homes) -> PathBuf {
    homes.root("claude").unwrap().join("CLAUDE.md")
}

fn written(ledger: &mut Ledger, homes: &Homes, docs: &[(String, String)], names: &[&str]) {
    let ids: Vec<String> = names
        .iter()
        .map(|name| gated(ledger, docs).id(name).to_string())
        .collect();
    ledger.applied_to(&claude_main(homes), ids);
}

#[test]
fn two_documents_for_the_same_file_hold_the_place_and_nothing_is_written() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let claude = placed
        .iter()
        .find(|one| one.target.assistant == "claude")
        .unwrap();
    assert!(claude.target.held());
    assert_eq!(gated.named(&claude.files), ["a", "b"]);
}

#[test]
fn the_side_this_computer_already_wrote_goes_on_and_the_other_is_not_placed() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let mut ledger = consented();
    written(&mut ledger, &homes, &docs, &["b"]);
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let claude = placed
        .iter()
        .find(|one| one.target.at == claude_main(&homes))
        .unwrap();
    assert_eq!(gated.named(&claude.files), ["b"]);
    assert!(!claude.target.held());
}

#[test]
fn with_both_sides_already_written_the_place_is_held() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let mut ledger = consented();
    written(&mut ledger, &homes, &docs, &["a", "b"]);
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let claude = placed
        .iter()
        .find(|one| one.target.at == claude_main(&homes))
        .unwrap();
    assert!(claude.target.held());
    assert_eq!(claude.files.len(), 2);
}

fn ids(gated: &super::gated::Gated, files: &BTreeSet<String>) -> Vec<String> {
    gated
        .named(&files.iter().cloned().collect::<Vec<_>>())
        .into_iter()
        .map(str::to_string)
        .collect()
}

#[test]
fn files_of_one_skill_stand_together_and_another_document_for_its_folder_is_a_rival() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        (
            "a".to_string(),
            doc("claude", Kind::Skill, "tidy/SKILL.md", "tidy\n"),
        ),
        (
            "b".to_string(),
            doc("claude", Kind::Skill, "tidy/notes.md", "notes\n"),
        ),
        (
            "c".to_string(),
            doc(GLOBAL, Kind::Skill, "tidy/SKILL.md", "other tidy\n"),
        ),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let sides = sides(&homes, &gated.gate, gated.id("a"));
    assert_eq!(ids(&gated, &sides.mine), ["a", "b"]);
    assert_eq!(ids(&gated, &sides.rival_files()), ["c"]);
}

#[test]
fn documents_that_share_nothing_are_never_rivals() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        (
            "a".to_string(),
            doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
        ),
        (
            "b".to_string(),
            doc("claude", Kind::Rule, "rules/tone.md", "Short.\n"),
        ),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let sides = sides(&homes, &gated.gate, gated.id("a"));
    assert!(sides.rivals.is_empty());
    assert_eq!(sides.mine.len(), 1);
}

#[test]
fn a_document_still_waiting_here_awaits_the_files_it_would_write() {
    let (_tmp, homes) = seeded();
    let docs = vec![(
        "a".to_string(),
        doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
    )];
    let mut ledger = consented();
    assert!(awaited(&homes, &gated(&ledger, &docs).gate).is_empty());
    ledger.wait_all([gated(&ledger, &docs).id("a")]);
    let gated = gated(&ledger, &docs);
    assert_eq!(
        gated.gate.seen(gated.id("a")),
        Some(agents::Seen::WaitingHere)
    );
    assert_eq!(
        awaited(&homes, &gated.gate),
        BTreeSet::from([agents::path_key(&claude_main(&homes))])
    );
    assert!(targets(&homes, &gated.gate, &ledger).is_empty());
}

fn computers_doc(level: &str, kind: Kind, name: &str, device: &str) -> String {
    let body = doc(level, kind, name, "Words.\n");
    tisty_core::agents::meant_for(&body, Some(&DeviceId(device.into()))).unwrap()
}

#[test]
fn two_computers_each_with_its_own_claude_file_never_write_each_others() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        (
            "a".to_string(),
            computers_doc("claude", Kind::Rule, "CLAUDE.md", ME),
        ),
        (
            "b".to_string(),
            computers_doc("claude", Kind::Rule, "CLAUDE.md", "dev_there"),
        ),
    ];
    let ledger = consented();
    let gated = gated_in(&ledger, &docs, &[("a", ME), ("b", "dev_there")]);
    let placed = targets(&homes, &gated.gate, &ledger);
    let [only] = placed.as_slice() else {
        panic!("{placed:?}");
    };
    assert_eq!(gated.named(&only.files), ["a"]);
    assert!(!only.target.held());
    assert_eq!(
        gated.gate.seen(gated.id("b")),
        Some(agents::Seen::OtherComputer)
    );
    assert!(gated.gate.elsewhere(gated.id("b")));
    assert!(sides(&homes, &gated.gate, gated.id("a")).rivals.is_empty());
}

#[test]
fn a_document_in_another_computers_folder_is_never_a_rival_nor_awaited() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        (
            "a".to_string(),
            doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
        ),
        (
            "b".to_string(),
            computers_doc("claude", Kind::Rule, "CLAUDE.md", "dev_there"),
        ),
    ];
    let mut ledger = consented();
    ledger.wait_all([gated(&ledger, &docs).id("b")]);
    let gated = gated_in(&ledger, &docs, &[("b", "dev_there")]);
    assert!(sides(&homes, &gated.gate, gated.id("a")).rivals.is_empty());
    assert!(awaited(&homes, &gated.gate).is_empty());
    let placed = targets(&homes, &gated.gate, &ledger);
    assert_eq!(placed.len(), 1);
    assert_eq!(gated.named(&placed[0].files), ["a"]);
}

#[test]
fn a_folder_and_a_text_naming_different_computers_place_nothing() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        (
            "a".to_string(),
            computers_doc("claude", Kind::Rule, "CLAUDE.md", "dev_there"),
        ),
        (
            "b".to_string(),
            computers_doc("claude", Kind::Rule, "rules/tone.md", ME),
        ),
    ];
    let ledger = consented();
    let gated = gated_in(&ledger, &docs, &[("a", ME)]);
    assert_eq!(gated.gate.seen(gated.id("a")), Some(agents::Seen::Apart));
    assert_eq!(
        gated.gate.seen(gated.id("b")),
        Some(agents::Seen::Apart),
        "in root with a computer's name"
    );
    assert!(targets(&homes, &gated.gate, &ledger).is_empty());
}

#[test]
fn a_computer_without_the_assistant_writes_nothing_for_it() {
    let tmp = tempfile::tempdir().unwrap();
    put(&tmp.path().join(".codex/AGENTS.md"), "Codex rules.\n");
    let homes = Homes::with(tmp.path(), &[]);
    let docs = vec![
        (
            "a".to_string(),
            doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
        ),
        (
            "b".to_string(),
            doc("codex", Kind::Rule, "AGENTS.md", "Codex.\n"),
        ),
        (
            "c".to_string(),
            doc(GLOBAL, Kind::Skill, "pr/SKILL.md", "pr\n"),
        ),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let claude = gated.gate.card_here(gated.id("a")).unwrap();
    assert!(!lands_here(&homes, claude));
    let placed = targets(&homes, &gated.gate, &ledger);
    let names: Vec<&str> = placed
        .iter()
        .flat_map(|one| gated.named(&one.files))
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"b") && names.contains(&"c"));
}

#[test]
fn card_at_names_the_rule_or_skill_each_assistant_reads_at_a_path() {
    let (tmp, homes) = seeded();
    let override_md = homes.root("codex").unwrap().join(CODEX_OVERRIDE);
    put(&override_md, "Override.\n");
    let in_claude = |name: &str| homes.root("claude").unwrap().join(name);
    assert_eq!(
        card_at(&homes, &in_claude("CLAUDE.md")),
        Some(("claude", Kind::Rule, "CLAUDE.md".to_string()))
    );
    assert_eq!(
        card_at(&homes, &in_claude("rules/style/tone.md")),
        Some(("claude", Kind::Rule, "rules/style/tone.md".to_string()))
    );
    assert_eq!(
        card_at(&homes, &override_md),
        Some(("codex", Kind::Rule, CODEX_OVERRIDE.to_string()))
    );
    assert_eq!(
        card_at(&homes, &in_claude("skills/pr/SKILL.md")),
        Some(("claude", Kind::Skill, "pr/SKILL.md".to_string()))
    );
    let shared = tmp.path().join(".agents/skills/lint/SKILL.md");
    assert_eq!(
        card_at(&homes, &shared),
        Some(("codex", Kind::Skill, "lint/SKILL.md".to_string()))
    );
}

#[test]
fn card_at_names_nothing_for_what_no_assistant_reads_as_a_rule() {
    let (_tmp, homes) = seeded();
    let claude = homes.root("claude").unwrap();
    for name in [
        "settings.json",
        "projects/repo/memory/MEMORY.md",
        "notes.md",
    ] {
        assert_eq!(card_at(&homes, &claude.join(name)), None, "{name}");
    }
}

#[test]
fn a_relative_folder_named_by_a_variable_counts_as_never_named() {
    let tmp = tempfile::tempdir().unwrap();
    let homes = Homes::with(
        tmp.path(),
        &[
            ("CLAUDE_CONFIG_DIR", Path::new(".claude2")),
            ("CODEX_HOME", &tmp.path().join("codex-here")),
        ],
    );
    assert_eq!(homes.root("claude"), Some(tmp.path().join(".claude")));
    assert_eq!(homes.root("codex"), Some(tmp.path().join("codex-here")));
    assert!(Homes::told(PathBuf::from("home"), []).is_none());
}

#[test]
fn every_assistant_the_app_writes_is_one_whose_folder_the_core_guards() {
    let ids: Vec<&str> = ASSISTANTS.iter().map(|one| one.id).collect();
    assert_eq!(ids, tisty_core::agents::KNOWN);
}

const TISTY_PLUGIN: &str =
    r#"{"version":2,"plugins":{"tisty@rgdevment":[{"scope":"user","version":"1.5.0"}]}}"#;

fn tistys_own(ledger: &mut Ledger) -> Gate {
    ledger.take_tisty_skills(true);
    let mut gate = gated(ledger, &[]).gate;
    let body = doc_of(GLOBAL, Kind::Skill, "tisty/SKILL.md", b"Tisty.\n").unwrap();
    gate.admit_builtins(
        ledger,
        &[agents::Bundled {
            skill: "tisty".into(),
            name: "tisty/SKILL.md".into(),
            body,
        }],
    );
    gate
}

fn shipped_to(homes: &Homes, gate: &Gate, ledger: &Ledger) -> Vec<String> {
    targets(homes, gate, ledger)
        .into_iter()
        .filter(|one| one.target.shipped())
        .map(|one| one.target.assistant)
        .collect()
}

#[test]
fn tistys_own_skills_reach_every_agent_and_the_shared_folder_once() {
    let (_tmp, homes) = seeded();
    let mut ledger = consented();
    let gate = tistys_own(&mut ledger);
    let mut reached = shipped_to(&homes, &gate, &ledger);
    reached.sort();
    assert_eq!(reached.len(), 2, "{reached:?}");
    assert!(reached.contains(&"claude".to_string()));
}

#[test]
fn claude_codes_tisty_plugin_keeps_tistys_copies_out_of_its_folder_alone() {
    let (tmp, homes) = seeded();
    put(
        &tmp.path().join(".claude/plugins/installed_plugins.json"),
        TISTY_PLUGIN,
    );
    let mut ledger = consented();
    let gate = tistys_own(&mut ledger);
    assert_eq!(homes.plugged(), BTreeSet::from(["claude"]));
    let reached = shipped_to(&homes, &gate, &ledger);
    assert_eq!(reached.len(), 1, "{reached:?}");
    assert!(!reached.contains(&"claude".to_string()));
}

#[test]
fn a_plugin_turned_off_installed_for_one_project_or_unreadable_brings_nothing() {
    let (tmp, homes) = seeded();
    let installed = tmp.path().join(".claude/plugins/installed_plugins.json");
    let settings = tmp.path().join(".claude/settings.json");
    assert!(homes.plugged().is_empty(), "no records, no plugin");

    put(&installed, TISTY_PLUGIN);
    put(&settings, r#"{"enabledPlugins":{"tisty@rgdevment":false}}"#);
    assert!(homes.plugged().is_empty(), "turned off");

    put(&settings, r#"{"enabledPlugins":{"tisty@rgdevment":true}}"#);
    assert_eq!(homes.plugged().len(), 1);

    put(&installed, &TISTY_PLUGIN.replace("\"user\"", "\"project\""));
    assert!(homes.plugged().is_empty(), "one project only");

    put(&installed, &TISTY_PLUGIN.replace("tisty@", "other@"));
    assert!(homes.plugged().is_empty(), "another plugin");

    put(
        &installed,
        &TISTY_PLUGIN.replace("tisty@rgdevment", "tisty@team/m~1"),
    );
    put(&settings, r#"{"enabledPlugins":{"tisty@team/m~1":false}}"#);
    assert!(
        homes.plugged().is_empty(),
        "turned off, whatever its key holds"
    );

    put(&installed, "not json");
    assert!(homes.plugged().is_empty());
}

#[test]
fn without_their_own_yes_no_target_carries_them_and_with_it_agents_yes_is_not_needed() {
    let (tmp, homes) = seeded();
    put(
        &tmp.path().join(".claude/plugins/installed_plugins.json"),
        TISTY_PLUGIN,
    );
    let mut ledger = consented();
    let switched_off = gated(&ledger, &[]).gate;
    assert!(shipped_to(&homes, &switched_off, &ledger).is_empty());
    ledger.consent(false);
    let skills_alone = tistys_own(&mut ledger);
    let reached = shipped_to(&homes, &skills_alone, &ledger);
    assert_eq!(reached.len(), 1, "{reached:?}");
    assert!(
        !reached.contains(&"claude".to_string()),
        "the plugin still counts"
    );
}

fn skill(level: &str) -> String {
    doc(level, Kind::Skill, "pr/SKILL.md", "pr\n")
}

fn claude_skill(placed: &[Placed], homes: &Homes) -> Vec<Placed> {
    let at = homes.root("claude").unwrap().join("skills/pr/SKILL.md");
    placed
        .iter()
        .filter(|one| one.target.at == at)
        .cloned()
        .collect()
}

#[test]
fn one_skill_brought_twice_in_the_same_place_is_written_once_from_the_smaller_id() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        ("b".to_string(), skill("claude")),
        ("a".to_string(), skill("claude")),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);

    let written = claude_skill(&targets(&homes, &gated.gate, &ledger), &homes);

    let [one] = written.as_slice() else {
        panic!("{written:?}");
    };
    assert_eq!(gated.named(&one.files), ["a"]);
    assert!(!one.target.held());
}

#[test]
fn a_skill_with_a_real_rival_is_still_held() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        ("a".to_string(), skill("claude")),
        ("b".to_string(), skill(GLOBAL)),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);

    let written = claude_skill(&targets(&homes, &gated.gate, &ledger), &homes);

    let [one] = written.as_slice() else {
        panic!("{written:?}");
    };
    assert!(one.target.held());
    assert_eq!(one.files.len(), 2);
}

fn side_of(
    homes: &Homes,
    gated: &super::gated::Gated,
    file: &str,
    other: &str,
) -> (Vec<String>, String) {
    let sides = sides(homes, &gated.gate, gated.id(file));
    let wanted = gated.id(other).to_string();
    let side = sides
        .rivals
        .iter()
        .find(|(_, files)| files.contains(&wanted))
        .map(|(side, _)| side.clone())
        .expect("the other document is a rival");
    let places = sides
        .places
        .iter()
        .map(|(place, _)| place.clone())
        .collect();
    (places, side)
}

fn choose_over(ledger: &mut Ledger, places: &[String], side: &str) {
    for place in places {
        ledger.choose(place, side);
    }
}

#[test]
fn a_recorded_choice_wins_over_what_was_written_and_puts_the_other_aside() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let mut ledger = consented();
    written(&mut ledger, &homes, &docs, &["a"]);
    let (place, side) = side_of(&homes, &gated(&ledger, &docs), "a", "b");
    choose_over(&mut ledger, &place, &side);
    let gated = gated(&ledger, &docs);

    let placed = targets(&homes, &gated.gate, &ledger);

    let claude = placed
        .iter()
        .find(|one| one.target.at == claude_main(&homes))
        .unwrap();
    assert_eq!(gated.named(&claude.files), ["b"]);
    assert!(!claude.target.held());
    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("b")),
        Stance::Goes
    );
    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("a")),
        Stance::Aside
    );
}

#[test]
fn a_document_disjoint_from_what_was_written_never_takes_over_even_alone() {
    let (_tmp, homes) = seeded();
    let docs = vec![(
        "a".to_string(),
        doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
    )];
    let mut ledger = consented();
    ledger.applied_to(&claude_main(&homes), ["gone-document".to_string()]);
    let gated = gated(&ledger, &docs);

    let placed = targets(&homes, &gated.gate, &ledger);

    let claude = placed
        .iter()
        .find(|one| one.target.at == claude_main(&homes))
        .unwrap();
    assert!(claude.target.held());
    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("a")),
        Stance::Asks
    );
}

#[test]
fn with_nothing_written_one_side_goes_and_several_ask() {
    let (_tmp, homes) = seeded();
    let ledger = consented();
    let alone = vec![(
        "a".to_string(),
        doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
    )];
    let one = gated(&ledger, &alone);
    assert_eq!(
        stance(&homes, &one.gate, &ledger, one.id("a")),
        Stance::Goes
    );

    let docs = twice();
    let several = gated(&ledger, &docs);
    assert_eq!(
        stance(&homes, &several.gate, &ledger, several.id("a")),
        Stance::Asks
    );
    assert_eq!(
        stance(&homes, &several.gate, &ledger, several.id("b")),
        Stance::Asks
    );
}

#[test]
fn a_written_side_that_is_still_credited_goes_and_the_other_is_put_aside() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let mut ledger = consented();
    written(&mut ledger, &homes, &docs, &["b"]);
    let gated = gated(&ledger, &docs);

    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("b")),
        Stance::Goes
    );
    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("a")),
        Stance::Aside
    );
}

#[test]
fn a_global_skill_put_aside_in_claude_keeps_writing_in_the_shared_folder() {
    let (_tmp, homes) = seeded();
    let docs = vec![
        ("a".to_string(), skill("claude")),
        ("b".to_string(), skill(GLOBAL)),
    ];
    let mut ledger = consented();
    let (place, side) = side_of(&homes, &gated(&ledger, &docs), "b", "a");
    choose_over(&mut ledger, &place, &side);
    let gated = gated(&ledger, &docs);

    let placed = targets(&homes, &gated.gate, &ledger);

    let in_claude = claude_skill(&placed, &homes);
    assert_eq!(gated.named(&in_claude[0].files), ["a"]);
    let shared = homes
        .root("codex")
        .unwrap()
        .parent()
        .unwrap()
        .join(".agents/skills/pr/SKILL.md");
    let in_shared = placed.iter().find(|one| one.target.at == shared).unwrap();
    assert_eq!(gated.named(&in_shared.files), ["b"]);
    assert!(!in_shared.target.held());
}
