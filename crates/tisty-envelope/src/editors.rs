use tisty_contracts::Editor;
use tisty_share_proto::identity::{self, Certificate, ShareKey, Signed};

const ABOUT: &str = "editor";
pub(crate) const AT_MOST: usize = 64;

fn said(id: &str, owned: bool, name: &str) -> Vec<u8> {
    let mut said = (id.len() as u64).to_be_bytes().to_vec();
    said.extend_from_slice(id.as_bytes());
    said.push(u8::from(owned));
    said.extend_from_slice(&(name.len() as u64).to_be_bytes());
    said.extend_from_slice(name.as_bytes());
    said
}

/// It answers only for this root having signed this name for this document, never for who the person is.
pub fn holds(editor: &Editor) -> bool {
    let Ok(since) = editor.since.parse() else {
        return false;
    };
    let signed = Signed {
        certificate: Certificate {
            root: editor.root.clone(),
            share: editor.machine.clone(),
            since,
            sig: editor.certificate.clone(),
        },
        sig: editor.signed.clone(),
    };
    let owned = editor.owned == Some(true);
    matches!(
        identity::check(
            &signed,
            ABOUT,
            &said(&editor.r#in, owned, &editor.name),
            &[]
        ),
        identity::Verdict::Holds { .. }
    )
}

pub fn signed(
    key: &ShareKey,
    certificate: &Certificate,
    id: &str,
    owned: bool,
    name: &str,
) -> Option<Editor> {
    let signed = key.sign(certificate, ABOUT, &said(id, owned, name))?;
    Some(Editor {
        r#in: id.to_owned(),
        root: signed.certificate.root,
        machine: signed.certificate.share,
        since: signed.certificate.since.to_string(),
        certificate: signed.certificate.sig,
        name: name.to_owned(),
        owned: owned.then_some(true),
        signed: signed.sig,
    })
}

pub(crate) fn with_mine(arrived: Vec<Editor>, mine: Option<Editor>) -> Vec<Editor> {
    let mut out: Vec<Editor> = arrived.into_iter().filter(holds).collect();
    if let Some(mine) = mine {
        match out
            .iter_mut()
            .find(|one| one.root == mine.root && one.r#in == mine.r#in)
        {
            Some(place) => *place = mine,
            None => out.push(mine),
        }
    }
    while out.len() > AT_MOST {
        let at = out
            .iter()
            .position(|one| one.owned != Some(true))
            .unwrap_or(0);
        out.remove(at);
    }
    out
}

#[cfg(test)]
#[path = "editors_test.rs"]
mod tests;
