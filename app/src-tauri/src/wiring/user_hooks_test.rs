use std::path::Path;

use super::*;
use crate::wiring::places::Os;

fn place(home: &Path) -> Place {
    Place {
        distro: None,
        os: Os::Linux,
        home: home.to_path_buf(),
        config: home.join(".config"),
        dsh: home.join(".dsh"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: "/opt/tisty".to_string(),
        open: true,
        packaged: false,
        mount: String::new(),
    }
}

const GUARD: Hook = Hook {
    name: "guard.sh",
    event: Event::BeforeTool,
    tools: Tools::Shell,
    timeout: 5,
};

#[test]
fn a_users_hook_never_reads_as_tistys_own_and_the_other_way_round() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in [
        Kind::Claude,
        Kind::Codex,
        Kind::Cursor,
        Kind::Gemini,
        Kind::Copilot,
        Kind::Antigravity,
    ] {
        let own = hooks::with(kind, "", &here).unwrap();
        let both = with(kind, &own, &here, &GUARD).unwrap();
        assert!(
            hooks::hooked(kind, &both) && !hooks::apart(kind, &both),
            "{kind:?}"
        );
        let own_gone = hooks::without(kind, &both).unwrap();
        assert!(
            standing_in(kind, &own_gone, &here, GUARD.name).present,
            "{kind:?}"
        );
        let user_gone = without(kind, &both, GUARD.name).unwrap();
        assert!(
            !standing_in(kind, &user_gone, &here, GUARD.name).present,
            "{kind:?}"
        );
        assert!(hooks::hooked(kind, &user_gone), "{kind:?}");
    }
}
