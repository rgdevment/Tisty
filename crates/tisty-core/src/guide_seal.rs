use std::collections::BTreeMap;
use std::path::Path;

use crate::attach::printed;
use crate::state::same_name;
use crate::untouched::SOWN_BY;
use serde::{Deserialize, Serialize};
use tisty_share_proto::identity::{Certificate, Person, ShareKey, Signed, Verdict, check};
use tisty_share_proto::name::named_well;

const ABOUT: &str = "tisty.guide.v1";
pub const MAKER_ROOT: &str = "AfnGoRnNecrr5RMnprcTbdZdk7_LABXtpqchY-L0b0s";
pub const SEAL: &str = "seal.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuideSeal {
    pub author: String,
    pub files: BTreeMap<String, String>,
    pub signed: Signed,
}

fn said(author: &str, files: &BTreeMap<String, String>) -> Vec<u8> {
    serde_json::to_vec(&(author, files)).unwrap_or_default()
}

pub fn guide_files(guide: &Path) -> std::io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    for tongue in std::fs::read_dir(guide)? {
        let tongue = tongue?;
        if !tongue.file_type()?.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(tongue.path())? {
            let file = file?;
            if file.file_type()?.is_file() {
                let named = format!(
                    "{}/{}",
                    tongue.file_name().to_string_lossy(),
                    file.file_name().to_string_lossy()
                );
                files.insert(named, printed(&std::fs::read(file.path())?));
            }
        }
    }
    Ok(files)
}

pub fn sealed(
    key: &ShareKey,
    certificate: &Certificate,
    author: &str,
    files: BTreeMap<String, String>,
) -> Option<GuideSeal> {
    let signed = key.sign(certificate, ABOUT, &said(author, &files))?;
    Some(GuideSeal {
        author: author.to_owned(),
        files,
        signed,
    })
}

// The name holds only over these very files, signed by a machine that root certified.
pub fn author_of<'a>(
    seal: &'a GuideSeal,
    files: &BTreeMap<String, String>,
    root: &str,
) -> Option<&'a str> {
    let root = Person::read(root)?;
    let named = named_well(&seal.author) && !same_name(&seal.author, SOWN_BY);
    if !named || seal.files != *files {
        return None;
    }
    match check(&seal.signed, ABOUT, &said(&seal.author, &seal.files), &[]) {
        Verdict::Holds { person, .. } if person == root => Some(&seal.author),
        _ => None,
    }
}

pub fn author_of_guide(guide: &Path, root: &str) -> Option<String> {
    let seal: GuideSeal = serde_json::from_slice(&std::fs::read(guide.join(SEAL)).ok()?).ok()?;
    let files = guide_files(guide).ok()?;
    author_of(&seal, &files, root).map(str::to_owned)
}
