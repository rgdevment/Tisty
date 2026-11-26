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
        dsh: home.join(".dsh"),
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
    assert!(!at.with_file_name(".claude.json.before-tisty").exists());
    let read: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(at).unwrap()).unwrap();
    assert_eq!(read["mcpServers"]["tisty"]["type"], "stdio");
}

#[test]
fn the_first_original_is_kept_for_good_and_the_last_beside_it() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");
    std::fs::write(&at, "{\n  \"numStartups\": 53\n}\n").unwrap();

    wired_here("claude-code", tmp.path(), "/opt/one/tisty");
    wired_here("claude-code", tmp.path(), "/opt/two/tisty");

    let first = std::fs::read_to_string(at.with_file_name(".claude.json.before-tisty")).unwrap();
    let last =
        std::fs::read_to_string(at.with_file_name(".claude.json.before-tisty-last")).unwrap();
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
    let os = match cfg!(windows) {
        true => places::Os::Windows,
        false => places::Os::Linux,
    };
    let here = Place {
        os,
        ..place(tmp.path(), "/opt/tisty")
    };
    let there = tmp.path().join("there").join("tisty");
    std::fs::create_dir_all(there.parent().unwrap()).unwrap();
    std::fs::write(&there, b"").unwrap();
    let there = there.display().to_string();
    let gone = tmp
        .path()
        .join("no-such")
        .join("tisty")
        .display()
        .to_string();
    let client = client("cursor");

    let (_, at) = wired_here("cursor", tmp.path(), &there);
    let seen = told(&client, &here, None).unwrap();
    assert!(seen.astray && !seen.gone, "another Tisty, still on disk");

    std::fs::remove_file(at).unwrap();
    wired_here("cursor", tmp.path(), &gone);
    let seen = told(&client, &here, None).unwrap();
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
        "{ \"mcpServers\": { \"tisty\": { \"command\": \"/old/tisty\", \"tools\": [\"find\"] } } }",
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
    std::fs::write(&at, "model = \"gpt\"\n\n[mcp_servers.tisty]\ncommand = \"/old/tisty\"\nargs = [\"mcp\"]\nstartup_timeout_sec = 30\n").unwrap();

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
fn deepseek_harness_gets_tistys_row_in_its_patch_layer() {
    let tmp = tempfile::tempdir().unwrap();

    let (seen, at) = wired_here("deepseek-harness", tmp.path(), "/opt/tisty");

    let said = std::fs::read_to_string(&at).unwrap();
    assert!(seen.wired && !seen.astray);
    assert!(said.contains("name: '@deepseek-ai/dsh-mcp-client'"));
    assert!(said.contains("serverName: tisty"));
    assert!(said.contains("command: /opt/tisty"));

    unwire_at(
        &client("deepseek-harness"),
        &place(tmp.path(), "/opt/tisty"),
        &at,
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "[]\n");
}

#[test]
fn a_patch_row_pointing_at_another_tisty_is_told_apart() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("deepseek-harness", tmp.path(), "/old/tisty");

    let seen = told(
        &client("deepseek-harness"),
        &place(tmp.path(), "/opt/tisty"),
        None,
    )
    .unwrap();

    assert!(seen.astray && !seen.wired);
    assert_eq!(seen.points.as_deref(), Some("/old/tisty"));
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

        files::laid(&at, "was", "now", packaged).unwrap();

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

#[test]
fn disconnecting_an_assistant_takes_its_hooks_out_too() {
    let tmp = tempfile::tempdir().unwrap();
    let client = client("claude-code");
    let here = place(tmp.path(), "/opt/tisty");
    let at = client.file(&here).unwrap();
    wire_at(&client, &here, &at).unwrap();
    hook_at(&client, &here).unwrap();
    assert!(told(&client, &here, None).unwrap().hooks.unwrap().on);

    unwire_at(&client, &here, &at).unwrap();

    let seen = told(&client, &here, None).unwrap();
    assert!(!seen.wired);
    assert!(!seen.hooks.unwrap().on);
}

#[test]
fn connecting_an_assistant_never_adds_hooks_the_person_did_not_ask_for() {
    let tmp = tempfile::tempdir().unwrap();
    let client = client("gemini-cli");
    let here = place(tmp.path(), "/opt/tisty");
    let at = client.file(&here).unwrap();

    wire_at(&client, &here, &at).unwrap();

    assert!(!told(&client, &here, None).unwrap().hooks.unwrap().on);
}

fn hooked_here(id: &str, home: &Path, command: &str) {
    hook_at(&client(id), &place(home, command)).unwrap();
}

fn looked(id: &str, home: &Path, command: &str) -> Seen {
    told(&client(id), &place(home, command), None).unwrap()
}

#[test]
fn hooks_left_behind_are_seen_with_the_connection_already_right_and_adding_them_again_mends_them() {
    for id in ALL_HOOKED {
        let tmp = tempfile::tempdir().unwrap();
        hooked_here(id, tmp.path(), "/old/tisty");
        wired_here(id, tmp.path(), "/opt/tisty");
        hooked_here(id, tmp.path(), "/old/tisty");

        let before = looked(id, tmp.path(), "/opt/tisty");
        let hooks = before.hooks.unwrap();
        assert!(before.wired && !before.astray, "{id}");
        assert!(hooks.on, "{id}");
        assert_eq!(hooks.stray.len(), 1, "{id}");
        assert_eq!(hooks.stray[0].said, "/old/tisty", "{id}");

        hooked_here(id, tmp.path(), "/opt/tisty");

        let hooks = looked(id, tmp.path(), "/opt/tisty").hooks.unwrap();
        assert!(hooks.on && hooks.stray.is_empty(), "{id}");
    }
}

const ALL_HOOKED: [&str; 6] = [
    "claude-code",
    "cursor",
    "gemini-cli",
    "copilot",
    "codex",
    "antigravity",
];

fn mended(id: &str, home: &Path, ask: &Mend) -> (Option<Stuck>, Option<Stuck>) {
    let place = place(home, "/opt/tisty");
    let client = client(id);
    let at = client.file(&place).unwrap();
    mend_at(&client, &place, &at, ask)
}

fn both() -> Mend {
    Mend {
        connection: Some("/old/tisty".into()),
        hooks: vec!["/old/tisty".into()],
        hooks_out: false,
    }
}

#[test]
fn one_fix_repoints_the_connection_and_the_hooks_it_named() {
    for id in ALL_HOOKED {
        let tmp = tempfile::tempdir().unwrap();
        wired_here(id, tmp.path(), "/old/tisty");
        hooked_here(id, tmp.path(), "/old/tisty");

        let (connection, hooks) = mended(id, tmp.path(), &both());

        assert!(connection.is_none() && hooks.is_none(), "{id}");
        let after = looked(id, tmp.path(), "/opt/tisty");
        let hooks = after.hooks.unwrap();
        assert!(after.wired && hooks.on && hooks.stray.is_empty(), "{id}");
    }
}

#[test]
fn a_fix_keeps_what_the_file_held_before_the_whole_of_it() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("gemini-cli", tmp.path(), "/old/tisty");
    hooked_here("gemini-cli", tmp.path(), "/old/tisty");
    let at = tmp.path().join(".gemini").join("settings.json");
    let was = read_of(&at);

    mended("gemini-cli", tmp.path(), &both());

    assert_eq!(read_of(&beside(&at, "before-tisty-last")), was);
}

#[test]
fn a_fix_never_writes_over_what_changed_since_it_was_named_nor_adds_hooks() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("claude-code", tmp.path(), "/other/tisty");
    let at = tmp.path().join(".claude.json");
    let was = read_of(&at);

    let (connection, hooks) = mended("claude-code", tmp.path(), &both());

    assert!(matches!(connection, Some(Stuck::Moved(_))));
    assert!(matches!(hooks, Some(Stuck::Moved(_))));
    assert_eq!(read_of(&at), was);
    assert!(!tmp.path().join(".claude").join("settings.json").exists());
}

#[test]
fn a_fix_takes_out_the_hooks_left_behind_when_the_plugin_brings_them() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("claude-code", tmp.path(), "/opt/tisty");
    hooked_here("claude-code", tmp.path(), "/old/tisty");

    let ask = Mend {
        connection: None,
        hooks_out: true,
        ..both()
    };
    let (connection, hooks) = mended("claude-code", tmp.path(), &ask);

    assert!(connection.is_none() && hooks.is_none());
    assert!(
        !looked("claude-code", tmp.path(), "/opt/tisty")
            .hooks
            .unwrap()
            .on
    );
}

#[test]
fn one_half_that_cannot_be_written_leaves_the_other_fixed_and_says_which() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("claude-code", tmp.path(), "/old/tisty");
    let settings = tmp.path().join(".claude").join("settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(&settings, b"{ \"WHO\": \"Jos\xE9\" }").unwrap();

    let (connection, hooks) = mended("claude-code", tmp.path(), &both());

    assert!(connection.is_none());
    assert!(matches!(hooks, Some(Stuck::Unread(_))));
    assert!(looked("claude-code", tmp.path(), "/opt/tisty").wired);
}

#[test]
fn a_connection_written_some_other_way_is_the_persons_and_is_not_called_broken() {
    for command in [
        "${userHome}/tisty",
        "~/.local/bin/tisty",
        "/usr/bin/env",
        "tisty",
    ] {
        let tmp = tempfile::tempdir().unwrap();
        wired_here("cursor", tmp.path(), command);

        let seen = looked("cursor", tmp.path(), "/opt/tisty");

        assert!(
            seen.apart && !seen.wired && !seen.astray && !seen.gone,
            "{command}"
        );
    }
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".cursor").join("mcp.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(
        &at,
        "{ \"mcpServers\": { \"tisty\": { \"url\": \"http://x\" } } }",
    )
    .unwrap();
    assert!(looked("cursor", tmp.path(), "/opt/tisty").apart);
}

#[test]
fn a_connection_that_cannot_be_read_is_said_so() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".cursor").join("mcp.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, b"Jos\xE9").unwrap();

    let seen = looked("cursor", tmp.path(), "/opt/tisty");

    assert!(seen.unread && !seen.wired && !seen.astray && !seen.apart);
}

#[test]
fn an_entry_that_cannot_be_taken_out_whole_is_said_so_and_left() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".codex").join("config.toml");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let was = "[mcp_servers.tisty]\ncommand = 't'\nargs = [\n  [\"a\"],\n]\n\n[other]\na = 1\n";
    std::fs::write(&at, was).unwrap();

    unwire_at(&client("codex"), &place(tmp.path(), "/opt/tisty"), &at).unwrap();

    let read: ::toml::Table = read_of(&at).parse().unwrap();
    assert_eq!(read["other"]["a"].as_integer(), Some(1));
    assert!(read.get("mcp_servers").is_none());
}

#[test]
fn a_file_that_already_says_it_is_not_written_again() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");
    std::fs::write(&at, "{\n  \"numStartups\": 53\n}\n").unwrap();
    wired_here("claude-code", tmp.path(), "/opt/tisty");
    let last = at.with_file_name(".claude.json.before-tisty-last");
    let kept = std::fs::read_to_string(&last).unwrap();

    wired_here("claude-code", tmp.path(), "/opt/tisty");

    assert_eq!(std::fs::read_to_string(&last).unwrap(), kept);
    assert!(!kept.contains("tisty"), "{kept}");
}

#[test]
fn a_bare_name_the_path_leads_here_is_this_tisty_and_any_other_is_not() {
    let tmp = tempfile::tempdir().unwrap();
    let (mine, other) = (tmp.path().join("mine"), tmp.path().join("other"));
    for dir in [&mine, &other] {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("tisty"), dir.display().to_string()).unwrap();
    }
    let command = mine.join("tisty").display().to_string();
    wired_here("cursor", tmp.path(), "tisty");
    let with_path = |bins: Vec<PathBuf>| {
        let place = Place {
            bins,
            os: match cfg!(windows) {
                true => places::Os::Windows,
                false => places::Os::Linux,
            },
            ..place(tmp.path(), &command)
        };
        told(&client("cursor"), &place, None).unwrap()
    };

    let here = with_path(vec![mine.clone(), other.clone()]);
    assert!(here.wired && !here.astray);
    for elsewhere in [vec![other, mine], Vec::new()] {
        let seen = with_path(elsewhere);
        assert!(seen.apart && !seen.wired && !seen.astray);
    }
}

#[cfg(unix)]
#[test]
fn a_link_to_this_tisty_is_this_tisty() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path().join("app");
    std::fs::create_dir(&app).unwrap();
    let real = app.join("tisty");
    let link = tmp.path().join("tisty");
    std::fs::write(&real, b"").unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    wired_here("cursor", tmp.path(), &link.display().to_string());

    let seen = looked("cursor", tmp.path(), &real.display().to_string());

    assert!(seen.wired && !seen.astray);
}

fn read_of(at: &Path) -> String {
    std::fs::read_to_string(at).unwrap()
}

fn beside(at: &Path, end: &str) -> PathBuf {
    let named = at.file_name().unwrap().to_string_lossy().into_owned();
    at.with_file_name(format!("{named}.{end}"))
}

#[test]
fn what_was_there_before_the_whole_operation_is_the_last_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".gemini").join("settings.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "{\n  \"theme\": \"first\"\n}\n").unwrap();
    wired_here("gemini-cli", tmp.path(), "/opt/old/tisty");
    hooked_here("gemini-cli", tmp.path(), "/opt/old/tisty");
    let months_later = read_of(&at).replace("first", "months later");
    std::fs::write(&at, &months_later).unwrap();

    wired_here("gemini-cli", tmp.path(), "/opt/tisty");

    assert_eq!(read_of(&beside(&at, "before-tisty-last")), months_later);
    assert!(read_of(&beside(&at, "before-tisty")).contains("first"));
    assert!(!read_of(&beside(&at, "before-tisty")).contains("tisty"));
}

#[test]
fn copilots_own_file_keeps_what_is_not_tistys_and_leaves_a_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".copilot").join("hooks").join("tisty.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let was = r#"{ "version": 1, "hooks": { "sessionStart": [ { "type": "command", "exec": "/opt/old/tisty", "args": ["hook","copilot","start"] }, { "type": "command", "exec": "mine.exe" } ] } }"#;
    std::fs::write(&at, was).unwrap();

    hooked_here("copilot", tmp.path(), "/opt/tisty");

    assert!(read_of(&at).contains("mine.exe"));
    assert_eq!(read_of(&beside(&at, "before-tisty-last")), was);
    let hooked = read_of(&at);
    unhook_at(&client("copilot"), &place(tmp.path(), "/opt/tisty")).unwrap();
    assert!(read_of(&at).contains("mine.exe"));
    assert_eq!(read_of(&beside(&at, "before-tisty-last")), hooked);
}

#[test]
fn copilots_own_file_goes_when_only_tistys_hooks_were_in_it_and_leaves_a_copy() {
    let tmp = tempfile::tempdir().unwrap();
    hooked_here("copilot", tmp.path(), "/opt/tisty");
    let at = tmp.path().join(".copilot").join("hooks").join("tisty.json");
    let was = read_of(&at);

    unhook_at(&client("copilot"), &place(tmp.path(), "/opt/tisty")).unwrap();

    assert!(!at.exists());
    assert_eq!(read_of(&beside(&at, "before-tisty-last")), was);
}

#[test]
fn a_file_that_cannot_be_read_is_never_taken_for_an_empty_one() {
    for (named, bytes) in [
        ("ansi", b"{ \"env\": { \"WHO\": \"Jos\xE9\" } }".to_vec()),
        (
            "utf-16",
            [0xFF, 0xFE]
                .into_iter()
                .chain("{ \"env\": {} }".encode_utf16().flat_map(u16::to_le_bytes))
                .collect(),
        ),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let claude = client("claude-code");
        let here = place(tmp.path(), "/opt/tisty");
        let settings = tmp.path().join(".claude").join("settings.json");
        let connection = claude.file(&here).unwrap();
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        std::fs::write(&settings, &bytes).unwrap();
        std::fs::write(&connection, &bytes).unwrap();

        assert!(hook_at(&claude, &here).is_err(), "{named}");
        assert!(unhook_at(&claude, &here).is_err(), "{named}");
        assert!(wire_at(&claude, &here, &connection).is_err(), "{named}");

        assert_eq!(std::fs::read(&settings).unwrap(), bytes, "{named}");
        assert_eq!(std::fs::read(&connection).unwrap(), bytes, "{named}");
    }
}

#[test]
fn codex_is_never_left_with_a_file_that_does_not_read() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".codex").join("config.toml");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(
        &at,
        "[mcp_servers.tisty]\ncommand = '/opt/old/tisty'\nargs = [\n  \"mcp\",\n]\n",
    )
    .unwrap();

    let (seen, _) = wired_here("codex", tmp.path(), "/opt/tisty");

    assert!(seen.wired);
    assert!(read_of(&at).parse::<::toml::Table>().is_ok());
}

#[test]
fn comments_and_trailing_commas_stay_where_they_were() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".cursor").join("mcp.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let was = "{\n // servers\n \"mcpServers\": {\n  \"tisty\": { \"command\": \"/old/tisty\", /* why */ \"args\": [\"mcp\", \"--mine\",], },\n  },\n}\n";
    std::fs::write(&at, was).unwrap();

    let ask = Mend {
        connection: Some("/old/tisty".into()),
        ..Mend::default()
    };
    let (connection, _) = mended("cursor", tmp.path(), &ask);

    assert!(connection.is_none());
    assert_eq!(read_of(&at), was.replace("/old/tisty", "/opt/tisty"));
}

#[test]
fn a_list_for_a_command_keeps_what_follows_the_program() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("opencode", tmp.path(), "/old/tisty");

    let ask = Mend {
        connection: Some("/old/tisty".into()),
        ..Mend::default()
    };
    let (connection, hooks) = mended("opencode", tmp.path(), &ask);

    assert!(connection.is_none() && hooks.is_none());
    assert!(looked("opencode", tmp.path(), "/opt/tisty").wired);
}

#[test]
fn a_distro_where_windows_may_not_run_is_never_mended() {
    let tmp = tempfile::tempdir().unwrap();
    let claude = client("claude-code");
    let shut = Place {
        open: false,
        ..place(tmp.path(), "/opt/tisty")
    };
    let at = claude.file(&shut).unwrap();

    let (connection, hooks) = mend_at(&claude, &shut, &at, &both());

    assert!(matches!(connection, Some(Stuck::Shut)));
    assert!(matches!(hooks, Some(Stuck::Shut)));
}

#[test]
fn claude_codes_hooks_are_written_only_once_its_plugin_is_known_not_to_bring_them() {
    let tmp = tempfile::tempdir().unwrap();
    let claude = client("claude-code");
    let homes = || agents::Homes::of(tmp.path());
    assert!(unplugged(&claude, homes()).is_ok());
    assert!(unplugged(&claude, None).is_ok());

    let records = tmp.path().join(".claude").join("plugins");
    std::fs::create_dir_all(&records).unwrap();
    let records = records.join("installed_plugins.json");
    std::fs::write(
        &records,
        "{\"plugins\":{\"tisty@x\":[{\"scope\":\"user\"}]}}",
    )
    .unwrap();
    assert!(matches!(unplugged(&claude, homes()), Err(Stuck::Plugged)));
    assert!(unplugged(&client("cursor"), homes()).is_ok());

    std::fs::write(&records, "{ not read").unwrap();
    assert!(matches!(
        unplugged(&claude, homes()),
        Err(Stuck::PluginUnread(_))
    ));
    std::fs::remove_file(&records).unwrap();
    let settings = tmp.path().join(".claude").join("settings.json");
    std::fs::write(&settings, b"Jos\xE9").unwrap();
    assert!(
        unplugged(&claude, homes()).is_ok(),
        "with no plugin of Tisty's there is nothing to ask its settings"
    );
}

#[test]
fn a_fix_leaves_claude_codes_hooks_alone_when_the_plugin_turns_out_to_bring_them() {
    let tmp = tempfile::tempdir().unwrap();
    wired_here("claude-code", tmp.path(), "/old/tisty");
    hooked_here("claude-code", tmp.path(), "/old/tisty");
    let settings = tmp.path().join(".claude").join("settings.json");
    let was = read_of(&settings);
    let records = tmp.path().join(".claude").join("plugins");
    std::fs::create_dir_all(&records).unwrap();
    std::fs::write(
        records.join("installed_plugins.json"),
        r#"{"plugins":{"tisty@x":[{"scope":"user"}]}}"#,
    )
    .unwrap();
    let claude = client("claude-code");
    let here = place(tmp.path(), "/opt/tisty");
    let at = claude.file(&here).unwrap();

    let (connection, hooks) =
        mend_guarded(&claude, &here, &at, &both(), agents::Homes::of(tmp.path()));

    assert!(connection.is_none());
    assert!(matches!(hooks, Some(Stuck::Plugged)));
    assert_eq!(read_of(&settings), was);
    std::fs::remove_dir_all(&records).unwrap();
    let only_hooks = Mend {
        connection: None,
        ..both()
    };
    let (_, hooks) = mend_guarded(
        &claude,
        &here,
        &at,
        &only_hooks,
        agents::Homes::of(tmp.path()),
    );
    assert!(hooks.is_none());
    assert!(!read_of(&settings).contains("/old/tisty"));
}

const PERSONS_HOOKS: &str = r#"{
  "hooks": {
    "Stop": [
      { "hooks": [ { "type": "command", "command": "say done" } ] }
    ]
  }
}
"#;

const PERSONS_NAMED_HOOK: &str = r#"{
  "my-linter": {
    "Stop": [ { "type": "command", "command": "lint --all" } ]
  }
}
"#;

fn json_at(at: &Path) -> serde_json::Value {
    json_at_text(&read_of(at))
}

fn json_at_text(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn hooking_then_unhooking_codex_leaves_the_persons_hooks_file_as_it_was() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".codex").join("hooks.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, PERSONS_HOOKS).unwrap();
    let codex = client("codex");
    let here = place(tmp.path(), "/opt/tisty");

    hook_at(&codex, &here).unwrap();

    let hooks = told(&codex, &here, None).unwrap().hooks.unwrap();
    assert!(
        hooks.on && !hooks.apart && hooks.stray.is_empty(),
        "{hooks:?}"
    );
    assert!(hooks.at.ends_with("hooks.json"), "{}", hooks.at);
    let now = json_at(&at);
    assert_eq!(
        now["hooks"]["Stop"][0],
        json_at_text(PERSONS_HOOKS)["hooks"]["Stop"][0]
    );
    assert!(read_of(&at).contains("'/opt/tisty' hook codex start"));

    unhook_at(&codex, &here).unwrap();

    assert!(!told(&codex, &here, None).unwrap().hooks.unwrap().on);
    assert_eq!(json_at(&at), json_at_text(PERSONS_HOOKS));
}

#[test]
fn antigravity_is_hooked_in_the_file_beside_its_connection_and_keeps_the_persons_named_hooks() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".gemini").join("config").join("hooks.json");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, PERSONS_NAMED_HOOK).unwrap();
    let antigravity = client("antigravity");
    let here = place(tmp.path(), "/opt/tisty");
    let connection = antigravity.file(&here).unwrap();

    wire_at(&antigravity, &here, &connection).unwrap();
    assert!(!told(&antigravity, &here, None).unwrap().hooks.unwrap().on);
    hook_at(&antigravity, &here).unwrap();

    let seen = told(&antigravity, &here, None).unwrap();
    let hooks = seen.hooks.unwrap();
    assert!(
        seen.wired && hooks.on && hooks.stray.is_empty(),
        "{hooks:?}"
    );
    assert!(!hooks.apart, "{hooks:?}");
    let now = json_at(&at);
    assert_eq!(
        now["my-linter"],
        json_at_text(PERSONS_NAMED_HOOK)["my-linter"]
    );
    assert_eq!(
        now["tisty"]["SessionStart"][0]["command"],
        "'/opt/tisty' hook antigravity start"
    );
    assert!(json_at(&connection)["mcpServers"]["tisty"].is_object());

    unhook_at(&antigravity, &here).unwrap();

    let seen = told(&antigravity, &here, None).unwrap();
    assert!(seen.wired && !seen.hooks.unwrap().on);
    assert_eq!(
        json_at(&at)["my-linter"],
        json_at_text(PERSONS_NAMED_HOOK)["my-linter"]
    );
    assert!(!read_of(&at).contains("/opt/tisty"));
}

#[test]
fn disconnecting_codex_or_antigravity_takes_its_hooks_out_too() {
    for id in ["codex", "antigravity"] {
        let tmp = tempfile::tempdir().unwrap();
        let client = client(id);
        let here = place(tmp.path(), "/opt/tisty");
        let at = client.file(&here).unwrap();
        wire_at(&client, &here, &at).unwrap();
        hook_at(&client, &here).unwrap();
        assert!(
            told(&client, &here, None).unwrap().hooks.unwrap().on,
            "{id}"
        );

        unwire_at(&client, &here, &at).unwrap();

        let seen = told(&client, &here, None).unwrap();
        assert!(!seen.wired, "{id}");
        assert!(!seen.hooks.unwrap().on, "{id}");
    }
}

#[test]
fn connecting_codex_or_antigravity_never_adds_hooks_the_person_did_not_ask_for() {
    for id in ["codex", "antigravity"] {
        let tmp = tempfile::tempdir().unwrap();
        let client = client(id);
        let here = place(tmp.path(), "/opt/tisty");
        let at = client.file(&here).unwrap();

        wire_at(&client, &here, &at).unwrap();

        let hooks = told(&client, &here, None).unwrap().hooks.unwrap();
        assert!(!hooks.on, "{id}");
        assert!(!Path::new(&hooks.at).exists(), "{id}");
    }
}

#[test]
fn a_hook_file_written_by_a_mend_keeps_the_copy_of_what_was_before_for_codex_and_antigravity() {
    for (id, rel) in [
        ("codex", ".codex/hooks.json"),
        ("antigravity", ".gemini/config/hooks.json"),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        wired_here(id, tmp.path(), "/old/tisty");
        hooked_here(id, tmp.path(), "/old/tisty");
        let at = tmp.path().join(rel);
        let was = read_of(&at);

        mended(id, tmp.path(), &both());

        assert_eq!(read_of(&beside(&at, "before-tisty-last")), was, "{id}");
        assert!(!read_of(&at).contains("/old/tisty"), "{id}");
    }
}

fn persons_hook_here(id: &str, home: &Path, command: &str) {
    let kind = hooks::kind(id).unwrap();
    let here = place(home, command);
    let at = kind.file(&here).unwrap();
    let was = std::fs::read_to_string(&at).unwrap_or_default();
    let hook = user_hooks::Hook {
        name: "guard.sh",
        event: tisty_core::agents::Event::BeforeTool,
        tools: tisty_core::agents::Tools::Shell,
        timeout: 5,
    };
    let now = user_hooks::with(kind, &was, &here, &hook).unwrap();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, now).unwrap();
}

#[test]
fn one_fix_repoints_the_persons_own_hooks_with_tistys_in_every_agent() {
    for id in ALL_HOOKED {
        let tmp = tempfile::tempdir().unwrap();
        wired_here(id, tmp.path(), "/old/tisty");
        hooked_here(id, tmp.path(), "/old/tisty");
        persons_hook_here(id, tmp.path(), "/old/tisty");
        let at = hooks::kind(id)
            .unwrap()
            .file(&place(tmp.path(), "/opt/tisty"))
            .unwrap();

        let (connection, hooks) = mended(id, tmp.path(), &both());

        assert!(connection.is_none() && hooks.is_none(), "{id}");
        let after = looked(id, tmp.path(), "/opt/tisty");
        let hooks = after.hooks.unwrap();
        assert!(
            after.wired && hooks.on && hooks.stray.is_empty(),
            "{id} {hooks:?}"
        );
        let text = read_of(&at);
        assert!(
            text.contains("run-hook") && !text.contains("/old/tisty"),
            "{id} {text}"
        );
    }
}

#[test]
fn only_the_persons_own_hook_left_behind_still_asks_for_a_fix() {
    for id in ALL_HOOKED {
        let tmp = tempfile::tempdir().unwrap();
        wired_here(id, tmp.path(), "/opt/tisty");
        persons_hook_here(id, tmp.path(), "/old/tisty");

        let hooks = looked(id, tmp.path(), "/opt/tisty").hooks.unwrap();

        assert!(!hooks.on, "{id}");
        assert_eq!(hooks.stray.len(), 1, "{id} {hooks:?}");
        assert_eq!(hooks.stray[0].said, "/old/tisty", "{id}");
        let ask = Mend {
            connection: None,
            hooks: vec!["/old/tisty".into()],
            hooks_out: false,
        };
        let (_, stuck) = mended(id, tmp.path(), &ask);
        assert!(stuck.is_none(), "{id} {stuck:?}");
        assert!(
            looked(id, tmp.path(), "/opt/tisty")
                .hooks
                .unwrap()
                .stray
                .is_empty(),
            "{id}"
        );
    }
}
