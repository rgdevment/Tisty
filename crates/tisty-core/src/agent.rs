use crate::{
    Config, Result,
    event::{DeviceId, DeviceKind, Op},
    paths::Paths,
    store::Store,
};

/// Minting is the person's act: nothing reachable over a wire calls this.
pub fn register(paths: &Paths) -> Result<DeviceId> {
    let mut config = Config::load_or_init(paths)?;
    if let Some(held) = config.agent_id.clone() {
        return Ok(held);
    }

    let who = DeviceId(crate::config::new_device_id());
    config.agent_id = Some(who.clone());
    config.save(paths)?;

    let mut store = Store::open(paths.store(), who.clone())?;
    store.append(Op::DeviceJoin {
        d: who.clone(),
        k: Some(DeviceKind::Agent),
    })?;
    Ok(who)
}

/// What it wrote stays: retiring takes the voice, never the words.
pub fn retire(paths: &Paths) -> Result<Option<DeviceId>> {
    let mut config = Config::load_or_init(paths)?;
    let Some(who) = config.agent_id.clone() else {
        return Ok(None);
    };

    config.agent_id = None;
    config.save(paths)?;

    let mut store = Store::open(paths.store(), config.device_id.clone())?;
    store.append(Op::DeviceRemove { d: who.clone() })?;
    Ok(Some(who))
}

pub fn registered(paths: &Paths) -> Result<Option<DeviceId>> {
    Ok(Config::load_or_init(paths)?.agent_id)
}

/// Where an agent may take a file from: what it attaches reaches the shared folder.
pub fn may_attach(source: &std::path::Path, paths: &Paths) -> Result<std::path::PathBuf> {
    let refused = || crate::Error::OutsideTheStore(source.display().to_string());
    let at = source.canonicalize().map_err(|_| refused())?;

    let mine = [paths.data(), paths.config(), paths.cache()];
    if mine
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| at.starts_with(one))
    {
        return Err(refused());
    }

    if !reachable().iter().any(|root| at.starts_with(root)) {
        return Err(refused());
    }
    fit_to_keep(&at)?;
    Ok(at)
}

/// What an assistant may keep, and how it is judged: by the bytes, never by the name. A denylist
/// of extensions is one rename away from useless, so this asks the file what it is.
const FOR_AN_AGENT: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "avif", "heic", "svg", "pdf", "txt", "md", "csv", "xml",
    "html", "htm", "docx", "xlsx", "pptx", "odt", "ods", "mp4", "m4v", "mov", "webm", "ogv", "mp3",
    "m4a", "wav", "ogg", "zip", "7z", "gz", "tgz", "tar",
];

fn named_type(at: &std::path::Path) -> Option<String> {
    Some(at.extension()?.to_str()?.to_ascii_lowercase())
}

/// A signature the bytes must carry for the name to be believed. Text formats have none, so they
/// are judged by what they must not contain instead.
fn signed_as(kind: &str, head: &[u8]) -> bool {
    let starts = |mark: &[u8]| head.starts_with(mark);
    match kind {
        "png" => starts(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => starts(&[0xFF, 0xD8, 0xFF]),
        "gif" => starts(b"GIF87a") || starts(b"GIF89a"),
        "webp" => starts(b"RIFF") && head.len() > 12 && &head[8..12] == b"WEBP",
        "avif" | "heic" | "mp4" | "m4v" | "mov" => head.len() > 12 && &head[4..8] == b"ftyp",
        "webm" | "ogv" | "ogg" => starts(&[0x1A, 0x45, 0xDF, 0xA3]) || starts(b"OggS"),
        "mp3" => starts(b"ID3") || (head.len() > 1 && head[0] == 0xFF && head[1] & 0xE0 == 0xE0),
        "m4a" => head.len() > 12 && &head[4..8] == b"ftyp",
        "wav" => starts(b"RIFF") && head.len() > 12 && &head[8..12] == b"WAVE",
        "pdf" => starts(b"%PDF-"),
        "docx" | "xlsx" | "pptx" | "odt" | "ods" | "zip" => starts(&[0x50, 0x4B]),
        "7z" => starts(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
        "gz" | "tgz" => starts(&[0x1F, 0x8B]),
        // A tar says what it is 257 bytes in, where the first entry's header carries the marker.
        "tar" => head.len() > 262 && &head[257..262] == b"ustar",
        _ => true,
    }
}

/// Secrets do not announce themselves by extension. These are the shapes they do carry.
pub fn holds_a_secret(head: &[u8]) -> bool {
    if head.starts_with(&[0x30, 0x82]) {
        return true;
    }
    let Ok(text) = std::str::from_utf8(head) else {
        return false;
    };
    if text.contains("-----BEGIN") && (text.contains("PRIVATE KEY") || text.contains("CERTIFICATE"))
    {
        return true;
    }
    let telling = |line: &str| {
        let line = line.trim();
        line.split_once('=').is_some_and(|(key, value)| {
            !key.is_empty()
                && key
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && value.len() > 12
        })
    };
    text.lines().filter(|one| telling(one)).count() >= 2
}

/// Whether the file itself, not its name, is something an assistant is allowed to copy.
pub fn fit_to_keep(at: &std::path::Path) -> Result<()> {
    let refused = || crate::Error::NotForAnAgent(at.display().to_string());
    let kind = named_type(at).ok_or_else(refused)?;
    if !FOR_AN_AGENT.contains(&kind.as_str()) {
        return Err(refused());
    }

    let head = read_head(at).map_err(|_| refused())?;
    if !signed_as(&kind, &head) || holds_a_secret(&head) {
        return Err(refused());
    }
    Ok(())
}

fn read_head(at: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut head = vec![0u8; 4096];
    let read = std::fs::File::open(at)?.read(&mut head)?;
    head.truncate(read);
    Ok(head)
}

pub fn may_reach(at: &std::path::Path, paths: &Paths) -> Result<std::path::PathBuf> {
    let refused = || crate::Error::OutsideTheStore(at.display().to_string());
    let found = at.canonicalize().map_err(|_| refused())?;

    let mine = [paths.data(), paths.config(), paths.cache()];
    if mine
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| found.starts_with(one))
    {
        return Err(refused());
    }
    if !reachable().iter().any(|root| found.starts_with(root)) {
        return Err(refused());
    }
    Ok(found)
}

pub fn reachable() -> Vec<std::path::PathBuf> {
    let mut roots = vec![std::env::temp_dir()];
    if let Some(dirs) = directories::UserDirs::new() {
        for one in [
            dirs.download_dir(),
            dirs.document_dir(),
            dirs.picture_dir(),
            dirs.desktop_dir(),
        ] {
            roots.extend(one.map(std::path::Path::to_path_buf));
        }
    }
    roots
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrote(at: &std::path::Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let one = at.join(name);
        std::fs::write(&one, bytes).unwrap();
        one
    }

    #[test]
    fn what_an_assistant_may_keep_is_judged_by_the_bytes_not_the_name() {
        let room = tempfile::tempdir().unwrap();
        let at = room.path();

        let png = wrote(at, "shot.png", b"\x89PNG\r\n\x1a\nrest of it");
        let pdf = wrote(at, "invoice.pdf", b"%PDF-1.7 and the rest");
        let note = wrote(at, "notes.md", b"# what the group said\n\nbring card stock");
        for one in [&png, &pdf, &note] {
            assert!(fit_to_keep(one).is_ok(), "{}", one.display());
        }

        // The four that were sitting in a real Downloads folder when this was written.
        let env = wrote(
            at,
            "shell-secrets.env",
            b"AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMIK7MDENGbPxRfiCY\nGITHUB_TOKEN=ghp_16C7e42F292c6912",
        );
        let p12 = wrote(at, "privada.p12", &[0x30, 0x82, 0x0A, 0x00]);
        let pem = wrote(at, "key.pem", b"-----BEGIN RSA PRIVATE KEY-----\nMIIEow\n");
        let conf = wrote(at, "vpn.conf", b"[main]\nserver=vpn.example.com");
        for one in [&env, &p12, &pem, &conf] {
            assert!(fit_to_keep(one).is_err(), "{}", one.display());
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
    fn renaming_a_secret_does_not_get_it_past() {
        let room = tempfile::tempdir().unwrap();
        let at = room.path();

        // The defect a denylist of extensions cannot fix: the name says one thing, the bytes another.
        let dressed = wrote(at, "holiday.png", &[0x30, 0x82, 0x0A, 0x00]);
        let also = wrote(
            at,
            "receipt.pdf",
            b"-----BEGIN PRIVATE KEY-----\nMIIEvQIBADAN\n",
        );
        assert!(
            fit_to_keep(&dressed).is_err(),
            "a p12 called .png is still a p12"
        );
        assert!(
            fit_to_keep(&also).is_err(),
            "a key called .pdf is still a key"
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
}
