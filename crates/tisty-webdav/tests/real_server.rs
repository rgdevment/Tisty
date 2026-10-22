//! Asked of a real server only when someone names one:
//! `TISTY_WEBDAV_URL=… TISTY_WEBDAV_USER=… TISTY_WEBDAV_PASSWORD=… cargo test -p tisty-webdav --test real_server -- --ignored`

use std::sync::Arc;

use tisty_carrier::testing::Kept;
use tisty_carrier::{Expect, Hitch, Remote, Tokens};
use tisty_webdav::{DavAt, Password, WebDav};

fn named() -> Option<(String, String, String)> {
    let one = |key: &str| std::env::var(key).ok().filter(|said| !said.is_empty());
    Some((
        one("TISTY_WEBDAV_URL")?,
        one("TISTY_WEBDAV_USER")?,
        one("TISTY_WEBDAV_PASSWORD")?,
    ))
}

fn body_of(dav: &WebDav, name: &str, from: u64) -> Vec<u8> {
    let mut got = Vec::new();
    dav.fetch(name, from, &mut got).unwrap();
    got
}

#[test]
#[ignore = "needs a WebDAV server named in the environment"]
fn a_real_server_keeps_the_contract_every_provider_keeps() {
    let Some((url, user, password)) = named() else {
        panic!("name the server in TISTY_WEBDAV_URL, TISTY_WEBDAV_USER and TISTY_WEBDAV_PASSWORD");
    };
    let room = tempfile::tempdir().unwrap();
    let tokens = Tokens::new(
        Arc::new(Kept::holding("refresh", &password)),
        Arc::new(Password),
        "refresh",
        room.path(),
    );
    let dav = WebDav::new(Arc::new(tokens), DavAt::under(&url, &user).unwrap()).unwrap();
    let folder = format!("contract-{}", std::process::id());
    let name = format!("{folder}/one note.md");
    let local = |body: &[u8]| {
        let at = room.path().join(format!("local-{}", body.len()));
        std::fs::write(&at, body).unwrap();
        at
    };

    let first = dav.put(&name, &local(b"first"), Expect::Absent).unwrap();
    assert!(matches!(
        dav.put(&name, &local(b"again"), Expect::Absent),
        Err(Hitch::Changed(_))
    ));
    let listed = dav.list(&folder).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, name);
    assert_eq!(listed[0].revision, first.revision);
    assert_eq!(body_of(&dav, &name, 2), b"rst");

    let second = dav
        .put(
            &name,
            &local(b"second!"),
            Expect::Revision(first.revision.clone()),
        )
        .unwrap();
    assert_ne!(second.revision, first.revision);
    assert!(matches!(
        dav.put(
            &name,
            &local(b"stale"),
            Expect::Revision(first.revision.clone())
        ),
        Err(Hitch::Changed(_))
    ));

    let moved = format!("{folder}/kept/moved.md");
    dav.renamed(&name, &moved).unwrap();
    assert!(dav.about(&name).unwrap().is_none());
    let there = dav.about(&moved).unwrap().unwrap();
    assert_eq!(body_of(&dav, &moved, 0), b"second!");

    assert!(matches!(
        dav.delete(&moved, Some(&first.revision)),
        Err(Hitch::Changed(_))
    ));
    dav.delete(&moved, Some(&there.revision)).unwrap();
    assert!(matches!(dav.delete(&moved, None), Err(Hitch::Missing(_))));
    assert!(dav.list(&folder).unwrap().is_empty());
    assert!(dav.guards_writes().unwrap());
}
