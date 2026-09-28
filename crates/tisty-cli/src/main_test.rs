use super::*;

fn normalised(args: &[&str]) -> Vec<String> {
    normalise(args.iter().map(|s| s.to_string()))
}

fn keys(facts: &[(&'static str, tisty_core::witness::Fact)]) -> Vec<&'static str> {
    facts.iter().map(|(name, _)| *name).collect()
}

#[test]
fn a_failure_from_the_core_carries_the_facts_the_core_deems_safe() {
    let broke = anyhow::Error::from(tisty_core::Error::MissingSegment {
        number: 7,
        device: "dev_a3f1".into(),
    });

    let facts = blamed("sync", &broke);

    assert_eq!(keys(&facts), ["command", "code", "number", "device"]);
}

#[test]
fn a_refusal_meant_for_the_screen_leaves_its_words_on_the_screen() {
    let said = "no list matches «la clínica de Juan»";
    let refused = anyhow::anyhow!("{said}");

    let facts = blamed("mv", &refused);

    assert_eq!(keys(&facts), ["command"]);
    assert!(!format!("{facts:?}").contains("Juan"), "{facts:?}");
}

#[test]
fn being_told_no_is_not_the_same_as_something_breaking() {
    let refused = anyhow::anyhow!("no list matches «la clínica de Juan»");
    let broke = anyhow::Error::from(tisty_core::Error::AlreadyRunning);

    assert!(
        refused.downcast_ref::<tisty_core::Error>().is_none(),
        "a refusal carries no error of ours, so it is noted rather than warned about"
    );
    assert!(
        broke.downcast_ref::<tisty_core::Error>().is_some(),
        "something of ours breaking is what a warning is for"
    );
}

#[test]
fn a_failed_command_is_written_down_by_the_name_of_its_subcommand() {
    assert_eq!(named(Some("sync")), "sync");
    assert_eq!(named(Some("doctor")), "doctor");
    assert_eq!(named(None), "none");
}

#[test]
fn a_bare_capture_is_written_down_as_the_capture_it_is() {
    assert_eq!(named(Some("comprar pan")), "add");
    assert_eq!(named(Some("--version")), "add");
}

#[test]
fn nothing_a_person_typed_can_reach_the_file() {
    let secret = "llamar a la clínica de Juan";

    let written = named(Some(secret));

    assert!(SUBCOMMANDS.contains(&written), "{written}");
    assert!(!written.contains("Juan"));
}

#[test]
fn a_known_subcommand_stays_a_subcommand() {
    assert_eq!(normalised(&["tisty", "ls"]), ["tisty", "ls"]);
    assert_eq!(normalised(&["tisty", "done", "2"]), ["tisty", "done", "2"]);
}

#[test]
fn free_text_becomes_a_capture() {
    assert_eq!(
        normalised(&["tisty", "deploy the release"]),
        ["tisty", "add", "deploy the release"]
    );
}

#[test]
fn flags_are_left_to_clap() {
    assert_eq!(normalised(&["tisty", "--version"]), ["tisty", "--version"]);
    assert_eq!(normalised(&["tisty", "--help"]), ["tisty", "--help"]);
}

#[test]
fn bare_invocation_is_untouched() {
    assert_eq!(normalised(&["tisty"]), ["tisty"]);
}

#[test]
fn every_subcommand_is_known_to_the_capture_guard() {
    use clap::CommandFactory;

    for sub in Cli::command().get_subcommands() {
        let name = sub.get_name();
        assert!(
            SUBCOMMANDS.contains(&name),
            "«{name}» would be swallowed as a capture"
        );
    }
}
