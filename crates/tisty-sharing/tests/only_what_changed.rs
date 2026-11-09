//! A Sync sends again only the envelopes whose words moved, and whatever names one of them, since every one Folio stores is paid for.

#[allow(dead_code)]
mod common;

use std::collections::BTreeSet;

use common::{Home, asked};
use tisty_core::model::DocId;
use tisty_sharing::{Asked, Sealed};

fn home() -> (tempfile::TempDir, Home) {
    let tmp = tempfile::tempdir().unwrap();
    let mut home = Home::at(tmp.path());
    home.with_a_name();
    (tmp, home)
}

fn sealed(home: &Home, asking: &Asked) -> Sealed {
    home.sealed(home.gathered(asking), Vec::new())
}

fn shared(home: &mut Home, asking: &Asked) {
    let first = sealed(home, asking);
    home.sent(&first.remembering);
}

fn rewritten(home: &Home, doc: DocId, body: &str) {
    let file = &home.state.docs[&doc].file;
    tisty_core::docs::write(&home.paths.docs(), file, body).unwrap();
}

fn going(home: &Home, asking: &Asked) -> BTreeSet<String> {
    let out = sealed(home, asking).only_what_changed();
    let remembered: BTreeSet<String> = out
        .remembering
        .iter()
        .map(|(file, _)| file.clone())
        .collect();
    let sent: BTreeSet<String> = out.going.iter().map(|one| one.id.clone()).collect();
    let ids: BTreeSet<String> = out
        .remembering
        .iter()
        .map(|(_, one)| one.id.clone())
        .collect();
    assert_eq!(sent, ids, "only what goes is remembered as sent");
    remembered
}

fn files(home: &Home, docs: &[DocId]) -> BTreeSet<String> {
    docs.iter()
        .map(|doc| home.state.docs[doc].file.clone())
        .collect()
}

fn two_languages(home: &mut Home) -> (DocId, DocId) {
    let es = home.wrote("---\nlang: es\n---\n\n# Paseos\n\nUno.\n", None);
    let en = home.wrote("---\nlang: en\n---\n\n# Walks\n\nOne.\n", None);
    home.joined(en, es);
    (es, en)
}

#[test]
fn a_first_share_sends_every_member_and_a_sync_with_no_change_sends_none() {
    let (_tmp, mut home) = home();
    let (es, en) = two_languages(&mut home);

    assert!(!home.gathered(&asked(es)).same_as_sent());
    assert_eq!(going(&home, &asked(es)), files(&home, &[es, en]));

    shared(&mut home, &asked(es));
    assert!(home.gathered(&asked(es)).same_as_sent());
    assert!(going(&home, &asked(es)).is_empty());
}

#[test]
fn of_two_languages_only_the_one_that_changed_goes_again() {
    let (_tmp, mut home) = home();
    let (es, en) = two_languages(&mut home);
    shared(&mut home, &asked(es));

    rewritten(&home, en, "---\nlang: en\n---\n\n# Walks\n\nOne and two.\n");
    assert!(!home.gathered(&asked(es)).same_as_sent());
    assert_eq!(going(&home, &asked(es)), files(&home, &[en]));

    rewritten(&home, en, "---\nlang: en\n---\n\n# Walks\n\nOne.\n");
    rewritten(&home, es, "---\nlang: es\n---\n\n# Paseos\n\nUno y dos.\n");
    assert_eq!(going(&home, &asked(es)), files(&home, &[es]));
}

#[test]
fn the_versions_that_go_again_follow_on_from_their_own() {
    let (_tmp, mut home) = home();
    let (es, en) = two_languages(&mut home);
    shared(&mut home, &asked(es));
    rewritten(&home, en, "---\nlang: en\n---\n\n# Walks\n\nOne and two.\n");

    let out = sealed(&home, &asked(es)).only_what_changed();
    home.sent(&out.remembering);

    let shelf = tisty_sharing::remembered(&home.state);
    assert_eq!(shelf[&home.state.docs[&en].file].version, 2);
    assert_eq!(
        shelf[&home.state.docs[&es].file].version, 1,
        "what did not go keeps the version Folio holds"
    );
    assert!(home.gathered(&asked(es)).same_as_sent());
}

#[test]
fn a_language_whose_title_changed_takes_its_anchor_along_since_the_map_names_it() {
    let (_tmp, mut home) = home();
    let (es, en) = two_languages(&mut home);
    shared(&mut home, &asked(es));

    rewritten(&home, en, "---\nlang: en\n---\n\n# Strolls\n\nOne.\n");
    assert_eq!(going(&home, &asked(es)), files(&home, &[es, en]));
}

#[test]
fn a_new_paper_on_the_anchor_goes_to_every_language_since_they_all_wear_it() {
    let (_tmp, mut home) = home();
    let (es, en) = two_languages(&mut home);
    shared(&mut home, &asked(es));

    rewritten(
        &home,
        es,
        "---\nlang: es\npaper: letter\n---\n\n# Paseos\n\nUno.\n",
    );
    assert_eq!(going(&home, &asked(es)), files(&home, &[es, en]));
}

#[test]
fn a_new_language_goes_with_its_anchor_and_leaves_the_others_where_they_are() {
    let (_tmp, mut home) = home();
    let (es, en) = two_languages(&mut home);
    shared(&mut home, &asked(es));

    let pt = home.wrote("---\nlang: pt\n---\n\n# Passeios\n\nUm.\n", None);
    home.joined(pt, es);
    let now = going(&home, &asked(es));
    assert_eq!(now, files(&home, &[es, pt]));
    assert!(!now.contains(&home.state.docs[&en].file));
}

fn citing(home: &mut Home) -> (DocId, DocId, Asked) {
    let other = home.wrote("# Otro\n\nAlgo aparte.\n", None);
    let doc = home.wrote("# Paseos\n\n[[Otro]]\n", None);
    let mut asking = asked(doc);
    asking.cited = vec![other];
    (doc, other, asking)
}

#[test]
fn a_cited_document_that_did_not_change_is_not_sent_again() {
    let (_tmp, mut home) = home();
    let (doc, other, asking) = citing(&mut home);
    assert_eq!(going(&home, &asking), files(&home, &[doc, other]));
    shared(&mut home, &asking);

    rewritten(&home, doc, "# Paseos\n\n[[Otro]], otra vez.\n");
    assert_eq!(going(&home, &asking), files(&home, &[doc]));
}

#[test]
fn a_cited_document_that_changed_goes_with_whoever_shows_its_title() {
    let (_tmp, mut home) = home();
    let (doc, other, asking) = citing(&mut home);
    shared(&mut home, &asking);

    rewritten(&home, other, "# Otro\n\nAlgo aparte, y más.\n");
    assert_eq!(going(&home, &asking), files(&home, &[doc, other]));
}
