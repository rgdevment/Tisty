use std::path::Path;

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

pub fn every_real_document() -> Vec<(String, String)> {
    let raw = std::fs::read_to_string(root().join("fixtures/markdown.json")).unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap();
    let mut out: Vec<(String, String)> = cases
        .into_iter()
        .map(|one| {
            let text = match one["file"].as_str() {
                Some(file) => std::fs::read_to_string(root().join(file)).unwrap(),
                None => one["text"].as_str().unwrap().to_owned(),
            };
            (one["name"].as_str().unwrap().to_owned(), text)
        })
        .collect();
    for folder in [
        "docs/help",
        "app/src-tauri/resources/guide/en",
        "app/src-tauri/resources/guide/es",
    ] {
        for file in std::fs::read_dir(root().join(folder)).unwrap().flatten() {
            let path = file.path();
            if path.extension().is_some_and(|one| one == "md") {
                out.push((
                    path.display().to_string(),
                    std::fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    assert!(out.len() > 100);
    out
}
