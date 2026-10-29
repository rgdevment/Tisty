use std::cell::Cell;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use jiff::Timestamp;
use tisty_share_proto::envelope::{self, Draft, Kind, LinkKey, Signer};
use tisty_share_proto::identity::{Root, ShareKey};

use super::*;
use crate::memory::Memory;

/// The memory store never waits, so one poll finishes any of the server's work.
fn done<T>(work: impl Future<Output = T>) -> T {
    match pin!(work).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(said) => said,
        Poll::Pending => panic!("the memory store never waits"),
    }
}

/// Counts up, so every id and token in a test is different and the same run after run.
struct Counting(u8);

impl Randomness for Counting {
    fn fill(&mut self, into: &mut [u8]) -> bool {
        for one in into {
            self.0 = self.0.wrapping_add(1) % 248;
            *one = self.0;
        }
        true
    }
}

struct Empty;

impl Randomness for Empty {
    fn fill(&mut self, _: &mut [u8]) -> bool {
        false
    }
}

const DAY: i64 = 24 * 60 * 60;
const T0: i64 = 1_800_000_000;

fn policy() -> Policy {
    Policy {
        slowness: Slowness { log_n: 4, r: 8 },
        ..Policy::default()
    }
}

fn server() -> Share<Memory, Counting> {
    Share::new(Memory::default(), Counting(0), policy())
}

fn key() -> LinkKey {
    LinkKey::from_bytes(&[9; 32])
}

fn sealed(id: &str, version: u64, from: Option<u64>, text: &str) -> Vec<u8> {
    let draft = Draft {
        id,
        version,
        from,
        text,
        pictures: &[],
    };
    envelope::seal(&key(), &draft, None, &Limits::default())
        .unwrap()
        .to_bytes()
}

fn opened(served: &Served) -> String {
    let arrived = Envelope::read(served.envelope.clone(), &Limits::default()).unwrap();
    envelope::open(&key(), &arrived, &[])
        .unwrap()
        .text
        .to_string()
}

/// A document already shared once, at T0.
fn shared(share: &mut Share<Memory, Counting>) -> Reserved {
    let reserved = done(share.reserve(T0)).unwrap();
    let first = sealed(&reserved.id, 1, None, "uno");
    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, first, T0)),
        Ok(1)
    );
    reserved
}

#[test]
fn a_document_is_reserved_shared_and_read_as_tisty_sealed_it() {
    let mut share = server();
    let reserved = shared(&mut share);
    assert_eq!(reserved.id.len(), 8);

    let served = done(share.read(&reserved.id, None, T0 + 10)).unwrap();
    assert_eq!(served.version, 1);
    assert_eq!(opened(&served), "uno");
    assert_eq!(done(share.latest(&reserved.id, T0)), Ok(Some(1)));
}

#[test]
fn only_the_edit_link_uploads() {
    let mut share = server();
    let reserved = shared(&mut share);
    let next = sealed(&reserved.id, 2, Some(1), "dos");

    for wrong in [reserved.admin.as_str(), "anything", ""] {
        assert_eq!(
            done(share.upload(&reserved.id, wrong, next.clone(), T0)),
            Err(Refused::Forbidden)
        );
    }
    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, next, T0)),
        Ok(2)
    );
}

#[test]
fn an_envelope_sealed_for_another_document_is_refused() {
    let mut share = server();
    let reserved = shared(&mut share);
    let elsewhere = sealed("someOther", 2, Some(1), "dos");

    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, elsewhere, T0)),
        Err(Refused::Misaddressed)
    );
}

#[test]
fn a_change_written_on_an_older_version_is_sent_back_to_be_chosen() {
    let mut share = server();
    let reserved = shared(&mut share);
    let mine = sealed(&reserved.id, 2, Some(1), "mía");
    let theirs = sealed(&reserved.id, 2, Some(1), "suya");

    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, mine, T0)),
        Ok(2)
    );
    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, theirs, T0)),
        Err(Refused::Behind { latest: Some(2) })
    );
    let skipping = sealed(&reserved.id, 4, Some(2), "salta");
    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, skipping, T0)),
        Err(Refused::Behind { latest: Some(2) })
    );
    assert_eq!(
        opened(&done(share.read(&reserved.id, None, T0)).unwrap()),
        "mía"
    );
}

#[test]
fn readers_get_only_the_latest_and_the_editors_see_the_last_five() {
    let mut share = server();
    let reserved = shared(&mut share);
    for version in 2..=7 {
        let next = sealed(
            &reserved.id,
            version,
            Some(version - 1),
            &format!("v{version}"),
        );
        done(share.upload(&reserved.id, &reserved.edit, next, T0)).unwrap();
    }

    let served = done(share.read(&reserved.id, None, T0)).unwrap();
    assert_eq!((served.version, opened(&served)), (7, "v7".to_owned()));
    assert_eq!(share.store.envelopes(), 5);
    let third = done(share.version(&reserved.id, &reserved.admin, 3, T0)).unwrap();
    assert_eq!(opened(&third), "v3");
    assert_eq!(
        done(share.version(&reserved.id, &reserved.edit, 2, T0)),
        Err(Refused::Missing)
    );
    assert_eq!(
        done(share.version(&reserved.id, "a reader", 3, T0)),
        Err(Refused::Forbidden)
    );
}

#[test]
fn an_envelope_over_the_server_s_limits_is_refused() {
    let mut share = server();
    share.policy.limits.text = 3;
    let reserved = done(share.reserve(T0)).unwrap();
    let long = sealed(&reserved.id, 1, None, "demasiado");

    // Sealed under Tisty's own limits, read under the server's tighter ones.
    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, long, T0)),
        Err(Refused::Envelope(Unread::TextTooBig))
    );
}

#[test]
fn a_signed_envelope_is_taken_with_its_signature() {
    let mut share = server();
    let reserved = done(share.reserve(T0)).unwrap();
    let root = Root::from_bytes(&[1; 32]);
    let machine = ShareKey::from_bytes(&[2; 32]);
    let certificate = root.certify(&machine.public(), Timestamp::from_second(1_000).unwrap());
    let signer = Signer {
        key: &machine,
        certificate: &certificate,
    };
    let draft = Draft {
        id: &reserved.id,
        version: 1,
        from: None,
        text: "firmado",
        pictures: &[(Kind::Png, b"\x89PNG\r\n\x1a\n...")],
    };
    let bytes = envelope::seal(&key(), &draft, Some(&signer), &Limits::default())
        .unwrap()
        .to_bytes();

    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, bytes, T0)),
        Ok(1)
    );
}

#[test]
fn a_password_is_asked_for_counted_and_after_five_misses_waits() {
    let mut share = server();
    let reserved = shared(&mut share);
    let gate = Manage::Password(Some("perro azul".into()));
    done(share.manage(&reserved.id, &reserved.admin, gate, T0)).unwrap();

    assert_eq!(
        done(share.read(&reserved.id, None, T0)),
        Err(Refused::PasswordAsked)
    );
    for _ in 0..4 {
        assert_eq!(
            done(share.read(&reserved.id, Some("gato"), T0)),
            Err(Refused::PasswordWrong)
        );
    }
    assert_eq!(
        done(share.read(&reserved.id, Some("gato"), T0)),
        Err(Refused::PasswordWrong)
    );
    // Even the right one waits until the lock lifts.
    assert_eq!(
        done(share.read(&reserved.id, Some("perro azul"), T0 + 60)),
        Err(Refused::PasswordLocked {
            until: T0 + share.policy.locked
        })
    );
    let later = T0 + share.policy.locked + 1;
    assert!(done(share.read(&reserved.id, Some("perro azul"), later)).is_ok());

    done(share.manage(&reserved.id, &reserved.admin, Manage::Password(None), later)).unwrap();
    assert!(done(share.read(&reserved.id, None, later)).is_ok());
}

#[test]
fn a_document_nobody_reads_for_thirty_days_is_gone() {
    let mut share = server();
    let reserved = shared(&mut share);

    assert!(done(share.read(&reserved.id, None, T0 + 29 * DAY)).is_ok());
    // That read moved the end to day 59.
    assert!(done(share.read(&reserved.id, None, T0 + 58 * DAY)).is_ok());
    assert_eq!(
        done(share.read(&reserved.id, None, T0 + 89 * DAY)),
        Err(Refused::Gone)
    );
    assert_eq!(
        done(share.read(&reserved.id, None, T0 + 89 * DAY)),
        Err(Refused::Missing)
    );
    assert_eq!(share.store.envelopes(), 0);
}

#[test]
fn tisty_asking_for_the_latest_version_is_not_activity() {
    let mut share = server();
    let reserved = shared(&mut share);

    for day in [10, 20, 29] {
        assert_eq!(
            done(share.latest(&reserved.id, T0 + day * DAY)),
            Ok(Some(1))
        );
    }
    assert_eq!(
        done(share.latest(&reserved.id, T0 + 31 * DAY)),
        Err(Refused::Gone)
    );
}

#[test]
fn the_owner_s_visit_and_a_new_version_count_as_activity() {
    let mut share = server();
    let reserved = shared(&mut share);
    done(share.manage(&reserved.id, &reserved.admin, Manage::Visit, T0 + 25 * DAY)).unwrap();
    let next = sealed(&reserved.id, 2, Some(1), "dos");
    done(share.upload(&reserved.id, &reserved.edit, next, T0 + 50 * DAY)).unwrap();

    assert!(done(share.read(&reserved.id, None, T0 + 79 * DAY)).is_ok());
}

#[test]
fn reads_close_together_write_the_activity_down_once() {
    let mut share = server();
    let reserved = shared(&mut share);
    let before = done(share.store.doc(&reserved.id))
        .unwrap()
        .unwrap()
        .revision;
    for minute in 1..=30 {
        done(share.read(&reserved.id, None, T0 + minute * 60)).unwrap();
    }
    let after = done(share.store.doc(&reserved.id))
        .unwrap()
        .unwrap()
        .revision;

    assert_eq!(after, before);
    done(share.read(&reserved.id, None, T0 + 2 * 60 * 60)).unwrap();
    assert_eq!(
        done(share.store.doc(&reserved.id))
            .unwrap()
            .unwrap()
            .revision,
        before + 1
    );
}

#[test]
fn an_owner_may_ask_for_a_shorter_life_never_a_longer_one() {
    let mut share = server();
    let reserved = shared(&mut share);
    done(share.manage(&reserved.id, &reserved.admin, Manage::Idle(7 * DAY), T0)).unwrap();
    assert_eq!(
        done(share.read(&reserved.id, None, T0 + 8 * DAY)),
        Err(Refused::Gone)
    );

    let reserved = shared(&mut share);
    done(share.manage(&reserved.id, &reserved.admin, Manage::Idle(365 * DAY), T0)).unwrap();
    assert_eq!(
        done(share.read(&reserved.id, None, T0 + 31 * DAY)),
        Err(Refused::Gone)
    );
}

#[test]
fn a_paused_document_is_not_served_and_its_trash_time_runs_instead() {
    let mut share = server();
    let reserved = shared(&mut share);
    done(share.manage(&reserved.id, &reserved.admin, Manage::Pause, T0 + DAY)).unwrap();

    assert_eq!(
        done(share.read(&reserved.id, None, T0 + DAY)),
        Err(Refused::Paused)
    );
    assert_eq!(
        done(share.latest(&reserved.id, T0 + DAY)),
        Err(Refused::Paused)
    );
    let next = sealed(&reserved.id, 2, Some(1), "dos");
    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, next, T0 + DAY)),
        Err(Refused::Paused)
    );
    done(share.manage(
        &reserved.id,
        &reserved.admin,
        Manage::Restore,
        T0 + 20 * DAY,
    ))
    .unwrap();
    assert!(done(share.read(&reserved.id, None, T0 + 45 * DAY)).is_ok());

    done(share.manage(&reserved.id, &reserved.admin, Manage::Pause, T0 + 46 * DAY)).unwrap();
    assert_eq!(
        done(share.read(&reserved.id, None, T0 + 77 * DAY)),
        Err(Refused::Gone)
    );
}

#[test]
fn deleting_takes_the_document_and_every_envelope_away() {
    let mut share = server();
    let reserved = shared(&mut share);
    assert_eq!(
        done(share.manage(&reserved.id, &reserved.edit, Manage::Delete, T0)),
        Err(Refused::Forbidden)
    );
    done(share.manage(&reserved.id, &reserved.admin, Manage::Delete, T0)).unwrap();

    assert_eq!(
        done(share.read(&reserved.id, None, T0)),
        Err(Refused::Missing)
    );
    assert_eq!(share.store.envelopes(), 0);
}

#[test]
fn a_sweep_takes_whatever_ran_out_and_leaves_the_rest() {
    let mut share = server();
    let old = shared(&mut share);
    let never_used = done(share.reserve(T0)).unwrap();
    let fresh = done(share.reserve(T0 + 20 * DAY)).unwrap();
    let first = sealed(&fresh.id, 1, None, "nuevo");
    done(share.upload(&fresh.id, &fresh.edit, first, T0 + 20 * DAY)).unwrap();

    assert_eq!(done(share.sweep(T0 + 31 * DAY)), Ok(2));
    assert_eq!(
        done(share.read(&old.id, None, T0 + 31 * DAY)),
        Err(Refused::Missing)
    );
    assert_eq!(
        done(share.read(&never_used.id, None, T0)),
        Err(Refused::Missing)
    );
    assert!(done(share.read(&fresh.id, None, T0 + 31 * DAY)).is_ok());
}

#[test]
fn a_reserved_id_with_nothing_uploaded_cannot_be_read() {
    let mut share = server();
    let reserved = done(share.reserve(T0)).unwrap();
    assert_eq!(
        done(share.read(&reserved.id, None, T0)),
        Err(Refused::Missing)
    );
    assert_eq!(done(share.latest(&reserved.id, T0)), Ok(None));
}

#[test]
fn nothing_is_reserved_without_randomness() {
    let mut share = Share::new(Memory::default(), Empty, policy());
    assert_eq!(done(share.reserve(T0)).err(), Some(Refused::NoRandomness));
}

/// Loses the next save once, as if another upload had saved between reading and writing.
struct Racing {
    inner: Memory,
    lose: Cell<bool>,
}

impl Store for Racing {
    async fn doc(&self, id: &str) -> Result<Option<Doc>, Fault> {
        self.inner.doc(id).await
    }
    async fn save(&self, doc: &Doc, revision: Option<u64>) -> Result<Saved, Fault> {
        if self.lose.replace(false) {
            return Ok(Saved::Raced);
        }
        self.inner.save(doc, revision).await
    }
    async fn forget(&self, id: &str) -> Result<(), Fault> {
        self.inner.forget(id).await
    }
    async fn envelope(&self, id: &str, kept: &Kept) -> Result<Option<Vec<u8>>, Fault> {
        self.inner.envelope(id, kept).await
    }
    async fn keep(&self, id: &str, kept: &Kept, bytes: &[u8]) -> Result<(), Fault> {
        self.inner.keep(id, kept, bytes).await
    }
    async fn drop_envelope(&self, id: &str, kept: &Kept) -> Result<(), Fault> {
        self.inner.drop_envelope(id, kept).await
    }
    async fn ending(&self, now: i64) -> Result<Vec<String>, Fault> {
        self.inner.ending(now).await
    }
}

#[test]
fn an_upload_that_loses_the_race_leaves_nothing_of_itself_behind() {
    let store = Racing {
        inner: Memory::default(),
        lose: Cell::new(false),
    };
    let mut share = Share::new(store, Counting(0), policy());
    let reserved = done(share.reserve(T0)).unwrap();
    share.store.lose.set(true);
    let first = sealed(&reserved.id, 1, None, "perdió");

    assert_eq!(
        done(share.upload(&reserved.id, &reserved.edit, first, T0)),
        Err(Refused::Behind { latest: None })
    );
    assert_eq!(share.store.inner.envelopes(), 0);
}

#[test]
fn two_readers_racing_to_write_their_visit_are_both_served() {
    let store = Racing {
        inner: Memory::default(),
        lose: Cell::new(false),
    };
    let mut share = Share::new(store, Counting(0), policy());
    let reserved = done(share.reserve(T0)).unwrap();
    let first = sealed(&reserved.id, 1, None, "uno");
    done(share.upload(&reserved.id, &reserved.edit, first, T0)).unwrap();
    share.store.lose.set(true);

    assert!(done(share.read(&reserved.id, None, T0 + DAY)).is_ok());
}
