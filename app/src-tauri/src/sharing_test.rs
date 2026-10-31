use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr};

use tisty_share_proto::envelope::{Envelope, LinkKey};
use tisty_share_proto::identity::{ShareKey, Verdict};
use tisty_share_proto::seed::Seed;
use tisty_share_server::guard::Work;
use tisty_share_server::keys::Slowness;
use tisty_share_server::memory::Memory;
use tisty_share_server::share::{Policy, Share};
use tisty_share_server::testing::{Counting, done};
use tisty_share_worker::api::{Asked, answer};

use super::*;

const T0: i64 = 1_800_000_000;

/// A machine's key and the certificate its person gave it, as `vouched_by` hands them over.
fn voucher() -> Voucher {
    let key = ShareKey::from_bytes(&[3; 32]);
    let since = jiff::Timestamp::from_second(T0 - 86_400).unwrap();
    let certificate = Seed::from_bytes(&[7; 32])
        .root()
        .certify(&key.public(), since);
    (key, certificate)
}

/// The server's API in memory, with little work so the tests run quickly.
struct Here {
    share: RefCell<Share<Memory, Counting>>,
}

impl Here {
    fn new() -> Self {
        let policy = Policy {
            slowness: Slowness { log_n: 4, r: 8 },
            work: Work {
                reserve: 4,
                upload: 4,
                per_megabyte: 1,
                report: 4,
                lasts: 10 * 60,
            },
            ..Policy::default()
        };
        Self {
            share: RefCell::new(Share::new(Memory::default(), Counting(0), policy, [5; 32])),
        }
    }

    fn read(&self, id: &str, password: Option<&str>) -> (u16, Vec<u8>) {
        let said = done(answer(
            &mut self.share.borrow_mut(),
            Asked {
                method: "GET".into(),
                path: format!("{API}docs/{id}"),
                token: None,
                password: password.map(|one| URL_SAFE_NO_PAD.encode(one)),
                work: None,
                body: Vec::new(),
                caller: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7)),
                now: T0,
            },
        ));
        (said.status, said.body)
    }
}

impl Wire for Here {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal> {
        let said = done(answer(
            &mut self.share.borrow_mut(),
            Asked {
                method: asked.method.into(),
                path: format!("{API}{}", asked.path),
                token: asked.token.map(str::to_owned),
                password: None,
                work: asked.work,
                body: asked.body,
                caller: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
                now: T0,
            },
        ));
        Ok(Said {
            status: said.status,
            body: said.body,
        })
    }
}

#[test]
fn a_shared_document_reads_back_with_the_key_in_its_link_and_nowhere_else() {
    let here = Here::new();
    let shared = share(
        &here,
        SERVER,
        "# Hola\n\nUn texto.",
        &Chosen::default(),
        &voucher(),
        Some("Ana Pérez"),
    )
    .unwrap();
    assert_eq!(shared.version, 1);
    assert!(!shared.editable && !shared.gated);

    let (status, body) = here.read(&shared.link, None);
    assert_eq!(status, 200);
    let envelope = Envelope::read(body, &Limits::default()).unwrap();
    let key = LinkKey::from_text(&shared.key).unwrap();
    let opened = envelope::open(&key, &envelope, &[]).unwrap();
    assert_eq!(opened.text.as_str(), "# Hola\n\nUn texto.");
    let me = Seed::from_bytes(&[7; 32]).root().person();
    assert!(matches!(opened.wrote, Some(Verdict::Holds { person, .. }) if person == me));
    assert!(matches!(opened.sent, Some(Verdict::Holds { person, .. }) if person == me));
    assert_eq!(
        opened.written.and_then(|one| one.name).as_deref(),
        Some("Ana Pérez")
    );

    let (read, edit) = links(&shared);
    assert_eq!(
        read,
        format!("https://tisty.cl/{}#{}", shared.link, shared.key)
    );
    assert_eq!(edit, None);
}

#[test]
fn the_edit_link_is_the_read_link_with_its_token_after_a_dot() {
    let shared = Shared {
        server: "https://tisty.cl/".into(),
        link: "k7Qm2xPa".into(),
        key: "llave".into(),
        edit: "token".into(),
        admin: "admin".into(),
        editable: true,
        gated: false,
        version: 1,
    };
    assert_eq!(
        links(&shared),
        (
            "https://tisty.cl/k7Qm2xPa#llave".into(),
            Some("https://tisty.cl/k7Qm2xPa#llave.token".into())
        )
    );
}

#[test]
fn a_password_and_a_shorter_life_are_set_on_the_server_as_it_is_shared() {
    let here = Here::new();
    let chosen = Chosen {
        password: Some("ñandú".into()),
        editable: true,
        idle_days: Some(7),
    };
    let shared = share(&here, SERVER, "Hola", &chosen, &voucher(), None).unwrap();
    assert!(shared.gated && shared.editable);

    assert_eq!(here.read(&shared.link, None).0, 401);
    assert_eq!(here.read(&shared.link, Some("ñandú")).0, 200);
    let told = info(&here, &shared).unwrap();
    assert!(told.gated);
    assert_eq!(told.latest, Some(1));
    assert!(told.ends <= T0 + 7 * 86_400);
}

#[test]
fn stopping_leaves_the_link_leading_nowhere() {
    let here = Here::new();
    let shared = share(&here, SERVER, "Hola", &Chosen::default(), &voucher(), None).unwrap();
    stop(&here, &shared).unwrap();
    assert_eq!(here.read(&shared.link, None).0, 404);
    let refused = info(&here, &shared).unwrap_err();
    assert_eq!(refused.code, "shareRefused");
    assert_eq!(refused.name.as_deref(), Some("missing"));
}

#[test]
fn a_key_taken_back_reaches_the_server_and_shares_nothing_more() {
    let here = Here::new();
    let (key, _) = voucher();
    let at = jiff::Timestamp::from_second(T0).unwrap();
    let taken = Seed::from_bytes(&[7; 32]).root().revoke(&key.public(), at);
    revoke(&here, &[taken.clone(), taken]).unwrap();

    let refused = share(&here, SERVER, "Hola", &Chosen::default(), &voucher(), None).unwrap_err();
    assert_eq!(refused.code, "shareUnsigned");
}
