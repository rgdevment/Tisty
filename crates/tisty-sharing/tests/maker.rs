//! What Tisty sows goes out only from the Tisty whose machine key the maker's root certified, and always under the maker's alias.

#[allow(dead_code)]
mod common;

use common::{Home, asked};
use tisty_core::event::{DocAdd, FolderAdd, Said, Signature};
use tisty_core::model::{DocId, FolderId};
use tisty_core::untouched::SOWN_BY;
use tisty_core::{Op, docs};
use tisty_share_proto::identity::Person;
use tisty_share_proto::seed::Seed;
use tisty_sharing::{Finished, Sealed, Told, Unshared, signs_for};

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

fn the_root() -> Person {
    Seed::from_bytes(&[0x33; 32]).root().person()
}

fn root_of_this_home() -> String {
    the_root().shown()
}

fn another_root() -> String {
    Seed::from_bytes(&[0x44; 32]).root().person().shown()
}

fn guide_folder(home: &mut Home) -> FolderId {
    let id = ulid::Ulid::generate();
    home.commit(Op::FolderAdd {
        id,
        d: FolderAdd {
            name: "Guía".into(),
            order: "a0".into(),
            parent: None,
            icon: None,
            color: None,
        },
    });
    id
}

fn added(
    home: &mut Home,
    body: &str,
    by: &str,
    folder: Option<FolderId>,
    page_of: Option<DocId>,
    sown: bool,
) -> DocId {
    let made = docs::create(&home.paths.docs(), &home.device, body).unwrap();
    let id = ulid::Ulid::generate();
    home.commit(Op::DocAdd {
        id,
        d: DocAdd {
            guest: true,
            sown,
            by: Some(by.into()),
            file: made.id,
            order: format!("s{id}"),
            said: Some(Said::of(body)),
            folder,
            page_of,
            ..Default::default()
        },
    });
    id
}

fn sown_guide(home: &mut Home) -> DocId {
    let leaf = home.attached("atajos.pdf", b"%PDF atajos");
    let folder = Some(guide_folder(home));
    let guide = added(
        home,
        &format!("# Guía\n\nEmpieza aquí.\n\n[Atajos]({leaf})\n"),
        SOWN_BY,
        folder,
        None,
        true,
    );
    added(
        home,
        "# Primeros pasos\n\nUna tarea.\n",
        SOWN_BY,
        folder,
        Some(guide),
        true,
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
    let signs = |home: &Home, root: &str| signs_for(&home.paths, &home.state, &home.device, root);
    assert_eq!(signs(&home, &root_of_this_home()), Some(the_root()));
    assert_eq!(signs(&home, &another_root()), None);
    assert_eq!(signs(&home, MAKER_ALIAS), None);

    let tmp = tempfile::tempdir().unwrap();
    let nameless = Home::at(tmp.path());
    assert_eq!(signs(&nameless, &root_of_this_home()), None);
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
    home.maker = Some(the_root());
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
    home.maker = Some(the_root());
    let guide = sown_guide(&mut home);
    let mut asking = asked(guide);
    asking.signed_as = Some(SOWN_BY.into());

    let out = sealed(&home, &asking);
    assert_eq!(out.remembering[0].1.signed_as.as_deref(), Some(MAKER_ALIAS));
}

#[test]
fn the_maker_still_cannot_share_what_someone_else_wrote() {
    let (_tmp, mut home) = home();
    home.maker = Some(the_root());
    let theirs = added(&mut home, "# De Ana\n", "Ana", None, None, false);
    assert!(matches!(refused(&home, theirs), Some(Unshared::NotOwn)));
}

#[test]
fn what_tisty_sowed_is_still_tistys_for_duplicating_even_for_the_maker() {
    let (_tmp, mut home) = home();
    home.maker = Some(the_root());
    let guide = sown_guide(&mut home);
    let me = home.state.persons.person(&home.device);
    let kept = &home.state.docs[&guide];

    assert!(kept.sown);
    assert!(home.state.theirs(kept, me.as_ref()));
    assert!(home.state.theirs(kept, None));
    assert!(refused(&home, guide).is_none());
}

#[test]
fn a_new_guide_text_goes_up_as_a_new_version_of_the_same_shared_document() {
    let (_tmp, mut home) = home();
    home.maker = Some(the_root());
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

#[test]
fn a_document_someone_signed_as_tisty_is_still_theirs_for_the_maker() {
    let (_tmp, mut home) = home();
    home.maker = Some(the_root());
    let received = added(
        &mut home,
        "# Guía\n\nOtra cosa.\n",
        SOWN_BY,
        None,
        None,
        false,
    );

    assert!(!home.state.docs[&received].sown);
    assert!(matches!(refused(&home, received), Some(Unshared::NotOwn)));
}

#[test]
fn a_sown_guide_cited_from_the_makers_own_document_goes_under_the_alias_too() {
    let (_tmp, mut home) = home();
    home.maker = Some(the_root());
    let guide = sown_guide(&mut home);
    let title = home.state.docs[&guide].title.clone().unwrap();
    let mine = home.wrote(&format!("# Mía\n\n[[{title}]]\n"), None);
    let mut asking = asked(mine);
    asking.cited = vec![guide];
    asking.signed_as = Some(SOWN_BY.into());

    let out = sealed(&home, &asking);
    assert_eq!(out.remembering.len(), 2);
    assert!(
        out.remembering
            .iter()
            .all(|(_, one)| one.signed_as.as_deref() == Some(MAKER_ALIAS))
    );
}

#[test]
fn the_sown_mark_from_a_machine_of_another_root_is_not_the_makers() {
    let (_tmp, mut home) = home();
    home.maker = Some(the_root());
    let body = "# Guía\n\nDe otra máquina.\n";
    let made = docs::create(&home.paths.docs(), &home.device, body).unwrap();
    let id = ulid::Ulid::generate();
    home.state.apply(&tisty_core::Event::new(
        tisty_core::DeviceId("mac_ajena".into()),
        jiff::Timestamp::now(),
        Op::DocAdd {
            id,
            d: DocAdd {
                guest: true,
                sown: true,
                by: Some(SOWN_BY.into()),
                file: made.id,
                order: "z0".into(),
                said: Some(Said::of(body)),
                ..Default::default()
            },
        },
    ));

    assert!(home.state.docs[&id].sown);
    assert!(matches!(refused(&home, id), Some(Unshared::NotOwn)));
}
