use std::path::Path;

use tisty_core::event::{DocAdd, Said, Signature};
use tisty_core::model::{DocId, Kept};
use tisty_core::parcel::{self, Along, Answer, Claimed, Landed};
use tisty_core::signed_parcel::Signer;
use tisty_core::{DeviceId, DeviceKind, Error, Event, Op, Paths, State, docs, person};
use tisty_share_proto::seed::Seed;

mod common;
use common::rewritten;

const ME: u8 = 7;
const SOMEONE_ELSE: u8 = 9;

struct Desk {
    paths: Paths,
    state: State,
    dev: DeviceId,
    seq: i64,
}

impl Desk {
    fn new(under: &Path, named: &str) -> Self {
        let paths = Paths::new(under.join(named), under.join(format!("{named}-cfg")));
        std::fs::create_dir_all(paths.data().join("docs")).unwrap();
        Self {
            paths,
            state: State::default(),
            dev: DeviceId(named.into()),
            seq: 0,
        }
    }

    fn tell(&mut self, op: Op) {
        self.seq += 1;
        let at = "2026-11-10T12:00:00Z"
            .parse::<jiff::Timestamp>()
            .unwrap()
            .checked_add(jiff::SignedDuration::from_millis(self.seq))
            .unwrap();
        let event = Event::new(self.dev.clone(), at, op);
        self.state.apply(&event);
    }

    fn named(mut self, seed: u8, alias: &str) -> Self {
        self.tell(Op::DeviceJoin {
            d: self.dev.clone(),
            k: Some(DeviceKind::Machine),
            p: None,
        });
        let now = "2026-11-10T12:00:00Z".parse().unwrap();
        let seed = Seed::from_bytes(&[seed; 32]);
        for op in person::begun(&self.paths, &self.dev, &seed, &self.state.persons, now).unwrap() {
            self.tell(op);
        }
        self.tell(Op::Signed {
            d: Signature {
                alias: Some(alias.into()),
                ..Default::default()
            },
        });
        self
    }

    fn wrote(&mut self, body: &str) -> DocId {
        let made = docs::create(&self.paths.docs(), &self.dev, body).unwrap();
        let id = ulid::Ulid::generate();
        self.tell(Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id,
                order: "a0001".into(),
                said: Some(Said::of(body)),
                ..Default::default()
            },
        });
        id
    }

    fn packed(&self, into: &Path, number: Option<&str>) {
        let (key, certificate) =
            person::signer(&self.paths, &self.state.persons, &self.dev).unwrap();
        let signer = Signer {
            key: &key,
            certificate: &certificate,
            name: self.state.signed.alias.as_deref(),
        };
        parcel::packed(
            &self.paths,
            &self.state,
            &[],
            into,
            &Along::default(),
            number,
            Some(&signer),
        )
        .unwrap();
    }

    fn opened(
        &mut self,
        from: &Path,
        number: Option<&str>,
        answer: Option<Answer>,
    ) -> Result<Landed, Error> {
        let (landed, ops) = parcel::taken_answering(
            &self.paths,
            &self.state,
            &self.dev.clone(),
            from,
            &Along::default(),
            number,
            answer,
        )?;
        for op in ops {
            self.tell(op);
        }
        Ok(landed)
    }

    fn the_one(&self) -> &Kept {
        let mut all = self.state.docs.values();
        let one = all.next().expect("nothing landed");
        assert!(all.next().is_none(), "more than one document landed");
        one
    }
}

fn print_of(seed: u8) -> String {
    Seed::from_bytes(&[seed; 32]).root().person().fingerprint()
}

fn entry(at: &Path, named: &str) -> Option<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(at).unwrap()).unwrap();
    let mut one = zip.by_name(named).ok()?;
    let mut body = Vec::new();
    std::io::Read::read_to_end(&mut one, &mut body).unwrap();
    Some(body)
}

#[test]
fn a_signed_parcel_lands_as_mine_in_another_tisty_of_the_same_person() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, None);

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    let landed = there.opened(&at, None, None).unwrap();

    assert_eq!(landed.docs, 1);
    assert_eq!(
        landed.signer, None,
        "my own parcel was named as someone else's"
    );
    assert!(!landed.broken);
    assert!(
        !there.the_one().guest,
        "my own writing came home as a guest"
    );
}

#[test]
fn a_signed_parcel_names_no_store_and_reads_as_a_version_3_parcel() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, None);

    let manifest: serde_json::Value =
        serde_json::from_slice(&entry(&at, "tisty-docs.json").unwrap()).unwrap();
    assert_eq!(manifest["version"], 3, "an older Tisty would refuse it");
    assert_eq!(manifest["from"], "", "the store's name went out with it");
    assert!(manifest.get("seal").is_none());
    assert!(entry(&at, "tisty-docs.signed.json").is_some());
}

#[test]
fn a_signed_parcel_from_another_person_lands_as_theirs_with_their_print() {
    let room = tempfile::tempdir().unwrap();
    let mut friend = Desk::new(room.path(), "friend").named(SOMEONE_ELSE, "beto");
    friend.wrote("# Receta\n\nde beto");
    let at = room.path().join("receta.tistyx");
    friend.packed(&at, None);

    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    let landed = here.opened(&at, None, None).unwrap();

    assert_eq!(
        landed.signer,
        Some(Claimed {
            root: Seed::from_bytes(&[SOMEONE_ELSE; 32])
                .root()
                .person()
                .shown(),
            print: print_of(SOMEONE_ELSE),
            name: Some("beto".into()),
        })
    );
    assert!(here.the_one().guest, "a friend's writing came in as mine");
}

#[test]
fn a_friends_locked_and_signed_parcel_stays_theirs_whoever_knows_the_number() {
    let room = tempfile::tempdir().unwrap();
    let mut friend = Desk::new(room.path(), "friend").named(SOMEONE_ELSE, "beto");
    friend.wrote("# Receta\n\nde beto");
    let at = room.path().join("receta.tistyx");
    friend.packed(&at, Some("123456"));

    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    let landed = here
        .opened(&at, Some("123456"), Some(Answer::Mine))
        .unwrap();

    assert!(landed.signer.is_some());
    assert!(
        here.the_one().guest,
        "the number made a friend's writing mine"
    );
}

#[test]
fn my_locked_and_signed_parcel_lands_as_mine_without_asking() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, Some("123456"));

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    let landed = there.opened(&at, Some("123456"), None).unwrap();

    assert_eq!(landed.signer, None);
    assert!(!there.the_one().guest);
}

#[test]
fn a_signed_parcel_whose_files_changed_lands_as_a_strangers() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, None);
    rewritten(&at, |named, body| match named.starts_with("docs/") {
        true => b"# Acta\n\nlo que otro puso".to_vec(),
        false => body,
    });

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    let landed = there.opened(&at, None, None).unwrap();

    assert!(landed.broken, "a changed body still read as signed");
    assert_eq!(landed.signer, None);
    assert!(
        there.the_one().guest,
        "what someone changed came in as mine"
    );
}

#[test]
fn a_signed_parcel_with_a_file_added_lands_as_a_strangers() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, None);
    let mut zip = zip::ZipWriter::new_append(
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&at)
            .unwrap(),
    )
    .unwrap();
    zip.start_file(
        "attachments/extra.txt",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    std::io::Write::write_all(&mut zip, b"slipped in").unwrap();
    zip.finish().unwrap();

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    let landed = there.opened(&at, None, None).unwrap();

    assert!(landed.broken, "a file nobody signed rode along unnoticed");
}

#[test]
fn a_signed_parcel_from_a_key_taken_back_lands_as_a_strangers() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, None);
    let (key, _) = person::signer(&here.paths, &here.state.persons, &here.dev).unwrap();

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    let now = "2026-11-10T13:00:00Z".parse().unwrap();
    there.state.persons.revocations.push(
        Seed::from_bytes(&[ME; 32])
            .root()
            .revoke(&key.public(), now),
    );
    let landed = there.opened(&at, None, None).unwrap();

    assert!(landed.broken, "a key taken back still signed for me");
    assert!(there.the_one().guest);
}

#[test]
fn a_tisty_with_no_phrase_asks_before_a_signed_parcel_lands() {
    let room = tempfile::tempdir().unwrap();
    let mut friend = Desk::new(room.path(), "friend").named(SOMEONE_ELSE, "beto");
    friend.wrote("# Receta\n\nde beto");
    let at = room.path().join("receta.tistyx");
    friend.packed(&at, None);

    let mut bare = Desk::new(room.path(), "bare");
    match bare.opened(&at, None, None) {
        Err(Error::ParcelSignedBy { print, name, .. }) => {
            assert_eq!(print, print_of(SOMEONE_ELSE));
            assert_eq!(name.as_deref(), Some("beto"));
        }
        other => panic!("it landed without asking: {other:?}"),
    }
    assert!(bare.state.docs.is_empty());

    let landed = bare.opened(&at, None, Some(Answer::Theirs)).unwrap();
    assert!(landed.signer.is_some());
    assert!(bare.the_one().guest);
}

#[test]
fn an_unsigned_locked_parcel_told_as_someone_elses_lands_as_a_strangers() {
    let room = tempfile::tempdir().unwrap();
    let mut bare = Desk::new(room.path(), "bare");
    bare.wrote("# Acta\n\nsin frase");
    let at = room.path().join("acta.tistyx");
    parcel::written(
        &bare.paths,
        &bare.state,
        &[],
        &at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();
    assert!(parcel::locked(&at));

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    assert!(matches!(
        there.opened(&at, Some("123456"), None),
        Err(Error::ParcelAsksIfMine)
    ));
    let landed = there
        .opened(&at, Some("123456"), Some(Answer::Theirs))
        .unwrap();
    assert_eq!(landed.signer, None);
    assert!(there.the_one().guest);
}

#[test]
fn a_signature_entry_too_big_to_read_lands_as_a_strangers() {
    let room = tempfile::tempdir().unwrap();
    let mut here = Desk::new(room.path(), "here").named(ME, "ana");
    here.wrote("# Acta\n\nlo mio");
    let at = room.path().join("acta.tistyx");
    here.packed(&at, None);
    rewritten(&at, |named, body| match named == "tisty-docs.signed.json" {
        true => [body, vec![b' '; 70 * 1024]].concat(),
        false => body,
    });

    let mut there = Desk::new(room.path(), "there").named(ME, "ana");
    let landed = there.opened(&at, None, None).unwrap();

    assert!(
        landed.broken,
        "an entry past its limit was read as a signature"
    );
    assert!(there.the_one().guest);
}
