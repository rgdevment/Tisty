use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr};

use tisty_share_proto::identity::ShareKey;
use tisty_share_proto::seed::Seed;
use tisty_share_server::guard::Work;
use tisty_share_server::keys::Slowness;
use tisty_share_server::memory::Memory;
use tisty_share_server::share::{Policy, Share};
use tisty_share_server::testing::{Counting, done};
use tisty_share_worker::api::{Asked, answer};

use super::*;
use crate::sharing::{Asking, Chosen, Said};

const T0: i64 = 1_800_000_000;
const KEY: &str = "_YQqHycdmpMvpG7q2Gmd3IjAeSc_Rbn-RvCb7NuqEB8";

fn opening(link: &str) -> String {
    let mut said = url::Url::parse("tisty://share").unwrap();
    said.query_pairs_mut().append_pair("link", link);
    said.to_string()
}

#[test]
fn only_a_link_to_tisty_share_s_own_server_is_taken() {
    let taken = asked(&opening(&format!("https://tisty.cl/oVD0Idz0#{KEY}"))).unwrap();
    assert_eq!(taken.id, "oVD0Idz0");
    assert_eq!(taken.read(), format!("https://tisty.cl/oVD0Idz0#{KEY}"));
    // The edit token stays behind: Tisty only reads a share for now.
    let editing = asked(&opening(&format!("https://tisty.cl/oVD0Idz0#{KEY}.token"))).unwrap();
    assert_eq!(editing.key, KEY);

    for wrong in [
        opening(&format!("https://evil.example/oVD0Idz0#{KEY}")),
        opening(&format!("http://tisty.cl/oVD0Idz0#{KEY}")),
        opening(&format!("https://tisty.cl:8443/oVD0Idz0#{KEY}")),
        opening(&format!("https://tisty.cl/../admin#{KEY}")),
        opening("https://tisty.cl/oVD0Idz0"),
        opening("https://tisty.cl/oVD0Idz0#not-a-key"),
        format!("tisty://open?link=https://tisty.cl/oVD0Idz0%23{KEY}"),
        format!(
            "{}&link=x",
            opening(&format!("https://tisty.cl/oVD0Idz0#{KEY}"))
        ),
    ] {
        assert_eq!(asked(&wrong), None, "{wrong}");
    }
}

/// The server's API in memory, as the share tests meet it.
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
}

impl Wire for Here {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal> {
        let said = done(answer(
            &mut self.share.borrow_mut(),
            Asked {
                method: asked.method.into(),
                path: format!("/api/v1/{}", asked.path),
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

fn voucher() -> crate::handed::Voucher {
    let key = ShareKey::from_bytes(&[3; 32]);
    let since = jiff::Timestamp::from_second(T0 - 86_400).unwrap();
    let certificate = Seed::from_bytes(&[7; 32])
        .root()
        .certify(&key.public(), since);
    (key, certificate)
}

fn link_of(shared: &tisty_core::event::Shared) -> Link {
    Link {
        id: shared.link.clone(),
        key: shared.key.clone(),
    }
}

#[test]
fn a_shared_document_opens_here_with_its_title_and_the_name_it_was_signed_with() {
    let here = Here::new();
    let text = "# Compras\n\nleche";
    let shared = sharing::share(
        &here,
        sharing::SERVER,
        text,
        &Chosen::default(),
        &voucher(),
        Some("Ana"),
    )
    .unwrap();
    let got = fetched(&here, &link_of(&shared)).unwrap();
    assert_eq!(got.title, "Compras");
    assert_eq!(got.body, text);
    assert_eq!(got.signer.as_deref(), Some("Ana"));
}

#[test]
fn a_share_behind_a_password_or_gone_is_told_as_such() {
    let here = Here::new();
    let chosen = Chosen {
        password: Some("perro azul".into()),
        ..Chosen::default()
    };
    let shared = sharing::share(&here, sharing::SERVER, "# x", &chosen, &voucher(), None).unwrap();
    assert_eq!(
        fetched(&here, &link_of(&shared)).unwrap_err().code,
        "shareOpenGated"
    );

    sharing::stop(&here, &shared).unwrap();
    assert_eq!(
        fetched(&here, &link_of(&shared)).unwrap_err().code,
        "shareOpenGone"
    );
    let nowhere = Link {
        id: "nadaaqui".into(),
        key: KEY.into(),
    };
    assert_eq!(fetched(&here, &nowhere).unwrap_err().code, "shareOpenGone");
}

#[test]
fn an_envelope_nobody_signed_is_not_shown() {
    let here = Here::new();
    let shared = sharing::share(
        &here,
        sharing::SERVER,
        "# x",
        &Chosen::default(),
        &voucher(),
        None,
    )
    .unwrap();
    // The right envelope opened with another key is not this writer's word.
    let other = Link {
        id: shared.link.clone(),
        key: LinkKey::from_bytes(&[1; 32]).to_text().to_string(),
    };
    assert_eq!(
        fetched(&here, &other).unwrap_err().code,
        "shareOpenUnsigned"
    );
}
