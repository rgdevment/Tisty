use tisty_core::Op;
use tisty_core::event::Shared;
use tisty_core::model::DocId;
use tisty_share_proto::envelope::{self, Envelope, LinkKey};
use tisty_share_proto::limits::Limits;
use tisty_share_server::store::Store;

use super::{
    CITED_AT_MOST, Citable, Going, Signing, Spent, cited_in, held_here, held_with, let_go_of,
    printed, reach_among, took_up,
};
use crate::Session;
use crate::sharing::here::{Here, voucher};
use crate::sharing::{self, Chosen, SERVER, Words};

fn going(file: &str, title: &str, page: Option<(u32, u32)>, text: &str) -> Going {
    Going {
        citable: Citable {
            doc: ulid::Ulid::generate(),
            file: file.into(),
            title: title.into(),
            page,
        },
        was: None,
        text: text.into(),
    }
}

fn signing() -> Signing {
    Signing {
        voucher: voucher(),
        name: Some("Ana".into()),
        data: std::env::temp_dir(),
    }
}

// What a link opens, with the key a reader holds for it.
fn opened(here: &Here, shared: &Shared) -> envelope::Opened {
    let (status, body) = here.read(&shared.link, None);
    assert_eq!(status, 200, "{} did not read", shared.link);
    let arrived = Envelope::read(body, &Limits::default()).unwrap();
    envelope::open(&LinkKey::from_text(&shared.key).unwrap(), &arrived, &[]).unwrap()
}

#[test]
fn a_share_puts_up_what_it_cites_holds_it_and_names_each_with_its_key() {
    let here = Here::new();
    let root = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    let going = [
        going(
            "guia-2.md",
            "Instalar",
            Some((2, 2)),
            "Antes, ![las notas](tisty:doc/notas.md)",
        ),
        going("notas.md", "Notas", None, "Unas notas"),
    ];
    let members = took_up(&here, &root, &going, &signing(), 10, &mut Spent::default()).unwrap();
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|(_, shared)| shared.held));

    let text = "![Instalar](tisty:doc/guia-2.md)\n\nY nada más.";
    let cited = cited_in(text, &members);
    let words = Words {
        text,
        cited: &cited,
        pictures: &[],
        voucher: &signing().voucher,
        name: None,
    };
    sharing::first_up(&here, &root, &words, &Chosen::default()).unwrap();

    let read = opened(&here, &root);
    assert_eq!(read.cited.len(), 1);
    assert_eq!(read.cited[0].doc, "guia-2.md");
    assert_eq!(read.cited[0].page, Some((2, 2)));
    let page = &members[0].1;
    assert_eq!(read.cited[0].id, page.link);
    // A cited page names what it cites in turn, all of it held by the one share.
    let page_read = opened(&here, page);
    assert_eq!(page_read.cited[0].id, members[1].1.link);
    assert_eq!(
        page_read.text.as_str(),
        "Antes, ![las notas](tisty:doc/notas.md)"
    );
    let holds = done(here.share.borrow().store.doc(&root.link))
        .unwrap()
        .unwrap()
        .holds;
    assert_eq!(holds, vec![page.link.clone(), members[1].1.link.clone()]);

    // Stopped, it takes what went only through it along.
    sharing::stop(&here, &root).unwrap();
    assert_eq!(here.read(&page.link, None).0, 404);
}

#[test]
fn a_document_shared_on_its_own_is_held_as_it_is_and_never_shared_again() {
    let here = Here::new();
    let root = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    let own = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    let words = Words {
        text: "Mío",
        cited: &[],
        pictures: &[],
        voucher: &signing().voucher,
        name: None,
    };
    sharing::first_up(&here, &own, &words, &Chosen::default()).unwrap();
    let mut one = going("mio.md", "Mío", None, "Mío");
    one.was = Some(own.clone());
    let members = took_up(&here, &root, &[one], &signing(), 10, &mut Spent::default()).unwrap();
    assert_eq!(members[0].1.link, own.link);

    // Let go, it keeps the life it had on its own.
    sharing::stop(&here, &root).unwrap();
    assert_eq!(opened(&here, &own).text.as_str(), "Mío");
}

#[test]
fn a_share_that_could_not_put_up_what_it_cites_leaves_none_of_it_behind() {
    let here = Here::new();
    let root = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    // Past what any envelope may say, so the second one cannot go up.
    let huge = "x".repeat(Limits::default().text as usize + 1);
    let going = [
        going("a.md", "A", None, "A"),
        going("b.md", "B", None, &huge),
    ];
    assert!(took_up(&here, &root, &going, &signing(), 10, &mut Spent::default()).is_err());
    // What went up and was held before the failure is let go and stopped, never left alive through the holder.
    assert_eq!(here.share.borrow().store.envelopes(), 0);
    let holds = done(here.share.borrow().store.doc(&root.link))
        .unwrap()
        .unwrap()
        .holds;
    assert!(holds.is_empty());
}

#[test]
fn words_name_only_what_goes_with_them_and_a_print_changes_with_what_they_cite() {
    let among = vec![(
        Citable {
            doc: ulid::Ulid::generate(),
            file: "notas.md".into(),
            title: "Notas".into(),
            page: None,
        },
        Shared {
            server: SERVER.into(),
            link: "nts9Qe".into(),
            key: LinkKey::from_bytes(&[4; 32]).to_text().to_string(),
            edit: "edit".into(),
            admin: "admin".into(),
            editable: false,
            gated: false,
            version: 1,
            still: false,
            taken: false,
            print: String::new(),
            signs: Default::default(),
            holds: Vec::new(),
            held: true,
            cites: false,
        },
    )];
    let text = "![Notas](tisty:doc/notas.md) y ![Otra](tisty:doc/otra.md)";
    let cited = cited_in(text, &among);
    assert_eq!(cited.len(), 1);
    assert_eq!(cited[0].title, "Notas");
    assert_eq!(printed(text, &[]), sharing::print_of(text));
    assert_ne!(printed(text, &cited), sharing::print_of(text));
}

fn done<T>(work: impl std::future::Future<Output = T>) -> T {
    tisty_share_server::testing::done(work)
}

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    (tmp, Session::at(paths).unwrap())
}

fn filed(session: &mut Session, body: &str) -> DocId {
    filed_in(session, body, None)
}

fn filed_in(session: &mut Session, body: &str, page_of: Option<DocId>) -> DocId {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    let id = ulid::Ulid::generate();
    session
        .commit(Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: made.id.clone(),
                order: made.id,
                folder: None,
                page_of,
                task_of: None,
            },
        })
        .unwrap();
    id
}

fn shared_as(session: &mut Session, doc: DocId, link: &str, holds: Vec<DocId>, held: bool) {
    let d = Shared {
        server: SERVER.into(),
        link: link.into(),
        key: LinkKey::from_bytes(&[4; 32]).to_text().to_string(),
        edit: "edit".into(),
        admin: "admin".into(),
        editable: false,
        gated: false,
        version: 1,
        still: false,
        taken: false,
        print: String::new(),
        signs: Default::default(),
        holds,
        held,
        cites: false,
    };
    session.commit(Op::ShareDoc { id: doc, d }).unwrap();
}

#[test]
fn a_document_two_shares_hold_leads_only_where_both_lead_and_goes_only_when_neither_holds_it() {
    let (_tmp, mut session) = session();
    let open = filed(&mut session, "# Abierto");
    let gated = filed(&mut session, "# Con clave");
    let page = filed(&mut session, "# Página");
    let only_gated = filed(&mut session, "# Solo con clave");
    shared_as(&mut session, open, "openAa", vec![page], false);
    shared_as(&mut session, gated, "gateBb", vec![page, only_gated], false);
    shared_as(&mut session, page, "pageCc", Vec::new(), true);
    shared_as(&mut session, only_gated, "onlyDd", Vec::new(), true);

    // The open share's readers reach the page, and never what only the gated share chose to take along.
    let led: Vec<DocId> = held_with(&session, page)
        .into_iter()
        .map(|(one, _)| one.doc)
        .collect();
    assert_eq!(led, vec![page]);
    assert_eq!(held_with(&session, gated).len(), 2);

    // Stopped, the gated share forgets what went only through it; the page another share holds stays.
    let was = session.state.shares[&gated].clone();
    let ops = let_go_of(&session, gated, &was, &[]);
    assert_eq!(
        ops,
        vec![Op::ShareStop {
            id: only_gated,
            link: "onlyDd".into()
        }]
    );
    assert!(held_here(&session, page) && !held_here(&session, open));
}

#[test]
fn a_share_takes_as_many_as_its_hour_has_room_for_and_leaves_the_rest_for_later() {
    let here = Here::new();
    let root = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    let going = [going("a.md", "A", None, "A"), going("b.md", "B", None, "B")];
    let mut spent = Spent::default();
    let members = took_up(&here, &root, &going, &signing(), 1, &mut spent).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].0.file, "a.md");
    assert_eq!(
        spent,
        Spent {
            reserves: 1,
            uploads: 1
        }
    );
}

#[test]
fn a_document_the_server_let_go_of_is_shared_anew_and_the_rest_held_as_they_are() {
    let here = Here::new();
    let root = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    let lost = sharing::reserve(&here, SERVER, &Chosen::default()).unwrap();
    sharing::stop(&here, &lost).unwrap();
    let mut one = going("perdido.md", "Perdido", None, "Otra vez");
    one.was = Some(Shared {
        held: true,
        ..lost.clone()
    });
    let members = took_up(&here, &root, &[one], &signing(), 1, &mut Spent::default()).unwrap();
    assert_ne!(members[0].1.link, lost.link);
    assert_eq!(opened(&here, &members[0].1).text.as_str(), "Otra vez");
}

// A document whose body names each of its pages with a card, as Tisty writes them.
fn paged(session: &mut Session, pages: &[&str]) -> (DocId, Vec<DocId>) {
    let root = filed(session, "# Libro");
    let made: Vec<DocId> = pages
        .iter()
        .map(|body| filed_in(session, body, Some(root)))
        .collect();
    let cards: Vec<String> = made
        .iter()
        .map(|one| format!("![Página](tisty:doc/{})", session.state.docs[one].file))
        .collect();
    let file = session.state.docs[&root].file.clone();
    tisty_core::docs::write(
        &session.paths.docs(),
        &file,
        &format!(
            "# Libro

{}
",
            cards.join(
                "

"
            )
        ),
    )
    .unwrap();
    (root, made)
}

#[test]
fn a_document_with_more_pages_than_one_round_takes_still_reaches_them_all() {
    let (_tmp, mut session) = session();
    let bodies: Vec<String> = (1..=12).map(|n| format!("# Capítulo {n}")).collect();
    let pages: Vec<&str> = bodies.iter().map(String::as_str).collect();
    let (root, made) = paged(&mut session, &pages);
    let reached = reach_among(&session, root, &|_| true).unwrap();
    assert_eq!(reached.pages.len(), 12);
    assert_eq!(reached.pages[0].doc, made[0]);
    assert_eq!(reached.pages[11].page, Some((12, 12)));
    assert!(reached.cited.is_empty());
}

#[test]
fn a_page_with_no_words_yet_is_named_by_its_file_and_past_an_envelope_s_list_is_refused() {
    let (_tmp, mut session) = session();
    let (root, made) = paged(&mut session, &[""]);
    let page = &reach_among(&session, root, &|_| true).unwrap().pages[0];
    assert_eq!(page.title, session.state.docs[&made[0]].file);

    let bodies: Vec<String> = (0..=CITED_AT_MOST).map(|n| format!("# {n}")).collect();
    let pages: Vec<&str> = bodies.iter().map(String::as_str).collect();
    let (book, _) = paged(&mut session, &pages);
    assert_eq!(
        reach_among(&session, book, &|_| true).unwrap_err().code,
        "shareTooManyCited"
    );
}
