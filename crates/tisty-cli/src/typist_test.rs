use super::*;

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

#[test]
fn a_script_under_a_runtime_is_called_by_its_name_without_the_ending() {
    assert_eq!(
        named(&strings(&["node", "/opt/gemini/bundle/gemini.js"])),
        "gemini"
    );
    assert_eq!(
        named(&strings(&[
            "node",
            "--max-old-space-size=4096",
            "/x/bin/codex.js"
        ])),
        "codex"
    );
    assert_eq!(
        named(&strings(&["python3", "/usr/local/bin/aider.py"])),
        "aider"
    );
    assert_eq!(named(&strings(&["/usr/local/bin/claude.exe"])), "claude");
    assert_eq!(named(&strings(&["node"])), "node");
}

#[test]
fn a_marker_in_the_environment_is_enough_on_its_own() {
    let seen = read(&strings(&["HOME", "CLAUDECODE"]), &[], true);

    assert_eq!(seen, Some(Sign::Env("CLAUDECODE")));
    assert_eq!(
        marked(&strings(&["HOME", "CLAUDECODE"])),
        Some(Sign::Env("CLAUDECODE"))
    );
    assert_eq!(marked(&strings(&["HOME", "PATH"])), None);
}

#[test]
fn an_assistant_up_the_tree_is_one_even_at_a_terminal() {
    let seen = read(
        &strings(&["HOME"]),
        &strings(&["zsh", "claude", "zsh"]),
        true,
    );

    assert_eq!(seen, Some(Sign::Ancestor("claude".into())));
}

#[test]
fn an_editor_up_the_tree_is_the_person_while_there_is_a_terminal() {
    let tree = strings(&["zsh", "cursor helper (plugin)", "cursor"]);

    assert_eq!(read(&strings(&["HOME"]), &tree, true), None);
    assert_eq!(
        read(&strings(&["HOME"]), &tree, false),
        Some(Sign::Ide("cursor helper (plugin)".into()))
    );
}

#[test]
fn a_name_that_merely_starts_like_an_editor_is_not_one() {
    let tree = strings(&["codeium-server", "xcode", "decode"]);

    assert_eq!(read(&strings(&["HOME"]), &tree, false), None);
    assert_eq!(
        read(&strings(&["HOME"]), &strings(&["code-insiders"]), false),
        Some(Sign::Ide("code-insiders".into()))
    );
}

#[test]
fn a_plain_shell_under_a_terminal_is_the_person() {
    let seen = read(
        &strings(&["HOME", "PATH"]),
        &strings(&["zsh", "tmux: server", "systemd"]),
        true,
    );

    assert_eq!(seen, None);
}

#[test]
fn a_script_with_no_terminal_and_no_assistant_is_still_the_person() {
    let seen = read(
        &strings(&["HOME"]),
        &strings(&["bash", "cron", "systemd"]),
        false,
    );

    assert_eq!(seen, None);
}

#[test]
fn the_program_behind_a_runtime_is_what_counts() {
    assert_eq!(
        named(&strings(&["node", "--no-warnings", "/usr/lib/claude"])),
        "claude"
    );
    assert_eq!(
        named(&strings(&["/usr/bin/python3", "-m", "aider"])),
        "aider"
    );
    assert_eq!(named(&strings(&["node"])), "node");
    assert_eq!(named(&[]), "");
}
