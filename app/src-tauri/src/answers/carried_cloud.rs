use std::sync::{Arc, Mutex};

use tisty_core::config::{CloudAt, Sync};

use crate::reaching::{KEPT_AS, WEBDAV, secrets_for};
use crate::{OneAtATime, Session, held};

pub(crate) type Asked = std::sync::mpsc::Receiver<(bool, bool)>;

pub(crate) fn a_session(home: &std::path::Path) -> Arc<Mutex<Session>> {
    let paths = tisty_core::Paths::new(home.join("data"), home.join("config"));
    Arc::new(Mutex::new(Session::at(paths).unwrap()))
}

pub(crate) fn restored(session: &Mutex<Session>) {
    held(session)
        .keep(|config| config.restored_at = Some(jiff::Timestamp::now()))
        .unwrap();
}

pub(crate) fn holding_our_store(session: &Mutex<Session>, folder: &std::path::Path) {
    let store = folder.join(tisty_carrier::STORE);
    std::fs::create_dir_all(&store).unwrap();
    let ours = tisty_core::store::identity(held(session).paths.store()).unwrap();
    std::fs::write(store.join(tisty_core::store::MARKER), ours).unwrap();
}

// The server of a cloud already carried here; each request says whether the session and the round's turn are free.
pub(crate) fn carried_cloud(
    session: &Arc<Mutex<Session>>,
    alone: &Arc<OneAtATime>,
) -> (CloudAt, Asked) {
    let (heard, asked) = std::sync::mpsc::channel();
    let (theirs, alone) = (session.clone(), alone.clone());
    let at = cloud_that(session, move || {
        let free = theirs.try_lock().is_ok();
        let turn = alone.claim().is_some();
        let _ = heard.send((free, turn));
    });
    (at, asked)
}

pub(crate) fn cloud_that(
    session: &Arc<Mutex<Session>>,
    before_answering: impl Fn() + Send + 'static,
) -> CloudAt {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!(
        "http://127.0.0.1:{}/dav/mario",
        listener.local_addr().unwrap().port()
    );
    let dav = tisty_webdav::DavAt::under(&base, "mario").unwrap();
    let at = CloudAt {
        provider: WEBDAV.into(),
        account: dav.account(),
        email: dav.shown(),
    };
    secrets_for(&at)
        .unwrap()
        .write(KEPT_AS, "r0ck4me!")
        .unwrap();
    {
        let mut session = held(session);
        session
            .keep(|config| config.sync = Some(Sync::cloud(at.clone())))
            .unwrap();
        let home = session.reach.home_of(&at);
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("carried-here"), "1").unwrap();
    }
    std::thread::spawn(move || {
        use std::io::Write;
        for stream in listener.incoming() {
            before_answering();
            let _ = stream.unwrap().write_all(
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        }
    });
    at
}

pub(crate) fn forgotten(at: &CloudAt) {
    secrets_for(at).unwrap().forget(KEPT_AS).unwrap();
}
