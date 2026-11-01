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
        &[],
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
        seen: None,
    };
    let shared = share(&here, SERVER, "Hola", &[], &chosen, &voucher(), None).unwrap();
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
    let shared = share(
        &here,
        SERVER,
        "Hola",
        &[],
        &Chosen::default(),
        &voucher(),
        None,
    )
    .unwrap();
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

    let refused = share(
        &here,
        SERVER,
        "Hola",
        &[],
        &Chosen::default(),
        &voucher(),
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "shareUnsigned");
}

#[test]
fn a_revocation_the_server_heard_is_not_told_again_and_the_others_are() {
    let cache = tempfile::tempdir().unwrap();
    let root = Seed::from_bytes(&[7; 32]).root();
    let at = jiff::Timestamp::from_second(T0).unwrap();
    let one = root.revoke(&ShareKey::from_bytes(&[3; 32]).public(), at);
    let two = root.revoke(&ShareKey::from_bytes(&[4; 32]).public(), at);

    assert_eq!(
        untold(cache.path(), vec![one.clone(), two.clone()]).len(),
        2
    );
    heard(cache.path(), std::slice::from_ref(&one));
    assert_eq!(
        untold(cache.path(), vec![one.clone(), two.clone()]),
        vec![two.clone()]
    );
    heard(cache.path(), std::slice::from_ref(&two));
    assert!(untold(cache.path(), vec![one, two]).is_empty());
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
    let refused = share(
        &Ungated(&here),
        SERVER,
        "Hola",
        &[],
        &chosen,
        &voucher(),
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "shareOffline");

    assert_eq!(here.share.borrow().store.envelopes(), 0);
}

/// The server takes the upload, and its answer is lost on the way back.
struct Unanswered<'a>(&'a Here);

impl Wire for Unanswered<'_> {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal> {
        let put = asked.method == "PUT";
        let said = self.0.send(asked)?;
        match put {
            true => Err(Refusal::of("shareOffline")),
            false => Ok(said),
        }
    }
}

#[test]
fn an_upload_whose_answer_was_lost_does_not_stay_up() {
    let here = Here::new();
    let refused = share(
        &Unanswered(&here),
        SERVER,
        "Hola",
        &[],
        &Chosen::default(),
        &voucher(),
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "shareOffline");

    assert_eq!(here.share.borrow().store.envelopes(), 0);
}

#[test]
fn a_document_over_the_server_s_limits_says_it_is_too_big() {
    let here = Here::new();
    let long = "a".repeat(Limits::default().text as usize + 1);

    let refused = share(
        &here,
        SERVER,
        &long,
        &[],
        &Chosen::default(),
        &voucher(),
        None,
    )
    .unwrap_err();
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
        seen: None,
    };
    let text = "# Prueba de Tisty Share\n\nEscrito desde la prueba que llega a tisty.cl.";
    let shared = share(&wire, SERVER, text, &[], &chosen, &voucher, Some("Prueba")).unwrap();

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

fn png(side: u32) -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_pixel(side, side, image::Rgb([200, 30, 30]))
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

#[test]
fn the_pictures_the_words_name_go_in_the_envelope_and_the_rest_stay_as_written() {
    let text = "# Viaje\n\n![Playa](attachments/playa-a1b2c3d4.png)\n\n![Otra vez](attachments/playa-a1b2c3d4.png)\n\n![Plano](attachments/plano-b2c3d4e5.pdf)\n\n![Web](https://example.com/x.png)\n\n```md\n![En código](attachments/playa-a1b2c3d4.png)\n```\n";
    let read = |target: &str| match target {
        "attachments/playa-a1b2c3d4.png" => Some(png(20)),
        "attachments/plano-b2c3d4e5.pdf" => Some(b"%PDF-1.7".to_vec()),
        _ => None,
    };
    let (said, pictures) = pictured(text, read);

    // The same picture named twice travels once.
    assert_eq!(pictures.len(), 1);
    assert_eq!(said.matches("tisty:picture/0").count(), 2);
    assert!(said.contains("![Plano](attachments/plano-b2c3d4e5.pdf)"));
    assert!(said.contains("![Web](https://example.com/x.png)"));
    assert!(said.contains("![En código](attachments/playa-a1b2c3d4.png)"));
}

#[test]
fn a_shared_document_carries_its_pictures_to_whoever_opens_it() {
    let here = Here::new();
    let (text, pictures) = pictured(
        "# Viaje\n\n![Playa](attachments/playa-a1b2c3d4.png)\n",
        |_| Some(png(20)),
    );
    let shared = share(
        &here,
        SERVER,
        &text,
        &pictures,
        &Chosen::default(),
        &voucher(),
        None,
    )
    .unwrap();

    let (status, body) = here.read(&shared.link, None);
    assert_eq!(status, 200);
    let envelope = Envelope::read(body, &Limits::default()).unwrap();
    let opened = envelope::open(&LinkKey::from_text(&shared.key).unwrap(), &envelope, &[]).unwrap();
    assert_eq!(opened.pictures.len(), 1);
    assert!(opened.text.contains("![Playa](tisty:picture/0)"));
}

#[test]
fn a_picture_named_with_spaces_or_angle_brackets_is_taken_and_a_stray_mark_is_not() {
    let read = |target: &str| (target == "attachments/playa-a1b2c3d4.png").then(|| png(20));
    let (said, pictures) = pictured("![a]( attachments/playa-a1b2c3d4.png )\n", read);
    assert_eq!(said, "![a]( tisty:picture/0 )\n");
    assert_eq!(pictures.len(), 1);
    let (said, _) = pictured("![a](<attachments/playa-a1b2c3d4.png>)\n", read);
    assert_eq!(said, "![a](tisty:picture/0)\n");
    let spaced = |target: &str| (target == "attachments/mi playa-a1b2c3d4.png").then(|| png(20));
    let (said, pictures) = pictured(
        "![a](<attachments/mi playa-a1b2c3d4.png> \"Playa\")\n",
        spaced,
    );
    assert_eq!(said, "![a](tisty:picture/0 \"Playa\")\n");
    assert_eq!(pictures.len(), 1);
    let bracketed = |target: &str| (target == "attachments/foto (1)-a1b2c3d4.png").then(|| png(20));
    let (said, pictures) = pictured("![a](<attachments/foto (1)-a1b2c3d4.png>)\n", bracketed);
    assert_eq!(said, "![a](tisty:picture/0)\n");
    assert_eq!(pictures.len(), 1);
    let (said, _) = pictured(
        "![[nested] alt (with) marks](attachments/playa-a1b2c3d4.png)\n",
        read,
    );
    assert_eq!(said, "![[nested] alt (with) marks](tisty:picture/0)\n");

    // A `![` with no image after it never reaches a plain link or a picture in code further on.
    let stray = "Say ![ and move on.\n\n[plano](attachments/playa-a1b2c3d4.png)\n\n```\n![x](attachments/playa-a1b2c3d4.png)\n```\n";
    let (said, pictures) = pictured(stray, read);
    assert_eq!(said, stray);
    assert!(pictures.is_empty());
    // Nor does an alt text that ends with the line.
    let (said, _) = pictured("![a\n](attachments/playa-a1b2c3d4.png)", read);
    assert_eq!(said, "![a\n](attachments/playa-a1b2c3d4.png)");
    // Words beyond ASCII before and after stay whole.
    let (said, _) = pictured("Ñandú ![ñ](attachments/playa-a1b2c3d4.png) €\n", read);
    assert_eq!(said, "Ñandú ![ñ](tisty:picture/0) €\n");
}
