use super::*;

const CODE: Client = Client {
    id: "claude-code",
    name: "Claude Code",
    kind: Kind::Json("mcpServers"),
    files: &[],
    signs: &[],
};

const CODEX: Client = Client {
    id: "codex",
    name: "Codex",
    kind: Kind::Toml("mcp_servers"),
    files: &[],
    signs: &[],
};

fn wire_at(client: &'static Client, at: &Path) -> Result<(), Stuck> {
    let was = std::fs::read_to_string(at).unwrap_or_default();
    let entry = entry(client.kind);
    let now = match client.kind {
        Kind::Json(key) => json::set(&was, key, NAME, &entry),
        Kind::Toml(key) => toml::set(&was, key, NAME, &entry),
    }
    .ok_or_else(|| Stuck::Puzzling(at.display().to_string()))?;
    kept(at, &was)?;
    laid(at, &now)
}

fn unwire_at(client: &'static Client, at: &Path) -> Option<()> {
    let was = std::fs::read_to_string(at).unwrap_or_default();
    let now = match client.kind {
        Kind::Json(key) => json::unset(&was, key, NAME),
        Kind::Toml(key) => toml::unset(&was, key, NAME),
    }?;
    laid(at, &now).ok()
}

#[test]
fn a_settings_file_that_is_not_there_yet_is_written() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("deeper/mcp.json");

    wire_at(&CODE, &at).unwrap();

    assert!(told(&CODE, &at).wired);
    assert!(!at.with_file_name(format!("mcp.json.{BEFORE}")).exists());
}

#[test]
fn what_was_written_before_is_kept_beside_it() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");
    let was = "{\n  \"numStartups\": 53,\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"s\" }\n  }\n}\n";
    std::fs::write(&at, was).unwrap();

    wire_at(&CODE, &at).unwrap();

    let beside = at.with_file_name(format!(".claude.json.{BEFORE}"));
    assert_eq!(std::fs::read_to_string(beside).unwrap(), was);
    let now = std::fs::read_to_string(&at).unwrap();
    assert!(now.contains("\"numStartups\": 53"));
    assert!(now.contains("\"sereno\""));
}

#[test]
fn wiring_twice_leaves_one_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");

    wire_at(&CODE, &at).unwrap();
    wire_at(&CODE, &at).unwrap();

    let now = std::fs::read_to_string(&at).unwrap();
    let read: serde_json::Value = serde_json::from_str(&now).unwrap();
    assert_eq!(read["mcpServers"].as_object().unwrap().len(), 1, "{now}");
}

#[test]
fn a_file_written_in_a_way_we_cannot_follow_is_left_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(".claude.json");
    let was = "{ \"mcpServers\": \"none of your business\" }";
    std::fs::write(&at, was).unwrap();

    let stuck = wire_at(&CODE, &at).unwrap_err();

    assert!(matches!(stuck, Stuck::Puzzling(_)));
    assert_eq!(std::fs::read_to_string(&at).unwrap(), was);
}

#[test]
fn a_table_is_written_and_taken_back_out_of_a_toml() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("config.toml");
    std::fs::write(&at, "model = \"gpt\"\n").unwrap();

    wire_at(&CODEX, &at).unwrap();
    assert!(told(&CODEX, &at).wired);

    unwire_at(&CODEX, &at).unwrap();
    assert!(!told(&CODEX, &at).wired);
    assert!(std::fs::read_to_string(&at).unwrap().contains("model"));
}

#[test]
fn what_it_points_at_is_reported_back() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("mcp.json");

    wire_at(&CODE, &at).unwrap();

    assert_eq!(told(&CODE, &at).points, Some(command::calling()));
}

#[test]
fn a_command_that_is_no_longer_there_is_astray() {
    let gone = match cfg!(windows) {
        true => "C:\\Programs\\Gone\\tisty.exe",
        false => "/Applications/Gone.app/Contents/MacOS/tisty",
    };

    assert!(adrift(gone));
}

#[test]
fn a_bare_name_is_left_to_the_path_rather_than_called_astray() {
    assert!(!adrift("tisty"));
}

#[test]
fn the_command_beside_the_window_is_not_astray() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join(command::CLI);
    std::fs::write(&at, b"a command").unwrap();

    assert!(!adrift(&at.display().to_string()));
}

#[test]
fn every_client_is_named_once() {
    let mut ids: Vec<&str> = CLIENTS.iter().map(|it| it.id).collect();
    ids.sort_unstable();
    let mut alone = ids.clone();
    alone.dedup();
    assert_eq!(ids, alone);
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn every_client_has_somewhere_to_write_on_this_machine() {
    for client in CLIENTS {
        assert!(
            client.files.iter().filter_map(spot).next().is_some(),
            "{} has nowhere to write",
            client.id
        );
    }
}
