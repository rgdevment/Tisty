use serde_json::json;

use super::received;

fn kept_as_named(name: &str, _: &[u8]) -> Option<String> {
    Some(format!("kept-{name}"))
}

#[test]
fn what_travelled_points_again_at_what_is_kept_here() {
    let document = json!({
        "title": "Viaje",
        "body": "![a](tisty:picture/0 \"a.png\") [b](tisty:file/0 \"b.pdf\")\n",
        "pictures": [{"at": 0, "name": "a.png"}],
        "files": [{"at": 0, "name": "b.pdf"}],
        "owner_name": "Rina",
        "signed_as": "Ana",
    });

    let told = received(&document, &[vec![1]], &[vec![2]], &mut kept_as_named);

    assert_eq!(
        told.body,
        "![a](kept-a.png \"a.png\") [b](kept-b.pdf \"b.pdf\")\n"
    );
    assert_eq!(told.title.as_deref(), Some("Viaje"));
    assert_eq!(told.owner_name.as_deref(), Some("Rina"));
    assert_eq!(told.signed_as.as_deref(), Some("Ana"));
}

#[test]
fn a_number_never_eats_the_start_of_a_longer_one() {
    let pictures: Vec<serde_json::Value> = (0..11)
        .map(|at| json!({"at": at, "name": format!("p{at}")}))
        .collect();
    let document = json!({"body": "tisty:picture/1 tisty:picture/10", "pictures": pictures});
    let bytes = vec![vec![0]; 11];

    let told = received(&document, &bytes, &[], &mut kept_as_named);

    assert_eq!(told.body, "kept-p1 kept-p10");
}

#[test]
fn pages_come_along_with_their_pictures() {
    let document = json!({
        "body": "arriba",
        "pages": [{"body": "![a](tisty:picture/0)", "at": 1}],
        "pictures": [{"at": 0, "name": "a.png"}],
    });

    let told = received(&document, &[vec![1]], &[], &mut kept_as_named);

    assert_eq!(told.pages, ["![a](kept-a.png)"]);
}

#[test]
fn what_could_not_be_kept_or_was_never_there_stays_as_it_travelled() {
    let document = json!({
        "body": "tisty:picture/0 tisty:picture/1",
        "pictures": [{"at": 0, "name": "a"}, {"at": 1, "name": "b"}],
    });

    let told = received(&document, &[vec![1]], &[], &mut |_, _| None);

    assert_eq!(told.body, "tisty:picture/0 tisty:picture/1");
}

#[test]
fn how_readers_reach_other_languages_and_cited_documents_comes_along() {
    let document = json!({
        "body": "[otro](tisty:cited/0)",
        "lang": "es",
        "cited": [{"document": "d", "key": "k"}],
        "languages": [{"lang": "en", "document": "e"}],
    });

    let told = received(&document, &[], &[], &mut kept_as_named);

    assert_eq!(told.body, "[otro](tisty:doc/folio-cited-0)");
    let carried: serde_json::Value = serde_json::from_str(&told.carried.unwrap()).unwrap();
    assert_eq!(carried["cited"][0]["document"], "d");
    assert_eq!(carried["languages"][0]["lang"], "en");
    assert_eq!(carried["lang"], "es");
}

#[test]
fn a_document_with_nothing_beyond_itself_carries_nothing() {
    let told = received(&json!({"body": "x"}), &[], &[], &mut kept_as_named);

    assert_eq!(told.carried, None);
}
