use super::*;

const CLAUDE: &[u8] = include_bytes!("fixtures/CLAUDE.md");
const AGENTS: &[u8] = include_bytes!("fixtures/AGENTS.md");
const SKILL: &[u8] = include_bytes!("fixtures/SKILL.md");
const NOTIFY: &[u8] = include_bytes!("fixtures/notify.cmd");
const FIELDS: &[u8] = include_bytes!("fixtures/fields.sh");
const COPILOT: &[u8] = include_bytes!("fixtures/copilot-instructions.md");
const MIXED: &[u8] = include_bytes!("fixtures/mixed.txt");

fn every_fixture() -> [(&'static str, Kind, &'static str, &'static [u8]); 7] {
    [
        ("claude-code", Kind::Rule, "CLAUDE.md", CLAUDE),
        (GLOBAL, Kind::Rule, "AGENTS.md", AGENTS),
        ("claude-code", Kind::Skill, "pdf-forms/SKILL.md", SKILL),
        ("claude-code", Kind::Rule, "hooks/notify.cmd", NOTIFY),
        ("codex", Kind::Skill, "pdf-forms/scripts/fields.sh", FIELDS),
        ("copilot", Kind::Rule, "copilot-instructions.md", COPILOT),
        ("gemini", Kind::Rule, "notes/mixed.txt", MIXED),
    ]
}

#[test]
fn a_real_agent_file_comes_back_byte_for_byte() {
    for (level, kind, name, file) in every_fixture() {
        let body = doc_of(level, kind, name, file).unwrap();
        assert_eq!(rendered(&body).unwrap(), file, "{name}");
    }
}

#[test]
fn the_round_trip_holds_after_the_document_is_stored() {
    let dir = tempfile::tempdir().unwrap();
    let device = crate::event::DeviceId("dev-a".into());
    for (level, kind, name, file) in every_fixture() {
        let doc = crate::docs::create(
            dir.path(),
            &device,
            &doc_of(level, kind, name, file).unwrap(),
        )
        .unwrap();
        let stored = crate::docs::read(dir.path(), &doc.id).unwrap();
        assert_eq!(rendered(&stored).unwrap(), file, "{name}");
    }
}

#[test]
fn the_round_trip_holds_after_the_editor_saves_it_untouched() {
    for (level, kind, name, file) in every_fixture() {
        let body = doc_of(level, kind, name, file).unwrap();
        let as_the_editor_hands_it = body.replace("\r\n", "\n");
        let as_the_editor_hands_it = as_the_editor_hands_it.trim_end_matches('\n');
        if Card::of(&body).unwrap().endings == Endings::AsIs {
            continue;
        }
        assert_eq!(rendered(as_the_editor_hands_it).unwrap(), file, "{name}");
    }
}

#[test]
fn the_markdown_an_agent_reads_is_kept_as_written_not_translated() {
    let body = doc_of("claude-code", Kind::Rule, "CLAUDE.md", CLAUDE).unwrap();
    let text = std::str::from_utf8(CLAUDE).unwrap();
    assert!(body.ends_with(text));
    assert!(body.contains("@~/.claude/rules/style.md"));
    assert!(body.contains("<important>"));
    assert!(body.contains("::: not a callout"));
}

#[test]
fn a_skill_keeps_its_own_front_matter_under_tistys_card() {
    let body = doc_of("claude-code", Kind::Skill, "pdf-forms/SKILL.md", SKILL).unwrap();
    let card = Card::of(&body).unwrap();
    assert_eq!(card.address(), "claude-code/skill/pdf-forms/SKILL.md");
    let written = rendered(&body).unwrap();
    assert!(written.starts_with(b"---\nname: pdf-forms\ndescription: "));
    assert!(!String::from_utf8(written).unwrap().contains("agent:"));
}

#[test]
fn a_file_without_front_matter_gets_none_back() {
    let body = doc_of(GLOBAL, Kind::Rule, "AGENTS.md", AGENTS).unwrap();
    assert!(rendered(&body).unwrap().starts_with(b"# AGENTS.md"));
}

#[test]
fn a_script_lives_in_a_code_block_wide_enough_for_the_fences_it_holds() {
    let body = doc_of("codex", Kind::Skill, "pdf-forms/scripts/fields.sh", FIELDS).unwrap();
    assert!(body.contains("\n````sh\n#!/usr/bin/env sh\n"));
    let card = Card::of(&body).unwrap();
    assert!(card.open_end);
    assert!(card.code());
}

#[test]
fn the_card_says_only_what_the_bytes_need() {
    let plain = doc_of(GLOBAL, Kind::Rule, "AGENTS.md", AGENTS).unwrap();
    assert!(plain.starts_with(
        "---\nagent: global/rule/AGENTS.md\ntitle: global · AGENTS.md\n---\n# AGENTS.md"
    ));
    let windows = doc_of("copilot", Kind::Rule, "copilot-instructions.md", COPILOT).unwrap();
    assert!(
        windows.starts_with(
            "---\nagent: [copilot/rule/copilot-instructions.md, crlf, bom, open-end]\n"
        )
    );
    let mixed = doc_of("gemini", Kind::Rule, "notes/mixed.txt", MIXED).unwrap();
    assert_eq!(Card::of(&mixed).unwrap().endings, Endings::AsIs);
}

#[test]
fn an_edit_in_tisty_reaches_the_file_with_the_files_own_endings() {
    let body = doc_of("claude-code", Kind::Rule, "hooks/notify.cmd", NOTIFY).unwrap();
    let edited = body.replace("\r\n", "\n").replace("exit /b 0", "exit /b 1");
    let written = rendered(&edited).unwrap();
    assert_eq!(
        written,
        String::from_utf8(NOTIFY.to_vec())
            .unwrap()
            .replace("exit /b 0", "exit /b 1")
            .into_bytes()
    );
}

#[test]
fn bringing_a_change_in_keeps_the_other_properties_of_the_card() {
    let body = doc_of(GLOBAL, Kind::Rule, "AGENTS.md", AGENTS).unwrap();
    let tagged = body.replacen("title:", "tags: [agents]\ntitle:", 1);
    let now = b"# AGENTS.md\r\n\r\nChanged outside.";
    let back = brought(&tagged, now).unwrap();
    assert!(back.contains("tags: [agents]\n"));
    assert!(back.contains("agent: [global/rule/AGENTS.md, crlf, open-end]\n"));
    assert_eq!(rendered(&back).unwrap(), now);
}

#[test]
fn a_name_that_could_leave_its_folder_or_break_the_card_is_refused() {
    for name in [
        "../escape.md",
        "/abs.md",
        "a//b.md",
        "C:\\x.md",
        "a,b.md",
        "x/./y.md",
        "",
    ] {
        assert_eq!(
            doc_of(GLOBAL, Kind::Rule, name, b"x\n"),
            Err(Refused::Name),
            "{name}"
        );
    }
    assert_eq!(
        doc_of("Claude Code", Kind::Rule, "a.md", b"x\n"),
        Err(Refused::Name)
    );
}

#[test]
fn a_file_that_is_not_text_is_refused_rather_than_mangled() {
    assert_eq!(
        doc_of(GLOBAL, Kind::Skill, "logo.png", &[0x89, b'P', 0xff, 0xfe]),
        Err(Refused::NotText)
    );
}

#[test]
fn a_card_with_a_flag_this_tisty_does_not_know_is_not_read() {
    assert!(Card::of("---\nagent: [global/rule/a.md, utf16]\n---\nx\n").is_none());
    assert!(Card::of("---\nagent: global/hook/a.json\n---\nx\n").is_none());
}

#[test]
fn a_code_block_broken_by_an_edit_is_never_written() {
    let body = doc_of("claude-code", Kind::Rule, "hooks/notify.cmd", NOTIFY).unwrap();
    let broken = body.replace("```cmd", "the fence is gone");
    assert_eq!(rendered(&broken), Err(Refused::BrokenFence));
}

#[test]
fn the_odd_endings_a_file_can_have_still_come_back_whole() {
    let odd: [(&str, &[u8]); 5] = [
        ("x.json", b"x\r"),
        ("x.md", b"x\r"),
        ("x.md", b"\xef\xbb\xbf\xef\xbb\xbfa"),
        ("notes.`", b"plain\n"),
        ("x.sh", b"a\r\nb"),
    ];
    for (name, file) in odd {
        let body = doc_of(GLOBAL, Kind::Rule, name, file).unwrap();
        assert_eq!(rendered(&body).unwrap(), file, "{name}");
        let stored = crate::docs::settled(&body);
        assert_eq!(rendered(&stored).unwrap(), file, "{name} stored");
    }
}

#[test]
fn bringing_a_change_in_rewrites_only_the_cards_own_line() {
    let body = "---\nagent: global/rule/AGENTS.md\nnotes:\n  agent: kept as written\n---\nOld.\n";
    let back = brought(body, b"New.\n").unwrap();
    assert!(back.contains("\n  agent: kept as written\n"));
    assert!(back.ends_with("---\nNew.\n"));
}
