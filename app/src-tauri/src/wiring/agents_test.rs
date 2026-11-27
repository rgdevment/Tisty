use super::gated::{ME, consented, gated, gated_in, id_of};
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
fn the_harness_alone_owns_the_shared_skills_folder() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("home/.dsh")).unwrap();
    let homes = Homes::with(&tmp.path().join("home"), &[]);
    let ids: Vec<&str> = homes.detected().iter().map(|one| one.id).collect();
    assert_eq!(ids, ["dsh"]);

    let ledger = consented();
    let docs = vec![(
        "a".to_string(),
        doc(GLOBAL, Kind::Skill, "pr/SKILL.md", "pr\n"),
    )];
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    let skills: Vec<&Placed> = placed
        .iter()
        .filter(|one| one.target.assistant == "dsh")
        .collect();
    assert_eq!(skills.len(), 1);
    assert_eq!(
        skills[0].target.at,
        tmp.path().join("home/.agents/skills/pr/SKILL.md")
    );
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
fn the_side_this_computer_already_wrote_no_longer_goes_on_when_another_arrives_and_the_place_is_held()
 {
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
    let mut named = gated.named(&claude.files);
    named.sort_unstable();
    assert_eq!(named, ["a", "b"]);
    assert!(claude.target.held());
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
) -> (Vec<(String, String)>, String) {
    let sides = sides(homes, &gated.gate, gated.id(file));
    let wanted = gated.id(other).to_string();
    let side = sides
        .rivals
        .iter()
        .find(|(_, files)| files.contains(&wanted))
        .map(|(side, _)| side.clone())
        .expect("the other document is a rival");
    (sides.places, side)
}

fn choose_over(ledger: &mut Ledger, places: &[(String, String)], side: &str) {
    for (place, over) in places {
        ledger.choose(place, side, [over.clone()]);
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
fn a_written_side_with_a_newcomer_asks_on_both_sides_until_one_is_chosen() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let mut ledger = consented();
    written(&mut ledger, &homes, &docs, &["b"]);
    let gated = gated(&ledger, &docs);

    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("b")),
        Stance::Asks
    );
    assert_eq!(
        stance(&homes, &gated.gate, &ledger, gated.id("a")),
        Stance::Asks
    );
}

#[test]
fn a_choice_made_over_the_other_side_stands_and_one_made_over_nobody_else_does_not() {
    let (_tmp, homes) = seeded();
    let docs = twice();
    let mut ledger = consented();
    let (place, side) = side_of(&homes, &gated(&ledger, &docs), "a", "b");
    choose_over(&mut ledger, &place, &side);
    let chosen = gated(&ledger, &docs);
    assert_eq!(
        stance(&homes, &chosen.gate, &ledger, chosen.id("b")),
        Stance::Goes
    );
    assert_eq!(
        stance(&homes, &chosen.gate, &ledger, chosen.id("a")),
        Stance::Aside
    );

    let mut forgotten = consented();
    for (place, _) in &place {
        forgotten.choose(place, &side, Vec::<String>::new());
    }
    let unweighed = gated(&forgotten, &docs);
    assert_eq!(
        stance(&homes, &unweighed.gate, &forgotten, unweighed.id("b")),
        Stance::Asks
    );
    assert_eq!(
        stance(&homes, &unweighed.gate, &forgotten, unweighed.id("a")),
        Stance::Asks
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

fn home_of(folders: &[&str]) -> (tempfile::TempDir, Homes) {
    let tmp = tempfile::tempdir().unwrap();
    for one in folders {
        std::fs::create_dir_all(tmp.path().join("home").join(one)).unwrap();
    }
    let homes = Homes::with(&tmp.path().join("home"), &[]);
    (tmp, homes)
}

fn landings_of(homes: &Homes, level: &str) -> Vec<(String, PathBuf)> {
    let docs = vec![(
        "a".to_string(),
        doc(level, Kind::Skill, "pdf/SKILL.md", "pdf\n"),
    )];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let mut all: Vec<(String, PathBuf)> = targets(homes, &gated.gate, &ledger)
        .into_iter()
        .map(|one| (one.target.assistant, one.target.at))
        .collect();
    all.sort();
    all
}

#[test]
fn a_global_skill_with_cursor_and_codex_lands_once_in_the_shared_folder_owned_by_codex() {
    let (tmp, homes) = home_of(&[".cursor", ".codex"]);
    let home = tmp.path().join("home");
    assert_eq!(
        landings_of(&homes, GLOBAL),
        [(
            "codex".to_string(),
            home.join(".agents/skills/pdf/SKILL.md")
        )]
    );
    assert!(!home.join(".cursor/skills").exists());
}

#[test]
fn a_global_skill_with_only_cursor_and_claude_lands_in_the_shared_folder_and_claudes() {
    let (tmp, homes) = home_of(&[".cursor", ".claude"]);
    let home = tmp.path().join("home");
    assert_eq!(
        landings_of(&homes, GLOBAL),
        [
            (
                "claude".to_string(),
                home.join(".claude/skills/pdf/SKILL.md")
            ),
            (
                "cursor".to_string(),
                home.join(".agents/skills/pdf/SKILL.md")
            ),
        ]
    );
}

#[test]
fn a_global_skill_with_only_claude_lands_only_in_claudes_folder() {
    let (tmp, homes) = home_of(&[".claude"]);
    assert_eq!(
        landings_of(&homes, GLOBAL),
        [(
            "claude".to_string(),
            tmp.path().join("home/.claude/skills/pdf/SKILL.md")
        )]
    );
}

#[test]
fn a_cursor_level_skill_still_lands_in_cursors_own_folder() {
    let (tmp, homes) = home_of(&[".cursor", ".codex"]);
    assert_eq!(
        landings_of(&homes, "cursor"),
        [(
            "cursor".to_string(),
            tmp.path().join("home/.cursor/skills/pdf/SKILL.md")
        )]
    );
}

#[test]
fn cursor_reads_the_shared_skills_folder_with_the_others() {
    let (tmp, homes) = home_of(&[".cursor", ".codex", ".gemini"]);
    let shared = tmp.path().join("home/.agents/skills/pdf/SKILL.md");
    let readers: Vec<&str> = readers_of(&homes, &shared, "codex")
        .iter()
        .map(|one| one.id)
        .collect();
    assert!(readers.contains(&"cursor"), "{readers:?}");
    assert!(readers.contains(&"gemini"), "{readers:?}");
}

#[test]
fn cursor_twin_maps_a_file_in_cursors_skills_to_its_shared_copy_and_nothing_else() {
    let (tmp, homes) = home_of(&[".cursor", ".codex"]);
    let home = tmp.path().join("home");
    assert_eq!(
        cursor_twin(&homes, &home.join(".cursor/skills/pdf/scripts/x.py")),
        Some((
            home.join(".cursor/skills/pdf"),
            home.join(".agents/skills/pdf/scripts/x.py")
        ))
    );
    assert_eq!(
        cursor_twin(&homes, &home.join(".agents/skills/pdf/SKILL.md")),
        None
    );
    assert_eq!(cursor_twin(&homes, &home.join(".cursor/AGENTS.md")), None);
    assert_eq!(cursor_twin(&homes, &home.join(".cursor/skills")), None);
}

#[test]
fn only_a_global_skill_is_left_for_the_shared_folder_in_cursors() {
    let (tmp, homes) = home_of(&[".cursor", ".codex"]);
    let home = tmp.path().join("home");
    let at = home.join(".cursor/skills/pdf/SKILL.md");
    assert!(left_for_shared(
        &homes,
        &card(GLOBAL, Kind::Skill, "pdf/SKILL.md"),
        &at
    ));
    assert!(!left_for_shared(
        &homes,
        &card("cursor", Kind::Skill, "pdf/SKILL.md"),
        &at
    ));
    assert!(!left_for_shared(
        &homes,
        &card(GLOBAL, Kind::Rule, "pdf.md"),
        &at
    ));
    let elsewhere = home.join(".claude/skills/pdf/SKILL.md");
    assert!(!left_for_shared(
        &homes,
        &card(GLOBAL, Kind::Skill, "pdf/SKILL.md"),
        &elsewhere
    ));
}

fn fenced(tmp: &Path, told: &[(&'static str, &Path)]) -> Homes {
    let root = tmp.join("distro");
    let home = root.join("home").join("mario");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::create_dir_all(home.join(".codex")).unwrap();
    Homes {
        fence: Some(root),
        ..Homes::with(&home, told)
    }
}

#[test]
fn a_home_outside_the_distro_root_has_no_homes_there() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("distro");
    assert!(Homes::in_distro(&root.join("home").join("mario"), &root).is_some());
    assert!(Homes::in_distro(&tmp.path().join("elsewhere"), &root).is_none());
    assert!(Homes::in_distro(Path::new("relative"), Path::new("relative")).is_none());
}

// The process's own variables name this computer's folders, never the distro's.
#[test]
fn a_distros_homes_never_read_the_variables_of_this_process() {
    const CHILD: &str = "TISTY_TEST_DISTRO_CHILD";
    let tmp = tempfile::tempdir().unwrap();
    let base = std::env::var_os(CHILD).map_or_else(|| tmp.path().to_path_buf(), PathBuf::from);
    let root = base.join("distro");
    let home = root.join("home").join("mario");
    let elsewhere = base.join("elsewhere");
    if std::env::var_os(CHILD).is_none() {
        let name =
            "wiring::agents::tests::a_distros_homes_never_read_the_variables_of_this_process";
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture"])
            .env(CHILD, tmp.path())
            .env("CLAUDE_CONFIG_DIR", &elsewhere)
            .env("USERPROFILE", &home)
            .env("HOME", &home)
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(
            out.status.success(),
            "{said}{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(said.contains("1 passed") || said.contains("ok"), "{said}");
        return;
    }
    let here = Homes::here().unwrap();
    assert_eq!(
        here.root("claude"),
        Some(elsewhere),
        "the variable moves this computer's Claude"
    );
    let distro = Homes::in_distro(&home, &root).unwrap();
    assert_eq!(distro.root("claude"), Some(home.join(".claude")));
    assert!(distro.told.is_empty());
}

#[test]
fn a_distro_target_is_marked_and_nothing_lands_outside_the_fence() {
    let tmp = tempfile::tempdir().unwrap();
    let elsewhere = tmp.path().join("windows-codex");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let homes = fenced(tmp.path(), &[("CODEX_HOME", &elsewhere)]);
    let docs = vec![
        (
            "a".to_string(),
            doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
        ),
        (
            "b".to_string(),
            doc("codex", Kind::Rule, "AGENTS.md", "# Codex\n"),
        ),
        (
            "c".to_string(),
            doc(GLOBAL, Kind::Skill, "pr/SKILL.md", "pr\n"),
        ),
    ];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    assert!(!placed.is_empty());
    let fence = homes.fence.clone().unwrap();
    for one in &placed {
        assert!(one.target.distro, "{:?}", one.target.at);
        assert!(one.target.at.starts_with(&fence), "{:?}", one.target.at);
    }
    let claude = homes.root("claude").unwrap().join("CLAUDE.md");
    assert!(placed.iter().any(|one| one.target.at == claude));
    assert!(
        !placed
            .iter()
            .any(|one| one.target.assistant == "codex" && one.target.at.starts_with(&elsewhere)),
        "codex moved out of the distro is out of reach"
    );
    assert!(
        !placed
            .iter()
            .any(|one| gated.named(&one.files).contains(&"b"))
    );
}

#[test]
fn a_computers_own_targets_are_not_marked_as_a_distros() {
    let (_tmp, homes) = seeded();
    let docs = vec![(
        "a".to_string(),
        doc("claude", Kind::Rule, "CLAUDE.md", "# Claude\n"),
    )];
    let ledger = consented();
    let gated = gated(&ledger, &docs);
    let placed = targets(&homes, &gated.gate, &ledger);
    assert!(!placed.is_empty() && placed.iter().all(|one| !one.target.distro));
}

#[test]
fn a_hook_reaches_no_distro() {
    let tmp = tempfile::tempdir().unwrap();
    let homes = fenced(tmp.path(), &[]);
    let trigger = agents::Trigger {
        event: agents::Event::BeforeTool,
        tools: agents::Tools::Shell,
        timeout: 5,
        agents: vec!["claude-code".into()],
    };
    let body = agents::hook_doc("guard.sh", None, &trigger, "exit 2\n").unwrap();
    let mut ledger = consented();
    let (yes, _) = agents::yes_of(&body).unwrap();
    let file = id_of("h");
    ledger.register_hook(&file, yes);
    ledger.take(&file, true);
    let gated = gated(&ledger, &[("h".to_string(), body)]);
    let landed = placed(&homes, &gated.gate, &ledger, &BTreeSet::new());
    assert!(landed.is_empty(), "{landed:?}");
}
