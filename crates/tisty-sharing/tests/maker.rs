//! What Tisty sows goes out only from the Tisty whose machine key the maker's root certified, and always under the maker's alias.

#[allow(dead_code)]
mod common;

use common::{Home, asked};
use tisty_core::event::{DocAdd, Said, Signature};
use tisty_core::model::DocId;
use tisty_core::untouched::SOWN_BY;
use tisty_core::{Op, docs};
use tisty_share_proto::seed::Seed;
use tisty_sharing::{Finished, Sealed, Told, Unshared, signs_for, sown_by_tisty};

const MAKER_ALIAS: &str = "rgdevment";

fn home() -> (tempfile::TempDir, Home) {
    let tmp = tempfile::tempdir().unwrap();
    let mut home = Home::at(tmp.path());
    home.with_a_name();
    home.commit(Op::Signed {
        d: Signature {
            alias: Some(MAKER_ALIAS.into()),
            ..Default::default()
        },
    });
    (tmp, home)
}

fn root_of_this_home() -> String {
    Seed::from_bytes(&[0x33; 32]).root().person().shown()
}

fn another_root() -> String {
    Seed::from_bytes(&[0x44; 32]).root().person().shown()
}

fn added(home: &mut Home, body: &str, by: &str, page_of: Option<DocId>) -> DocId {
    let made = docs::create(&home.paths.docs(), &home.device, body).unwrap();
    let id = ulid::Ulid::generate();
    home.commit(Op::DocAdd {
        id,
        d: DocAdd {
            guest: true,
            by: Some(by.into()),
            file: made.id,
            order: format!("s{id}"),
            said: Some(Said::of(body)),
            page_of,
            ..Default::default()
        },
    });
    id
}

fn sown_guide(home: &mut Home) -> DocId {
    let leaf = home.attached("atajos.pdf", b"%PDF atajos");
    let guide = added(
        home,
        &format!("# Guía\n\nEmpieza aquí.\n\n[Atajos]({leaf})\n"),
        SOWN_BY,
        None,
    );
    added(
        home,
        "# Primeros pasos\n\nUna tarea.\n",
        SOWN_BY,
        Some(guide),
    );
    guide
}

fn sealed(home: &Home, asking: &tisty_sharing::Asked) -> Sealed {
    let gathered = home.gathered(asking);
    let answers = gathered
        .unfinished()
        .into_iter()
        .map(|one| Finished {
            name: one.name,
            html: one.html,
            not_drawn: 0,
        })
        .collect();
    home.sealed(gathered, answers)
}

fn refused(home: &Home, doc: DocId) -> Option<Unshared> {
    let desk = tisty_sharing::Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
        maker: home.maker,
    };
    tisty_sharing::gathered(&desk, &asked(doc)).err()
}

#[test]
fn only_a_machine_key_the_root_certified_answers_for_that_root() {
    let (_tmp, home) = home();
    assert!(signs_for(
        &home.paths,
        &home.state,
        &home.device,
        &root_of_this_home()
    ));
    assert!(!signs_for(
        &home.paths,
        &home.state,
        &home.device,
        &another_root()
    ));
    assert!(!signs_for(
        &home.paths,
        &home.state,
        &home.device,
        MAKER_ALIAS
    ));

    let tmp = tempfile::tempdir().unwrap();
    let nameless = Home::at(tmp.path());
    assert!(!signs_for(
        &nameless.paths,
        &nameless.state,
        &nameless.device,
        &root_of_this_home()
    ));
}

#[test]
fn under_another_root_what_tisty_sowed_stays_tistys_and_is_not_shared() {
    let (_tmp, mut home) = home();
    let guide = sown_guide(&mut home);
    assert!(matches!(refused(&home, guide), Some(Unshared::NotOwn)));
}

#[test]
fn the_maker_shares_what_tisty_sowed_with_its_pages_and_files() {
    let (_tmp, mut home) = home();
    home.maker = true;
    let guide = sown_guide(&mut home);

    let out = sealed(&home, &asked(guide));
    assert!(
        !out.told
            .iter()
            .any(|one| matches!(one, Told::PageLeftBehind { .. })),
        "{:?}",
        out.told
    );
    let (_, went) = &out.remembering[0];
    assert!(went.files_hash.is_some(), "its attachment goes with it");
    assert_eq!(went.signed_as.as_deref(), Some(MAKER_ALIAS));
}

#[test]
fn what_tisty_sowed_never_goes_out_signed_as_tisty() {
    let (_tmp, mut home) = home();
    home.maker = true;
    let guide = sown_guide(&mut home);
    let mut asking = asked(guide);
    asking.signed_as = Some(SOWN_BY.into());

    let out = sealed(&home, &asking);
    assert_eq!(out.remembering[0].1.signed_as.as_deref(), Some(MAKER_ALIAS));
}

#[test]
fn the_maker_still_cannot_share_what_someone_else_wrote() {
    let (_tmp, mut home) = home();
    home.maker = true;
    let theirs = added(&mut home, "# De Ana\n", "Ana", None);
    assert!(matches!(refused(&home, theirs), Some(Unshared::NotOwn)));
}

#[test]
fn what_tisty_sowed_is_still_tistys_for_duplicating_even_for_the_maker() {
    let (_tmp, mut home) = home();
    home.maker = true;
    let guide = sown_guide(&mut home);
    let me = home.state.persons.person(&home.device);
    let kept = &home.state.docs[&guide];

    assert!(sown_by_tisty(kept));
    assert!(home.state.theirs(kept, me.as_ref()));
    assert!(home.state.theirs(kept, None));
    assert!(refused(&home, guide).is_none());
}

#[test]
fn a_new_guide_text_goes_up_as_a_new_version_of_the_same_shared_document() {
    let (_tmp, mut home) = home();
    home.maker = true;
    let guide = sown_guide(&mut home);
    let first = sealed(&home, &asked(guide));
    home.sent(&first.remembering);
    let was = first.remembering[0].1.clone();

    let file = home.state.docs[&guide].file.clone();
    let body = docs::read(&home.paths.docs(), &file).unwrap();
    docs::write(
        &home.paths.docs(),
        &file,
        &body.replace("Empieza aquí.", "Empieza aquí, con la versión nueva."),
    )
    .unwrap();

    let second = sealed(&home, &asked(guide));
    let now = &second.remembering[0].1;
    assert_eq!(now.id, was.id);
    assert_eq!(now.version, was.version + 1);
    assert_eq!(now.signed_as.as_deref(), Some(MAKER_ALIAS));
}
