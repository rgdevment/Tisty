use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tisty_share_proto::identity::ShareKey;
use tisty_share_proto::seed::Seed;
use tisty_share_server::guard::Work;
use tisty_share_server::keys::Slowness;
use tisty_share_server::memory::Memory;
use tisty_share_server::share::{Policy, Share};
use tisty_share_server::testing::{Counting, done};
use tisty_share_worker::api::{Asked, answer};

use super::{API, Asking, Said, Wire};
use crate::handed::Voucher;
use crate::refusing::Refusal;

pub(crate) const T0: i64 = 1_800_000_000;

/// A machine's key and the certificate its person gave it, as `vouched_by` hands them over.
pub(crate) fn voucher() -> Voucher {
    let key = ShareKey::from_bytes(&[3; 32]);
    let since = jiff::Timestamp::from_second(T0 - 86_400).unwrap();
    let certificate = Seed::from_bytes(&[7; 32])
        .root()
        .certify(&key.public(), since);
    (key, certificate)
}

/// The server's API in memory, with little work so the tests run quickly.
pub(crate) struct Here {
    pub(crate) share: RefCell<Share<Memory, Counting>>,
}

impl Here {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn read(&self, id: &str, password: Option<&str>) -> (u16, Vec<u8>) {
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
