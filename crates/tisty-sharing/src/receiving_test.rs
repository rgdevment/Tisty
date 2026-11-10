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
fn an_anchors_reviews_come_along_and_a_later_version_replaces_them() {
    let document = json!({ "body": "x", "reviews": [{ "seq": 1 }] });

    let told = received(&document, &[], &[], &mut kept_as_named);

    let carried: serde_json::Value = serde_json::from_str(&told.carried.unwrap()).unwrap();
    assert_eq!(carried["reviews"][0]["seq"], 1);
    let later = r#"{"reviews":[{"seq":2}]}"#;
    let kept: serde_json::Value =
        serde_json::from_str(&super::map_brought(Some(&carried.to_string()), Some(later)).unwrap())
            .unwrap();
    assert_eq!(kept["reviews"][0]["seq"], 2);
    let gone: serde_json::Value =
        serde_json::from_str(&super::map_brought(Some(later), Some(r#"{"lang":"es"}"#)).unwrap())
            .unwrap();
    assert!(gone.get("reviews").is_none());
}

#[test]
fn a_document_with_nothing_beyond_itself_carries_nothing() {
    let told = received(&json!({"body": "x"}), &[], &[], &mut kept_as_named);

    assert_eq!(told.carried, None);
}

#[test]
fn another_version_brings_its_editors_and_leaves_what_this_copy_carries() {
    let theirs = r#"{"lang":"en","languages":[{"lang":"de"}],"editors":[{"name":"Ana"}]}"#;
    let mine = r#"{"lang":"es","languages":[{"lang":"en"}],"cited":null}"#;

    let kept: serde_json::Value =
        serde_json::from_str(&super::editors_brought(Some(mine), Some(theirs)).unwrap()).unwrap();
    assert_eq!(kept["lang"], "es");
    assert_eq!(kept["languages"][0]["lang"], "en");
    assert_eq!(kept["editors"][0]["name"], "Ana");

    let owners: serde_json::Value =
        serde_json::from_str(&super::editors_brought(None, Some(theirs)).unwrap()).unwrap();
    assert_eq!(
        owners,
        serde_json::json!({ "editors": [{ "name": "Ana" }] })
    );

    assert_eq!(
        super::editors_brought(Some(mine), None).as_deref(),
        Some(mine)
    );
    assert_eq!(super::editors_brought(None, Some(r#"{"lang":"en"}"#)), None);
}

#[test]
fn the_owner_joins_the_languages_others_added_by_id_and_never_by_lang() {
    let mine = r#"{"lang":"es","languages":[{"lang":"en","document":"marioEn1"},{"lang":"fr","document":"marioFr1"}],"cited":[{"document":"c"}]}"#;
    let theirs = r#"{"languages":[{"lang":"en","document":"anaEngl1"},{"lang":"fr","document":"marioFr1","title":"Nuevo"}]}"#;

    let joined: serde_json::Value =
        serde_json::from_str(&super::languages_joined(Some(mine), Some(theirs)).unwrap()).unwrap();

    let documents: Vec<&str> = joined["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["document"].as_str().unwrap())
        .collect();
    assert_eq!(documents, ["marioEn1", "marioFr1", "anaEngl1"]);
    assert_eq!(joined["languages"][1]["title"], "Nuevo");
    assert_eq!(joined["cited"][0]["document"], "c");
    let untouched = super::languages_joined(Some(mine), Some(r#"{"lang":"de"}"#)).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&untouched).unwrap(),
        serde_json::from_str::<serde_json::Value>(mine).unwrap()
    );
}

#[test]
fn the_reviews_a_map_carries_are_written_and_read_back_and_none_leaves_no_field() {
    let review: tisty_contracts::Review = serde_json::from_value(serde_json::json!({
        "of": "o", "kept": false, "seq": 2, "root": "r", "machine": "m",
        "since": "2026-11-03T12:00:00Z", "certificate": "c", "signed": "s",
    }))
    .unwrap();

    let with = super::with_reviews(Some(r#"{"lang":"es"}"#), std::slice::from_ref(&review));

    assert_eq!(super::reviews_carried(with.as_deref()), [review]);
    let without: serde_json::Value =
        serde_json::from_str(&super::with_reviews(with.as_deref(), &[]).unwrap()).unwrap();
    assert!(without.get("reviews").is_none());
    assert_eq!(without["lang"], "es");
}
