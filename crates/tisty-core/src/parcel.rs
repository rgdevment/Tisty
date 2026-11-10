use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tisty_share_proto::cloud::nonced;
use ulid::Ulid;

use crate::{
    Error, Paths, Result,
    event::{DeviceId, DocAdd, FolderAdd, Op, Said},
    model::{DEEPEST, DocId, FolderId, Kept},
    signed_parcel::{self, Asked, Closed, Hashing, Prints, Signer},
    state::State,
};

pub const EXTENSION: &str = "tistyx";
const KIND: &str = "tisty-docs";
const VERSION: u32 = 3;
/// From here a parcel speaks Tisty's own words; what an older one says is turned on its way in.
const SPOKEN_FROM: u32 = 3;
const MANIFEST: &str = "tisty-docs.json";
const READING: &str = "README.txt";
const READ_ME: &str = "This is a Tisty parcel.

It is an ordinary zip file: open it and the documents are inside `docs/`, as
plain Markdown you can read in any editor. Nothing here needs Tisty to be read.

Tisty opens it whole, with the folders, the titles and the attachments back in
place: File > Open a parcel. It is free and open source, for Windows and macOS.

  https://github.com/rgdevment/Tisty

---

Esto es un paquete de Tisty.

Es un zip corriente: abrelo y los documentos estan en `docs/`, como Markdown
que puedes leer en cualquier editor. Nada de aqui necesita Tisty para leerse.

Tisty lo abre entero, con las carpetas, los titulos y los adjuntos en su sitio:
Archivo > Abrir un paquete. Es gratis y de codigo abierto, para Windows y macOS.

  https://github.com/rgdevment/Tisty
";
const CARRIED: [&str; 2] = ["docs", "attachments"];
const AT_MOST: u64 = 8 * 1024 * 1024 * 1024;
const AT_MOST_FILES: usize = 200_000;
const MANIFEST_AT_MOST: u64 = 16 * 1024 * 1024;
const PAPERS_AT_MOST: usize = 50_000;
const TITLE_AT_MOST: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub kind: String,
    pub version: u32,
    pub from: String,
    /// Proof that the store named in `from` really wrote this, and not somebody who read its
    /// name off a parcel it once handed out. Absent, or wrong, and it lands as a stranger's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seal: Option<String>,
    pub folders: Vec<Shelf>,
    pub docs: Vec<Paper>,
}

fn sealed(manifest: &Manifest, keep: &[u8]) -> Option<String> {
    use hmac::{KeyInit, Mac};
    let bare = Manifest {
        seal: None,
        ..manifest.clone()
    };
    let said = serde_json::to_vec(&bare).ok()?;
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(keep).ok()?;
    mac.update(&said);
    Some(
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|one| format!("{one:02x}"))
            .collect(),
    )
}

/// Written here, or only claiming to be. A parcel with no seal at all is somebody else's by
/// definition: every Tisty that writes one seals it.
fn ours(manifest: &Manifest, paths: &Paths) -> bool {
    let mine = crate::store::peek_identity(paths.store());
    let said = manifest.from.trim();
    if said.is_empty() || mine.as_deref() != Some(said) {
        return false;
    }
    let Some(keep) = crate::store::secret(paths) else {
        return false;
    };
    match (&manifest.seal, sealed(manifest, &keep)) {
        (Some(theirs), Some(ours)) => theirs.as_bytes().ct_eq(ours.as_bytes()),
        _ => false,
    }
}

trait Steady {
    fn ct_eq(&self, other: &Self) -> bool;
}

impl Steady for [u8] {
    /// Compared to the end however early it differs, so the time it takes says nothing about
    /// how much of the seal was right.
    fn ct_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other)
                .fold(0u8, |told, (a, b)| told | (a ^ b))
                == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shelf {
    pub id: String,
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marked {
    pub at: jiff::Timestamp,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paper {
    pub file: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrote: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub made: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    /// What the archive holds, so a reader that knows nothing of shelved folders still closes it.
    pub archived: bool,
    /// Set when only the folder above put it away, so bringing that folder back opens it again.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub by_folder: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub away_alone: Option<bool>,
    /// What an assistant said had had its day, so the person still has it to answer wherever the
    /// parcel lands. Without the device that said it: a parcel goes to other people.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flagged: Option<Marked>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub locked: bool,
    /// Somebody else's writing that this store is only holding: it stays theirs wherever it
    /// goes next, rather than turning native by passing through here.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub guest: bool,
    /// The file of the anchor whose languages it is one of, when that one travels too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation_of: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub done: usize,
    pub whole: usize,
    pub bytes: u64,
}

#[derive(Default)]
pub struct Along<'a> {
    pub also: Option<&'a Path>,
    /// Locking a parcel, and opening one, are half the work and count in bytes rather than in
    /// documents. Told apart so the window can name the stage instead of sitting at 100 %.
    pub then: Option<&'a dyn Fn(Step)>,
    pub say: Option<&'a dyn Fn(Step)>,
}

impl Along<'_> {
    fn at(&self, done: usize, whole: usize, bytes: u64) {
        if let Some(say) = self.say {
            say(Step { done, whole, bytes });
        }
    }

    fn sealing(&self, done: u64, whole: u64) {
        if let Some(then) = self.then.or(self.say) {
            then(Step {
                done: done as usize,
                whole: whole as usize,
                bytes: done,
            });
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Sent {
    pub docs: usize,
    pub pages: usize,
    pub folders: usize,
    pub files: usize,
    pub bytes: u64,
    pub missed: usize,
    pub left: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Landed {
    pub docs: usize,
    pub pages: usize,
    pub folders: usize,
    pub joined: usize,
    pub files: usize,
    pub missed: usize,
    pub signer: Option<Claimed>,
    pub broken: bool,
}

pub use crate::signed_parcel::{Answer, Claimed};

/// What a locked parcel says it is, before anybody is asked for a number.
const LOCKED: &[u8; 8] = b"TISTYX1
";
/// scrypt is deliberately slow: a short number is worth little if a key is cheap to try.
const WORK: u8 = 16;
/// What we will grind for somebody else's file, either way: 2^14 is a fifth of a second, and
/// 2^18 already asks for a quarter of a gigabyte.
const WORK_AT_LEAST: u8 = 14;
const WORK_AT_MOST: u8 = 18;
const BLOCK: usize = 64 * 1024;
const TAG: usize = 16;

pub fn locked(at: &Path) -> bool {
    let mut head = [0u8; 8];
    std::fs::File::open(at)
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut head))
        .is_ok_and(|()| head.starts_with(&LOCKED[..6]))
}

/// The key is worth more than the number it came from — a short number is only slow to guess,
/// while the key it grinds out is not — so it is wiped rather than left in freed memory.
fn keyed(number: &str, salt: &[u8], work: u8) -> Result<zeroize::Zeroizing<[u8; 32]>> {
    let mut key = zeroize::Zeroizing::new([0u8; 32]);
    let how = scrypt::Params::new(work, 8, 1).map_err(|_| Error::WrongNumber)?;
    // Composed first, so a number typed on a Mac and the same one typed on Windows are the same
    // number rather than two spellings that grind out different keys.
    let said = crate::text::composed(number);
    scrypt::scrypt(said.as_bytes(), salt, &how, key.as_mut()).map_err(|_| Error::WrongNumber)?;
    Ok(key)
}

fn shut(from: &Path, into: &Path, number: &str, along: &Along) -> Result<()> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use rand_core::TryRngCore;
    use std::io::{Read, Write};

    let whole = std::fs::metadata(from).map(|one| one.len()).unwrap_or(0);
    let mut done = 0u64;

    let mut salt = [0u8; 16];
    let mut head = [0u8; 19];
    rand_core::OsRng
        .try_fill_bytes(&mut salt)
        .and_then(|()| rand_core::OsRng.try_fill_bytes(&mut head))
        .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

    let key = keyed(number, &salt, WORK)?;
    let sealer = chacha20poly1305::XChaCha20Poly1305::new_from_slice(key.as_ref())
        .map_err(|_| Error::WrongNumber)?;

    let mut plain = std::io::BufReader::new(std::fs::File::open(from)?);
    let mut out = std::io::BufWriter::new(std::fs::File::create(into)?);
    out.write_all(LOCKED)?;
    out.write_all(&[WORK])?;
    out.write_all(&salt)?;
    out.write_all(&head)?;

    let mut block = vec![0u8; BLOCK];
    let mut at = 0u32;
    let mut held: Option<Vec<u8>> = None;
    loop {
        let mut filled = 0;
        while filled < BLOCK {
            match plain.read(&mut block[filled..])? {
                0 => break,
                got => filled += got,
            }
        }
        let last = filled < BLOCK;
        if let Some(before) = held.take() {
            let sealed = sealer
                .encrypt(&nonced(&head, at, false).into(), before.as_slice())
                .map_err(|_| Error::WrongNumber)?;
            out.write_all(&sealed)?;
            at = at.checked_add(1).ok_or(Error::TooBig)?;
        }
        if last {
            let sealed = sealer
                .encrypt(&nonced(&head, at, true).into(), &block[..filled])
                .map_err(|_| Error::WrongNumber)?;
            out.write_all(&sealed)?;
            break;
        }
        held = Some(block[..filled].to_vec());
        done = done.saturating_add(filled as u64);
        along.sealing(done, whole);
    }
    out.flush()?;
    Ok(())
}

fn opened(from: &Path, into: &Path, number: Option<&str>, along: &Along) -> Result<()> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use std::io::{Read, Write};

    let sealed = std::fs::metadata(from).map(|one| one.len()).unwrap_or(0);
    let mut read = 0u64;

    let Some(number) = number.filter(|one| !one.is_empty()) else {
        return Err(Error::ParcelLocked);
    };
    let mut held = std::io::BufReader::new(std::fs::File::open(from)?);
    let mut mark = [0u8; 8];
    let mut work = [0u8; 1];
    let mut salt = [0u8; 16];
    let mut head = [0u8; 19];
    let short = |_| Error::NotAParcel(from.display().to_string());
    held.read_exact(&mut mark).map_err(short)?;
    held.read_exact(&mut work).map_err(short)?;
    held.read_exact(&mut salt).map_err(short)?;
    held.read_exact(&mut head).map_err(short)?;
    if &mark != LOCKED {
        // Locked by a Tisty that seals them some other way: say so, rather than let it read as
        // a wrong number or a broken file.
        return match mark.starts_with(&LOCKED[..6]) {
            true => Err(Error::ParcelNewer(0)),
            false => Err(Error::NotAParcel(from.display().to_string())),
        };
    }

    // The file says how hard its key was to make, and a file is not to be trusted: a number a
    // stranger wrote there would have us grind a gigabyte of memory on their say-so.
    if work[0] < WORK_AT_LEAST || work[0] > WORK_AT_MOST {
        return Err(Error::NotAParcel(from.display().to_string()));
    }
    let key = keyed(number, &salt, work[0])?;
    let sealer = chacha20poly1305::XChaCha20Poly1305::new_from_slice(key.as_ref())
        .map_err(|_| Error::WrongNumber)?;

    let mut out = std::io::BufWriter::new(std::fs::File::create(into)?);
    let mut block = vec![0u8; BLOCK + TAG];
    let mut at = 0u32;
    let mut bytes = 0u64;
    loop {
        let mut filled = 0;
        while filled < block.len() {
            match held.read(&mut block[filled..])? {
                0 => break,
                got => filled += got,
            }
        }
        let last = filled < block.len();
        let plain = sealer
            .decrypt(&nonced(&head, at, last).into(), &block[..filled])
            // The first block answers for the key: forging its seal would take the key itself,
            // so anything that fails later is a parcel that came apart, not a number typed wrong.
            .map_err(|_| match at {
                0 => Error::WrongNumber,
                _ => Error::ParcelTorn,
            })?;
        bytes = bytes.saturating_add(plain.len() as u64);
        if bytes > AT_MOST {
            return Err(Error::TooBig);
        }
        out.write_all(&plain)?;
        if last {
            break;
        }
        at = at.checked_add(1).ok_or(Error::TooBig)?;
        read = read.saturating_add(filled as u64);
        along.sealing(read, sealed);
    }
    out.flush()?;
    Ok(())
}

pub fn write(
    paths: &Paths,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
    written(paths, state, which, into, along, None)
}

pub fn written(
    paths: &Paths,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
    number: Option<&str>,
) -> Result<Sent> {
    packed(paths, state, which, into, along, number, None)
}

/// `number` locks the parcel against whoever finds the file; the signer says whose it is wherever it lands.
pub fn packed(
    paths: &Paths,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
    number: Option<&str>,
    signer: Option<&Signer>,
) -> Result<Sent> {
    let data = paths.data();
    if into.starts_with(data) || data.starts_with(into) {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }
    let papers = chosen(state, which);
    if papers.is_empty() {
        return Err(Error::NothingToCarry);
    }

    if number.is_some_and(str::is_empty) {
        return Err(Error::WrongNumber);
    }

    // Moved onto the destination only at the end, so exporting over last week's parcel cannot
    // cost it. A locked one is built inside the store: the folder it lands in — a stick, a
    // shared drive — is the one place it must never sit in the clear.
    let name = into.file_name().unwrap_or_default().to_string_lossy();
    let aside = Aside(match number {
        Some(_) => data.join(format!(".packing-{}.part", std::process::id())),
        None => into.with_file_name(format!(".{name}.{}.part", std::process::id())),
    });
    if number.is_some() {
        std::fs::create_dir_all(data)?;
    }

    let signer = signer.filter(|one| one.signs());
    let made = filled(paths, state, &papers, &aside.0, along, signer);
    match (made, number) {
        (Ok(sent), None) => std::fs::rename(&aside.0, into)
            .map(|()| sent)
            .map_err(Error::Io),
        (Ok(sent), Some(number)) => {
            let sealed =
                Aside(into.with_file_name(format!(".{name}.{}.locked", std::process::id())));
            shut(&aside.0, &sealed.0, number, along)
                .and_then(|()| std::fs::rename(&sealed.0, into).map_err(Error::Io))
                .map(|()| sent)
        }
        (Err(e), _) => Err(e),
    }
}

/// A file that must not outlive the call that made it, whatever happens in between — a panic
/// while packing would otherwise leave the whole of it lying about in the clear.
struct Aside(std::path::PathBuf);

impl Drop for Aside {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn filled(
    paths: &Paths,
    state: &State,
    papers: &[&Kept],
    into: &Path,
    along: &Along,
    signer: Option<&Signer>,
) -> Result<Sent> {
    let data = paths.data();
    let root = data.join("docs");
    let mut sent = Sent::default();
    let mut bodies: Vec<(&Kept, String)> = Vec::new();
    for one in papers {
        let Ok(body) = crate::docs::read(&root, &one.file) else {
            sent.missed += 1;
            continue;
        };
        bodies.push((one, body));
    }

    let mut beside: BTreeMap<String, PathBuf> = BTreeMap::new();
    for (_, body) in &bodies {
        for one in crate::refs::extract(body).into_iter().map(|one| one.target) {
            if !crate::attach::names_an_attachment(&one) || beside.contains_key(&one) {
                continue;
            }
            match crate::attach::found(&one, data, along.also) {
                Ok(from) if from.is_file() => {
                    let held = along.also.unwrap_or(data).join("attachments");
                    let named = from
                        .strip_prefix(data.join("attachments"))
                        .or_else(|_| from.strip_prefix(&held))
                        .map(|rest| {
                            format!("attachments/{}", rest.display()).replace(char::from(92), "/")
                        })
                        .unwrap_or_else(|_| one.clone());
                    beside.insert(named, from);
                }
                _ => {
                    if !sent.left.contains(&one) {
                        sent.left.push(one);
                    }
                }
            }
        }
    }

    let held: BTreeSet<&str> = bodies.iter().map(|(one, _)| one.file.as_str()).collect();
    let mut manifest = Manifest {
        kind: KIND.into(),
        version: VERSION,
        // Named stores tie every parcel they write together, and a signature already says whose it is.
        from: match signer {
            Some(_) => String::new(),
            None => crate::store::peek_identity(data.join("store")).unwrap_or_default(),
        },
        seal: None,
        folders: shelves(state, &bodies),
        docs: bodies
            .iter()
            .map(|(one, body)| Paper {
                file: one.file.clone(),
                order: one.order.clone(),
                title: Some(
                    crate::docs::titled(body)
                        .chars()
                        .take(TITLE_AT_MOST)
                        .collect(),
                ),
                folder: one.folder.map(|at| at.to_string()),
                page_of: one
                    .page_of
                    .and_then(|up| state.docs.get(&up))
                    .map(|up| up.file.clone())
                    .filter(|up| held.contains(up.as_str())),
                translation_of: carried_anchor(state, one, &held),
                wrote: one.wrote,
                made: one.made,
                by: one.by.clone(),
                archived: state.held_away(one),
                by_folder: !one.archived && state.held_away(one),
                away_alone: one.archived.then_some(true),
                flagged: one.flagged.as_ref().map(|said| Marked {
                    at: said.at,
                    body: said.body.clone(),
                    via: said.via.clone(),
                }),
                locked: one.locked,
                guest: one.guest,
            })
            .collect(),
    };

    manifest.version = VERSION;
    if signer.is_none() {
        manifest.seal = crate::store::secret(paths).and_then(|keep| sealed(&manifest, &keep));
    }

    let weighs = serde_json::to_string(&manifest)?.len() as u64;
    if weighs > MANIFEST_AT_MOST || manifest.docs.len() > PAPERS_AT_MOST {
        return Err(Error::TooBig);
    }

    let file = std::fs::File::create(into)?;
    let _ = crate::paths::ours_alone(into);
    let mut zip = zip::ZipWriter::new(file);
    let plain = zip::write::SimpleFileOptions::default();
    let kept = plain.compression_method(zip::CompressionMethod::Stored);

    let said = serde_json::to_string_pretty(&manifest)?;
    zip.start_file(MANIFEST, plain).map_err(zipped)?;
    std::io::Write::write_all(&mut zip, said.as_bytes())?;
    let mut prints = Prints::default();

    zip.start_file(READING, plain).map_err(zipped)?;
    std::io::Write::write_all(&mut zip, READ_ME.as_bytes())?;

    let whole = bodies.len() + beside.len();
    for (one, body) in &bodies {
        let named = format!("docs/{}.md", one.file);
        zip.start_file(named.as_str(), plain).map_err(zipped)?;
        std::io::Write::write_all(&mut zip, body.as_bytes())?;
        prints.add(&named, signed_parcel::digest(body.as_bytes()));
        sent.bytes = sent.bytes.saturating_add(body.len() as u64);
        match one.page_of.is_some() {
            true => sent.pages += 1,
            false => sent.docs += 1,
        }
        along.at(sent.docs + sent.pages, whole, sent.bytes);
    }

    for (named, from) in &beside {
        let Ok(weighs) = std::fs::metadata(from).map(|one| one.len()) else {
            left_behind(&mut sent.left, named.clone());
            continue;
        };
        sent.files += 1;
        sent.bytes = sent.bytes.saturating_add(weighs);
        if sent.bytes > AT_MOST || sent.files + bodies.len() > AT_MOST_FILES {
            return Err(Error::TooBig);
        }
        let Ok(file) = std::fs::File::open(from) else {
            sent.files -= 1;
            sent.bytes = sent.bytes.saturating_sub(weighs);
            left_behind(&mut sent.left, named.clone());
            continue;
        };
        zip.start_file(named, kept).map_err(zipped)?;
        let mut hashing = Hashing::new(file);
        std::io::copy(&mut hashing, &mut zip)?;
        if safe(named).is_some() {
            prints.add(named, hashing.digest());
        }
        along.at(bodies.len() + sent.files, whole, sent.bytes);
    }

    if let Some(signer) = signer {
        let signature = signed_parcel::signed(signer, said.as_bytes(), &prints)
            .ok_or_else(|| Error::Io(std::io::Error::other("the parcel could not be signed")))?;
        zip.start_file(signed_parcel::ENTRY, plain)
            .map_err(zipped)?;
        std::io::Write::write_all(&mut zip, &signature)?;
    }

    sent.folders = manifest.folders.len();
    zip.finish().map_err(zipped)?;
    match sent.docs + sent.pages {
        0 => Err(Error::NothingToCarry),
        _ => Ok(sent),
    }
}

fn left_behind(left: &mut Vec<String>, one: String) {
    if !left.contains(&one) {
        left.push(one);
    }
}

fn chosen<'a>(state: &'a State, which: &[String]) -> Vec<&'a Kept> {
    let asked: BTreeSet<&str> = which.iter().map(String::as_str).collect();
    // A task's words travel with the task, never as a document a parcel hands to another store.
    let mut found: Vec<&Kept> = state
        .papers()
        .filter(|one| {
            asked.is_empty()
                || asked.contains(one.file.as_str())
                || one.page_of.is_some_and(|up| {
                    state
                        .docs
                        .get(&up)
                        .is_some_and(|up| asked.contains(up.file.as_str()))
                })
        })
        .collect();
    found.sort_by(|a, b| {
        a.page_of
            .is_some()
            .cmp(&b.page_of.is_some())
            .then(a.order.cmp(&b.order))
            .then(a.id.cmp(&b.id))
    });
    found
}

fn shelves(state: &State, bodies: &[(&Kept, String)]) -> Vec<Shelf> {
    let mut wanted: BTreeSet<FolderId> = BTreeSet::new();
    for (one, _) in bodies {
        let mut at = one.folder;
        while let Some(id) = at {
            if !wanted.insert(id) {
                break;
            }
            at = state.folders.get(&id).and_then(|one| one.parent);
        }
    }

    let mut found: Vec<Shelf> = wanted
        .iter()
        .filter_map(|id| state.folders.get(id))
        .map(|one| Shelf {
            id: one.id.to_string(),
            name: one.name.clone(),
            order: one.order.clone(),
            parent: one
                .parent
                .filter(|up| wanted.contains(up))
                .map(|up| up.to_string()),
            icon: one.icon.clone(),
            color: one.color.clone(),
            archived: one.archived,
        })
        .collect();
    found.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    found
}

pub fn read(
    paths: &Paths,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
) -> Result<(Landed, Vec<Op>)> {
    taken(paths, state, device, from, along, None)
}

pub fn taken(
    paths: &Paths,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
    number: Option<&str>,
) -> Result<(Landed, Vec<Op>)> {
    taken_answering(paths, state, device, from, along, number, None)
}

/// Whose a parcel is rests on its signature; `answer` is the person's word when nothing signed can say.
pub fn taken_answering(
    paths: &Paths,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
    number: Option<&str>,
    answer: Option<Answer>,
) -> Result<(Landed, Vec<Op>)> {
    let data = paths.data();
    let staged = data.join(format!(".landing-{}", std::process::id()));
    swept(data);

    // Opening one takes the parcel out whole and then unpacks it, so the store needs twice what
    // the file weighs. Better said before than as an i/o error halfway through.
    if let Ok(weighs) = std::fs::metadata(from).map(|one| one.len()) {
        let needs = weighs.saturating_mul(2);
        if let Ok(free) = fs4::available_space(data)
            && free < needs
        {
            return Err(Error::NoRoom { needs, free });
        }
    }

    let shut = locked(from);
    // Inside the landing directory, so the sweep that clears an interrupted landing carries the
    // opened copy out with it: what a locked parcel holds must not be left lying in the clear.
    let plain = staged.join("opened.tistyx");
    if shut {
        std::fs::create_dir_all(&staged)?;
        let _ = crate::paths::ours_alone(&staged);
        if let Err(e) = opened(from, &plain, number, along) {
            let _ = std::fs::remove_dir_all(&staged);
            return Err(e);
        }
    }
    let at = match shut {
        true => plain.as_path(),
        false => from,
    };

    let opening = Opening {
        staged: &staged,
        unlocked: shut,
        answer,
    };
    let done = carried(paths, state, device, at, along, &opening);
    let _ = std::fs::remove_dir_all(&staged);
    done
}

struct Opening<'a> {
    staged: &'a Path,
    unlocked: bool,
    answer: Option<Answer>,
}

fn carried(
    paths: &Paths,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
    opening: &Opening,
) -> Result<(Landed, Vec<Op>)> {
    let data = paths.data();
    let staged = opening.staged;
    let file = std::fs::File::open(from)?;
    // Something that is not an archive at all is not a parcel either, and saying so beats
    // handing back whatever the zip reader made of it.
    let mut zip =
        zip::ZipArchive::new(file).map_err(|_| Error::NotAParcel(from.display().to_string()))?;
    let (manifest, said) = manifest_in(&mut zip, from)?;
    let signature = signature_in(&mut zip)?;

    let whole = zip.len() + manifest.docs.len();
    let prints = unpack(&mut zip, staged, along, whole)?;
    let closed = signed_parcel::closed(
        signature.as_deref(),
        &said,
        prints.as_ref(),
        &state.persons.revocations,
    );
    let asked = Asked {
        unlocked: opening.unlocked,
        answer: opening.answer,
        same_store: closed == Closed::Unsigned && ours(&manifest, paths),
    };
    let mine = signed_parcel::mine(&closed, &state.persons, device, &asked)?;
    let (mut landed, ops) = taken_in(
        data,
        state,
        device,
        &Landing {
            manifest: &manifest,
            staged,
            along,
            whole,
            elsewhere: !mine,
        },
    )?;
    landed.broken = closed == Closed::Broken;
    if let (false, Closed::By { person, name }) = (mine, closed) {
        landed.signer = Some(Claimed {
            root: person.shown(),
            print: person.fingerprint(),
            name,
        });
    }
    Ok((landed, ops))
}

pub fn swept(data: &Path) {
    let Ok(entries) = std::fs::read_dir(data) else {
        return;
    };
    let mine = format!(".landing-{}", std::process::id());
    let packing = format!(".packing-{}.part", std::process::id());
    for at in entries.filter_map(|one| one.ok()).map(|one| one.path()) {
        let named = at.file_name().and_then(|one| one.to_str()).unwrap_or("");
        // A parcel half built by a process that is gone holds everything in the clear.
        if !at.is_dir() && named.starts_with(".packing-") && named != packing {
            let _ = std::fs::remove_file(&at);
            continue;
        }
        let stale = at.is_dir()
            && at
                .file_name()
                .and_then(|one| one.to_str())
                .is_some_and(|one| one.starts_with(".landing-") && one != mine);
        if stale && std::fs::remove_dir_all(&at).is_err() {
            crate::witness::warn(
                crate::witness::channel::BACKUP,
                "what an interrupted landing left behind could not be swept",
                &[("at", crate::witness::Fact::Path(at.clone()))],
            );
        }
    }
}

fn manifest_in<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    from: &Path,
) -> Result<(Manifest, Vec<u8>)> {
    let mut held = zip
        .by_name(MANIFEST)
        .map_err(|_| Error::NotAParcel(from.display().to_string()))?;
    let mut said = String::new();
    let read = held
        .by_ref()
        .take(MANIFEST_AT_MOST + 1)
        .read_to_string(&mut said)? as u64;
    if read > MANIFEST_AT_MOST {
        return Err(Error::TooBig);
    }
    let manifest: Manifest =
        serde_json::from_str(&said).map_err(|_| Error::NotAParcel(from.display().to_string()))?;
    if manifest.kind != KIND {
        return Err(Error::NotAParcel(from.display().to_string()));
    }
    if manifest.version > VERSION {
        return Err(Error::ParcelNewer(manifest.version));
    }
    if manifest.docs.len() > PAPERS_AT_MOST || manifest.folders.len() > PAPERS_AT_MOST {
        return Err(Error::TooBig);
    }
    Ok((manifest, said.into_bytes()))
}

fn signature_in<R: Read + Seek>(zip: &mut zip::ZipArchive<R>) -> Result<Option<Vec<u8>>> {
    let Ok(held) = zip.by_name(signed_parcel::ENTRY) else {
        return Ok(None);
    };
    let mut said = Vec::new();
    held.take(signed_parcel::ENTRY_AT_MOST + 1)
        .read_to_end(&mut said)?;
    if said.len() as u64 > signed_parcel::ENTRY_AT_MOST {
        said.clear();
    }
    Ok(Some(said))
}

struct Landing<'a> {
    manifest: &'a Manifest,
    staged: &'a Path,
    along: &'a Along<'a>,
    whole: usize,
    elsewhere: bool,
}

fn taken_in(
    data: &Path,
    state: &State,
    device: &DeviceId,
    landing: &Landing,
) -> Result<(Landed, Vec<Op>)> {
    let Landing {
        manifest,
        staged,
        along,
        whole,
        elsewhere,
    } = *landing;
    let mut landed = Landed::default();
    let mut ops: Vec<Op> = Vec::new();

    let mut shut: BTreeSet<FolderId> = BTreeSet::new();
    let filed = shelved(state, manifest, &mut landed, &mut ops, &mut shut);

    let root = data.join("docs");
    let mut carried: BTreeMap<String, String> = BTreeMap::new();
    let mut named: BTreeMap<String, (String, DocId)> = BTreeMap::new();
    let mut ordered: Vec<(Option<FolderId>, String)> = Vec::new();
    let mut written: Vec<(String, String, usize)> = Vec::new();
    let mut tongued: Vec<(Ulid, String)> = Vec::new();

    for paper in ordering(manifest) {
        let Ok(body) = crate::docs::read(&staged.join("docs"), &paper.file) else {
            landed.missed += 1;
            continue;
        };
        let flat = manifest.version >= SPOKEN_FROM;
        let body = brought(
            data,
            staged,
            &body,
            flat,
            &mut carried,
            &mut landed,
            &mut ops,
        );
        let body = match manifest.version < SPOKEN_FROM {
            true => tisty_markdown::to_tisty(&body),
            false => body,
        };

        let up = paper
            .page_of
            .as_ref()
            .and_then(|file| named.get(file))
            .map(|(_, id)| *id);
        if paper.page_of.is_some() && up.is_none() {
            landed.missed += 1;
            continue;
        }
        let folder = match up {
            Some(_) => None,
            None => paper.folder.as_ref().and_then(|at| filed.get(at)).copied(),
        };

        let Ok(made) = crate::docs::create(&root, device, &body) else {
            landed.missed += 1;
            continue;
        };
        let id = Ulid::generate();
        named.insert(paper.file.clone(), (made.id.clone(), id));

        let order = crate::order::last_of(
            state
                .docs
                .values()
                .filter(|one| one.folder == folder && one.page_of.is_none())
                .map(|one| one.order.as_str())
                .chain(
                    ordered
                        .iter()
                        .filter(|(at, _)| *at == folder)
                        .map(|(_, key)| key.as_str()),
                ),
        );
        ordered.push((folder, order.clone()));

        let at = ops.len();
        ops.push(Op::DocAdd {
            id,
            d: DocAdd {
                wrote: paper.wrote,
                file: made.id.clone(),
                order,
                made: paper.made,
                by: paper
                    .by
                    .as_deref()
                    .map(signed_as)
                    .filter(|one| !one.is_empty()),
                guest: elsewhere || paper.guest,
                sown: false,
                said: Some(Said {
                    title: made.title.clone(),
                    ..Said::of(&body)
                }),
                folder,
                page_of: up,
                task_of: None,
            },
        });
        // A folder that lands closed answers for what it holds; marking the document again would
        // outlive the folder and never come back with it.
        let by_folder = paper.by_folder && folder.is_some_and(|at| shut.contains(&at));
        let alone = paper
            .away_alone
            .unwrap_or(paper.archived && paper.page_of.is_none());
        if alone && !by_folder {
            ops.push(Op::DocArchive { id });
        }
        if let Some(said) = paper.flagged.clone() {
            let mut mark = crate::event::Flag::new(said.body).through(said.via);
            mark.at = Some(said.at);
            ops.push(Op::DocFlag { id, d: mark });
        }
        if paper.locked {
            ops.push(Op::DocLock { id });
        }
        match up.is_some() {
            true => landed.pages += 1,
            false => landed.docs += 1,
        }
        along.at(
            whole - manifest.docs.len() + landed.docs + landed.pages,
            whole,
            0,
        );
        written.push((made.id, body, at));
        if let Some(anchor) = paper.translation_of.clone() {
            tongued.push((id, anchor));
        }
    }
    // Every document is named by now, so a language finds its anchor wherever it sat in the parcel.
    for (id, anchor) in tongued {
        if let Some((_, of)) = named.get(&anchor) {
            ops.push(Op::DocTongue {
                id,
                d: crate::event::Tongue { of: Some(*of) },
            });
        }
    }

    for (file, body, at) in written {
        let told = pointed(&body, &named);
        if told == body {
            continue;
        }
        if crate::docs::write(&root, &file, &told).is_err() {
            landed.missed += 1;
            continue;
        }
        // The references inside it changed length, so the note taken before must say what the
        // file now holds; otherwise the first read counts as news and stamps a hand on it.
        if let Some(Op::DocAdd { d, .. }) = ops.get_mut(at)
            && let Some(said) = d.said.as_mut()
        {
            *said = crate::event::Said {
                by: said.by.clone(),
                ..crate::event::Said::of(&told)
            };
        }
    }

    Ok((landed, ops))
}

fn ordering(manifest: &Manifest) -> Vec<&Paper> {
    let mut found: Vec<&Paper> = manifest
        .docs
        .iter()
        .filter(|one| one.page_of.is_none())
        .collect();
    for one in &manifest.docs {
        if one.page_of.is_some() {
            found.push(one);
        }
    }
    found
}

/// Which folders the parcel lands in end up closed — the ones it brings closed, and the ones
/// already here that were closed before it arrived.
fn shelved(
    state: &State,
    manifest: &Manifest,
    landed: &mut Landed,
    ops: &mut Vec<Op>,
    shut: &mut BTreeSet<FolderId>,
) -> BTreeMap<String, FolderId> {
    let mut filed: BTreeMap<String, FolderId> = BTreeMap::new();
    let mut deep: BTreeMap<FolderId, usize> = BTreeMap::new();
    let mut fresh: Vec<(FolderId, Option<FolderId>, String, String)> = Vec::new();

    for shelf in downwards(manifest) {
        let parent = shelf.parent.as_ref().and_then(|up| filed.get(up)).copied();
        let under = match parent {
            Some(at) => deep
                .get(&at)
                .copied()
                .unwrap_or_else(|| state.depth(Some(at))),
            None => 0,
        };
        if under >= DEEPEST {
            if let Some(at) = parent {
                filed.insert(shelf.id.clone(), at);
            }
            continue;
        }

        let standing = state
            .folders
            .values()
            .find(|one| one.parent == parent && alike(&one.name, &shelf.name))
            .map(|one| one.id)
            .or_else(|| {
                fresh
                    .iter()
                    .find(|(_, up, name, _)| *up == parent && alike(name, &shelf.name))
                    .map(|(id, ..)| *id)
            });
        if let Some(id) = standing {
            landed.joined += 1;
            if state.folder_away(id) || parent.is_some_and(|up| shut.contains(&up)) {
                shut.insert(id);
            }
            deep.insert(id, under + 1);
            filed.insert(shelf.id.clone(), id);
            continue;
        }

        let mut keys: Vec<String> = state
            .under(parent)
            .iter()
            .map(|one| one.order.clone())
            .collect();
        keys.extend(
            fresh
                .iter()
                .filter(|(_, up, _, _)| *up == parent)
                .map(|(_, _, _, key)| key.clone()),
        );
        let order = crate::order::last_of(keys.iter().map(String::as_str));

        let id = Ulid::generate();
        ops.push(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: named(&shelf.name),
                order: order.clone(),
                parent,
                icon: shelf
                    .icon
                    .clone()
                    .filter(|one| crate::model::icon::known(one)),
                color: shelf
                    .color
                    .as_ref()
                    .and_then(|one| crate::model::hue::kept(one))
                    .map(str::to_string),
            },
        });
        // A folder a parcel brings closed comes in closed. One that was already here decides for
        // itself: joining by name must not shelve what somebody is still using.
        if shelf.archived {
            ops.push(Op::FolderArchive { id });
        }
        if shelf.archived || parent.is_some_and(|up| shut.contains(&up)) {
            shut.insert(id);
        }
        fresh.push((id, parent, shelf.name.clone(), order));
        deep.insert(id, under + 1);
        filed.insert(shelf.id.clone(), id);
        landed.folders += 1;
    }
    filed
}

fn named(said: &str) -> String {
    let plain = crate::text::plainly(said);
    let mut cut: String = plain
        .chars()
        .take(crate::model::FOLDER_NAME_AT_MOST)
        .collect();
    if cut.trim().is_empty() {
        cut = "?".into();
    }
    cut.trim().to_string()
}

fn signed_as(said: &str) -> String {
    crate::text::plainly(said)
        .chars()
        .take(crate::event::ALIAS_AT_MOST)
        .collect::<String>()
        .trim()
        .to_string()
}

fn downwards(manifest: &Manifest) -> Vec<&Shelf> {
    let mut found: Vec<&Shelf> = Vec::new();
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut left: Vec<&Shelf> = manifest.folders.iter().collect();
    while !left.is_empty() {
        let (ready, waiting): (Vec<&Shelf>, Vec<&Shelf>) = left.into_iter().partition(|one| {
            one.parent
                .as_ref()
                .is_none_or(|up| done.contains(up.as_str()))
        });
        if ready.is_empty() {
            found.extend(waiting);
            break;
        }
        for one in &ready {
            done.insert(one.id.as_str());
        }
        found.extend(ready);
        left = waiting;
    }
    found
}

fn alike(one: &str, other: &str) -> bool {
    crate::text::composed(one.trim()).to_lowercase()
        == crate::text::composed(other.trim()).to_lowercase()
}

fn brought(
    data: &Path,
    staged: &Path,
    body: &str,
    flat: bool,
    carried: &mut BTreeMap<String, String>,
    landed: &mut Landed,
    ops: &mut Vec<Op>,
) -> String {
    let mut told = body.to_string();
    // The link as the parcel wrote it, which is also where its file sits inside the parcel.
    for one in tisty_markdown::refs::extract(body)
        .into_iter()
        .map(|one| one.target)
    {
        // A parcel from Tisty's own words keeps every file flat; an older one kept them where the link said.
        let Some(packed) =
            crate::attach::leaf_of(&one).map(|leaf| match flat || !one.contains('/') {
                true => format!("{}/{leaf}", crate::attach::SHED),
                false => one
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .split(['?', '#'])
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            })
        else {
            continue;
        };
        if !carried.contains_key(&one) {
            // Not `resolve`, which reads any attachment as its flat name: an older parcel kept its shelves.
            let Some(at) = crate::attach::decoded(&packed).and_then(|one| safe(&one)) else {
                continue;
            };
            let from = staged.join(at);
            if !from.is_file() {
                landed.missed += 1;
                continue;
            }
            let Ok(kept) = crate::attach::keep(&from, data, crate::attach::HELD_UP_TO) else {
                landed.missed += 1;
                continue;
            };
            landed.files += 1;
            ops.push(kept.told(crate::attach::Via::Parcel));
            let leaf = crate::attach::leaf_of(&kept.at).unwrap_or(&kept.at);
            let anchor = one.find(['?', '#']).map_or("", |at| &one[at..]);
            carried.insert(
                one.clone(),
                format!("{leaf}{}", anchor.trim_end_matches('>')),
            );
        }
        if let Some(now) = carried.get(&one)
            && *now != one
        {
            told = told.replace(&one, now);
        }
    }
    told
}

const MARK: char = '\u{0}';

fn pointed(body: &str, named: &BTreeMap<String, (String, DocId)>) -> String {
    // Every reference, card or mention alike: a name that moved has to move everywhere it is
    // written, or what was a link becomes a dead one.
    let mut found: Vec<String> = crate::refs::extract(body)
        .into_iter()
        .filter_map(|one| {
            one.target
                .strip_prefix(crate::refs::DOC)
                .map(str::to_string)
        })
        .collect();
    // The long name first: one id can be a prefix of another, and a plain replace would rewrite
    // the middle of the longer one.
    found.sort_by_key(|one| std::cmp::Reverse(one.len()));
    let mut told = body.replace(MARK, "");
    for file in found {
        if let Some((now, _)) = named.get(&file) {
            told = told.replace(
                &format!("{}{file}", crate::refs::DOC),
                &format!("{MARK}{now}"),
            );
        }
    }
    told.replace(MARK, crate::refs::DOC)
}

// None in place of the prints when a name came twice, since only one of them would land.
fn unpack<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    into: &Path,
    along: &Along,
    whole: usize,
) -> Result<Option<Prints>> {
    if zip.len() > AT_MOST_FILES {
        return Err(Error::TooBig);
    }
    let mut bytes = 0u64;
    let mut prints = Some(Prints::default());

    for i in 0..zip.len() {
        let mut held = zip.by_index(i).map_err(zipped)?;
        if held.is_dir() {
            continue;
        }
        let named = held.name().map(|one| one.into_owned()).ok();
        let Some(rest) = named.as_deref().and_then(safe) else {
            continue;
        };
        if held.size() > AT_MOST.saturating_sub(bytes) {
            return Err(Error::TooBig);
        }

        let at = into.join(&rest);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // A name this system will not take is one entry lost, not the whole parcel: what needed
        // it is counted as missing when nothing turns up under that name.
        let file = std::fs::File::create(&at).ok();
        if file.is_some() {
            let _ = crate::paths::ours_alone(&at);
        }
        let room = AT_MOST.saturating_sub(bytes).saturating_add(1);
        let mut hashing = Hashing::new(held.by_ref().take(room));
        let written = match file {
            Some(mut file) => std::io::copy(&mut hashing, &mut file)?,
            None => std::io::copy(&mut hashing, &mut std::io::sink())?,
        };
        if written >= room {
            return Err(Error::TooBig);
        }
        let digest = hashing.digest();
        let entry = named.as_deref().unwrap_or_default();
        if prints.as_mut().is_some_and(|all| !all.add(entry, digest)) {
            prints = None;
        }
        bytes = bytes.saturating_add(written);
        along.at(i + 1, whole, bytes);
    }
    Ok(prints)
}

fn safe(named: &str) -> Option<PathBuf> {
    let at = Path::new(named);
    if !at
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let head = at.components().next()?.as_os_str().to_str()?;
    CARRIED.contains(&head).then(|| at.to_path_buf())
}

fn zipped(e: zip::result::ZipError) -> Error {
    Error::Io(std::io::Error::other(e.to_string()))
}
// The anchor of its languages that travels with it: the anchor itself, or the first of them carried.
fn carried_anchor(
    state: &State,
    one: &crate::model::Kept,
    held: &BTreeSet<&str>,
) -> Option<String> {
    let tongues = state.tongues_of(one.id);
    let anchor = state.anchor_of(one);
    let first = tongues
        .iter()
        .find(|kin| kin.id == anchor && held.contains(kin.file.as_str()))
        .or_else(|| tongues.iter().find(|kin| held.contains(kin.file.as_str())))?;
    (first.id != one.id).then(|| first.file.clone())
}
