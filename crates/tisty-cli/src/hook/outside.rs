use std::path::{Component, Path, PathBuf};

const PROSE: [&str; 6] = ["md", "markdown", "txt", "rst", "adoc", "org"];
const BARE: [&str; 6] = ["handoff", "notes", "todo", "plan", "prompt", "brief"];
const READ_BY_PROGRAMS: [&str; 9] = [
    "agents.md",
    "claude.md",
    "gemini.md",
    "skill.md",
    "memory.md",
    "cmakelists.txt",
    "robots.txt",
    "llms.txt",
    "constraints.txt",
];
const STARTS_READ_BY_PROGRAMS: [&str; 4] = ["readme", "license", "changelog", "requirements"];
const SET_APART: [&str; 10] = [
    "tmp",
    "temp",
    "node_modules",
    "target",
    "vendor",
    "site-packages",
    "appdata",
    "programdata",
    "program files",
    "program files (x86)",
];
const SYSTEM: [&str; 6] = ["/etc", "/usr", "/var", "/opt", "/private", "/Library"];
const LEFT_BETWEEN_SESSIONS: &str = "side-session-notes";
const HELD_BY: [&str; 18] = [
    ".git",
    ".hg",
    ".svn",
    ".jj",
    ".obsidian",
    ".logseq",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "setup.py",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "composer.json",
    "Gemfile",
    "CMakeLists.txt",
    "Makefile",
];
const ABOVE_AT_MOST: usize = 40;
const HOMES: [&str; 6] = [
    "~",
    "$home",
    "${home}",
    "$env:userprofile",
    "${env:userprofile}",
    "%userprofile%",
];

pub struct Here {
    pub home: Option<PathBuf>,
    pub temp: PathBuf,
}

impl Here {
    pub fn now() -> Self {
        Self {
            home: std::env::home_dir(),
            temp: std::env::temp_dir(),
        }
    }

    pub fn outside(&self, said: &str, cwd: &str) -> Option<PathBuf> {
        let at = placed(said, cwd)?;
        if !is_prose(&at) || at.symlink_metadata().is_ok() {
            return None;
        }
        let folder = at.parent()?;
        let for_another_session = names(folder).any(|one| one == LEFT_BETWEEN_SESSIONS);
        let claimed = (!for_another_session && self.set_apart(folder)) || self.held(folder);
        (!claimed).then(|| folder.to_path_buf())
    }

    pub fn outside_in_shell(&self, said: &str, cwd: &str) -> Option<PathBuf> {
        let cwd = match cwd {
            "~" => self.home.as_ref()?.to_string_lossy().into_owned(),
            _ if cwd.starts_with("~/") || cwd.starts_with("~\\") => self.filled(cwd)?,
            _ => cwd.to_string(),
        };
        self.outside(&self.filled(said)?, &cwd)
    }

    fn filled(&self, said: &str) -> Option<String> {
        let under_home = HOMES.iter().find_map(|one| {
            let (start, rest) = said.split_at_checked(one.len())?;
            (start.eq_ignore_ascii_case(one) && rest.starts_with(['/', '\\'])).then_some(rest)
        });
        if under_home
            .unwrap_or(said)
            .contains(['$', '%', '`', '*', '?', '~'])
        {
            return None;
        }
        Some(match under_home {
            Some(rest) => format!("{}{rest}", self.home.as_ref()?.display()),
            None => said.to_string(),
        })
    }

    fn set_apart(&self, folder: &Path) -> bool {
        let library = |home: &PathBuf| folder.starts_with(home.join("Library"));
        folder.starts_with(&self.temp)
            || SYSTEM.iter().any(|one| folder.starts_with(one))
            || (cfg!(target_os = "macos") && self.home.as_ref().is_some_and(library))
            || names(folder).any(|one| one.starts_with('.') || SET_APART.contains(&one.as_str()))
    }

    // The person's own folder and the disk's root hold everything, so what they carry claims nothing.
    fn held(&self, folder: &Path) -> bool {
        folder
            .ancestors()
            .take(ABOVE_AT_MOST)
            .take_while(|one| !self.is_home(one) && one.parent().is_some())
            .any(|one| HELD_BY.iter().any(|name| one.join(name).exists()))
    }

    fn is_home(&self, folder: &Path) -> bool {
        self.home
            .as_deref()
            .is_some_and(|home| match cfg!(windows) {
                true => names(home).eq(names(folder)),
                false => home == folder,
            })
    }
}

fn names(folder: &Path) -> impl Iterator<Item = String> {
    folder.components().filter_map(|one| match one {
        Component::Normal(name) => Some(name.to_string_lossy().to_lowercase()),
        _ => None,
    })
}

fn is_prose(at: &Path) -> bool {
    let lower = |it: &std::ffi::OsStr| it.to_string_lossy().to_lowercase();
    let Some(name) = at.file_name().map(lower) else {
        return false;
    };
    let for_programs = READ_BY_PROGRAMS.contains(&name.as_str())
        || STARTS_READ_BY_PROGRAMS
            .iter()
            .any(|one| name.starts_with(one));
    let prose = match at.extension().map(lower) {
        Some(extension) => PROSE.contains(&extension.as_str()),
        None => BARE.contains(&name.as_str()),
    };
    prose && !for_programs
}

fn placed(said: &str, cwd: &str) -> Option<PathBuf> {
    let at = PathBuf::from(native(said)?);
    let at = match at.is_absolute() {
        true => at,
        false => {
            let from = PathBuf::from(native(cwd)?);
            from.is_absolute().then(|| from.join(at))?
        }
    };
    let mut plain = PathBuf::new();
    for one in at.components() {
        match one {
            Component::CurDir => {}
            Component::ParentDir => {
                plain.pop();
            }
            one => plain.push(one),
        }
    }
    Some(plain)
}

// A network path could hold the turn up, and one of another system cannot be told from here.
fn native(said: &str) -> Option<String> {
    if !cfg!(windows) {
        return Some(said.to_string());
    }
    let plain = said.replace('\\', "/");
    if plain.starts_with("//") {
        return None;
    }
    let Some(rooted) = plain.strip_prefix('/') else {
        return Some(plain);
    };
    let mut left = rooted.strip_prefix("mnt/").unwrap_or(rooted).chars();
    let disk = left.next().filter(char::is_ascii_alphabetic)?;
    let rest = left.as_str().strip_prefix(':').unwrap_or(left.as_str());
    rest.starts_with('/').then(|| format!("{disk}:{rest}"))
}

pub fn depends_on_where_it_runs(said: &str) -> bool {
    let homed = HOMES.iter().any(|one| {
        said.split_at_checked(one.len())
            .is_some_and(|(start, _)| start.eq_ignore_ascii_case(one))
    });
    !homed && native(said).is_some_and(|it| !Path::new(&it).is_absolute())
}

#[cfg(test)]
#[path = "outside_test.rs"]
mod tests;
