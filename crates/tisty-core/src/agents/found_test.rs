use super::*;

fn claude(home: &Path) -> Vec<Place> {
    vec![
        Place {
            assistant: "claude-code".into(),
            kind: Kind::Rule,
            at: home.join(".claude").join("CLAUDE.md"),
            name: "CLAUDE.md".into(),
        },
        Place {
            assistant: "claude-code".into(),
            kind: Kind::Skill,
            at: home.join(".claude").join("skills"),
            name: String::new(),
        },
        Place {
            assistant: "copilot".into(),
            kind: Kind::Rule,
            at: home.join(".copilot").join("hooks"),
            name: "hooks".into(),
        },
    ]
}

fn put(at: &Path, said: &str) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, said).unwrap();
}

#[test]
fn every_file_at_each_place_is_named_as_the_agent_reads_it() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/CLAUDE.md"), "# Rules\n");
    put(
        &home.path().join(".claude/skills/pdf/SKILL.md"),
        "---\nname: pdf\n---\n",
    );
    put(
        &home.path().join(".claude/skills/pdf/scripts/run.sh"),
        "echo\n",
    );
    let found = found(&claude(home.path()));
    let names: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(
        names,
        [
            ("CLAUDE.md", None),
            ("pdf/SKILL.md", None),
            ("pdf/scripts/run.sh", None)
        ]
    );
}

#[test]
fn settings_credentials_and_tistys_own_files_are_left_out() {
    let home = tempfile::tempdir().unwrap();
    let pdf = home.path().join(".claude/skills/pdf");
    put(&pdf.join("SKILL.md"), "x\n");
    put(&pdf.join("tisty.json"), "{}");
    put(&pdf.join("mine.json.before-tisty"), "{}");
    put(&pdf.join("credentials.json"), "{}");
    put(&pdf.join("deploy.pem"), "x");
    put(&home.path().join(".copilot/hooks/settings.json"), "{}");
    let found = found(&claude(home.path()));
    let skips: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(
        skips,
        [
            ("pdf/SKILL.md", None),
            ("pdf/credentials.json", Some(Skip::Credentials)),
            ("pdf/deploy.pem", Some(Skip::Credentials)),
            ("pdf/mine.json.before-tisty", Some(Skip::Tisty)),
            ("pdf/tisty.json", Some(Skip::Tisty)),
        ]
    );
}

#[test]
fn looking_opens_nothing_and_bringing_in_needs_consent() {
    let home = tempfile::tempdir().unwrap();
    let at = home.path().join(".claude/CLAUDE.md");
    put(&at, "# Rules\n");
    let found = found(&claude(home.path()));
    std::fs::remove_file(&at).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(
        brought_in(&Ledger::fresh("machine-a"), &found[0]),
        Err(Refused::NoConsent)
    );
}

#[test]
fn a_chosen_file_comes_in_as_its_document() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/CLAUDE.md"), "# Rules\n");
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    let body = brought_in(&ledger, &found(&claude(home.path()))[0]).unwrap();
    assert!(body.starts_with("---\nagent: claude-code/rule/CLAUDE.md\n"));
    assert_eq!(super::super::rendered(&body).unwrap(), b"# Rules\n");
}

#[test]
fn a_file_that_reads_like_it_holds_a_token_is_skipped_with_a_warning() {
    let home = tempfile::tempdir().unwrap();
    put(
        &home.path().join(".claude/CLAUDE.md"),
        "Use this key: ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789\n",
    );
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    let refused = brought_in(&ledger, &found(&claude(home.path()))[0]);
    assert!(matches!(refused, Err(Refused::Secret(_))), "{refused:?}");
}

#[test]
fn a_skipped_file_cannot_be_brought_in_even_when_chosen() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/skills/pdf/SKILL.md"), "x\n");
    put(&home.path().join(".claude/skills/pdf/deploy.pem"), "x");
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    let found = found(&claude(home.path()));
    assert_eq!(found[1].name, "pdf/deploy.pem");
    assert_eq!(
        brought_in(&ledger, &found[1]),
        Err(Refused::Skipped(Skip::Credentials))
    );
}

#[test]
fn only_tistys_own_files_are_left_out_not_everything_named_like_it() {
    let home = tempfile::tempdir().unwrap();
    put(
        &home.path().join(".claude/skills/tisty-news/SKILL.md"),
        "---\nname: tisty-news\n---\n",
    );
    put(
        &home.path().join(".claude/skills/tisty-voice/SKILL.md"),
        "notes\n",
    );
    let found = found(&claude(home.path()));
    let names: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(
        names,
        [
            ("tisty-news/SKILL.md", None),
            ("tisty-voice/SKILL.md", None)
        ]
    );
}

#[test]
fn a_file_left_out_by_its_name_stays_out_whatever_the_caller_says() {
    let home = tempfile::tempdir().unwrap();
    let at = home.path().join(".env");
    std::fs::write(&at, "API_KEY=abc\n").unwrap();
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    let forged = Found {
        assistant: "claude-code".into(),
        kind: Kind::Rule,
        at,
        name: "env.md".into(),
        skip: None,
    };
    assert_eq!(
        brought_in(&ledger, &forged),
        Err(Refused::Skipped(Skip::Credentials))
    );
}

#[test]
fn a_skill_keeps_its_own_config_files() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/skills/pdf/SKILL.md"), "x\n");
    put(&home.path().join(".claude/skills/pdf/config.json"), "{}");
    put(&home.path().join(".claude/skills/pdf/.env"), "KEY=x\n");
    let found = found(&claude(home.path()));
    let names: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(names, [("pdf/SKILL.md", None), ("pdf/config.json", None)]);
}

#[test]
fn hidden_folders_and_node_modules_are_not_walked() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/skills/pdf/.git/HEAD"), "ref\n");
    put(
        &home
            .path()
            .join(".claude/skills/pdf/node_modules/x/index.js"),
        "x\n",
    );
    put(&home.path().join(".claude/skills/pdf/SKILL.md"), "x\n");
    let found = found(&claude(home.path()));
    let names: Vec<&str> = found.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["pdf/SKILL.md"]);
}

#[test]
fn a_crowded_place_says_so_once_and_stops() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/skills/many/SKILL.md"), "x\n");
    for n in 0..FOUND_AT_MOST + 5 {
        put(
            &home.path().join(format!(".claude/skills/many/{n:05}.md")),
            "x\n",
        );
    }
    put(&home.path().join(".copilot/hooks/later.md"), "x\n");
    let found = found(&claude(home.path()));
    let crowded = found
        .iter()
        .filter(|one| one.skip == Some(Skip::Crowded))
        .count();
    assert_eq!(crowded, 1);
    assert_eq!(found.len(), FOUND_AT_MOST + 1);
}

fn names_of(found: &[Found]) -> Vec<&str> {
    found.iter().map(|one| one.name.as_str()).collect()
}

#[test]
fn the_skills_claude_keeps_synced_from_claude_ai_are_left_out() {
    let home = tempfile::tempdir().unwrap();
    let synced = home.path().join(".claude/skills/synced/bucket-1");
    let skills = home.path().join(".claude/skills");
    put(&skills.join("Synced/SKILL.md"), "x\n");
    put(&skills.join(".trash/old/SKILL.md"), "x\n");
    put(&synced.join("manifest.json"), "{}");
    put(&synced.join("review/SKILL.md"), "x\n");
    put(&synced.join("review/notes.md"), "x\n");
    put(&home.path().join(".claude/skills/synced/SKILL.md"), "x\n");
    put(&home.path().join(".claude/skills/pr/SKILL.md"), "x\n");
    assert_eq!(names_of(&found(&claude(home.path()))), ["pr/SKILL.md"]);
}

#[test]
fn a_hidden_file_never_shows_at_any_depth() {
    let home = tempfile::tempdir().unwrap();
    put(&home.path().join(".claude/skills/.bucket-x"), "x\n");
    put(&home.path().join(".claude/skills/pr/SKILL.md"), "x\n");
    put(&home.path().join(".claude/skills/pr/.bucket-y"), "x\n");
    put(&home.path().join(".claude/rules/.draft.md"), "x\n");
    put(&home.path().join(".claude/rules/tone.md"), "x\n");
    let mut places = claude(home.path());
    places.push(Place {
        assistant: "claude".into(),
        kind: Kind::Rule,
        at: home.path().join(".claude/rules"),
        name: "rules".into(),
    });
    assert_eq!(names_of(&found(&places)), ["pr/SKILL.md", "rules/tone.md"]);
}

#[test]
fn a_skill_is_a_folder_with_its_skill_md_and_brings_every_file_it_holds() {
    let home = tempfile::tempdir().unwrap();
    let skills = home.path().join(".claude/skills");
    put(&skills.join("pr/SKILL.md"), "x\n");
    put(&skills.join("pr/plantilla.md"), "x\n");
    put(&skills.join("pr/scripts/abrir.sh"), "x\n");
    put(&skills.join("notes/readme.md"), "x\n");
    put(&skills.join("loose.md"), "x\n");
    put(&skills.join("deeper/inner/SKILL.md"), "x\n");
    assert_eq!(
        names_of(&found(&claude(home.path()))),
        ["pr/SKILL.md", "pr/plantilla.md", "pr/scripts/abrir.sh"]
    );
}

#[test]
fn every_assistant_s_skills_follow_the_same_rules() {
    let home = tempfile::tempdir().unwrap();
    let places: Vec<Place> = [
        ("codex", ".agents/skills"),
        ("copilot", ".copilot/skills"),
        ("gemini", ".gemini/skills"),
        ("cursor", ".cursor/skills"),
    ]
    .into_iter()
    .map(|(who, at)| {
        let skills = home.path().join(at);
        put(&skills.join("pr/SKILL.md"), "x\n");
        put(&skills.join(".hidden/SKILL.md"), "x\n");
        put(&skills.join(".bucket-x"), "x\n");
        put(&skills.join("synced/b/s/SKILL.md"), "x\n");
        put(&skills.join("notes/a.md"), "x\n");
        Place {
            assistant: who.into(),
            kind: Kind::Skill,
            at: skills,
            name: String::new(),
        }
    })
    .collect();
    let found = found(&places);
    let each: Vec<(&str, &str)> = found
        .iter()
        .map(|one| (one.assistant.as_str(), one.name.as_str()))
        .collect();
    assert_eq!(
        each,
        [
            ("codex", "pr/SKILL.md"),
            ("copilot", "pr/SKILL.md"),
            ("gemini", "pr/SKILL.md"),
            ("cursor", "pr/SKILL.md")
        ]
    );
}

#[test]
fn a_rules_folder_brings_only_what_its_assistant_reads() {
    let home = tempfile::tempdir().unwrap();
    let rules = home.path().join(".claude/rules");
    put(&rules.join("tone.md"), "x\n");
    put(&rules.join("deep/style.MD"), "x\n");
    put(&rules.join("notes.txt"), "x\n");
    put(&rules.join("settings.json"), "{}");
    put(&rules.join("plugin/manifest.json"), "{}");
    put(&rules.join("plugin/kept.md"), "x\n");
    put(&rules.join("synced/team.md"), "x\n");
    let instructions = home.path().join(".copilot/instructions");
    put(&instructions.join("tests.instructions.md"), "x\n");
    put(&instructions.join("readme.md"), "x\n");
    let places: Vec<Place> = [
        ("claude", &rules, "rules"),
        ("copilot", &instructions, "instructions"),
    ]
    .into_iter()
    .map(|(who, at, name)| Place {
        assistant: who.into(),
        kind: Kind::Rule,
        at: at.clone(),
        name: name.into(),
    })
    .collect();
    assert_eq!(
        names_of(&found(&places)),
        [
            "rules/deep/style.MD",
            "rules/synced/team.md",
            "rules/tone.md",
            "instructions/tests.instructions.md"
        ]
    );
}

fn skill_places(home: &Path, who: &[(&str, &str)]) -> Vec<Place> {
    who.iter()
        .map(|(assistant, at)| Place {
            assistant: (*assistant).into(),
            kind: Kind::Skill,
            at: home.join(at),
            name: String::new(),
        })
        .collect()
}

#[test]
fn a_skill_in_a_category_folder_shows_only_where_its_assistant_looks_that_deep() {
    let home = tempfile::tempdir().unwrap();
    let places = skill_places(
        home.path(),
        &[
            ("claude", ".claude/skills"),
            ("codex", ".agents/skills"),
            ("copilot", ".copilot/skills"),
            ("gemini", ".gemini/skills"),
            ("cursor", ".cursor/skills"),
        ],
    );
    for place in &places {
        put(&place.at.join("trabajo/pr/SKILL.md"), "x\n");
    }
    let each: Vec<(String, String)> = found(&places)
        .into_iter()
        .map(|one| (one.assistant, one.name))
        .collect();
    assert_eq!(
        each,
        [
            ("codex".to_string(), "trabajo/pr/SKILL.md".to_string()),
            ("cursor".to_string(), "trabajo/pr/SKILL.md".to_string())
        ]
    );
}

#[test]
fn codex_looks_six_levels_down_and_cursor_as_deep_as_tisty_walks() {
    let home = tempfile::tempdir().unwrap();
    let places = skill_places(
        home.path(),
        &[("codex", ".agents/skills"), ("cursor", ".cursor/skills")],
    );
    for place in &places {
        put(&place.at.join("a/b/c/d/e/six/SKILL.md"), "x\n");
        put(&place.at.join("a/b/c/d/e/f/seven/SKILL.md"), "x\n");
    }
    let each: Vec<(String, String)> = found(&places)
        .into_iter()
        .map(|one| (one.assistant, one.name))
        .collect();
    assert_eq!(
        each,
        [
            ("codex".to_string(), "a/b/c/d/e/six/SKILL.md".to_string()),
            (
                "cursor".to_string(),
                "a/b/c/d/e/f/seven/SKILL.md".to_string()
            ),
            ("cursor".to_string(), "a/b/c/d/e/six/SKILL.md".to_string())
        ]
    );
}

#[test]
fn a_folder_the_assistant_keeps_stays_out_at_every_level_and_a_skill_holds_its_inner_ones() {
    let home = tempfile::tempdir().unwrap();
    let places = skill_places(home.path(), &[("codex", ".agents/skills")]);
    let skills = &places[0].at;
    put(&skills.join("trabajo/synced/pr/SKILL.md"), "x\n");
    put(&skills.join("trabajo/.hidden/pr/SKILL.md"), "x\n");
    put(&skills.join("pr/SKILL.md"), "x\n");
    put(&skills.join("pr/examples/mini/SKILL.md"), "x\n");
    assert_eq!(
        names_of(&found(&places)),
        ["pr/SKILL.md", "pr/examples/mini/SKILL.md"]
    );
}
