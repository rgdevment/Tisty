use jiff::Timestamp;

use super::*;
use crate::identity::Root;

fn key(n: u8) -> LinkKey {
    LinkKey::from_bytes(&[n; 32])
}

fn png(fill: u8, len: usize) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    bytes.resize(len, fill);
    bytes
}

fn jpeg(len: usize) -> Vec<u8> {
    let mut bytes = vec![0xff, 0xd8, 0xff, 0xe0];
    bytes.resize(len, 7);
    bytes
}

fn draft<'a>(text: &'a str, pictures: &'a [(Kind, &'a [u8])]) -> Draft<'a> {
    Draft {
        id: "k7Qm2xPa",
        version: 3,
        from: Some(2),
        text,
        pictures,
    }
}

struct Machine {
    root: Root,
    key: ShareKey,
    certificate: Certificate,
}

fn machine(n: u8) -> Machine {
    let root = Root::from_bytes(&[n; 32]);
    let key = ShareKey::from_bytes(&[n + 1; 32]);
    let certificate = root.certify(&key.public(), Timestamp::from_second(1_000).unwrap());
    Machine {
        root,
        key,
        certificate,
    }
}

impl Machine {
    fn signer(&self) -> Signer<'_> {
        Signer {
            key: &self.key,
            certificate: &self.certificate,
        }
    }
}

fn holds_for(verdict: Option<Verdict>, by: &Machine) -> bool {
    matches!(verdict, Some(Verdict::Holds { person, .. }) if person == by.root.person())
}

#[test]
fn a_document_with_pictures_crosses_from_one_tisty_to_another_signed() {
    let (one, two) = (png(1, 300), jpeg(200));
    let pictures = [(Kind::Png, one.as_slice()), (Kind::Jpeg, two.as_slice())];
    let mario = machine(1);
    let sealed = seal(
        &key(9),
        &draft("# Plan\n\nHola.", &pictures),
        Some(&mario.signer()),
    )
    .unwrap();

    let arrived = Envelope::read(&sealed.to_bytes()).unwrap();
    let opened = open(&key(9), &arrived, &[]).unwrap();

    assert_eq!(opened.text.as_str(), "# Plan\n\nHola.");
    assert_eq!(opened.pictures, vec![(Kind::Png, one), (Kind::Jpeg, two)]);
    assert_eq!(opened.manifest.id, "k7Qm2xPa");
    assert_eq!(
        (opened.manifest.version, opened.manifest.from),
        (3, Some(2))
    );
    assert!(holds_for(opened.wrote, &mario));
    assert!(holds_for(opened.sent, &mario));
}

#[test]
fn the_server_reads_every_limit_and_the_sender_without_the_key() {
    let picture = png(1, 300);
    let pictures = [(Kind::Png, picture.as_slice())];
    let mario = machine(1);
    let sealed = seal(&key(9), &draft("hola", &pictures), Some(&mario.signer())).unwrap();
    let bytes = sealed.to_bytes();

    let arrived = Envelope::read(&bytes).unwrap();
    let manifest = arrived.manifest();
    let kinds: Vec<Kind> = manifest.parts.iter().map(|one| one.kind).collect();
    assert_eq!(kinds, vec![Kind::Text, Kind::Signature, Kind::Png]);
    assert_eq!(manifest.parts[2].plain(), 300);
    assert!(holds_for(arrived.sender(&[]), &mario));
    assert!(
        !String::from_utf8_lossy(&bytes).contains("hola"),
        "the words travel sealed"
    );
}

#[test]
fn an_envelope_without_a_signature_opens_and_says_nobody_signed() {
    let sealed = seal(&key(9), &draft("hola", &[]), None).unwrap();
    let arrived = Envelope::read(&sealed.to_bytes()).unwrap();
    let opened = open(&key(9), &arrived, &[]).unwrap();

    assert_eq!(opened.text.as_str(), "hola");
    assert!(opened.wrote.is_none() && opened.sent.is_none());
    assert!(!arrived.manifest().signed());
}

#[test]
fn another_key_never_opens_it() {
    let sealed = seal(&key(9), &draft("hola", &[]), None).unwrap();
    let arrived = Envelope::read(&sealed.to_bytes()).unwrap();

    assert_eq!(open(&key(8), &arrived, &[]).err(), Some(Refused::Locked));
}

/// Where the sealed parts lie in the bytes: past the manifest, before the outer signature.
fn sealed_span(envelope: &Envelope) -> std::ops::Range<usize> {
    let start = 4 + 1 + 4 + envelope.written.len();
    start..start + envelope.parts.iter().map(Vec::len).sum::<usize>()
}

fn replaced(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    assert_eq!(from.len(), to.len());
    let at = bytes
        .windows(from.len())
        .position(|one| one == from)
        .expect("the pattern is there");
    let mut out = bytes.to_vec();
    out[at..at + to.len()].copy_from_slice(to);
    out
}

#[test]
fn a_changed_byte_in_any_sealed_part_is_refused() {
    let picture = png(1, 300);
    let pictures = [(Kind::Png, picture.as_slice())];
    let sealed = seal(
        &key(9),
        &draft("hola", &pictures),
        Some(&machine(1).signer()),
    )
    .unwrap();
    let bytes = sealed.to_bytes();
    let span = sealed_span(&sealed);
    assert!(span.len() > 300);

    for at in span.step_by(7) {
        let mut changed = bytes.clone();
        changed[at] ^= 1;
        let arrived = Envelope::read(&changed).unwrap();
        assert_eq!(
            open(&key(9), &arrived, &[]).err(),
            Some(Refused::Locked),
            "byte {at}"
        );
        assert!(
            matches!(arrived.sender(&[]), Some(Verdict::Refused)),
            "byte {at}"
        );
    }
}

#[test]
fn a_manifest_changed_on_the_way_breaks_the_seal_and_the_sender() {
    let mario = machine(1);
    let bytes = seal(&key(9), &draft("hola", &[]), Some(&mario.signer()))
        .unwrap()
        .to_bytes();
    let forged = replaced(&bytes, b"\"version\":3", b"\"version\":4");

    let arrived = Envelope::read(&forged).unwrap();
    assert_eq!(arrived.manifest().version, 4);
    assert!(matches!(arrived.sender(&[]), Some(Verdict::Refused)));
    assert_eq!(open(&key(9), &arrived, &[]).err(), Some(Refused::Locked));
}

#[test]
fn a_picture_declared_as_another_kind_on_the_way_is_refused() {
    let picture = png(1, 300);
    let pictures = [(Kind::Png, picture.as_slice())];
    let bytes = seal(&key(9), &draft("hola", &pictures), None)
        .unwrap()
        .to_bytes();
    let forged = replaced(&bytes, b"\"png\"", b"\"gif\"");

    let arrived = Envelope::read(&forged).unwrap();
    assert_eq!(open(&key(9), &arrived, &[]).err(), Some(Refused::Locked));
}

#[test]
fn a_signature_taken_to_another_upload_does_not_hold() {
    let mario = machine(1);
    let one = seal(&key(9), &draft("hola", &[]), Some(&mario.signer())).unwrap();
    let mut other = seal(&key(9), &draft("chao", &[]), None).unwrap();
    other.sent = one.sent.clone();

    assert!(matches!(other.sender(&[]), Some(Verdict::Refused)));
}

#[test]
fn two_pictures_swapped_or_one_cut_off_are_refused() {
    let (one, two) = (png(1, 300), png(2, 300));
    let pictures = [(Kind::Png, one.as_slice()), (Kind::Png, two.as_slice())];
    let sealed = seal(&key(9), &draft("hola", &pictures), None).unwrap();

    let mut swapped = sealed.clone();
    swapped.parts.swap(1, 2);
    assert_eq!(open(&key(9), &swapped, &[]).err(), Some(Refused::Locked));

    let mut cut = sealed.clone();
    cut.parts.pop();
    cut.manifest.parts.pop();
    assert!(open(&key(9), &cut, &[]).is_err());
}

#[test]
fn every_limit_is_held_when_sealing() {
    let big_text = "a".repeat(TEXT_AT_MOST + 1);
    assert_eq!(
        seal(&key(9), &draft(&big_text, &[]), None).err(),
        Some(Refused::TextTooBig)
    );
    assert!(seal(&key(9), &draft(&"a".repeat(TEXT_AT_MOST), &[]), None).is_ok());

    let big = png(1, PICTURE_AT_MOST + 1);
    assert_eq!(
        seal(&key(9), &draft("x", &[(Kind::Png, &big)]), None).err(),
        Some(Refused::PictureTooBig)
    );

    let small = png(1, 10);
    let eleven: Vec<(Kind, &[u8])> = (0..=PICTURES_AT_MOST)
        .map(|_| (Kind::Png, small.as_slice()))
        .collect();
    assert_eq!(
        seal(&key(9), &draft("x", &eleven), None).err(),
        Some(Refused::TooManyPictures)
    );

    let half = png(1, 450 * 1024);
    let five: Vec<(Kind, &[u8])> = (0..5).map(|_| (Kind::Png, half.as_slice())).collect();
    assert_eq!(
        seal(&key(9), &draft("x", &five), None).err(),
        Some(Refused::PicturesTooBig)
    );
}

#[test]
fn a_file_passed_off_as_a_picture_never_leaves() {
    let pdf = b"%PDF-1.7 a document".to_vec();
    let disguised = png(1, 20);
    assert_eq!(
        seal(&key(9), &draft("x", &[(Kind::Png, &pdf)]), None).err(),
        Some(Refused::NotAPicture)
    );
    assert_eq!(
        seal(&key(9), &draft("x", &[(Kind::Jpeg, &disguised)]), None).err(),
        Some(Refused::NotAPicture)
    );
    assert_eq!(
        seal(&key(9), &draft("x", &[(Kind::Text, &disguised)]), None).err(),
        Some(Refused::NotAPicture)
    );
}

#[test]
fn a_manifest_over_the_limits_is_refused_by_the_server_before_anything_is_kept() {
    let manifest = Manifest {
        v: SAID,
        id: "k7Qm2xPa".into(),
        version: 1,
        from: None,
        head: URL_SAFE_NO_PAD.encode([0u8; HEAD]),
        parts: std::iter::once(Part {
            kind: Kind::Text,
            size: 20,
        })
        .chain((0..=PICTURES_AT_MOST).map(|_| Part {
            kind: Kind::Png,
            size: 30,
        }))
        .collect(),
    };
    let written = serde_json::to_vec(&manifest).unwrap();
    let mut bytes = MAGIC.to_vec();
    bytes.push(VERSION);
    bytes.extend_from_slice(&(written.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&written);

    assert_eq!(Envelope::read(&bytes).err(), Some(Refused::TooManyPictures));
}

#[test]
fn a_version_must_come_after_the_one_it_was_written_on() {
    let mut twisted = draft("x", &[]);
    twisted.from = Some(3);
    assert_eq!(seal(&key(9), &twisted, None).err(), Some(Refused::Damaged));
    let mut first = draft("x", &[]);
    first.version = 1;
    first.from = None;
    assert!(seal(&key(9), &first, None).is_ok());
}

#[test]
fn an_id_is_only_letters_digits_dashes_and_underscores() {
    for id in ["", "a/b", "dos palabras", &"x".repeat(ID_AT_MOST + 1)] {
        let mut odd = draft("x", &[]);
        odd.id = id;
        assert_eq!(
            seal(&key(9), &odd, None).err(),
            Some(Refused::Damaged),
            "{id:?}"
        );
    }
}

#[test]
fn a_revoked_machine_reads_as_revoked() {
    let mario = machine(1);
    let sealed = seal(&key(9), &draft("hola", &[]), Some(&mario.signer())).unwrap();
    let taken_back = mario
        .root
        .revoke(&mario.key.public(), Timestamp::from_second(2_000).unwrap());

    let opened = open(&key(9), &sealed, std::slice::from_ref(&taken_back)).unwrap();
    assert!(matches!(opened.wrote, Some(Verdict::Revoked { .. })));
    assert!(matches!(
        sealed.sender(&[taken_back]),
        Some(Verdict::Revoked { .. })
    ));
}

#[test]
fn a_machine_whose_certificate_is_another_key_s_cannot_sign() {
    let mario = machine(1);
    let stranger = ShareKey::from_bytes(&[42; 32]);
    let borrowed = Signer {
        key: &stranger,
        certificate: &mario.certificate,
    };
    assert_eq!(
        seal(&key(9), &draft("hola", &[]), Some(&borrowed)).err(),
        Some(Refused::Unsigned)
    );
}

#[test]
fn bytes_not_sealed_by_tisty_share_or_by_a_newer_one_say_so() {
    assert_eq!(Envelope::read(b"# hola").err(), Some(Refused::Foreign));
    let mut bytes = seal(&key(9), &draft("hola", &[]), None).unwrap().to_bytes();
    bytes[4] = VERSION + 1;
    assert_eq!(Envelope::read(&bytes).err(), Some(Refused::Newer));
    assert_eq!(Envelope::read(&bytes[..3]).err(), Some(Refused::Foreign));
}

#[test]
fn trailing_or_missing_bytes_are_refused() {
    let bytes = seal(&key(9), &draft("hola", &[]), None).unwrap().to_bytes();
    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(Envelope::read(&longer).err(), Some(Refused::Damaged));
    assert_eq!(
        Envelope::read(&bytes[..bytes.len() - 1]).err(),
        Some(Refused::Damaged)
    );
}

#[test]
fn the_link_key_travels_as_text_and_back() {
    let minted = LinkKey::minted().unwrap();
    let said = minted.to_text();
    assert_eq!(said.len(), 43);
    let back = LinkKey::from_text(&said).unwrap();
    assert_eq!(back.to_text().as_str(), said.as_str());
    assert!(LinkKey::from_text("short").is_none());
}

#[test]
fn two_seals_of_the_same_words_never_look_alike() {
    let one = seal(&key(9), &draft("hola", &[]), None).unwrap();
    let two = seal(&key(9), &draft("hola", &[]), None).unwrap();
    assert_ne!(one.parts, two.parts);
    assert_ne!(one.manifest.head, two.manifest.head);
}
