use super::*;

fn cases() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../fixtures/tags.json")).unwrap()
}

fn said(one: &serde_json::Value) -> &str {
    one.as_str().unwrap()
}

// The window reads the same file, so the two cannot drift apart.
#[test]
fn a_word_becomes_the_tag_it_says() {
    for one in cases()["named"].as_array().unwrap() {
        assert_eq!(
            named(said(&one[0])).as_deref(),
            Some(said(&one[1])),
            "{}",
            one[0]
        );
    }
}

#[test]
fn a_tag_read_again_is_the_same_tag() {
    for one in cases()["named"].as_array().unwrap() {
        assert_eq!(
            named(said(&one[1])).as_deref(),
            Some(said(&one[1])),
            "{}",
            one[1]
        );
    }
}

#[test]
fn a_sentence_gives_the_tags_it_says() {
    for one in cases()["sentences"].as_array().unwrap() {
        let found: Vec<String> = spots(said(&one[0]))
            .into_iter()
            .map(|(_, tag)| tag)
            .collect();
        let told: Vec<&str> = one[1].as_array().unwrap().iter().map(said).collect();
        assert_eq!(found, told, "{}", one[0]);
    }
}

#[test]
fn a_tag_takes_the_whole_word_it_was_written_with() {
    let line = "hoy #\u{939}\u{93F}\u{928}\u{94D}\u{926}\u{940} fin";
    let (at, _) = &spots(line)[0];
    assert_eq!(
        &line[at.clone()],
        "#\u{939}\u{93F}\u{928}\u{94D}\u{926}\u{940}"
    );
}
