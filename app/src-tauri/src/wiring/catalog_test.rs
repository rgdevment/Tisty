use super::*;

#[test]
fn the_catalog_reads_and_names_each_assistant_once() {
    let all = all();
    assert!(all.len() >= 25, "{}", all.len());
    let mut ids: Vec<&str> = all.iter().map(|it| it.id.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), before);
}

#[test]
fn every_assistant_has_a_file_on_each_system_it_runs_on() {
    let here = Place::here("tisty".to_string()).unwrap();
    for client in all() {
        let any = client.everywhere.len()
            + client.windows.len()
            + client.macos.len()
            + client.linux.len();
        assert!(any > 0, "{} has nowhere to write", client.id);
    }
    assert!(all().iter().any(|client| client.file(&here).is_some()));
}

#[test]
fn only_its_program_says_an_assistant_is_still_here() {
    let tmp = tempfile::tempdir().unwrap();
    let mut here = Place {
        distro: None,
        os: Os::Linux,
        home: tmp.path().to_path_buf(),
        config: tmp.path().join(".config"),
        dsh: tmp.path().join(".dsh"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: "/opt/tisty".into(),
        open: true,
        packaged: false,
        mount: String::new(),
    };
    let cursor = all().into_iter().find(|it| it.id == "cursor").unwrap();
    let left = tmp.path().join(".cursor").join("mcp.json");
    std::fs::create_dir_all(left.parent().unwrap()).unwrap();
    std::fs::write(left, "{}").unwrap();

    assert_eq!(cursor.found(&here), Some(Found::Sure));
    assert!(!cursor.installed(&here), "its settings outlived it");
    std::fs::write(tmp.path().join("cursor"), "").unwrap();
    here.bins.push(tmp.path().to_path_buf());
    assert!(cursor.installed(&here));
}

#[test]
fn antigravity_is_found_by_its_agy_program_as_well_as_by_its_own_name() {
    for program in ["antigravity", "agy"] {
        let tmp = tempfile::tempdir().unwrap();
        let mut here = Place {
            distro: None,
            os: Os::Linux,
            home: tmp.path().to_path_buf(),
            config: tmp.path().join(".config"),
            dsh: tmp.path().join(".dsh"),
            appdata: None,
            local: None,
            bins: Vec::new(),
            command: "/opt/tisty".into(),
            open: true,
            packaged: false,
            mount: String::new(),
        };
        let antigravity = all().into_iter().find(|it| it.id == "antigravity").unwrap();

        assert!(!antigravity.installed(&here), "{program}");
        std::fs::write(tmp.path().join(program), "").unwrap();
        here.bins.push(tmp.path().to_path_buf());

        assert!(antigravity.installed(&here), "{program}");
    }
}

#[test]
fn the_assistants_whose_program_the_catalog_names_nowhere_are_known() {
    let nowhere: Vec<String> = all()
        .into_iter()
        .filter(|it| it.bins.is_empty() && it.apps.is_empty())
        .map(|it| it.id)
        .collect();
    assert_eq!(nowhere, ["cline", "roo", "continue"]);
}

#[test]
fn a_store_package_still_here_says_so_by_its_own_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let here = Place {
        distro: None,
        os: Os::Windows,
        home: tmp.path().to_path_buf(),
        config: tmp.path().join(".config"),
        dsh: tmp.path().join(".dsh"),
        appdata: Some(tmp.path().join("Roaming")),
        local: Some(tmp.path().join("Local")),
        bins: Vec::new(),
        command: "C:\\Tisty\\tisty.exe".into(),
        open: true,
        packaged: false,
        mount: String::new(),
    };
    let claude = all()
        .into_iter()
        .find(|it| it.id == "claude-desktop")
        .unwrap();

    assert!(!claude.installed(&here));
    std::fs::create_dir_all(tmp.path().join("Local/Packages/Claude_abc")).unwrap();
    assert!(claude.installed(&here));
}

#[test]
fn a_file_another_program_reads_still_matters_while_that_program_is_here() {
    let tmp = tempfile::tempdir().unwrap();
    let mut here = Place {
        distro: None,
        os: Os::Windows,
        home: tmp.path().to_path_buf(),
        config: tmp.path().join(".config"),
        dsh: tmp.path().join(".dsh"),
        appdata: Some(tmp.path().join("Roaming")),
        local: Some(tmp.path().join("Local")),
        bins: Vec::new(),
        command: "C:\\Tisty\\tisty.exe".into(),
        open: true,
        packaged: false,
        mount: String::new(),
    };
    let named = |id: &str| all().into_iter().find(|it| it.id == id).unwrap();

    assert!(!named("copilot").installed(&here) && !named("codex").installed(&here));
    std::fs::write(tmp.path().join("code.cmd"), "").unwrap();
    here.bins.push(tmp.path().to_path_buf());
    assert!(named("copilot").installed(&here), "VS Code reads it");
    std::fs::create_dir_all(tmp.path().join("Local/Packages/OpenAI.ChatGPT_x")).unwrap();
    assert!(named("codex").installed(&here), "ChatGPT reads it");
    assert!(!named("cursor").installed(&here));
}
