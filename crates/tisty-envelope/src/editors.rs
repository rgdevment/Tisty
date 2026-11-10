use tisty_contracts::Editor;
use tisty_share_proto::editing::{self, Entry};
use tisty_share_proto::identity::{Certificate, ShareKey};

pub(crate) const AT_MOST: usize = 64;

/// It answers only for this root having signed this name for this document, never for who the person is.
pub fn holds(editor: &Editor) -> bool {
    editing::holds(&Entry {
        id: &editor.r#in,
        owned: editor.owned == Some(true),
        name: &editor.name,
        root: &editor.root,
        machine: &editor.machine,
        since: &editor.since,
        certificate: &editor.certificate,
        signed: &editor.signed,
    })
}

pub fn signed(
    key: &ShareKey,
    certificate: &Certificate,
    id: &str,
    owned: bool,
    name: &str,
) -> Option<Editor> {
    let signed = editing::signed(key, certificate, id, owned, name)?;
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
