fn strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(|it| it.as_str().map(str::to_string))
        .collect()
}

#[test]
fn the_core_reads_every_case_of_the_markdown_contract_as_it_says() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw = std::fs::read_to_string(root.join("fixtures/markdown.json"))
        .expect("the contract the window keeps too");
    let cases: Vec<serde_json::Value> = serde_json::from_str(&raw).expect("a list of cases");
    assert!(cases.len() > 50);

    for one in cases {
        let name = one["name"].as_str().expect("name");
        let text = match one["file"].as_str() {
            Some(file) => {
                std::fs::read_to_string(root.join(file)).expect("a file the contract names")
            }
            None => one["text"].as_str().expect("text").to_string(),
        };
        let text = text.as_str();
        let tags: Vec<String> = tisty_core::tagging::tags_in(text)
            .iter()
            .map(ToString::to_string)
            .collect();
        let refs: Vec<String> = tisty_core::refs::extract(text)
            .iter()
            .map(|it| format!("{:?}:{}", it.kind, it.target))
            .collect();

        assert_eq!(
            tisty_core::docs::titled(text),
            one["title"].as_str().expect("title"),
            "{name}"
        );
        assert_eq!(tags, strings(&one["tags"]), "{name}");
        assert_eq!(refs, strings(&one["refs"]), "{name}");
    }
}
