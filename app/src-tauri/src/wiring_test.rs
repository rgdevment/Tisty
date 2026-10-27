use super::*;

fn client(id: &str) -> Client {
    catalog::all().into_iter().find(|it| it.id == id).unwrap()
}

fn place(home: &Path, command: &str) -> Place {
    Place {
        distro: None,
        os: places::Os::Linux,
        home: home.to_path_buf(),
        config: home.join(".config"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: command.to_string(),
        open: true,
        packaged: false,
        mount: String::new(),
    }
}

fn wired_here(id: &str, home: &Path, command: &str) -> (Seen, PathBuf) {
    let client = client(id);
    let place = place(home, command);
    let at = client.file(&place).unwrap();
    wire_at(&client, &place, &at).unwrap();
    (told(&client, &place, None).unwrap(), at)
}

#[test]
fn a_settings_file_that_is_not_there_yet_is_written() {
    let tmp = tempfile::tempdir().unwrap();

    let (seen, at) = wired_here("claude-code", tmp.path(), "/opt/tisty");

    assert!(seen.wired && !seen.astray);
    assert!(!at.with_file_name(format!(".claude.json.{BEFORE}")).exists());
    let read: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(at).unwrap()).unwrap();
    assert_eq!(read["mcpServers"]["tisty"]["type"], "stdio");
}

#[test]
fn the_first_original_is_kept_for_good_and_the_last_beside_it() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");
    std::fs::write(&at, "{\n  \"numStartups\": 53\n}\n").unwrap();

    wired_here("claude-code", tmp.path(), "/opt/one");
    wired_here("claude-code", tmp.path(), "/opt/two");

    let first =
        std::fs::read_to_string(at.with_file_name(format!(".claude.json.{BEFORE}"))).unwrap();
    let last = std::fs::read_to_string(at.with_file_name(format!(".claude.json.{LAST}"))).unwrap();
    assert!(!first.contains("tisty"), "{first}");
    assert!(last.contains("/opt/one"), "{last}");
}

#[test]
fn what_the_person_added_to_the_entry_stays() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");
    std::fs::write(
        &at,
        "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"/old/tisty\", \"args\": [\"mcp\"], \"env\": { \"A\": \"1\" } }\n  }\n}\n",
    )
    .unwrap();

    let (seen, _) = wired_here("claude-code", tmp.path(), "/opt/tisty");

    let read: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&at).unwrap()).unwrap();
    assert!(seen.wired);
    assert_eq!(read["mcpServers"]["tisty"]["env"]["A"], "1");
    assert_eq!(read["mcpServers"]["tisty"]["command"], "/opt/tisty");
}

#[test]
fn an_entry_pointing_at_another_tisty_is_told_apart() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("cursor", tmp.path(), "/old/tisty");

    let client = client("cursor");
    let seen = told(&client, &place(tmp.path(), "/opt/tisty"), None).unwrap();

    assert!(seen.astray && !seen.wired);
    assert_eq!(seen.points.as_deref(), Some("/old/tisty"));
}

#[test]
fn an_entry_pointing_at_a_file_that_is_gone_says_so() {
    let tmp = tempfile::tempdir().unwrap();
    let there = tmp.path().join("debug-tisty");
    std::fs::write(&there, b"").unwrap();
    let there = there.display().to_string();
    let gone = tmp.path().join("no-such-tisty").display().to_string();
    let client = client("cursor");

    wired_here("cursor", tmp.path(), &there);
    let seen = told(&client, &place(tmp.path(), "/opt/tisty"), None).unwrap();
    assert!(seen.astray && !seen.gone, "another Tisty, still on disk");

    wired_here("cursor", tmp.path(), &gone);
    let seen = told(&client, &place(tmp.path(), "/opt/tisty"), None).unwrap();
    assert!(seen.astray && seen.gone, "a Tisty that is no longer there");
}

#[test]
fn opencode_is_written_its_own_way_and_read_back() {
    let tmp = tempfile::tempdir().unwrap();

    let (seen, at) = wired_here("opencode", tmp.path(), "/opt/tisty");

    let read: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(at).unwrap()).unwrap();
    assert!(seen.wired);
    assert_eq!(read["mcp"]["tisty"]["command"][1], "mcp");
    assert_eq!(read["mcp"]["tisty"]["enabled"], true);
}

#[test]
fn copilot_keeps_the_tools_the_person_chose() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".copilot/mcp-config.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(
        &at,
        "{ \"mcpServers\": { \"tisty\": { \"command\": \"x\", \"tools\": [\"find\"] } } }",
    )
    .unwrap();

    wired_here("copilot", tmp.path(), "/opt/tisty");

    let read: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&at).unwrap()).unwrap();
    assert_eq!(read["mcpServers"]["tisty"]["tools"][0], "find");
    assert_eq!(read["mcpServers"]["tisty"]["type"], "local");
}

#[test]
fn a_toml_table_keeps_what_the_person_put_in_it() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".codex/config.toml");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "model = \"gpt\"\n\n[mcp_servers.tisty]\ncommand = \"old\"\nargs = [\"mcp\"]\nstartup_timeout_sec = 30\n").unwrap();

    let (seen, _) = wired_here("codex", tmp.path(), "/opt/tisty");

    let read: ::toml::Table = std::fs::read_to_string(&at).unwrap().parse().unwrap();
    assert!(seen.wired);
    assert_eq!(
        read["mcp_servers"]["tisty"]["startup_timeout_sec"].as_integer(),
        Some(30)
    );
    assert_eq!(read["model"].as_str(), Some("gpt"));

    unwire_at(&client("codex"), &place(tmp.path(), "/opt/tisty"), &at).unwrap();
    assert!(!std::fs::read_to_string(&at).unwrap().contains("tisty"));
}

#[test]
fn a_distro_where_windows_may_not_run_is_never_written() {
    let tmp = tempfile::tempdir().unwrap();
    let client = client("claude-code");
    let mut place = place(tmp.path(), "/mnt/c/tisty.exe");
    place.open = false;
    let at = client.file(&place).unwrap();

    assert!(matches!(wire_at(&client, &place, &at), Err(Stuck::Shut)));
    assert!(!at.exists());
}

#[test]
fn an_assistant_connected_by_hand_is_never_written() {
    let tmp = tempfile::tempdir().unwrap();
    let client = client("goose");
    let place = place(tmp.path(), "/opt/tisty");
    let at = client.file(&place).unwrap();

    assert!(matches!(wire_at(&client, &place, &at), Err(Stuck::NoSuch)));
    assert!(told(&client, &place, None).unwrap().by_hand);
}

fn windows(room: &Path) -> Place {
    Place {
        os: places::Os::Windows,
        appdata: Some(room.join("Roaming")),
        local: Some(room.join("Local")),
        ..place(
            room,
            "C:\\Users\\Mario\\AppData\\Local\\Microsoft\\WindowsApps\\tisty.exe",
        )
    }
}

#[test]
fn claude_desktop_with_its_data_out_of_its_package_is_written_where_it_reads() {
    let room = tempfile::tempdir().unwrap();
    let place = windows(room.path());
    let claude = client("claude-desktop");
    let packaged = room
        .path()
        .join("Local/Packages/Claude_pzs8sxrjxfjjc/LocalCache/Roaming/Claude");
    std::fs::create_dir_all(&packaged).unwrap();
    std::fs::write(packaged.join("claude_desktop_config.json"), "{}").unwrap();
    assert_eq!(
        claude.file(&place).unwrap(),
        packaged.join("claude_desktop_config.json")
    );

    let moved = room.path().join("Local/Claude-Data/claude_pzs8sxrjxfjjc");
    std::fs::create_dir_all(&moved).unwrap();
    let at = moved.join("claude_desktop_config.json");
    std::fs::write(
        &at,
        "{\n  \"coworkUserFilesPath\": \"C:\\\\Users\\\\Mario\\\\Claude\",\n  \"preferences\": {\n    \"sidebarMode\": \"chat\",\n    \"epitaxyPrefs\": { \"a\": { \"state\": { \"rowSplit\": 0.5 } } }\n  }\n}",
    )
    .unwrap();

    assert_eq!(claude.file(&place).unwrap(), at);
    assert_eq!(claude.found(&place), Some(catalog::Found::Sure));
    wire_at(&claude, &place, &at).unwrap();

    let said: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&at).unwrap()).unwrap();
    assert_eq!(
        said["mcpServers"]["tisty"]["command"],
        place.command.as_str()
    );
    assert_eq!(said["mcpServers"]["tisty"]["args"][0], "mcp");
    assert_eq!(said["preferences"]["sidebarMode"], "chat");
    assert_eq!(
        said["preferences"]["epitaxyPrefs"]["a"]["state"]["rowSplit"],
        0.5
    );
}

#[test]
fn a_step_ending_in_a_star_takes_the_first_folder_named_so() {
    let room = tempfile::tempdir().unwrap();
    let place = windows(room.path());
    let data = room.path().join("Local/Claude-Data");
    std::fs::create_dir_all(data.join("claude_b")).unwrap();
    std::fs::create_dir_all(data.join("claude_a")).unwrap();
    std::fs::write(data.join("claude_0"), "a file, not a folder").unwrap();

    assert_eq!(
        place.resolve("localappdata:Claude-Data/claude_*/x.json"),
        Some(data.join("claude_a").join("x.json"))
    );
    assert_eq!(
        place.resolve("localappdata:Claude-Data/other_*/x.json"),
        None
    );
    assert_eq!(place.resolve("localappdata:Nowhere/claude_*/x.json"), None);
}

#[cfg(unix)]
#[test]
fn a_store_copy_edits_a_file_already_there_in_place() {
    let room = tempfile::tempdir().unwrap();
    for (packaged, seen) in [(true, "now"), (false, "was")] {
        let at = room.path().join(format!("settings-{packaged}.json"));
        let other = room.path().join(format!("beside-{packaged}.json"));
        std::fs::write(&at, "was").unwrap();
        std::fs::hard_link(&at, &other).unwrap();

        laid(&at, "now", packaged).unwrap();

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "now");
        assert_eq!(std::fs::read_to_string(&other).unwrap(), seen);
    }
}

#[test]
fn only_a_store_copy_writing_under_appdata_writes_in_place() {
    let room = tempfile::tempdir().unwrap();
    let store = Place {
        packaged: true,
        ..windows(room.path())
    };
    let moved = room
        .path()
        .join("Local/Claude-Data/claude_a/claude_desktop_config.json");

    assert!(store.moved_aside(&moved));
    assert!(store.moved_aside(&room.path().join("Roaming/Code/User/mcp.json")));
    assert!(!store.moved_aside(&room.path().join(".claude.json")));
    assert!(!windows(room.path()).moved_aside(&moved));
    let distro = Place {
        distro: Some("Ubuntu".into()),
        ..store.clone()
    };
    assert!(!distro.moved_aside(&moved));
}

#[test]
fn a_store_copy_does_not_make_a_settings_file_the_assistant_never_made() {
    let room = tempfile::tempdir().unwrap();
    let place = Place {
        packaged: true,
        ..windows(room.path())
    };
    let claude = client("claude-desktop");
    std::fs::create_dir_all(room.path().join("Local/Claude-Data/claude_pzs8sxrjxfjjc")).unwrap();
    let at = claude.file(&place).unwrap();

    assert!(matches!(
        wire_at(&claude, &place, &at),
        Err(Stuck::Unborn(_))
    ));
    assert!(!at.exists());

    std::fs::write(&at, "{\"preferences\": {}}").unwrap();
    wire_at(&claude, &place, &at).unwrap();
    assert!(std::fs::read_to_string(&at).unwrap().contains("\"tisty\""));
}

#[test]
fn of_several_folders_named_alike_the_one_holding_the_settings_is_taken() {
    let room = tempfile::tempdir().unwrap();
    let place = windows(room.path());
    let data = room.path().join("Local/Claude-Data");
    std::fs::create_dir_all(data.join("claude_a1beta")).unwrap();
    std::fs::create_dir_all(data.join("claude_pzs8sxrjxfjjc")).unwrap();
    let live = data
        .join("claude_pzs8sxrjxfjjc")
        .join("claude_desktop_config.json");
    std::fs::write(&live, "{}").unwrap();

    assert_eq!(client("claude-desktop").file(&place).unwrap(), live);
}

#[test]
fn a_folder_is_named_alike_whatever_its_case() {
    let room = tempfile::tempdir().unwrap();
    let place = windows(room.path());
    let data = room.path().join("Local/Claude-Data");
    std::fs::create_dir_all(data.join("Claude_PZS")).unwrap();

    assert_eq!(
        place.resolve("localappdata:Claude-Data/claude_*/x.json"),
        Some(data.join("Claude_PZS").join("x.json"))
    );
}
