use tisty_core::event::Shared;
use tisty_share_proto::envelope::{self, Envelope, LinkKey};
use tisty_share_proto::limits::Limits;
use tisty_share_server::store::Store;

use super::*;
use crate::sharing::here::{Here, voucher};
use crate::sharing::{Chosen, SERVER};

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
    let members = took_up(&here, &root, &going, &signing()).unwrap();
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
    let members = took_up(&here, &root, &[one], &signing()).unwrap();
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
    assert!(took_up(&here, &root, &going, &signing()).is_err());
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
