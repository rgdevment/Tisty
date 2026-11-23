use super::gated::{consented, gated};
use super::*;
use tisty_core::agents::{doc_of, found};

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
fn global_rules_lead_each_main_file_and_global_skills_land_once_where_they_are_shared() {
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
    assert_eq!(gated.named(&claude.files), ["b", "a"]);
    let codex = placed
        .iter()
        .find(|one| one.target.at == homes.root("codex").unwrap().join("AGENTS.md"))
        .unwrap();
    assert_eq!(gated.named(&codex.files), ["b"]);
    let gemini = placed
        .iter()
        .find(|one| one.target.assistant == "gemini" && one.target.at.ends_with(GEMINI_RULES))
        .unwrap();
    assert_eq!(
        gemini.lead_in.as_ref().unwrap().at,
        homes.root("gemini").unwrap().join(GEMINI_MEMORY)
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
fn global_rules_go_to_the_codex_override_when_codex_reads_it() {
    let (_tmp, homes) = seeded();
    put(
        &homes.root("codex").unwrap().join("AGENTS.override.md"),
        "Override.\n",
    );
    let docs = vec![(
        "g".to_string(),
        doc(GLOBAL, Kind::Rule, "style.md", "Spanish.\n"),
    )];
    let ledger = consented();
    let placed = targets(&homes, &gated(&ledger, &docs).gate, &ledger);
    let codex: Vec<&PathBuf> = placed
        .iter()
        .filter(|one| one.target.assistant == "codex")
        .map(|one| &one.target.at)
        .collect();
    assert_eq!(
        codex,
        [&homes.root("codex").unwrap().join("AGENTS.override.md")]
    );
}

#[test]
fn the_same_file_brought_on_two_computers_is_written_once() {
    let (_tmp, homes) = seeded();
    let twice = doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n");
    let docs = vec![("b".to_string(), twice.clone()), ("a".to_string(), twice)];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let claude = placed
        .iter()
        .find(|one| one.target.assistant == "claude")
        .unwrap();
    assert_eq!(gated.named(&claude.files), ["a"]);
    assert_eq!(claude.target.pieces.len(), 1);
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
