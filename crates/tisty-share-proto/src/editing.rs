use crate::identity::{self, Certificate, ShareKey, Signed, Verdict};

const ABOUT: &str = "editor";

fn said(id: &str, owned: bool, name: &str) -> Vec<u8> {
    let mut said = (id.len() as u64).to_be_bytes().to_vec();
    said.extend_from_slice(id.as_bytes());
    said.push(u8::from(owned));
    said.extend_from_slice(&(name.len() as u64).to_be_bytes());
    said.extend_from_slice(name.as_bytes());
    said
}

pub struct Entry<'a> {
    pub id: &'a str,
    pub owned: bool,
    pub name: &'a str,
    pub root: &'a str,
    pub machine: &'a str,
    pub since: &'a str,
    pub certificate: &'a str,
    pub signed: &'a str,
}

/// It answers only for this root having signed this name for this document, never for who the person is.
pub fn holds(entry: &Entry) -> bool {
    let Ok(since) = entry.since.parse() else {
        return false;
    };
    let signed = Signed {
        certificate: Certificate {
            root: entry.root.to_owned(),
            share: entry.machine.to_owned(),
            since,
            sig: entry.certificate.to_owned(),
        },
        sig: entry.signed.to_owned(),
    };
    matches!(
        identity::check(
            &signed,
            ABOUT,
            &said(entry.id, entry.owned, entry.name),
            &[]
        ),
        Verdict::Holds { .. }
    )
}

pub fn signed(
    key: &ShareKey,
    certificate: &Certificate,
    id: &str,
    owned: bool,
    name: &str,
) -> Option<Signed> {
    key.sign(certificate, ABOUT, &said(id, owned, name))
}
