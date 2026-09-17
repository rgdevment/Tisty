use std::io::IsTerminal;

/// What gave the assistant away, for the log the person reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sign {
    Env(&'static str),
    Ancestor(String),
    Ide(String),
}

impl Sign {
    pub fn kind(&self) -> &'static str {
        match self {
            Sign::Env(_) => "env",
            Sign::Ancestor(_) => "ancestor",
            Sign::Ide(_) => "ide",
        }
    }

    pub fn name(&self) -> String {
        match self {
            Sign::Env(mark) => (*mark).to_string(),
            Sign::Ancestor(name) | Sign::Ide(name) => name.clone(),
        }
    }
}

const ENV_MARKS: &[&str] = &[
    "CLAUDECODE",
    "CLAUDE_CODE_ENTRYPOINT",
    "CODEX_SANDBOX",
    "CODEX_SANDBOX_NETWORK_DISABLED",
    "GEMINI_CLI",
    "CURSOR_AGENT",
    "AI_AGENT",
];

const ASSISTANTS: &[&str] = &[
    "claude",
    "codex",
    "gemini",
    "opencode",
    "aider",
    "goose",
    "amp",
    "copilot",
    "cursor-agent",
];

// An editor is an ancestor of the person's own terminal too: only a shell with no
// terminal at all is the editor's assistant rather than the person.
const IDES: &[&str] = &["cursor", "windsurf", "code", "antigravity", "zed"];

const RUNTIMES: &[&str] = &["node", "bun", "deno", "python", "python3"];

const ANCESTORS_AT_MOST: usize = 16;

/// A store the person did not choose is not the person's list: tests and `demo` run there.
/// Judged by where the data resolved to, not by how: `TISTY_DATA` aimed at the person's own
/// store is still the person's own store.
pub fn at_the_persons_store(paths: &tisty_core::Paths) -> bool {
    paths.the_persons_own()
}

pub fn assistant() -> Option<Sign> {
    let env: Vec<String> = std::env::vars_os()
        .filter(|(_, value)| !value.is_empty())
        .map(|(key, _)| key.to_string_lossy().into_owned())
        .collect();
    if let Some(mark) = marked(&env) {
        return Some(mark);
    }
    let at_a_terminal = std::io::stdin().is_terminal()
        || std::io::stdout().is_terminal()
        || std::io::stderr().is_terminal();
    read(&env, &ancestors(), at_a_terminal)
}

fn marked(env: &[String]) -> Option<Sign> {
    ENV_MARKS
        .iter()
        .find(|mark| env.iter().any(|key| key == *mark))
        .map(|mark| Sign::Env(mark))
}

fn read(env: &[String], ancestors: &[String], at_a_terminal: bool) -> Option<Sign> {
    if let Some(mark) = marked(env) {
        return Some(mark);
    }
    for name in ancestors {
        if ASSISTANTS.contains(&name.as_str()) {
            return Some(Sign::Ancestor(name.clone()));
        }
        if !at_a_terminal && IDES.iter().any(|ide| is_or_helps(name, ide)) {
            return Some(Sign::Ide(name.clone()));
        }
    }
    None
}

fn is_or_helps(name: &str, ide: &str) -> bool {
    name == ide
        || name
            .strip_prefix(ide)
            .is_some_and(|rest| rest.starts_with([' ', '-']))
}

/// What a script is called is the program: `gemini.js` under node is gemini.
const SCRIPT_ENDINGS: &[&str] = &[".exe", ".js", ".mjs", ".cjs", ".py"];

/// The program behind a command line: `node /usr/bin/claude` is claude, not node.
fn named(args: &[String]) -> String {
    let base = |arg: &String| {
        let name = std::path::Path::new(arg)
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        SCRIPT_ENDINGS
            .iter()
            .find_map(|ending| name.strip_suffix(ending))
            .unwrap_or(&name)
            .to_string()
    };
    let Some(first) = args.first() else {
        return String::new();
    };
    let program = base(first);
    if !RUNTIMES.contains(&program.as_str()) {
        return program;
    }
    args[1..]
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(base)
        .unwrap_or(program)
}

#[cfg(target_os = "linux")]
fn ancestors() -> Vec<String> {
    let mut out = Vec::new();
    let mut pid = std::process::id();
    for _ in 0..ANCESTORS_AT_MOST {
        let Some(ppid) = parent_of(pid) else { break };
        if ppid <= 1 {
            break;
        }
        let cmdline = std::fs::read(format!("/proc/{ppid}/cmdline")).unwrap_or_default();
        let args: Vec<String> = cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg).into_owned())
            .collect();
        out.push(named(&args));
        pid = ppid;
    }
    out
}

// The command name sits in parentheses and may hold spaces or parentheses of its own, so
// the fields are counted from the last closing one: state first, then the parent.
#[cfg(target_os = "linux")]
fn parent_of(pid: u32) -> Option<u32> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat.rsplit(')')
        .next()?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

// `comm=` is the executable alone, spaces and all — `/Applications/Visual Studio Code.app/…`
// split on whitespace would read as «visual» — and only a runtime needs its arguments read,
// one process at a time.
#[cfg(target_os = "macos")]
fn ancestors() -> Vec<String> {
    let ps = |args: &[&str]| {
        std::process::Command::new("ps")
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .ok()
            .filter(|done| done.status.success())
            .map(|done| String::from_utf8_lossy(&done.stdout).into_owned())
            .unwrap_or_default()
    };
    let table: std::collections::HashMap<u32, (u32, String)> = ps(&["-axo", "pid=,ppid=,comm="])
        .lines()
        .filter_map(|line| {
            let mut parts = line.trim_start().splitn(3, ' ');
            let pid: u32 = parts.next()?.trim().parse().ok()?;
            let ppid: u32 = parts.next()?.trim().parse().ok()?;
            let program = parts.next()?.trim().to_string();
            Some((pid, (ppid, program)))
        })
        .collect();
    let mut out = Vec::new();
    let mut pid = std::process::id();
    for _ in 0..ANCESTORS_AT_MOST {
        let Some((ppid, _)) = table.get(&pid) else {
            break;
        };
        if *ppid <= 1 {
            break;
        }
        let Some((_, program)) = table.get(ppid) else {
            break;
        };
        let mut args = vec![program.clone()];
        if RUNTIMES.contains(&named(&args).as_str()) {
            args = ps(&["-o", "command=", "-p", &ppid.to_string()])
                .split_whitespace()
                .map(ToString::to_string)
                .collect();
        }
        out.push(named(&args));
        pid = *ppid;
    }
    out
}

// One process at a time, not the whole table: reading every command line on the machine
// is what makes a process scan slow on Windows. A parent that started after its child is a
// pid Windows has handed out again since the real parent went away.
#[cfg(windows)]
fn ancestors() -> Vec<String> {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
    let mut system = System::new();
    let wants = ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always);
    let mut out = Vec::new();
    let mut pid = Pid::from_u32(std::process::id());
    for _ in 0..ANCESTORS_AT_MOST {
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), false, wants);
        let Some(child) = system.process(pid) else {
            break;
        };
        let born = child.start_time();
        let Some(ppid) = child.parent() else {
            break;
        };
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[ppid]), false, wants);
        let Some(above) = system.process(ppid).filter(|one| one.start_time() <= born) else {
            break;
        };
        let mut args: Vec<String> = above
            .cmd()
            .iter()
            .map(|one| one.to_string_lossy().into_owned())
            .collect();
        if args.is_empty() {
            args.push(above.name().to_string_lossy().into_owned());
        }
        out.push(named(&args));
        pid = ppid;
    }
    out
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn ancestors() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
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
}
