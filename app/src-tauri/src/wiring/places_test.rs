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
        dsh: PathBuf::from("/home/mario/.dsh"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: "/mnt/c/tisty.exe".into(),
        open: true,
        packaged: false,
        mount: "/mnt/".into(),
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
    assert_eq!(
        place.resolve("dshhome:cordis.patch.yml"),
        Some(PathBuf::from("/home/mario/.dsh/cordis.patch.yml"))
    );
    assert_eq!(place.resolve("appdata:Claude/x.json"), None);
    assert_eq!(place.resolve("/Applications/Claude.app"), None);
}

#[test]
fn the_deepseek_harness_home_is_the_variable_or_dot_dsh() {
    let home = Path::new("/home/mario");
    assert_eq!(dsh_home_of(home, None), home.join(".dsh"));
    let elsewhere = match cfg!(windows) {
        true => PathBuf::from(r"C:\srv\dsh"),
        false => PathBuf::from("/srv/dsh"),
    };
    assert_eq!(dsh_home_of(home, Some(&elsewhere)), elsewhere);
    assert_eq!(
        dsh_home_of(home, Some(Path::new("dsh"))),
        home.join(".dsh"),
        "a relative folder counts as never said"
    );
}

#[test]
fn a_command_in_a_distro_is_reached_where_windows_keeps_it() {
    let place = Place {
        distro: Some("Ubuntu".into()),
        os: Os::Linux,
        home: PathBuf::from("/home/mario"),
        config: PathBuf::from("/home/mario/.config"),
        dsh: PathBuf::from("/home/mario/.dsh"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: "/mnt/c/tisty.exe".into(),
        open: true,
        packaged: false,
        mount: "/mnt/".into(),
    };

    assert_eq!(
        place.reached("/mnt/c/Users/Mario/Tisty/tisty.exe"),
        Some(PathBuf::from(r"C:\Users\Mario\Tisty\tisty.exe"))
    );
    assert_eq!(
        place.reached("/home/mario/.local/bin/tisty-mcp"),
        Some(
            distro_root("Ubuntu")
                .join("home")
                .join("mario")
                .join(".local")
                .join("bin")
                .join("tisty-mcp")
        )
    );
    assert_eq!(
        place.reached("tisty"),
        None,
        "a bare name is found on the PATH, not here"
    );
}

#[test]
fn a_bare_path_on_a_mac_is_widened_to_where_commands_are_installed() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join(".nvm/versions/node/v22.1.0/bin")).unwrap();

    let found = usual_bins(Os::Macos, home.path());

    assert!(found.contains(&PathBuf::from("/opt/homebrew/bin")));
    assert!(found.contains(&home.path().join(".local/bin")));
    assert!(found.contains(&home.path().join(".nvm/versions/node/v22.1.0").join("bin")));
    assert!(usual_bins(Os::Windows, home.path()).is_empty());
}

fn distro_with_bins_under(root: &Path) -> Place {
    let home = root.join("home").join("mario");
    Place {
        distro: Some("Ubuntu".into()),
        os: Os::Linux,
        config: home.join(".config"),
        dsh: home.join(".dsh"),
        appdata: None,
        local: None,
        bins: distro_bins(root, &home),
        home,
        command: "/mnt/c/tisty.exe".into(),
        open: true,
        packaged: false,
        mount: "/mnt/".into(),
    }
}

fn install(root: &Path, dir: &str, name: &str) {
    let bin = dir
        .split('/')
        .fold(root.to_path_buf(), |at, step| at.join(step));
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join(name), "").unwrap();
}

#[test]
fn an_assistant_installed_with_npm_under_nvm_counts_as_installed_in_a_distro() {
    let root = tempfile::tempdir().unwrap();
    install(
        root.path(),
        "home/mario/.nvm/versions/node/v22.1.0/bin",
        "codex",
    );

    assert!(distro_with_bins_under(root.path()).runs("codex"));
}

#[test]
fn an_assistant_installed_under_the_homes_package_managers_counts_as_installed_in_a_distro() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "home/mario/.npm-global/bin", "gemini");
    install(root.path(), "home/mario/.bun/bin", "bunny");
    install(root.path(), "home/mario/.volta/bin", "volty");
    install(root.path(), "home/mario/.cargo/bin", "crabby");
    let place = distro_with_bins_under(root.path());

    for name in ["gemini", "bunny", "volty", "crabby"] {
        assert!(place.runs(name), "{name} is on the distro's usual path");
    }
}

#[test]
fn the_systems_and_the_homes_own_bins_are_still_searched_in_a_distro() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "usr/bin", "from-usr-bin");
    install(root.path(), "usr/local/bin", "from-usr-local-bin");
    install(root.path(), "bin", "from-bin");
    install(root.path(), "home/mario/.local/bin", "from-local-bin");
    install(root.path(), "home/mario/bin", "from-home-bin");
    let place = distro_with_bins_under(root.path());

    for name in [
        "from-usr-bin",
        "from-usr-local-bin",
        "from-bin",
        "from-local-bin",
        "from-home-bin",
    ] {
        assert!(place.runs(name), "{name} is on the distro's path");
    }
}

#[test]
fn an_assistant_that_is_nowhere_in_a_distro_is_not_found() {
    let root = tempfile::tempdir().unwrap();
    install(
        root.path(),
        "home/mario/.nvm/versions/node/v22.1.0/bin",
        "codex",
    );
    let place = distro_with_bins_under(root.path());

    assert!(!place.runs("claude"));
    assert_eq!(place.on_path("claude"), None);
}

fn store_copy(bins: Vec<PathBuf>, command: &str) -> Place {
    Place {
        distro: None,
        os: Os::Windows,
        home: PathBuf::from("C:/Users/Mario"),
        config: PathBuf::from("C:/Users/Mario/.config"),
        dsh: PathBuf::from("C:/Users/Mario/.dsh"),
        appdata: None,
        local: None,
        bins,
        command: command.into(),
        open: true,
        packaged: true,
        mount: String::new(),
    }
}

#[test]
fn a_store_copy_with_the_bare_name_and_nothing_on_the_path_has_its_alias_off() {
    let empty = tempfile::tempdir().unwrap();
    assert!(store_copy(Vec::new(), "tisty").alias_off());
    assert!(store_copy(vec![empty.path().to_path_buf()], "tisty").alias_off());
}

#[test]
fn the_alias_is_not_off_when_the_name_is_found_or_the_command_is_something_else() {
    let bin = tempfile::tempdir().unwrap();
    std::fs::write(bin.path().join("tisty.exe"), b"").unwrap();
    let found = store_copy(vec![bin.path().to_path_buf()], "tisty");
    assert!(!found.alias_off());

    let unpackaged = Place {
        packaged: false,
        ..store_copy(Vec::new(), "tisty")
    };
    assert!(!unpackaged.alias_off());

    assert!(!store_copy(Vec::new(), r"C:\Apps\tisty.exe").alias_off());
    assert!(!store_copy(Vec::new(), "C:/Apps/tisty.exe").alias_off());

    let in_distro = Place {
        distro: Some("Ubuntu".into()),
        ..store_copy(Vec::new(), "tisty")
    };
    assert!(!in_distro.alias_off());

    let linux = Place {
        os: Os::Linux,
        ..store_copy(Vec::new(), "tisty")
    };
    assert!(!linux.alias_off());
}
