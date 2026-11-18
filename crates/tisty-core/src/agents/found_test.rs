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
    put(&home.path().join(".copilot/hooks/tisty.json"), "{}");
    put(
        &home.path().join(".copilot/hooks/mine.json.before-tisty"),
        "{}",
    );
    put(&home.path().join(".copilot/hooks/settings.json"), "{}");
    put(&home.path().join(".copilot/hooks/.credentials.json"), "{}");
    put(&home.path().join(".copilot/hooks/deploy.pem"), "x");
    let found = found(&claude(home.path()));
    let skips: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(
        skips,
        [
            ("hooks/.credentials.json", Some(Skip::Credentials)),
            ("hooks/deploy.pem", Some(Skip::Credentials)),
            ("hooks/mine.json.before-tisty", Some(Skip::Tisty)),
            ("hooks/settings.json", Some(Skip::Settings)),
            ("hooks/tisty.json", Some(Skip::Tisty)),
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
    put(&home.path().join(".copilot/hooks/settings.json"), "{}");
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    let found = found(&claude(home.path()));
    assert_eq!(
        brought_in(&ledger, &found[0]),
        Err(Refused::Skipped(Skip::Settings))
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
        &home.path().join(".claude/skills/tisty-voice.md"),
        "notes\n",
    );
    let found = found(&claude(home.path()));
    let names: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(
        names,
        [("tisty-news/SKILL.md", None), ("tisty-voice.md", None)]
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
    put(&home.path().join(".claude/skills/pdf/config.json"), "{}");
    put(&home.path().join(".claude/skills/pdf/.env"), "KEY=x\n");
    let found = found(&claude(home.path()));
    let names: Vec<(&str, Option<Skip>)> = found
        .iter()
        .map(|one| (one.name.as_str(), one.skip))
        .collect();
    assert_eq!(
        names,
        [
            ("pdf/.env", Some(Skip::Credentials)),
            ("pdf/config.json", None)
        ]
    );
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
