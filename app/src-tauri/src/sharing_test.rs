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
                password: 4,
                password_most: 8,
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
        let revoked = said
            .headers
            .iter()
            .find(|(name, _)| *name == "Tisty-Revoked")
            .map(|(_, value)| value.clone());
        Ok(Said {
            status: said.status,
            body: said.body,
            revoked,
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

/// The server as it is, except that it never takes a password.
struct Ungated<'a>(&'a Here);

impl Wire for Ungated<'_> {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal> {
        if asked.path.ends_with("/manage") && asked.body.starts_with(b"{\"do\":\"password\"") {
            return Err(Refusal::of("shareOffline"));
        }
        self.0.send(asked)
    }
}

#[test]
fn a_share_that_could_not_take_its_password_does_not_stay_up() {
    let here = Here::new();
    let chosen = Chosen {
        password: Some("clave".into()),
        ..Chosen::default()
    };
    let refused = share(&Ungated(&here), SERVER, "Hola", &chosen, &voucher(), None).unwrap_err();
    assert_eq!(refused.code, "shareOffline");

    assert_eq!(here.share.borrow().store.envelopes(), 0);
}

#[test]
fn a_document_over_the_server_s_limits_says_it_is_too_big() {
    let here = Here::new();
    let long = "a".repeat(Limits::default().text as usize + 1);

    let refused = share(&here, SERVER, &long, &Chosen::default(), &voucher(), None).unwrap_err();
    assert_eq!(refused.code, "shareTooBig");
}

/// Run by hand after a Tisty Share deploy: a share goes up to tisty.cl, comes down whole behind its password, and stops.
/// With `TISTY_SHARE_KEEP=1` it goes up without a password, stays up and prints its link, to be opened in a browser.
#[test]
#[ignore = "reaches tisty.cl"]
fn a_share_goes_up_to_tisty_cl_and_comes_down_whole() {
    let wire = Net::to(SERVER).unwrap();
    let key = ShareKey::from_bytes(&[5; 32]);
    let since = jiff::Timestamp::now() - jiff::SignedDuration::from_hours(24);
    let certificate = Seed::from_bytes(&[8; 32])
        .root()
        .certify(&key.public(), since);
    let voucher = (key, certificate);
    let keep = std::env::var("TISTY_SHARE_KEEP").is_ok();
    let password = (!keep).then_some("perro azul");
    let chosen = Chosen {
        password: password.map(str::to_owned),
        editable: false,
        idle_days: Some(7),
    };
    let text = "# Prueba de Tisty Share\n\nEscrito desde la prueba que llega a tisty.cl.";
    let shared = share(&wire, SERVER, text, &chosen, &voucher, Some("Prueba")).unwrap();

    let said = info(&wire, &shared).unwrap();
    assert_eq!(said.gated, !keep);
    assert!(!said.paused && !said.blocked);
    let mut asking =
        reqwest::blocking::Client::new().get(format!("{SERVER}{API}docs/{}", shared.link));
    if let Some(password) = password {
        asking = asking.header("Tisty-Password", URL_SAFE_NO_PAD.encode(password));
    }
    let read = asking.send().unwrap();
    assert_eq!(read.status().as_u16(), 200);
    let envelope = Envelope::read(read.bytes().unwrap().to_vec(), &Limits::default()).unwrap();
    let opened = envelope::open(&LinkKey::from_text(&shared.key).unwrap(), &envelope, &[]).unwrap();
    assert_eq!(opened.text.as_str(), text);
    assert!(matches!(opened.wrote, Some(Verdict::Holds { .. })));

    if keep {
        println!("{}", links(&shared).0);
        return;
    }
    stop(&wire, &shared).unwrap();
    assert!(already_gone(&info(&wire, &shared).unwrap_err()));
}
