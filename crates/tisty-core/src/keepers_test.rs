use super::*;

fn offering(key: &'static str, at: &str) -> Offer {
    Offer {
        key,
        named: "",
        at: Some(PathBuf::from(at)),
    }
}

#[test]
fn a_folder_inside_a_provider_belongs_to_it() {
    let offers = vec![offering(DROPBOX, "/home/mario/Dropbox")];
    assert_eq!(
        whose(Path::new("/home/mario/Dropbox/tasks"), &offers),
        Keeper::Cloud(DROPBOX)
    );
}

#[test]
fn the_provider_folder_itself_belongs_to_it() {
    let offers = vec![offering(DROPBOX, "/home/mario/Dropbox")];
    assert_eq!(
        whose(Path::new("/home/mario/Dropbox"), &offers),
        Keeper::Cloud(DROPBOX)
    );
}

#[test]
fn a_name_that_only_starts_the_same_is_somebody_else() {
    let offers = vec![offering(DROPBOX, "/home/mario/Dropbox")];
    assert_eq!(
        whose(Path::new("/home/mario/Dropbox-old"), &offers),
        Keeper::Plain
    );
}

#[test]
fn the_closest_provider_wins_when_one_sits_inside_another() {
    let offers = vec![
        offering(DROPBOX, "/home/mario/Dropbox"),
        offering(DRIVE, "/home/mario/Dropbox/Drive"),
    ];
    assert_eq!(
        whose(Path::new("/home/mario/Dropbox/Drive/tasks"), &offers),
        Keeper::Cloud(DRIVE)
    );
}

#[test]
fn a_provider_we_could_not_find_never_claims_a_folder() {
    let offers = vec![Offer {
        key: DRIVE,
        named: "",
        at: None,
    }];
    assert_eq!(
        whose(Path::new("/home/mario/tasks"), &offers),
        Keeper::Plain
    );
}

#[test]
fn our_own_folder_hangs_from_the_one_that_was_chosen() {
    assert_eq!(
        suggested(Path::new("/home/mario/Dropbox")),
        Path::new("/home/mario/Dropbox").join(OURS)
    );
}

#[test]
fn the_shared_root_is_what_we_take_from_a_drive() {
    let room = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(room.path().join("My Drive")).unwrap();
    std::fs::create_dir_all(room.path().join("Other computers").join("salvia 07")).unwrap();

    assert_eq!(mine(room.path()), Some(room.path().join("My Drive")));
}

#[test]
fn a_drive_holding_only_machine_backups_gives_us_nowhere_both_could_meet() {
    let room = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(room.path().join("Other computers").join("salvia 07")).unwrap();

    assert_eq!(mine(room.path()), None);
}

#[cfg(not(windows))]
#[test]
fn the_drive_we_offer_is_the_one_the_other_machine_will_also_reach() {
    let room = tempfile::tempdir().unwrap();
    let drive = room
        .path()
        .join("Library/CloudStorage/GoogleDrive-mario@example.com");
    std::fs::create_dir_all(drive.join("My Drive")).unwrap();

    assert_eq!(found(DRIVE, room.path()), Some(drive.join("My Drive")));
}

#[cfg(windows)]
#[test]
fn a_share_is_away_and_a_local_disk_is_not() {
    assert!(away(Path::new(r"\\nas\tasks")));
    assert!(!away(Path::new(r"D:\tasks")));
}

#[cfg(windows)]
#[test]
fn windows_does_not_read_case_and_neither_do_we() {
    let offers = vec![offering(ONEDRIVE, r"C:\Users\Mario\OneDrive")];
    assert_eq!(
        whose(Path::new(r"c:\users\mario\onedrive\tasks"), &offers),
        Keeper::Cloud(ONEDRIVE)
    );
}

#[cfg(target_os = "macos")]
#[test]
fn a_mounted_volume_is_away_and_a_home_folder_is_not() {
    assert!(away(Path::new("/Volumes/nas/tasks")));
    assert!(!away(Path::new("/Users/mario/tasks")));
}
