use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    Error, Result,
    event::{DeviceId, DocAdd, FolderAdd, Op, Said},
    model::{DEEPEST, DocId, FolderId, Kept},
    state::State,
};

pub const EXTENSION: &str = "tistyx";
const KIND: &str = "tisty-docs";
const VERSION: u32 = 1;
const MANIFEST: &str = "tisty-docs.json";
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
    use hmac::Mac;
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
fn ours(manifest: &Manifest, data: &Path) -> bool {
    let mine = crate::store::peek_identity(data.join("store"));
    let said = manifest.from.trim();
    if said.is_empty() || mine.as_deref() != Some(said) {
        return false;
    }
    let Some(keep) = crate::store::secret(data.join("store")) else {
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
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub locked: bool,
    /// Somebody else's writing that this store is only holding: it stays theirs wherever it
    /// goes next, rather than turning native by passing through here.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub guest: bool,
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
}

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
    let how = scrypt::Params::new(work, 8, 1, 32).map_err(|_| Error::WrongNumber)?;
    // Composed first, so a number typed on a Mac and the same one typed on Windows are the same
    // number rather than two spellings that grind out different keys.
    let said = crate::text::composed(number);
    scrypt::scrypt(said.as_bytes(), salt, &how, key.as_mut()).map_err(|_| Error::WrongNumber)?;
    Ok(key)
}

/// Each block is sealed under its own nonce: bytes of its own, then the block's number, then a
/// byte that is only set on the last one. That byte lives in the nonce and not in the file, so a
/// parcel cut short has a surviving tail that no longer authenticates as an ending.
fn nonced(head: &[u8; 19], at: u32, last: bool) -> [u8; 24] {
    let mut nonce = [0u8; 24];
    nonce[..19].copy_from_slice(head);
    nonce[19..23].copy_from_slice(&at.to_be_bytes());
    nonce[23] = u8::from(last);
    nonce
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
    let sealer = chacha20poly1305::XChaCha20Poly1305::new(key.as_ref().into());

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
    let sealer = chacha20poly1305::XChaCha20Poly1305::new(key.as_ref().into());

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
    data: &Path,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
    written(data, state, which, into, along, None)
}

/// `number` locks the parcel: only a store whose person knows it takes what is inside as their
/// own writing rather than as a guest's.
pub fn written(
    data: &Path,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
    number: Option<&str>,
) -> Result<Sent> {
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

    let made = filled(data, state, &papers, &aside.0, along);
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
    data: &Path,
    state: &State,
    papers: &[&Kept],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
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
        from: crate::store::peek_identity(data.join("store")).unwrap_or_default(),
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
                wrote: one.wrote,
                made: one.made,
                by: one.by.clone(),
                archived: state.held_away(one),
                by_folder: !one.archived && state.held_away(one),
                locked: one.locked,
                guest: one.guest,
            })
            .collect(),
    };

    manifest.seal =
        crate::store::secret(data.join("store")).and_then(|keep| sealed(&manifest, &keep));

    let weighs = serde_json::to_string(&manifest)?.len() as u64;
    if weighs > MANIFEST_AT_MOST || manifest.docs.len() > PAPERS_AT_MOST {
        return Err(Error::TooBig);
    }

    let file = std::fs::File::create(into)?;
    let _ = crate::paths::ours_alone(into);
    let mut zip = zip::ZipWriter::new(file);
    let plain = zip::write::SimpleFileOptions::default();
    let kept = plain.compression_method(zip::CompressionMethod::Stored);

    zip.start_file(MANIFEST, plain).map_err(zipped)?;
    std::io::Write::write_all(
        &mut zip,
        serde_json::to_string_pretty(&manifest)?.as_bytes(),
    )?;

    let whole = bodies.len() + beside.len();
    for (one, body) in &bodies {
        zip.start_file(format!("docs/{}.md", one.file), plain)
            .map_err(zipped)?;
        std::io::Write::write_all(&mut zip, body.as_bytes())?;
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
        let Ok(mut file) = std::fs::File::open(from) else {
            sent.files -= 1;
            sent.bytes = sent.bytes.saturating_sub(weighs);
            left_behind(&mut sent.left, named.clone());
            continue;
        };
        zip.start_file(named, kept).map_err(zipped)?;
        std::io::copy(&mut file, &mut zip)?;
        along.at(bodies.len() + sent.files, whole, sent.bytes);
    }

    sent.folders = manifest.folders.len();
    zip.finish().map_err(zipped)?;
    match sent.docs + sent.pages {
        0 => Err(Error::NothingToCarry),
        _ => Ok(sent),
    }
}

pub fn plainly(
    data: &Path,
    state: &State,
    which: &[String],
    into: &Path,
    along: &Along,
) -> Result<Sent> {
    if into.starts_with(data) || data.starts_with(into) {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }
    let papers = chosen(state, which);
    if papers.is_empty() {
        return Err(Error::NothingToCarry);
    }

    let mut sent = Sent::default();
    let mut shelves: BTreeSet<PathBuf> = BTreeSet::new();
    let fresh = !into.exists();
    let shelved = trails(state, into);
    let whole = papers.iter().filter(|one| one.page_of.is_none()).count();
    for one in papers.iter().filter(|one| one.page_of.is_none()) {
        let under = one
            .folder
            .and_then(|at| shelved.get(&at).cloned())
            .unwrap_or_else(|| into.to_path_buf());
        if let Err(e) = std::fs::create_dir_all(&under) {
            if fresh {
                let _ = std::fs::remove_dir_all(into);
            }
            return Err(Error::Io(e));
        }
        for at in under.ancestors().take_while(|at| *at != into) {
            shelves.insert(at.to_path_buf());
        }

        let pages: Vec<String> = state
            .pages_of(one.id)
            .iter()
            .map(|page| page.file.clone())
            .collect();
        let named = free(
            &under,
            &crate::docs::spelled(match one.title.as_deref() {
                Some(said) if !said.is_empty() => said,
                _ => one.file.as_str(),
            }),
        );
        let Ok(took) =
            crate::docs::laid_out_as(data, &one.file, &pages, &under, Some(&named), along.also)
        else {
            sent.missed += 1;
            continue;
        };
        sent.docs += 1;
        sent.missed += took.missed;
        sent.pages += pages.len() - took.missed;
        sent.files += took.files;
        along.at(sent.docs, whole, sent.bytes);
        for gone in took.left {
            left_behind(&mut sent.left, gone);
        }
    }
    sent.folders = shelves.len();
    Ok(sent)
}

fn trails(state: &State, into: &Path) -> BTreeMap<FolderId, PathBuf> {
    let mut found: BTreeMap<FolderId, PathBuf> = BTreeMap::new();
    let mut left: Vec<(Option<FolderId>, PathBuf)> = vec![(None, into.to_path_buf())];
    while let Some((parent, at)) = left.pop() {
        let mut taken: BTreeSet<String> = BTreeSet::new();
        for one in state.under(parent) {
            if found.contains_key(&one.id) {
                continue;
            }
            let mut named = crate::docs::spelled(&one.name);
            // Windows and macOS hand back one directory for «Casa» and «CASA», so telling them
            // apart by their exact spelling would pour two folders into the same one.
            if !taken.insert(crate::text::composed(&named).to_lowercase()) {
                for n in 2..100 {
                    let tried = format!("{named} {n}");
                    if taken.insert(crate::text::composed(&tried).to_lowercase()) {
                        named = tried;
                        break;
                    }
                }
            }
            let mine = at.join(&named);
            found.insert(one.id, mine.clone());
            left.push((Some(one.id), mine));
        }
    }
    found
}

fn free(under: &Path, named: &str) -> String {
    if !under.join(named).exists() {
        return named.to_string();
    }
    for n in 2..100 {
        let tried = format!("{named} {n}");
        if !under.join(&tried).exists() {
            return tried;
        }
    }
    named.to_string()
}

fn left_behind(left: &mut Vec<String>, one: String) {
    if !left.contains(&one) {
        left.push(one);
    }
}

fn chosen<'a>(state: &'a State, which: &[String]) -> Vec<&'a Kept> {
    let asked: BTreeSet<&str> = which.iter().map(String::as_str).collect();
    let mut found: Vec<&Kept> = state
        .docs
        .values()
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
    data: &Path,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
) -> Result<(Landed, Vec<Op>)> {
    taken(data, state, device, from, along, None)
}

/// A parcel that opens with the number was locked by whoever holds it, and what is inside is
/// theirs: only writing that was already a guest where it came from stays one.
pub fn taken(
    data: &Path,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
    number: Option<&str>,
) -> Result<(Landed, Vec<Op>)> {
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

    let done = carried(data, state, device, at, along, &staged, shut);
    let _ = std::fs::remove_dir_all(&staged);
    done
}

fn carried(
    data: &Path,
    state: &State,
    device: &DeviceId,
    from: &Path,
    along: &Along,
    staged: &Path,
    unlocked: bool,
) -> Result<(Landed, Vec<Op>)> {
    let file = std::fs::File::open(from)?;
    // Something that is not an archive at all is not a parcel either, and saying so beats
    // handing back whatever the zip reader made of it.
    let mut zip =
        zip::ZipArchive::new(file).map_err(|_| Error::NotAParcel(from.display().to_string()))?;
    let manifest = manifest_in(&mut zip, from)?;

    let elsewhere = !unlocked && !ours(&manifest, data);
    let whole = zip.len() + manifest.docs.len();
    unpack(&mut zip, staged, along, whole).and_then(|_| {
        taken_in(
            data,
            state,
            device,
            &Landing {
                manifest: &manifest,
                staged,
                along,
                whole,
                elsewhere,
            },
        )
    })
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

fn manifest_in<R: Read + Seek>(zip: &mut zip::ZipArchive<R>, from: &Path) -> Result<Manifest> {
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
    Ok(manifest)
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

    for paper in ordering(manifest) {
        let Ok(body) = crate::docs::read(&staged.join("docs"), &paper.file) else {
            landed.missed += 1;
            continue;
        };
        let body = brought(data, staged, &body, &mut carried, &mut landed);

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
                said: Some(Said {
                    title: made.title.clone(),
                    bytes: Some(crate::docs::settled(&body).len() as u64),
                    tags: Some(crate::tagging::tags_in(&body)),
                    by: None,
                }),
                folder,
                page_of: up,
            },
        });
        // A folder that lands closed answers for what it holds; marking the document again would
        // outlive the folder and never come back with it.
        let by_folder = paper.by_folder && folder.is_some_and(|at| shut.contains(&at));
        if paper.archived && !by_folder {
            ops.push(Op::DocArchive { id });
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
            said.bytes = Some(crate::docs::settled(&told).len() as u64);
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
    carried: &mut BTreeMap<String, String>,
    landed: &mut Landed,
) -> String {
    let mut told = body.to_string();
    for one in crate::refs::extract(body).into_iter().map(|one| one.target) {
        if !crate::attach::names_an_attachment(&one) {
            continue;
        }
        if !carried.contains_key(&one) {
            let Ok(said) = crate::attach::resolve(&one, Path::new("")) else {
                continue;
            };
            let Some(at) = safe(&said.to_string_lossy().replace(char::from(92), "/")) else {
                continue;
            };
            let from = staged.join(at);
            if !from.is_file() {
                landed.missed += 1;
                continue;
            }
            let Ok(kept) = crate::attach::keep(&from, data, crate::attach::COPIED_IN_DOC) else {
                landed.missed += 1;
                continue;
            };
            landed.files += 1;
            carried.insert(one.clone(), kept.at);
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
    let mut found: Vec<String> = crate::refs::papers(body);
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

fn unpack<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    into: &Path,
    along: &Along,
    whole: usize,
) -> Result<usize> {
    if zip.len() > AT_MOST_FILES {
        return Err(Error::TooBig);
    }
    let mut files = 0;
    let mut bytes = 0u64;

    for i in 0..zip.len() {
        let mut held = zip.by_index(i).map_err(zipped)?;
        if held.is_dir() {
            continue;
        }
        let Some(rest) = safe(held.name()) else {
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
        let Ok(mut file) = std::fs::File::create(&at) else {
            continue;
        };
        let _ = crate::paths::ours_alone(&at);
        let room = AT_MOST.saturating_sub(bytes).saturating_add(1);
        let written = std::io::copy(&mut held.by_ref().take(room), &mut file)?;
        if written >= room {
            return Err(Error::TooBig);
        }
        bytes = bytes.saturating_add(written);
        files += 1;
        along.at(i + 1, whole, bytes);
    }
    Ok(files)
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
