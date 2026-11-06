use super::*;

#[test]
fn an_engine_starts_with_no_name_to_resolve_and_no_udp() {
    assert!(ENGINE_ARGS.contains("MAP * ~NOTFOUND\""));
    assert!(ENGINE_ARGS.contains("--webrtc-ip-handling-policy=disable_non_proxied_udp"));
}

#[test]
fn the_only_address_an_engine_can_name_is_the_dev_server() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(config["build"]["devUrl"], "http://localhost:1420");

    assert!(
        ENGINE_ARGS.contains(
            "--host-resolver-rules=\"MAP localhost:1420 localhost:1420, MAP * ~NOTFOUND\""
        )
    );
}

#[test]
fn naming_the_engine_arguments_keeps_what_the_webview_passed_by_default() {
    assert!(ENGINE_ARGS.contains("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"));
    assert!(ENGINE_ARGS.contains("--autoplay-policy=no-user-gesture-required"));
}

#[test]
fn every_window_the_config_opens_starts_its_engine_cut_off() {
    let mut config: tauri::Config =
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert!(config.app.windows.len() >= 2);

    cut_off(&mut config);

    for window in &config.app.windows {
        assert_eq!(
            window.additional_browser_args,
            Some(ENGINE_ARGS.to_string()),
            "{} would start its engine differently and WebView2 would refuse it",
            window.label
        );
    }
}

fn sources_under(at: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(at).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources_under(&path, found);
        } else if path
            .extension()
            .is_some_and(|one| one == "rs" || one == "ts" || one == "tsx")
        {
            found.push(path);
        }
    }
}

fn holding(folder: &str, words: &str) -> Vec<std::path::PathBuf> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(folder);
    let mut sources = Vec::new();
    sources_under(&root, &mut sources);
    assert!(!sources.is_empty(), "nothing to look through at {folder}");
    sources
        .into_iter()
        .filter(|path| {
            let inside = path.strip_prefix(&root).unwrap_or(path).to_string_lossy();
            let text = std::fs::read_to_string(path).unwrap_or_default();
            !inside.contains("tests") && !inside.ends_with("engine_test.rs") && text.contains(words)
        })
        .collect()
}

#[test]
fn no_window_is_built_but_through_the_one_that_cuts_the_engine_off() {
    let elsewhere: Vec<_> = holding("src", "WebviewWindowBuilder::new(")
        .into_iter()
        .filter(|path| !path.ends_with("engine.rs"))
        .collect();

    assert!(
        elsewhere.is_empty(),
        "these build a window of their own, which WebView2 would refuse: {elsewhere:?}"
    );
}

#[test]
fn nothing_a_document_holds_ever_runs_in_a_window() {
    let running = holding("../src", "allow-scripts");
    assert!(
        running.is_empty(),
        "these let a frame run a script: {running:?}"
    );

    let served = holding("src", "register_uri_scheme_protocol(\"widget\"");
    assert!(
        served.is_empty(),
        "a scheme to serve a page from is back: {served:?}"
    );

    let config = include_str!("../tauri.conf.json");
    assert!(
        !config.contains("widget:") && !config.contains("widget.localhost"),
        "the window may frame a page again"
    );
}
