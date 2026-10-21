use super::*;

#[test]
fn a_windows_path_is_reached_from_linux_under_its_mount() {
    assert_eq!(
        from_linux(
            "C:\\Users\\Mario\\AppData\\Local\\Programs\\Tisty\\tisty.exe",
            "/mnt/"
        )
        .as_deref(),
        Some("/mnt/c/Users/Mario/AppData/Local/Programs/Tisty/tisty.exe")
    );
    assert_eq!(
        from_linux("D:\\x\\tisty.exe", "/").as_deref(),
        Some("/d/x/tisty.exe")
    );
    assert_eq!(from_linux("tisty", "/mnt/"), None);
}

#[test]
fn wsl_conf_says_where_drives_are_and_whether_windows_may_run() {
    assert_eq!(told_by(""), ("/mnt/".to_string(), true));
    let conf = "[automount]\nroot = /win/ # moved\n\n[interop]\nenabled=false\n";
    assert_eq!(told_by(conf), ("/win/".to_string(), false));
}

#[test]
fn the_default_users_home_is_read_from_passwd() {
    let passwd = "root:x:0:0:root:/root:/bin/bash\nmario:x:1000:1000:,,,:/home/mario:/bin/bash\n";
    assert_eq!(home_in(passwd, 1000).as_deref(), Some("/home/mario"));
    assert_eq!(home_in(passwd, 1001), None);
}

#[test]
fn the_user_wsl_conf_names_wins_over_the_uid_the_registry_keeps() {
    let conf = "[boot]\nsystemd=true\n\n[user]\ndefault=mario\n";
    assert_eq!(user_in(conf).as_deref(), Some("mario"));
    assert_eq!(user_in("[user]\ndefault=\n"), None);
    assert_eq!(user_in("[boot]\ndefault=mario\n"), None);
    let passwd = "root:x:0:0:root:/root:/bin/bash\nmario:x:1000:1000::/home/mario:/usr/bin/zsh\n";
    assert_eq!(home_of(passwd, "mario").as_deref(), Some("/home/mario"));
    assert_eq!(home_of(passwd, "nadie"), None);
}

#[test]
fn roots_resolve_only_where_they_exist() {
    let place = Place {
        distro: Some("Ubuntu".into()),
        os: Os::Linux,
        home: PathBuf::from("/home/mario"),
        config: PathBuf::from("/home/mario/.config"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: "/mnt/c/tisty.exe".into(),
        open: true,
    };
    assert_eq!(
        place.resolve("~/.claude.json"),
        Some(PathBuf::from("/home/mario").join(".claude.json"))
    );
    assert_eq!(
        place.resolve("config:zed/settings.json"),
        Some(
            PathBuf::from("/home/mario/.config")
                .join("zed")
                .join("settings.json")
        )
    );
    assert_eq!(place.resolve("appdata:Claude/x.json"), None);
    assert_eq!(place.resolve("/Applications/Claude.app"), None);
}
