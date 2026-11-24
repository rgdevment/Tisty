use super::*;

#[cfg(windows)]
const HOST: (places::Os, &str, &str) = (
    places::Os::Windows,
    "C:\\New\\tisty.exe",
    "C:\\NoSuch\\tisty.exe",
);
#[cfg(not(windows))]
const HOST: (places::Os, &str, &str) = (places::Os::Linux, "/New/tisty", "/NoSuch/tisty");
const NEW: &str = HOST.1;
const OLD: &str = HOST.2;

fn in_json(said: &str) -> String {
    serde_json::to_string(said).unwrap()
}

fn client(id: &str) -> Client {
    catalog::all().into_iter().find(|it| it.id == id).unwrap()
}

fn place(home: &Path, command: &str) -> Place {
    Place {
        distro: None,
        os: HOST.0,
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

struct Room {
    home: tempfile::TempDir,
}

impl Room {
    fn with(rel: &str, text: &str) -> Self {
        let room = Room {
            home: tempfile::tempdir().unwrap(),
        };
        room.put(rel, text.as_bytes());
        room
    }

    fn put(&self, rel: &str, bytes: &[u8]) {
        let at = self.home.path().join(rel);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, bytes).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.home.path().join(rel)).unwrap()
    }

    fn here(&self) -> Place {
        place(self.home.path(), NEW)
    }

    fn row(&self, id: &str) -> Seen {
        told(&client(id), &self.here(), None).unwrap()
    }

    fn at(&self, id: &str) -> PathBuf {
        client(id).file(&self.here()).unwrap()
    }

    fn wire(&self, id: &str) -> Result<(), Stuck> {
        wire_at(&client(id), &self.here(), &self.at(id))
    }

    fn unwire(&self, id: &str) -> Result<(), Stuck> {
        unwire_at(&client(id), &self.here(), &self.at(id))
    }

    fn hook(&self) -> Result<(), Stuck> {
        hook_at(&client("claude-code"), &self.here())
    }

    fn unhook(&self) -> Result<(), Stuck> {
        unhook_at(&client("claude-code"), &self.here())
    }

    fn mend(&self, id: &str, ask: Mend) -> (Option<Stuck>, Option<Stuck>) {
        mend_at(&client(id), &self.here(), &self.at(id), &ask)
    }

    fn listed(&self, rel: &str) -> Vec<String> {
        let mut all: Vec<String> = std::fs::read_dir(self.home.path().join(rel))
            .unwrap()
            .flatten()
            .map(|one| one.file_name().to_string_lossy().into_owned())
            .collect();
        all.sort();
        all
    }
}

fn connection() -> Mend {
    Mend {
        connection: Some(OLD.into()),
        ..Mend::default()
    }
}

fn hooks() -> Mend {
    Mend {
        hooks: vec![OLD.into()],
        ..Mend::default()
    }
}

const CODEX: &str = ".codex/config.toml";
const CLAUDE: &str = ".claude.json";
const SETTINGS: &str = ".claude/settings.json";

#[test]
fn every_way_codex_is_written_is_fixed_changing_the_program_alone() {
    let o = OLD;
    for (named, text) in [
        (
            "inline",
            format!(
                "# top\r\n[mcp_servers]\r\ntisty = {{ command = '{o}', args = [\"mcp\"] }} # mine\r\nother = {{ command = \"x\" }}\r\n"
            ),
        ),
        (
            "dotted",
            format!(
                "model = \"m\"\nmcp_servers.tisty.command = '{o}' # why\nmcp_servers.tisty.args = [ \"mcp\" , ] # tail\n# note\nmcp_servers.other.command = \"x\"\n"
            ),
        ),
        (
            "dotted in a table",
            format!(
                "[mcp_servers]\n# a\ntisty.command = '{o}'\n# b\ntisty.args = [\"mcp\"]\ntisty.env.X = \"1\"\n"
            ),
        ),
        (
            "several lines",
            format!(
                "[mcp_servers.tisty] # head\n# before\ncommand = '{o}'   # cmd\nargs = [\n  # first\n  \"mcp\", # the arg\n]\n\n# env\n[mcp_servers.tisty.env]\nA = \"1\"\n"
            ),
        ),
        (
            "a text that looks like it",
            format!(
                "note = \"\"\"\n[mcp_servers.tisty]\ncommand = 'fake'\n\"\"\"\n\n[mcp_servers.tisty]\ncommand = '{o}'\nargs = [\"mcp\"]\n"
            ),
        ),
        (
            "keys in quotes",
            format!("[mcp_servers.\"tisty\"]\n\"command\" = '{o}'\n'args' = [\"mcp\"]\n"),
        ),
        (
            "Windows ends",
            format!("[mcp_servers.tisty]\r\ncommand = '{o}'\r\nargs = [\r\n\"mcp\",\r\n]\r\n"),
        ),
        (
            "ends of both kinds",
            format!("a = 1\r\n[mcp_servers.tisty]\ncommand = '{o}'\r\nargs = [\"mcp\"]\n"),
        ),
        (
            "a mark at the start",
            format!("\u{feff}[mcp_servers.tisty]\ncommand = '{o}'\nargs = [\"mcp\"]\n"),
        ),
        (
            "no end to its last line",
            format!("[mcp_servers.tisty]\nargs=[\"mcp\"]\ncommand='{o}'"),
        ),
        (
            "a last table with no end",
            format!("[mcp_servers.tisty]\ncommand='{o}'\n[other]"),
        ),
        (
            "a subtable",
            format!(
                "[mcp_servers.tisty]\ncommand = '{o}'\n\n[mcp_servers.tisty.tools.read]\napproval_mode = \"approve\"\n"
            ),
        ),
        (
            "a text over two lines",
            format!("note = '''a\r\nb'''\r\n[mcp_servers.tisty]\r\ncommand = '{o}'\r\n"),
        ),
    ] {
        let room = Room::with(CODEX, &text);
        assert!(room.row("codex").gone, "{named}: {:?}", room.row("codex"));

        let (failed, _) = room.mend("codex", connection());

        assert!(failed.is_none(), "{named}: {failed:?}");
        let expected = text.replace(&format!("'{o}'"), "@");
        let now = room.read(CODEX);
        let now = now
            .replace(&in_json(NEW), "@")
            .replace(&format!("'{NEW}'"), "@");
        assert_eq!(now, expected, "{named}");
        assert!(room.row("codex").wired, "{named}");
    }
}

#[test]
fn codex_keeps_every_comment_when_tisty_is_taken_out() {
    for text in [
        "# top\nmodel = \"a\"\n\n[mcp_servers.tisty]\ncommand = 'C:\\New\\tisty.exe'\nargs = [\"mcp\"]\n\n# my other server\n[mcp_servers.other]\ncommand = \"x\"\n",
        "# MY CONFIG\n# second line\n[mcp_servers.tisty]\ncommand = 'C:\\New\\tisty.exe'\nargs = [\"mcp\"]\n\n[mcp_servers.tisty.tools.read]\napproval_mode = \"approve\"\n\n# mine\n[mcp_servers.other]\ncommand = \"x\"\n",
        "[mcp_servers.tisty]\ncommand = 'C:\\New\\tisty.exe'\nargs = [\"mcp\"]\n\n# TODO: add the postgres server\n# [mcp_servers.pg]\n# command = \"pg\"\n",
    ] {
        let room = Room::with(CODEX, text);

        room.unwire("codex").unwrap();

        let now = room.read(CODEX);
        assert!(!now.contains("tisty"), "{now}");
        assert!(now.parse::<::toml::Table>().is_ok(), "{now}");
        for note in text.lines().filter(|line| line.starts_with('#')) {
            assert!(now.contains(note), "«{note}» was lost:\n{now}");
        }
    }
}

const FILES_THAT_ARE_NOT_JSON: [(&str, &str); 6] = [
    (
        "cut short",
        "{ \"projects\": { \"C:/x\": { \"history\": [1,2",
    ),
    (
        "cut inside its hooks",
        "{ \"a\": 1, \"hooks\": { \"Stop\": [ ",
    ),
    (
        "a conflict",
        "{\n<<<<<<< HEAD\n  \"a\": 1,\n=======\n  \"a\": 2,\n>>>>>>> x\n  \"b\": {}\n}\n",
    ),
    (
        "a comma missing",
        "{\n  \"a\": 1\n  \"mcpServers\": { \"mine\": { \"command\": \"m\" } }\n}\n",
    ),
    (
        "another kind of JSON",
        "{\n  mcpServers: { mine: { command: 'm' } },\n}\n",
    ),
    ("two of them", "{ \"a\": 1 }\n{ \"b\": 2 }\n"),
];

#[test]
fn a_file_that_is_not_json_is_never_written() {
    for (named, text) in FILES_THAT_ARE_NOT_JSON {
        let room = Room::with(CLAUDE, text);
        room.put(SETTINGS, text.as_bytes());

        assert!(room.wire("claude-code").is_err(), "{named}");
        assert!(room.hook().is_err(), "{named}");

        assert_eq!(room.read(CLAUDE), text, "{named}");
        assert_eq!(room.read(SETTINGS), text, "{named}");
        assert_eq!(room.listed(".claude"), ["settings.json"], "{named}");
    }
}

fn utf16(text: &str, mark: bool) -> Vec<u8> {
    let mut all = if mark { vec![0xFF, 0xFE] } else { Vec::new() };
    all.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    all
}

#[test]
fn a_file_in_another_encoding_is_never_written() {
    for mark in [true, false] {
        let json = utf16("{ \"permissions\": { \"allow\": [\"Bash\"] } }", mark);
        let toml = utf16("model = \"x\"\n", mark);
        let room = Room::with("x", "");
        room.put(CLAUDE, &json);
        room.put(SETTINGS, &json);
        room.put(CODEX, &toml);

        assert!(room.wire("claude-code").is_err(), "{mark}");
        assert!(room.hook().is_err(), "{mark}");
        assert!(room.wire("codex").is_err(), "{mark}");

        let bytes = |rel: &str| std::fs::read(room.home.path().join(rel)).unwrap();
        assert_eq!(bytes(CLAUDE), json, "{mark}");
        assert_eq!(bytes(SETTINGS), json, "{mark}");
        assert_eq!(bytes(CODEX), toml, "{mark}");
    }
}

const FILES_THAT_STAY_WRITABLE: [(&str, &str); 9] = [
    ("empty", ""),
    ("spaces", "  \r\n \t"),
    ("plain", "{ \"a\": 1 }"),
    ("a mark at the start", "\u{feff}{\r\n  \"a\": 1\r\n}\r\n"),
    (
        "comments",
        "{\n  // mine: http://x//y\n  \"a\": \"http://x//y\", /* z */\n  \"b\": 2\n}\n",
    ),
    ("a comma at the end", "{\n  \"a\": [1, 2,],\n}\n"),
    ("half an emoji", "{ \"history\": \"\\ud83d\", \"a\": 1 }"),
    (
        "a number past any",
        "{ \"id\": 12345678901234567890123, \"e\": 1E400 }",
    ),
    ("no end to its last line", "{\n  \"a\": 1\n}"),
];

#[test]
fn what_could_be_written_before_still_can() {
    for (named, text) in FILES_THAT_STAY_WRITABLE {
        let room = Room::with(CLAUDE, text);
        room.put(SETTINGS, text.as_bytes());

        room.wire("claude-code")
            .unwrap_or_else(|why| panic!("{named}: {why:?}"));
        room.hook().unwrap_or_else(|why| panic!("{named}: {why:?}"));

        let seen = room.row("claude-code");
        assert!(seen.wired && seen.hooks.unwrap().on, "{named}");
        room.unhook().unwrap();
        room.unwire("claude-code").unwrap();
        let seen = room.row("claude-code");
        assert!(!seen.wired && !seen.hooks.unwrap().on, "{named}");
        for word in ["\"a\"", "mine", "history", "\"id\""] {
            assert_eq!(
                text.contains(word),
                room.read(CLAUDE).contains(word) && room.read(SETTINGS).contains(word),
                "{named}: {word}"
            );
        }
    }
}

#[test]
fn codex_is_written_into_whatever_toml_it_had() {
    for text in [
        "",
        "model = \"x\"",
        "\u{feff}model = \"x\"\r\n",
        "a = 1\r\nb = 2\n",
    ] {
        let room = Room::with(CODEX, text);
        room.wire("codex").unwrap();
        assert!(room.row("codex").wired, "{text:?}");
    }
}

#[test]
fn a_hook_of_the_persons_stays_whatever_shape_it_is_in() {
    for text in [
        r#"{ "hooks": { "SessionStart": { "hooks": [ { "type": "command", "command": "mine.exe" } ] } } }"#,
        r#"{ "hooks": { "Stop": "echo mine" } }"#,
        r#"{ "hooks": { "SessionStart": null, "Stop": [] } }"#,
        r#"{ "hooks": [ { "mine": 1 } ] }"#,
        r#"{ "hooks": "mine" }"#,
        r#"{ "hooks": null }"#,
    ] {
        let room = Room::with(SETTINGS, text);
        assert!(room.hook().is_err(), "{text}");
        assert_eq!(room.read(SETTINGS), text);
    }
    let text = r#"{ "hooks": { "SessionStart": ["mine.exe", 3, null, {"hooks": "mine"}], "PreToolUse": [ { "hooks": [ { "command": "mine2" } ] } ] } }"#;
    let room = Room::with(SETTINGS, text);
    room.hook().unwrap();
    room.unhook().unwrap();
    let now: serde_json::Value = serde_json::from_str(&room.read(SETTINGS)).unwrap();
    let was: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(now["hooks"]["SessionStart"], was["hooks"]["SessionStart"]);
    assert_eq!(now["hooks"]["PreToolUse"], was["hooks"]["PreToolUse"]);
}

const MINE: &str = r#"{ "type": "command", "command": "C:\\mine\\my-own-hook.exe" }"#;
fn ours() -> String {
    let old = in_json(OLD);
    format!(
        "{{ \"type\": \"command\", \"command\": {old}, \"args\": [\"hook\", \"claude-code\", \"start\"], \"timeout\": 99 }}"
    )
}

#[test]
fn taking_the_hooks_out_leaves_every_comment_and_a_list_that_still_reads() {
    let ours = ours();
    for text in [
        format!(
            "{{\n  \"hooks\": {{\n    \"SessionStart\": [\n      {{ \"hooks\": [\n        {ours}, // tisty\n        // MY NOTE: keep this one\n        {MINE}\n      ] }}\n    ]\n  }}\n}}\n"
        ),
        format!(
            "{{\n  \"hooks\": {{\n    \"SessionStart\": [\n      {{ \"hooks\": [\n        {MINE}, // MY NOTE\n        {ours}\n      ] }}\n    ]\n  }}\n}}\n"
        ),
        format!(
            "{{\n  \"hooks\": {{\n    \"SessionStart\": [\n      // MY NOTE about session start\n      {{ \"hooks\": [ {ours} ] }}\n      // disabled: {MINE}\n    ]\n  }}\n}}\n"
        ),
        format!(
            "{{\n  \"hooks\": {{\n    \"SessionStart\": [\n      {{ \"hooks\": [\n        // MY NOTE\n        {ours}\n        /* and after */\n      ] }}\n    ]\n  }}\n}}\n"
        ),
    ] {
        let room = Room::with(SETTINGS, &text);

        room.unhook().unwrap();

        let now = room.read(SETTINGS);
        assert!(!now.contains("NoSuch"), "{now}");
        assert!(now.contains("MY NOTE"), "{now}");
        assert_eq!(
            text.contains("my-own-hook"),
            now.contains("my-own-hook"),
            "{now}"
        );
        assert_eq!(
            text.contains("and after"),
            now.contains("and after"),
            "{now}"
        );
        assert!(!json::commented(&now) || json::valid(&now), "{now}");
        let bare: String = now
            .lines()
            .map(|line| line.split("//").next().unwrap())
            .collect();
        let bare = bare.replace("/* and after */", "");
        assert!(
            serde_json::from_str::<serde_json::Value>(&bare).is_ok(),
            "no comma is left hanging: {now}"
        );
    }
}

#[test]
fn a_line_the_person_wrapped_stays_when_hooks_or_connection_are_taken_out() {
    let ours = ours();
    let wrapped = r#"{ "type": "command", "command": "my-logger && $HOME/bin/tisty hook claude-code start" }"#;
    let text = format!(
        "{{ \"hooks\": {{ \"SessionStart\": [ {{ \"hooks\": [ {wrapped}, {ours} ] }} ] }} }}"
    );
    let only =
        format!("{{ \"hooks\": {{ \"SessionStart\": [ {{ \"hooks\": [ {wrapped} ] }} ] }} }}");

    let room = Room::with(SETTINGS, &text);
    let seen = room.row("claude-code").hooks.unwrap();
    assert!(seen.on && seen.apart);
    room.unhook().unwrap();
    assert!(room.read(SETTINGS).contains("my-logger"));
    let seen = room.row("claude-code").hooks.unwrap();
    assert!(!seen.on && seen.apart);
    assert!(room.hook().is_err(), "a second one would run beside it");
    assert!(room.read(SETTINGS).contains("my-logger"));

    let room = Room::with(SETTINGS, &only);
    room.wire("claude-code").unwrap();
    room.unwire("claude-code").unwrap();
    assert_eq!(room.read(SETTINGS), only);
}

#[test]
fn nothing_is_taken_out_when_one_of_the_two_files_cannot_be_read() {
    let room = Room::with("x", "");
    room.wire("claude-code").unwrap();
    let was = room.read(CLAUDE);
    room.put(SETTINGS, &utf16("{}", true));

    assert!(matches!(room.unwire("claude-code"), Err(Stuck::Unread(_))));

    assert_eq!(room.read(CLAUDE), was);
}

#[test]
fn connection_and_hooks_in_one_file_are_taken_out_in_one_write() {
    let room = Room::with(".gemini/settings.json", "{\n  \"theme\": \"dark\"\n}\n");
    room.wire("gemini-cli").unwrap();
    hook_at(&client("gemini-cli"), &room.here()).unwrap();
    let was = room.read(".gemini/settings.json");

    room.unwire("gemini-cli").unwrap();

    let now = room.read(".gemini/settings.json");
    assert!(!now.contains("tisty") && now.contains("dark"), "{now}");
    assert_eq!(room.read(".gemini/settings.json.before-tisty-last"), was);
}

#[test]
fn an_entry_the_person_wrote_their_own_way_is_never_connected_over() {
    for (id, rel, text) in [
        (
            "codex",
            CODEX,
            "[mcp_servers.tisty]\ncommand = \"wsl.exe\"\nargs = [\"-d\", \"Ubuntu\", \"tisty\", \"mcp\"] # mine\n",
        ),
        (
            "claude-code",
            CLAUDE,
            "{ \"mcpServers\": { \"tisty\": { \"command\": \"wsl.exe\", \"args\": [\"tisty\", \"mcp\"] } } }",
        ),
        (
            "claude-code",
            CLAUDE,
            "{ \"mcpServers\": { \"tisty\": { \"url\": \"http://x\" } } }",
        ),
    ] {
        let room = Room::with(rel, text);
        assert!(matches!(room.wire(id), Err(Stuck::Moved(_))), "{text}");
        assert_eq!(room.read(rel), text);
    }
}

#[test]
fn an_entry_already_there_that_cannot_be_changed_is_never_told_to_be_pasted_again() {
    let text = format!(
        "{{ \"mcpServers\": {{ \"tisty\": {{ \"command\": \"{}\" }} }} \"a\": 1 }}",
        OLD.replace('\\', "\\\\")
    );
    let room = Room::with(CLAUDE, &text);

    let (failed, _) = room.mend("claude-code", connection());

    assert!(matches!(failed, Some(Stuck::Unfixable(_))), "{failed:?}");
    assert_eq!(room.read(CLAUDE), text);
}

#[test]
fn what_the_agent_wrote_meanwhile_is_never_written_over() {
    let room = Room::with(CLAUDE, "{ \"numStartups\": 2 }");
    let at = room.at("claude-code");

    for (file, was) in [(&at, "{ \"numStartups\": 1 }"), (&at, "")] {
        let late = files::laid(file, was, "{ \"tisty\": 1 }", false);
        assert!(matches!(late, Err(Stuck::Moved(_))), "{late:?}");
    }
    let moved = Writes::default().change(&at, "{ \"numStartups\": 1 }", "{ \"tisty\": 1 }", false);
    assert!(matches!(moved, Err(Stuck::Moved(_))), "{moved:?}");
    let aside = Writes::default().change(&at, "{ \"numStartups\": 1 }", "{ \"tisty\": 1 }", true);
    assert!(matches!(aside, Err(Stuck::Moved(_))), "{aside:?}");
    let removed = Writes::default().remove(&at, "{ \"numStartups\": 1 }");
    assert!(matches!(removed, Err(Stuck::Moved(_))), "{removed:?}");

    assert_eq!(room.read(CLAUDE), "{ \"numStartups\": 2 }");
    assert_eq!(
        room.listed("."),
        [".claude.json"],
        "nothing is left beside it"
    );
    let missing = room.home.path().join("gone.json");
    let gone = Writes::default().change(&missing, "{ \"was\": 1 }", "{}", false);
    assert!(matches!(gone, Err(Stuck::Moved(_))), "{gone:?}");
}

#[cfg(windows)]
#[test]
fn a_hook_claude_runs_through_a_shell_is_written_so_that_shell_reads_it() {
    for (was, new, now) in [
        (
            "C:/NoSuch/tisty.exe hook claude-code start",
            NEW,
            "C:/New/tisty.exe hook claude-code start",
        ),
        (
            "\"C:/NoSuch/tisty.exe\" hook claude-code start",
            NEW,
            "\"C:/New/tisty.exe\" hook claude-code start",
        ),
        (
            "'C:\\\\NoSuch\\\\tisty.exe' hook claude-code start",
            NEW,
            "\"C:/New/tisty.exe\" hook claude-code start",
        ),
        (
            "C:/NoSuch/tisty.exe hook claude-code start && echo hi",
            "C:\\Program Files\\Tisty\\tisty.exe",
            "\"C:/Program Files/Tisty/tisty.exe\" hook claude-code start && echo hi",
        ),
    ] {
        let line = |said: &str| {
            format!(
                "{{ \"hooks\": {{ \"SessionStart\": [ {{ \"hooks\": [ {{ \"type\": \"command\", \"command\": {} }} ] }} ] }} }}",
                serde_json::to_string(said).unwrap()
            )
        };
        let room = Room::with(SETTINGS, &line(&was.replace("\\\\", "\\")));
        let here = place(room.home.path(), new);
        let claude = client("claude-code");
        let said = told(&claude, &here, None).unwrap().hooks.unwrap().stray;
        assert_eq!(said.len(), 1, "{was}");
        let ask = Mend {
            hooks: vec![said[0].said.clone()],
            ..Mend::default()
        };

        let (_, failed) = mend_at(&claude, &here, &room.at("claude-code"), &ask);

        assert!(failed.is_none(), "{failed:?}");
        assert_eq!(room.read(SETTINGS), line(now));
    }
    let named = |said: &str| {
        format!(
            "{{ \"hooks\": {{ \"SessionStart\": [ {{ \"hooks\": [ {{ \"type\": \"command\", \"shell\": \"powershell\", \"command\": \"{said}\" }} ] }} ] }} }}"
        )
    };
    let room = Room::with(
        SETTINGS,
        &named("& 'C:\\\\NoSuch\\\\tisty.exe' hook claude-code start"),
    );
    let (_, failed) = room.mend("claude-code", hooks());
    assert!(failed.is_none(), "{failed:?}");
    assert_eq!(
        room.read(SETTINGS),
        named("& 'C:\\\\New\\\\tisty.exe' hook claude-code start")
    );
    let powershell = r#"{ "hooks": { "SessionStart": [ { "hooks": [ { "type": "command", "command": "& 'C:\\NoSuch\\tisty.exe' hook claude-code start" } ] } ] } }"#;
    let room = Room::with(SETTINGS, powershell);
    let seen = room.row("claude-code").hooks.unwrap();
    assert!(
        seen.apart && seen.stray.is_empty(),
        "bash runs no such line"
    );
    let (_, failed) = room.mend("claude-code", hooks());
    assert!(matches!(failed, Some(Stuck::Moved(_))));
    assert_eq!(room.read(SETTINGS), powershell);
}

#[test]
fn one_agents_file_still_matters_while_an_app_that_runs_it_is_here() {
    let room = Room::with("x", "");
    let mut here = room.here();
    here.os = places::Os::Windows;
    here.local = Some(room.home.path().join("Local"));
    for (id, app) in [
        ("claude-code", "Local/AnthropicClaude/claude.exe"),
        ("claude-code", "Local/Programs/Microsoft VS Code/Code.exe"),
        ("claude-code", "Local/Programs/cursor/Cursor.exe"),
        ("codex", "Local/Programs/Microsoft VS Code/Code.exe"),
        ("codex", "Local/Programs/cursor/Cursor.exe"),
    ] {
        let _ = std::fs::remove_dir_all(room.home.path().join("Local"));
        assert!(!client(id).installed(&here), "{id}");
        room.put(app, b"x");
        assert!(client(id).installed(&here), "{id} through {app}");
    }
}

#[test]
fn only_a_record_that_names_tistys_plugin_is_asked_about_further() {
    let room = Room::with("x", "");
    let homes = || Some(agents::Homes::with(room.home.path(), &[]));
    let claude = client("claude-code");
    let records = ".claude/plugins/installed_plugins.json";
    let user = r#"{"version":2,"plugins":{"tisty@mcp":[{"scope":"user"}]}}"#;
    let project = r#"{"version":2,"plugins":{"tisty@mcp":[{"scope":"project"}]}}"#;
    let off = "\u{feff}{ \"enabledPlugins\": { \"tisty@mcp\": false } }";
    for (named, installed, settings, brings) in [
        ("neither", None, None, Some(false)),
        (
            "settings with a comment",
            None,
            Some("{ // x\n \"a\": 1 }"),
            Some(false),
        ),
        ("settings blank", None, Some(" \n"), Some(false)),
        (
            "settings with a mark",
            None,
            Some("\u{feff}{}"),
            Some(false),
        ),
        ("settings unreadable", None, Some("<<<<"), Some(false)),
        (
            "no such plugin",
            Some(r#"{"plugins":{}}"#),
            Some("<<<<"),
            Some(false),
        ),
        ("of one project", Some(project), Some("<<<<"), Some(false)),
        ("of the person", Some(user), Some("{}"), Some(true)),
        (
            "of the person, settings with a comment",
            Some(user),
            Some("{ // mine\n \"a\": 1 }"),
            None,
        ),
        ("of the person, no settings", Some(user), None, Some(true)),
        (
            "turned off, with a mark",
            Some(user),
            Some(off),
            Some(false),
        ),
        (
            "of the person, settings blank",
            Some(user),
            Some(" \n"),
            Some(true),
        ),
        (
            "of the person, settings unreadable",
            Some(user),
            Some("<<<<"),
            None,
        ),
        ("records unreadable", Some("{"), Some("{}"), None),
    ] {
        let _ = std::fs::remove_dir_all(room.home.path().join(".claude"));
        if let Some(text) = installed {
            room.put(records, text.as_bytes());
        }
        if let Some(text) = settings {
            room.put(SETTINGS, text.as_bytes());
        }
        let said = match unplugged(&claude, homes()) {
            Ok(()) => Some(false),
            Err(Stuck::Plugged) => Some(true),
            Err(Stuck::PluginUnread(_)) => None,
            Err(why) => panic!("{named}: {why:?}"),
        };
        assert_eq!(said, brings, "{named}");
        if let Some(brings) = brings {
            assert_eq!(
                homes().unwrap().plugged().contains("claude"),
                brings,
                "{named}"
            );
        }
    }
}

#[test]
fn hooks_the_plugin_no_longer_brings_are_not_taken_out() {
    let ours = ours();
    let text = format!("{{ \"hooks\": {{ \"SessionStart\": [ {{ \"hooks\": [ {ours} ] }} ] }} }}");
    let room = Room::with(SETTINGS, &text);
    let homes = || Some(agents::Homes::with(room.home.path(), &[]));
    let claude = client("claude-code");
    let ask = Mend {
        hooks_out: true,
        ..hooks()
    };
    let mend = || {
        mend_guarded(
            &claude,
            &room.here(),
            &room.at("claude-code"),
            &ask,
            homes(),
        )
    };

    let (_, failed) = mend();
    assert!(matches!(failed, Some(Stuck::Unplugged)), "{failed:?}");
    assert_eq!(room.read(SETTINGS), text);

    let user = r#"{"plugins":{"tisty@mcp":[{"scope":"user"}]}}"#;
    room.put(".claude/plugins/installed_plugins.json", user.as_bytes());
    let (_, failed) = mend();
    assert!(failed.is_none(), "{failed:?}");
    assert!(!room.read(SETTINGS).contains("NoSuch"));
}
