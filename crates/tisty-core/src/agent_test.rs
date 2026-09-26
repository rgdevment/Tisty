use super::*;

fn wrote(at: &std::path::Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let one = at.join(name);
    std::fs::write(&one, bytes).unwrap();
    one
}

#[test]
fn what_an_assistant_may_keep_is_anything_whose_bytes_match_its_name() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path();

    let png = wrote(at, "shot.png", b"\x89PNG\r\n\x1a\nrest of it");
    let pdf = wrote(at, "invoice.pdf", b"%PDF-1.7 and the rest");
    let note = wrote(at, "notes.md", b"# what the group said\n\nbring card stock");
    for one in [&png, &pdf, &note] {
        assert!(fit_to_keep(one).is_ok(), "{}", one.display());
    }

    let code = wrote(at, "subir.py", b"import os\n\nprint(os.getcwd())\n");
    let stack = wrote(
        at,
        "compose.yaml",
        b"services:\n  db:\n    image: postgres\n",
    );
    let conf = wrote(at, "vpn.conf", b"[main]\nserver=vpn.example.com");
    let built = wrote(at, "tisty.exe", b"MZ\x90\x00this is a binary");
    for one in [&code, &stack, &conf, &built] {
        assert!(fit_to_keep(one).is_ok(), "{}", one.display());
    }

    let p12 = wrote(at, "privada.p12", &[0x30, 0x82, 0x0A, 0x00]);
    let pem = wrote(at, "key.pem", b"-----BEGIN RSA PRIVATE KEY-----\nMIIEow\n");
    for one in [&p12, &pem] {
        assert!(fit_to_keep(one).is_ok(), "{}", one.display());
    }
}

#[test]
fn a_container_is_kept_when_its_bytes_say_it_is_one() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path();
    let mut tar = vec![0u8; 512];
    tar[257..262].copy_from_slice(b"ustar");

    let zip = wrote(at, "actas.zip", b"PKrest of it");
    let seven = wrote(at, "actas.7z", &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00]);
    let gz = wrote(at, "logs.gz", &[0x1F, 0x8B, 0x08, 0x00]);
    let tarred = wrote(at, "todo.tar", &tar);
    for one in [&zip, &seven, &gz, &tarred] {
        assert!(fit_to_keep(one).is_ok(), "{}", one.display());
    }

    let lying = wrote(at, "actas.zip", b"not a zip at all");
    let short = wrote(at, "todo.tar", b"nowhere near 512 bytes");
    for one in [&lying, &short] {
        assert!(fit_to_keep(one).is_err(), "{}", one.display());
    }
}

#[test]
fn a_file_dressed_as_another_kind_does_not_get_it_past() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path();

    let dressed = wrote(at, "holiday.png", &[0x30, 0x82, 0x0A, 0x00]);
    let also = wrote(
        at,
        "receipt.pdf",
        b"-----BEGIN PRIVATE KEY-----\nMIIEvQIBADAN\n",
    );
    assert!(
        fit_to_keep(&dressed).is_err(),
        "un p12 llamado .png no es un png"
    );
    assert!(
        fit_to_keep(&also).is_err(),
        "una clave llamada .pdf no es un pdf"
    );
}

fn paths(at: &std::path::Path) -> Paths {
    Paths::new(at.join("data"), at.join("config"))
}

#[test]
fn registering_twice_does_not_mint_a_second_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(tmp.path());

    let first = register(&paths).unwrap();
    let again = register(&paths).unwrap();

    assert_eq!(first, again);
    let events = crate::store::read_all(paths.store()).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(&e.op, Op::DeviceJoin { .. }))
            .count(),
        1
    );
}

#[test]
fn an_agent_joins_as_one_and_the_state_says_so() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(tmp.path());

    let who = register(&paths).unwrap();
    let state = crate::State::replay(&crate::store::read_all(paths.store()).unwrap());

    assert!(
        state.agents.contains(&who),
        "a machine would not be listed here"
    );
    assert!(state.devices.contains(&who));
}

#[test]
fn retiring_takes_the_voice_and_leaves_the_words() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(tmp.path());
    let who = register(&paths).unwrap();

    let mut store = Store::open(paths.store(), who.clone()).unwrap();
    let id = ulid::Ulid::generate();
    store
        .append(Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new("what it filed", "a0"),
        })
        .unwrap();

    assert_eq!(retire(&paths).unwrap(), Some(who.clone()));
    assert_eq!(registered(&paths).unwrap(), None);

    let state = crate::State::replay(&crate::store::read_all(paths.store()).unwrap());
    assert!(state.tasks.contains_key(&id), "retiring is not a purge");
    assert!(!state.agents.contains(&who));
}

#[test]
fn retiring_when_none_was_registered_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    assert_eq!(retire(&paths(tmp.path())).unwrap(), None);
}

fn headed(mark: &[u8]) -> Vec<u8> {
    let mut head = vec![b'-'; 320];
    head[..mark.len()].copy_from_slice(mark);
    head
}

fn marked(at: usize, mark: &[u8]) -> Vec<u8> {
    let mut head = vec![b'-'; 320];
    head[at..at + mark.len()].copy_from_slice(mark);
    head
}

#[test]
fn every_kind_is_known_by_the_bytes_it_opens_with() {
    let riff = |tail: &[u8]| {
        let mut head = headed(b"RIFF");
        head[8..12].copy_from_slice(tail);
        head
    };

    let known: Vec<(&str, Vec<u8>)> = vec![
        ("png", headed(b"\x89PNG\r\n\x1a\n")),
        ("jpg", headed(&[0xFF, 0xD8, 0xFF])),
        ("jpeg", headed(&[0xFF, 0xD8, 0xFF])),
        ("gif", headed(b"GIF87a")),
        ("gif", headed(b"GIF89a")),
        ("webp", riff(b"WEBP")),
        ("wav", riff(b"WAVE")),
        ("avif", marked(4, b"ftyp")),
        ("heic", marked(4, b"ftyp")),
        ("mp4", marked(4, b"ftyp")),
        ("m4v", marked(4, b"ftyp")),
        ("mov", marked(4, b"ftyp")),
        ("m4a", marked(4, b"ftyp")),
        ("webm", headed(&[0x1A, 0x45, 0xDF, 0xA3])),
        ("ogv", headed(b"OggS")),
        ("ogg", headed(b"OggS")),
        ("mp3", headed(b"ID3")),
        ("mp3", headed(&[0xFF, 0xE0])),
        ("pdf", headed(b"%PDF-")),
        ("docx", headed(&[0x50, 0x4B])),
        ("xlsx", headed(&[0x50, 0x4B])),
        ("pptx", headed(&[0x50, 0x4B])),
        ("odt", headed(&[0x50, 0x4B])),
        ("ods", headed(&[0x50, 0x4B])),
        ("zip", headed(&[0x50, 0x4B])),
        ("7z", headed(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C])),
        ("gz", headed(&[0x1F, 0x8B])),
        ("tgz", headed(&[0x1F, 0x8B])),
        ("tar", marked(257, b"ustar")),
    ];

    for (kind, head) in known {
        assert!(
            signed_as(kind, &head),
            "{kind} was not read from its own bytes"
        );
    }
}

#[test]
fn a_name_that_lies_about_its_bytes_is_refused() {
    let plain = vec![b'-'; 320];

    for kind in [
        "png", "jpg", "jpeg", "gif", "webp", "wav", "avif", "heic", "mp4", "m4v", "mov", "m4a",
        "webm", "ogv", "ogg", "mp3", "pdf", "docx", "xlsx", "pptx", "odt", "ods", "zip", "7z",
        "gz", "tgz", "tar",
    ] {
        assert!(
            !signed_as(kind, &plain),
            "{kind} took bytes that are not its own"
        );
    }
}

#[test]
fn a_signature_that_runs_off_the_end_is_no_signature() {
    assert!(!signed_as("webp", b"RIFF"));
    assert!(!signed_as("wav", b"RIFF"));
    assert!(!signed_as("mp4", b"ftyp"));
    assert!(!signed_as("tar", b"ustar"));
    assert!(!signed_as("mp3", &[0xFF]));
}

#[test]
fn a_kind_with_no_signature_of_its_own_is_taken_at_its_word() {
    assert!(signed_as("md", &vec![b'-'; 320]));
    assert!(signed_as("txt", b""));
    assert!(signed_as("rs", b"fn main() {}"));
}

#[test]
fn a_file_cut_back_to_its_bare_signature_is_refused() {
    let mut webp = b"RIFF----WEBP".to_vec();
    assert!(!signed_as("webp", &webp));
    webp.push(b'-');
    assert!(signed_as("webp", &webp));

    let mut wav = b"RIFF----WAVE".to_vec();
    assert!(!signed_as("wav", &wav));
    wav.push(b'-');
    assert!(signed_as("wav", &wav));

    let mut ftyp = b"----ftyp----".to_vec();
    assert!(!signed_as("mp4", &ftyp));
    assert!(!signed_as("m4a", &ftyp));
    ftyp.push(b'-');
    assert!(signed_as("mp4", &ftyp));
    assert!(signed_as("m4a", &ftyp));

    let mut tar = vec![b'-'; 262];
    tar[257..262].copy_from_slice(b"ustar");
    assert!(!signed_as("tar", &tar));
    tar.push(b'-');
    assert!(signed_as("tar", &tar));
}

#[test]
fn an_mp3_frame_is_read_by_the_bits_that_are_set_and_not_the_ones_that_are_not() {
    assert!(signed_as("mp3", &headed(&[0xFF, 0xFB])));
    assert!(signed_as("mp3", &headed(&[0xFF, 0xE0])));
    assert!(!signed_as("mp3", &headed(&[0xFF, 0x1F])));
}

fn guarded_by(word: &str) -> String {
    format!("DATABASE_URL=postgres://admin:{word}@db.example.com/app\n")
}

#[test]
fn what_reads_as_a_credential_is_told() {
    let told = |text: &str| secrets_in(text.as_bytes());

    let aws = told("AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMIK7MDENGbPxRfiCYEXAMPLEKEY\n");
    assert_eq!(aws.len(), 1);
    assert_eq!(aws[0].named, "AWS_SECRET_ACCESS_KEY");

    let prefixed = told("ghp_1234567890abcdefghijklmnopqrstuvwx\n");
    assert_eq!(prefixed.len(), 1);
    assert_eq!(prefixed[0].named, "ghp_");

    let linked = told(&guarded_by("Sup3rSecret"));
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].why, "a link with a password written into it");

    assert_eq!(told("-----BEGIN PRIVATE KEY-----\nMIIEow\n").len(), 1);
    assert_eq!(secrets_in(&[0x30, 0x82, 0x0A, 0x00]).len(), 1);
}

#[test]
fn what_only_looks_like_a_credential_is_left_alone() {
    let quiet = |text: &str| {
        let found = secrets_in(text.as_bytes());
        assert!(
            found.is_empty(),
            "{text:?} was told as {}",
            found.first().map(|one| one.why).unwrap_or_default()
        );
    };

    quiet("API_KEY=${MyToken1}\nAUTH_TOKEN=${OtherValue2}\n");
    quiet("API_KEY=$(read MyToken1)\nAUTH_TOKEN=$(read Other2)\n");
    quiet("SECRET_PATH=./Config/Local1\nTOKEN_FILE=../Shared/Creds2\n");
    quiet("PASSWORD=~/Keys/mine1\nTOKEN=<YourValue1>\n");
    quiet("BUILD_ID=a1b2c3D4e5f6xyz\nRELEASE_TAG=v2Beta7xyzQ\n");
    quiet("HOMEPAGE=https://example.com/a/very/long/path\nDOCS_URL=https://docs.example.com/x\n");
    quiet("PASSWORD=abc\nTOKEN=xy\n");
    quiet("SECRET=one two three four\nTOKEN=five six seven\n");
    quiet("my-key!=Sup3rSecretValue1234567890\n");
    quiet("SECRET=abcdefghijklmnop\nTOKEN=qrstuvwxyzabcdef\n");
}

#[test]
fn the_identity_an_agent_registered_is_the_one_read_back() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(tmp.path());

    assert_eq!(registered(&paths).unwrap(), None);

    let who = register(&paths).unwrap();
    assert_eq!(registered(&paths).unwrap(), Some(who));
}

#[test]
fn what_an_assistant_may_take_comes_from_outside_the_store() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(tmp.path());
    std::fs::create_dir_all(paths.data()).unwrap();

    let outside = wrote(tmp.path(), "shot.png", b"\x89PNG\r\n\x1a\nrest of it");
    assert_eq!(
        may_attach(&outside, &paths).unwrap(),
        outside.canonicalize().unwrap()
    );

    let inside = wrote(paths.data(), "shot.png", b"\x89PNG\r\n\x1a\nrest of it");
    assert!(may_attach(&inside, &paths).is_err());

    assert!(may_attach(&tmp.path().join("nowhere.png"), &paths).is_err());
}

#[test]
fn a_placeholder_written_into_the_middle_is_still_a_placeholder() {
    let quiet = |text: &str| {
        let found = secrets_in(text.as_bytes());
        assert!(
            found.is_empty(),
            "{text:?} was told as {}",
            found.first().map(|one| one.why).unwrap_or_default()
        );
    };

    quiet("SECRET=abc${VAR}defGH1\nTOKEN=xyz${OTHER}uvwJK2\n");
    quiet("SECRET=abc$(cat one)dGH1\nTOKEN=xyz$(cat two)uJK2\n");
    quiet("SECRET=One Two Three4\nTOKEN=Four Five Six7\n");
    quiet("PASSWORD=aB3\nTOKEN=cD4\n");
    quiet("AUTH_URL=https://Example.com/Path1\nTOKEN_URL=https://Other.com/Way2\n");
}

#[test]
fn a_value_as_long_as_the_shortest_allowed_is_still_read() {
    let found = secrets_in("SECRET=aB3dEf7h\nTOKEN=jK5mNp9q\n".as_bytes());
    assert_eq!(found.len(), 2);
}

#[test]
fn a_link_whose_password_is_too_short_to_be_one_is_left_alone() {
    let found = secrets_in(guarded_by("sh0Rt").as_bytes());
    assert!(found.is_empty());
}

#[test]
fn a_key_needs_the_line_it_opens_with_and_not_just_its_name() {
    assert!(secrets_in(b"note: my PRIVATE KEY is in the safe upstairs\n").is_empty());
    assert!(secrets_in(b"we keep the CERTIFICATE in the drawer downstairs\n").is_empty());
}

#[test]
fn a_key_is_told_at_the_line_it_opens_on() {
    let deep = secrets_in(b"first line\nsecond line\n-----BEGIN PRIVATE KEY-----\nMIIEow\n");
    assert_eq!(deep.len(), 1);
    assert_eq!(deep[0].line, 3);

    let top = secrets_in(b"-----BEGIN PRIVATE KEY-----\nMIIEow\n");
    assert_eq!(top[0].line, 1);
}

#[test]
fn what_an_assistant_may_read_comes_from_outside_the_store() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(tmp.path());
    std::fs::create_dir_all(paths.data()).unwrap();

    let outside = wrote(tmp.path(), "notes.md", b"# what the group said");
    assert_eq!(
        may_reach(&outside, &paths).unwrap(),
        outside.canonicalize().unwrap()
    );

    let inside = wrote(paths.data(), "notes.md", b"# what the group said");
    assert!(may_reach(&inside, &paths).is_err());

    assert!(may_reach(&tmp.path().join("nowhere.md"), &paths).is_err());
}

#[test]
fn the_places_an_assistant_may_reach_are_real_and_named() {
    let roots = reachable();

    assert!(!roots.is_empty());
    for one in &roots {
        assert!(
            one.is_absolute(),
            "{} is not an absolute path",
            one.display()
        );
        assert!(one.exists(), "{} is not there", one.display());
    }
    assert!(roots.contains(&std::env::temp_dir().canonicalize().unwrap()));
}
