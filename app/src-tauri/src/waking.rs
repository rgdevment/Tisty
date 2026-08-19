use tisty_core::witness::{self, Fact, channel};

pub const HUSHED: &str = "--hushed";

pub fn hushed() -> bool {
    std::env::args().any(|one| one == HUSHED)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Waking {
    pub offered: bool,
    pub wakes: bool,
    pub theirs: bool,
}

impl Waking {
    fn none() -> Self {
        Self {
            offered: false,
            wakes: false,
            theirs: false,
        }
    }
}

pub fn waking() -> Waking {
    there::waking()
}

pub fn wake(wanted: bool) -> std::io::Result<Waking> {
    there::wake(wanted).inspect_err(|why| {
        witness::error(
            channel::WINDOW,
            "the machine was not told whether to open Tisty",
            &[("why", Fact::Why(why.to_string()))],
        );
    })?;
    Ok(there::waking())
}

#[cfg(target_os = "macos")]
mod there {
    use super::{HUSHED, Waking};
    use std::path::{Path, PathBuf};

    const LABEL: &str = "dev.rgdevment.tisty";

    pub fn waking() -> Waking {
        let Some(app) = bundle() else {
            return Waking::none();
        };
        let wakes = plist()
            .and_then(|at| std::fs::read_to_string(at).ok())
            .is_some_and(|text| names(&text, &app));
        Waking {
            offered: true,
            wakes,
            theirs: false,
        }
    }

    pub fn wake(wanted: bool) -> std::io::Result<()> {
        let (Some(app), Some(at)) = (bundle(), plist()) else {
            return Ok(());
        };
        wake_at(&app, &at, wanted)
    }

    pub fn wake_at(app: &Path, at: &Path, wanted: bool) -> std::io::Result<()> {
        if wanted {
            if let Some(folder) = at.parent() {
                std::fs::create_dir_all(folder)?;
            }
            return std::fs::write(at, written(app));
        }
        match std::fs::remove_file(at) {
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn bundle() -> Option<PathBuf> {
        within(&std::env::current_exe().ok()?)
    }

    pub fn within(exe: &Path) -> Option<PathBuf> {
        exe.ancestors()
            .find(|at| at.extension().is_some_and(|kind| kind == "app"))
            .map(Path::to_path_buf)
    }

    fn plist() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("LaunchAgents")
                .join(format!("{LABEL}.plist"))
        })
    }

    pub fn names(text: &str, app: &Path) -> bool {
        let at = escaped(&app.display().to_string());
        text.contains(&format!("<string>{at}</string>"))
    }

    pub fn written(app: &Path) -> String {
        let at = escaped(&app.display().to_string());
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/open</string>
        <string>-a</string>
        <string>{at}</string>
        <string>--args</string>
        <string>{HUSHED}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>LimitLoadToSessionType</key>
    <string>Aqua</string>
</dict>
</plist>
"#
        )
    }

    fn escaped(one: &str) -> String {
        one.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

#[cfg(windows)]
mod there {
    use super::{HUSHED, Waking};
    use std::path::Path;
    use windows::ApplicationModel::{StartupTask, StartupTaskState};
    use windows::core::HSTRING;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const NAME: &str = "Tisty";
    const TASK: &str = "Tisty";

    pub fn waking() -> Waking {
        if packaged() { tasked() } else { shelved() }
    }

    pub fn wake(wanted: bool) -> std::io::Result<()> {
        if packaged() {
            return task(wanted);
        }
        shelve(wanted)
    }

    fn packaged() -> bool {
        crate::update::route().route == crate::update::Route::Store
    }

    fn tasked() -> Waking {
        let Some(state) = asked().and_then(|task| task.State().ok()) else {
            return Waking::none();
        };
        Waking {
            offered: true,
            wakes: state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy,
            theirs: state != StartupTaskState::Disabled,
        }
    }

    fn task(wanted: bool) -> std::io::Result<()> {
        let Some(task) = asked() else {
            return Ok(());
        };
        if wanted {
            task.RequestEnableAsync()
                .and_then(|asking| asking.get())
                .map(|_| ())
                .map_err(sour)
        } else {
            task.Disable().map_err(sour)
        }
    }

    fn asked() -> Option<StartupTask> {
        StartupTask::GetAsync(&HSTRING::from(TASK)).ok()?.get().ok()
    }

    fn sour(why: windows::core::Error) -> std::io::Error {
        std::io::Error::other(why.message())
    }

    fn shelved() -> Waking {
        let Ok(exe) = std::env::current_exe() else {
            return Waking::none();
        };
        let wakes = value().is_some_and(|one| ours(&one, &exe));
        Waking {
            offered: true,
            wakes,
            theirs: wakes && !approved(),
        }
    }

    fn shelve(wanted: bool) -> std::io::Result<()> {
        let exe = std::env::current_exe()?;
        let key =
            winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN, KEY_WRITE)?;
        if wanted {
            return key.set_value(NAME, &format!("\"{}\" {HUSHED}", exe.display()));
        }
        match key.delete_value(NAME) {
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn value() -> Option<String> {
        winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(RUN)
            .ok()?
            .get_value::<String, _>(NAME)
            .ok()
    }

    fn approved() -> bool {
        let read = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(APPROVED)
            .ok()
            .and_then(|key| key.get_raw_value(NAME).ok());
        read.is_none_or(|held| approves(&held.bytes))
    }

    pub fn approves(bytes: &[u8]) -> bool {
        !matches!(bytes.first(), Some(2 | 3))
    }

    pub fn ours(value: &str, exe: &Path) -> bool {
        let said = value.trim();
        let named = exe.display().to_string();
        if let Some(rest) = said.strip_prefix('"') {
            let quoted = rest.split('"').next().unwrap_or_default();
            return !quoted.is_empty() && quoted.eq_ignore_ascii_case(&named);
        }
        if said.eq_ignore_ascii_case(&named) {
            return true;
        }
        let first = said.split_whitespace().next().unwrap_or_default();
        !first.is_empty() && first.eq_ignore_ascii_case(&named)
    }
}

#[cfg(all(not(windows), not(target_os = "macos")))]
mod there {
    use super::Waking;

    pub fn waking() -> Waking {
        Waking::none()
    }

    pub fn wake(_wanted: bool) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::there::{names, wake_at, within, written};
    use std::path::{Path, PathBuf};

    fn app(at: &str) -> PathBuf {
        PathBuf::from(at)
    }

    #[test]
    fn the_bundle_is_the_folder_the_binary_sits_inside() {
        assert_eq!(
            within(Path::new("/Applications/Tisty.app/Contents/MacOS/Tisty")),
            Some(app("/Applications/Tisty.app"))
        );
    }

    #[test]
    fn a_binary_that_lives_in_no_bundle_offers_nothing() {
        assert!(within(Path::new("/Users/someone/code/target/debug/tisty-gui")).is_none());
    }

    #[test]
    fn the_plist_opens_the_bundle_that_wrote_it_and_keeps_it_quiet() {
        let text = written(&app("/Applications/Tisty.app"));

        assert!(text.contains("<string>/Applications/Tisty.app</string>"));
        assert!(text.contains("<string>--hushed</string>"));
        assert!(text.contains("<key>RunAtLoad</key>"));
    }

    #[test]
    fn a_plist_that_opens_a_copy_somewhere_else_is_not_this_one() {
        let text = written(&app("/Users/someone/Applications/Tisty.app"));

        assert!(names(&text, &app("/Users/someone/Applications/Tisty.app")));
        assert!(!names(&text, &app("/Applications/Tisty.app")));
    }

    #[test]
    fn the_agent_is_written_where_launchd_reads_and_taken_back_out() {
        let home = tempfile::tempdir().unwrap();
        let at = home
            .path()
            .join("Library/LaunchAgents/dev.rgdevment.tisty.plist");
        let bundle = app("/Applications/Tisty.app");

        wake_at(&bundle, &at, true).unwrap();
        assert!(names(&std::fs::read_to_string(&at).unwrap(), &bundle));

        wake_at(&bundle, &at, false).unwrap();
        assert!(!at.exists());
    }

    #[test]
    fn taking_out_an_agent_that_was_never_written_is_not_a_failure() {
        let home = tempfile::tempdir().unwrap();
        let at = home
            .path()
            .join("Library/LaunchAgents/dev.rgdevment.tisty.plist");

        assert!(wake_at(&app("/Applications/Tisty.app"), &at, false).is_ok());
    }

    #[test]
    fn a_folder_named_with_an_ampersand_survives_the_plist() {
        let at = app("/Applications/Work & Play/Tisty.app");
        let text = written(&at);

        assert!(text.contains("Work &amp; Play"));
        assert!(names(&text, &at));
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::there::{approves, ours};
    use std::path::Path;

    const EXE: &str = r"C:\Program Files\Tisty\Tisty.exe";

    #[test]
    fn the_quoted_program_is_read_apart_from_its_arguments() {
        assert!(ours(&format!("\"{EXE}\" --hushed"), Path::new(EXE)));
    }

    #[test]
    fn a_program_written_without_quotes_keeps_the_spaces_in_its_folder() {
        assert!(ours(EXE, Path::new(EXE)));
    }

    #[test]
    fn a_program_without_quotes_is_read_apart_from_its_arguments() {
        let at = r"C:\Tisty\Tisty.exe";
        assert!(ours(&format!("{at} --hushed"), Path::new(at)));
    }

    #[test]
    fn windows_does_not_mind_the_case_of_the_program() {
        assert!(ours(&format!("\"{}\"", EXE.to_lowercase()), Path::new(EXE)));
    }

    #[test]
    fn an_entry_left_by_another_copy_is_not_this_one() {
        assert!(!ours(
            r#""C:\Users\Someone\Tisty\Tisty.exe" --hushed"#,
            Path::new(EXE)
        ));
    }

    #[test]
    fn an_empty_entry_claims_nothing() {
        assert!(!ours("   ", Path::new(EXE)));
    }

    #[test]
    fn the_task_manager_says_disabled_with_a_two_or_a_three() {
        assert!(!approves(&[2, 0, 0, 0]));
        assert!(!approves(&[3, 0, 0, 0]));
        assert!(approves(&[0, 0, 0, 0]));
        assert!(approves(&[6, 0, 0, 0]));
        assert!(approves(&[]));
    }
}
