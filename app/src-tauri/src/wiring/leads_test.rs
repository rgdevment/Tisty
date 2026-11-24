use super::*;

const MINE: &str = "C:\\New\\tisty.exe";

fn place(os: Os, command: &str) -> Place {
    Place {
        distro: None,
        os,
        home: PathBuf::from("/home/someone"),
        config: PathBuf::from("/home/someone/.config"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: command.to_string(),
        open: true,
        packaged: false,
        mount: String::new(),
    }
}

#[test]
fn what_nothing_here_can_tell_is_never_called_broken() {
    let windows = place(Os::Windows, MINE);
    for said in [
        "${userHome}\\AppData\\Local\\Tisty\\tisty.exe",
        "%LOCALAPPDATA%\\Tisty\\tisty.exe",
        "~/.local/bin/tisty",
        "$HOME/.local/bin/tisty",
        "C:\\Windows\\System32\\wsl.exe",
        "C:\\Windows\\System32\\cmd.exe",
        "C:\\Tools\\my-tisty.exe",
        "\\\\no-such-host-zz\\share\\tisty.exe",
        "/c/New/tisty.exe",
        "tisty",
        "bin/tisty",
        "",
    ] {
        assert_eq!(leads(&windows, said), Leads::Unsaid, "{said}");
    }
    let unix = place(Os::Linux, "/opt/new/tisty");
    for said in [
        "$HOME/.local/bin/tisty",
        "~/.local/bin/tisty",
        "/usr/bin/nice",
        "/Volumes/Stick/Tisty.app/Contents/MacOS/tisty",
        "/media/someone/stick/tisty",
        "/mnt/stick/tisty",
        "//server/share/tisty",
        "tisty",
    ] {
        assert_eq!(leads(&unix, said), Leads::Unsaid, "{said}");
    }
}

#[test]
fn a_network_path_is_never_waited_for() {
    let windows = place(Os::Windows, MINE);
    let began = std::time::Instant::now();

    leads(&windows, "\\\\no-such-host-zz\\share\\tisty.exe");

    assert!(began.elapsed().as_millis() < 200);
}

#[cfg(windows)]
#[test]
fn a_drive_that_is_not_there_today_says_nothing_of_what_it_holds() {
    let missing = ('D'..='Z')
        .rev()
        .find(|one| !Path::new(&format!("{one}:\\")).exists())
        .unwrap();

    let said = format!("{missing}:\\Tisty\\tisty.exe");

    assert_eq!(leads(&place(Os::Windows, MINE), &said), Leads::Unsaid);
}

#[test]
fn another_tisty_is_told_apart_from_one_that_is_gone() {
    let tmp = tempfile::tempdir().unwrap();
    let os = match cfg!(windows) {
        true => Os::Windows,
        false => Os::Linux,
    };
    let other = tmp.path().join("with space").join("tisty.exe");
    std::fs::create_dir_all(other.parent().unwrap()).unwrap();
    std::fs::write(&other, b"").unwrap();
    let here = place(os, MINE);
    let said = other.display().to_string();

    assert_eq!(leads(&here, &said), Leads::Another);
    assert!(leads(&here, &said).astray());
    std::fs::remove_file(&other).unwrap();
    assert_eq!(leads(&here, &said), Leads::Gone);
    assert_eq!(leads(&here, &said.replace('\\', "/")), Leads::Gone);
    assert_eq!(leads(&place(os, &said.to_uppercase()), &said), Leads::Here);
}

#[test]
fn inside_a_distro_what_it_mounts_of_windows_is_looked_for_on_its_drive() {
    let distro = Place {
        distro: Some("Ubuntu".into()),
        mount: "/mnt/".into(),
        ..place(Os::Linux, "/mnt/c/New/tisty.exe")
    };

    assert_eq!(leads(&distro, "/mnt/c/New/tisty.exe"), Leads::Here);
    assert_eq!(leads(&distro, "C:\\Old\\tisty.exe"), Leads::Unsaid);
    #[cfg(windows)]
    assert_eq!(
        leads(&distro, "/mnt/c/no-such-folder/tisty.exe"),
        Leads::Gone
    );
}
