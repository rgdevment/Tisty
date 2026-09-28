use super::*;

fn under_home(rest: &str) -> String {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/home/someone".into());
    std::path::Path::new(&home).join(rest).display().to_string()
}

fn fact_named<'a>(facts: &'a [(&'static str, witness::Fact)], key: &str) -> &'a witness::Fact {
    &facts.iter().find(|(name, _)| *name == key).expect(key).1
}

#[test]
fn a_broken_segment_names_its_file_without_naming_the_account() {
    let at = under_home("tisty/store/dev_a3f1/000001.tisty");
    let error = Error::MalformedEvent {
        file: at.clone(),
        line: 35,
        source: serde_json::from_str::<u8>("nonsense").unwrap_err(),
    };

    let facts = error.told();

    assert!(
        matches!(fact_named(&facts, "file"), witness::Fact::Path(_)),
        "a path carried as an id skips the redaction and takes the home directory with it"
    );
    assert!(matches!(
        fact_named(&facts, "line"),
        witness::Fact::Count(35)
    ));
}

#[test]
fn a_torn_segment_names_its_file_the_same_way() {
    let error = Error::TruncatedSegment {
        file: under_home("tisty/store/dev_a3f1/000001.tisty"),
        found: 12,
        declared: Some(20),
    };

    assert!(matches!(
        fact_named(&error.told(), "file"),
        witness::Fact::Path(_)
    ));
}

#[test]
fn every_error_says_which_one_it_is() {
    let facts = Error::NoHomeDirectory.told();

    assert!(matches!(
        fact_named(&facts, "code"),
        witness::Fact::Code("noHomeDirectory")
    ));
}
