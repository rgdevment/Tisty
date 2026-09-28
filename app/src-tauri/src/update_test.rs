use super::*;

const FEED: &str = r#"{"schema":1,"latest":"0.3.0","latestPrerelease":"0.4.0-rc1"}"#;

#[test]
fn a_copy_under_rosetta_asks_for_the_native_build_and_no_other_copy_chooses() {
    assert_eq!(platform(true), Some("darwin-aarch64"));
    assert_eq!(
        platform(false),
        None,
        "the plugin's own architecture stands"
    );
}

#[test]
fn a_stable_copy_is_never_pointed_at_a_candidate() {
    let found = newer("0.2.0", FEED, Kept::plain(Route::Download), None).expect("0.3.0 is newer");

    assert_eq!(found.version, "0.3.0");
}

/// The one a candidate is waiting for: the release it was a candidate of. Nothing else takes
/// it off the track it is on, and a comparison that read the two as the same version would
/// strand it there on the very day it was meant to be replaced.
#[test]
fn a_candidate_is_replaced_by_the_release_it_was_a_candidate_of() {
    let feed = r#"{"schema":1,"latest":"1.5.0"}"#;
    let kept = Kept::plain(Route::Download);

    assert_eq!(
        newer("1.5.0-rc1", feed, kept, None).unwrap().version,
        "1.5.0"
    );
    assert_eq!(
        newer("1.5.0-rc9", feed, kept, Some(false)).unwrap().version,
        "1.5.0",
        "and it arrives whether or not the candidates were asked to stop"
    );
    assert!(
        newer("1.5.0", feed, kept, None).is_none(),
        "and the release itself is not offered to itself"
    );
}

/// The answer can be written down while somebody is turning the candidates off, and what was
/// true when the question went out is not what decides whether it may be shown.
#[test]
fn a_candidate_written_down_before_the_answer_changed_is_not_offered_after_it() {
    let kept = Kept::plain(Route::Download);

    assert_eq!(
        remembered("1.12.0", Some("1.13.0-rc1"), kept, Some(true))
            .unwrap()
            .version,
        "1.13.0-rc1"
    );
    assert!(
        remembered("1.12.0", Some("1.13.0-rc1"), kept, Some(false)).is_none(),
        "a copy that asked to leave the candidates is not handed one it was promised earlier"
    );
    assert_eq!(
        remembered("1.12.0", Some("1.13.0"), kept, Some(false))
            .unwrap()
            .version,
        "1.13.0",
        "and a stable one it was promised still arrives"
    );
}

#[test]
fn nobody_having_said_is_not_somebody_having_said_no() {
    assert!(
        tracking("0.4.0-rc1", None),
        "a candidate installed by hand goes on being offered candidates"
    );
    assert!(!tracking("0.3.0", None), "and a stable copy is left alone");

    assert!(
        tracking("0.3.0", Some(true)),
        "asking for them is the way in"
    );
    assert!(
        !tracking("0.4.0-rc1", Some(false)),
        "and saying no is the way back out, which the next stable release completes"
    );
    assert!(
        !tracking("tomorrow", None),
        "an unreadable version is stable"
    );
}

#[test]
fn a_copy_that_asked_to_leave_the_candidates_is_offered_the_stable_one() {
    let feed = r#"{"schema":1,"latest":"0.3.0","latestPrerelease":"0.4.0-rc2"}"#;
    let kept = Kept::plain(Route::Download);

    assert_eq!(
        newer("0.4.0-rc1", feed, kept, None).unwrap().version,
        "0.4.0-rc2",
        "still on the track it was installed on"
    );
    assert_eq!(
        newer("0.2.0", feed, kept, Some(false)).unwrap().version,
        "0.3.0",
        "and off it, only the stable one is offered"
    );
    assert!(
        newer("0.4.0-rc1", feed, kept, Some(false)).is_none(),
        "a candidate that asked to leave waits for a stable release that passes it"
    );
}

#[test]
fn a_stable_copy_takes_the_candidates_track_only_by_asking_for_it() {
    let found = newer("0.3.0", FEED, Kept::plain(Route::Download), Some(true))
        .expect("asked for the candidates");

    assert_eq!(found.version, "0.4.0-rc1");
    assert!(
        newer("0.3.0", FEED, Kept::plain(Route::Download), None).is_none(),
        "and without asking, the same manifest says nothing"
    );
}

#[test]
fn a_manifest_cannot_walk_a_stable_copy_onto_the_candidates_track() {
    let feed = r#"{"latest":"9.9.9-rc1"}"#;

    assert!(newer("0.3.0", feed, Kept::plain(Route::Download), None).is_none());
}

#[test]
fn a_candidate_is_offered_the_newest_of_either() {
    assert_eq!(
        newer("0.3.0-rc1", FEED, Kept::plain(Route::Download), None)
            .unwrap()
            .version,
        "0.4.0-rc1"
    );
}

#[test]
fn a_candidate_takes_the_stable_one_when_it_is_ahead() {
    let feed = r#"{"latest":"0.5.0","latestPrerelease":"0.4.0-rc1"}"#;

    assert_eq!(
        newer("0.4.0-rc1", feed, Kept::plain(Route::Download), None)
            .unwrap()
            .version,
        "0.5.0"
    );
}

#[test]
fn the_same_version_is_not_an_update() {
    assert!(newer("0.3.0", FEED, Kept::plain(Route::Download), None).is_none());
    assert!(newer("0.4.0-rc1", FEED, Kept::plain(Route::Download), None).is_none());
}

#[test]
fn a_manifest_that_makes_no_sense_says_nothing() {
    assert!(newer("0.1.0", "not json", Kept::plain(Route::Download), None).is_none());
    assert!(
        newer(
            "0.1.0",
            r#"{"latest":"tomorrow"}"#,
            Kept::plain(Route::Download),
            None
        )
        .is_none()
    );
}

#[test]
fn a_manifest_without_a_candidate_still_reads() {
    let feed = r#"{"latest":"0.3.0"}"#;

    assert_eq!(
        newer("0.2.0-rc1", feed, Kept::plain(Route::Download), None)
            .unwrap()
            .version,
        "0.3.0"
    );
}

#[test]
fn a_project_with_no_stable_release_yet_still_works() {
    let feed = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"0.2.0-rc6"}"#;

    assert_eq!(
        newer("0.2.0-rc5", feed, Kept::plain(Route::Download), None)
            .unwrap()
            .version,
        "0.2.0-rc6"
    );
    assert!(newer("0.1.0", feed, Kept::plain(Route::Download), None).is_none());
}

#[test]
fn only_a_copy_that_owns_its_own_folder_replaces_itself() {
    assert!(self_installs(Route::Download));
    assert!(self_installs(Route::Brew));
    assert!(!self_installs(Route::Store), "the store keeps its own");
    assert!(!self_installs(Route::BrewCli), "brew keeps the formula");
}

#[test]
fn an_installer_is_only_ever_taken_from_where_our_releases_live() {
    assert!(ours(
        "https://github.com/rgdevment/Tisty/releases/download/v1/tisty.exe"
    ));
    assert!(ours("https://objects.githubusercontent.com/whatever"));

    assert!(
        !ours("http://github.com/rgdevment/Tisty/x.exe"),
        "plain http"
    );
    assert!(
        !ours("https://github.com.example.invalid/x.exe"),
        "a lookalike host"
    );
    assert!(
        !ours("https://raw.githubusercontent.com/x.exe"),
        "not where releases live"
    );
    assert!(!ours("file:///C:/x.exe"));
    assert!(!ours("nonsense"));
}

#[test]
fn a_candidate_asks_the_candidates_feed_and_a_stable_one_the_stable_feed() {
    assert_eq!(feeds_for("0.3.0"), vec![LATEST]);
    assert_eq!(
        feeds_for("tomorrow"),
        vec![LATEST],
        "unreadable means stable"
    );
}

/// The release that retires a candidate deletes its channel. A copy still holding that offer
/// asks for a file that is no longer served, and a refusal from the network is not something
/// anybody can act on.
#[test]
fn a_candidate_that_was_retired_is_told_it_is_gone_rather_than_met_with_a_refusal() {
    assert_eq!(feeds_for("0.4.0-rc1"), vec![CANDIDATE, LATEST]);
}

#[test]
fn the_feed_follows_what_is_offered_rather_than_what_is_running() {
    let feed = r#"{"latest":"0.5.0","latestPrerelease":"0.4.0-rc1"}"#;
    let found =
        newer("0.4.0-rc1", feed, Kept::plain(Route::Download), None).expect("0.5.0 is newer");

    assert_eq!(
        feeds_for(&found.version),
        vec![LATEST],
        "a candidate sent to a stable release must be pointed at the stable feed"
    );
}

#[test]
fn what_was_found_is_still_offered_after_the_window_closed() {
    let kept = Kept::plain(Route::Download);

    assert_eq!(
        remembered("0.2.0", Some("0.3.0"), kept, None)
            .unwrap()
            .version,
        "0.3.0"
    );
    assert!(remembered("0.3.0", Some("0.3.0"), kept, None).is_none());
    assert!(
        remembered("0.4.0", Some("0.3.0"), kept, None).is_none(),
        "a copy updated by hand is not owed the old offer"
    );
    assert!(remembered("0.2.0", None, kept, None).is_none());
    assert!(remembered("0.2.0", Some("tomorrow"), kept, None).is_none());
}

/// A release the Store is still certifying is out for everyone else; told of it, a copy the
/// Store keeps has a button that does nothing. Only the Store speaks for what it sells.
#[test]
fn the_manifest_says_nothing_to_a_copy_the_store_keeps() {
    assert!(
        remembered(
            "1.13.0",
            Some("1.13.3"),
            Kept::plain(Route::Store),
            Some(false)
        )
        .is_none()
    );
    let feed = r#"{"schema":1,"latest":"1.13.3"}"#;
    assert!(newer("1.13.0", feed, Kept::plain(Route::Store), Some(false)).is_none());
}

#[test]
fn an_offer_the_store_itself_made_is_one_this_copy_can_take() {
    let offer = from_the_shop("0.3.0", "0.2.0").expect("0.3.0 is newer");

    assert_eq!(offer.route, Route::Store);
    assert!(offer.installs);
    assert!(
        !self_installs(Route::Store),
        "and the manifest still cannot offer one on the store's behalf"
    );
}

#[test]
fn the_store_is_held_to_the_same_rule_as_the_manifest() {
    assert!(
        from_the_shop("0.3.0", "0.3.0").is_none(),
        "the version already running is not an update"
    );
    assert!(
        from_the_shop("0.2.0", "0.3.0").is_none(),
        "nor is one behind it"
    );
    assert!(from_the_shop("tomorrow", "0.3.0").is_none());
    assert!(from_the_shop("0.3.0", "tomorrow").is_none());
}

/// The offer and the refusal read the same thing, or the button is shown to somebody it will
/// then be taken from, with a message saying somebody else looks after this copy — when the
/// truth is that nobody does.
#[test]
#[cfg(target_os = "macos")]
fn a_copy_that_cannot_replace_itself_is_not_offered_a_button() {
    let feed = r#"{"latest":"0.3.0"}"#;
    let offer = newer("0.2.0", feed, Kept::plain(Route::Download), None).unwrap();

    assert_eq!(
        offer.installs,
        !from_a_mount(),
        "what the offer promises is what the install will allow"
    );
}

#[test]
fn candidates_are_only_asked_for_where_one_could_arrive() {
    assert!(takes_candidates(Route::Download));
    assert!(takes_candidates(Route::Brew));
    assert!(
        !takes_candidates(Route::Store),
        "the store is sent finished versions only"
    );
    assert!(
        !takes_candidates(Route::BrewCli),
        "and the formula keeps its candidates under another name"
    );
}

#[test]
fn an_offer_says_whether_this_copy_can_take_it() {
    let feed = r#"{"latest":"0.3.0"}"#;

    assert!(
        newer("0.2.0", feed, Kept::plain(Route::Download), None)
            .unwrap()
            .installs
    );
    assert!(
        newer("0.2.0", feed, Kept::plain(Route::Store), None).is_none(),
        "the manifest makes no offer on the Store's behalf"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn a_copy_still_inside_its_disk_image_knows_it_cannot_replace_itself() {
    let at = std::path::Path::new("/Volumes/Tisty/Tisty.app/Contents/MacOS/tisty");

    assert!(mounted(Some(at)));
    assert!(!mounted(Some(std::path::Path::new(APP))));
    assert!(!mounted(None));
}

#[test]
#[cfg(not(target_os = "macos"))]
fn nothing_is_mounted_anywhere_but_a_mac() {
    assert!(!mounted(Some(std::path::Path::new(
        "/Volumes/Tisty/Tisty.app"
    ))));
}

fn nowhere(_: &std::path::Path) -> bool {
    false
}

fn at(said: &str) -> Option<&std::path::Path> {
    Some(std::path::Path::new(said))
}

fn only(named: &'static str) -> impl Fn(&std::path::Path) -> bool {
    move |what| what == std::path::Path::new(named)
}

const APP: &str = "/Applications/Tisty.app/Contents/MacOS/tisty";

const MSIX: &str =
    r"C:\Program Files\WindowsApps\rgdevment.Tisty_0.2.0.0_x64__8wekyb3d8bbwe\tisty.exe";

#[test]
fn a_copy_under_windowsapps_is_kept_by_the_store() {
    assert_eq!(chosen(at(MSIX), nowhere), Kept::plain(Route::Store));
}

#[test]
fn the_separator_is_read_the_same_on_every_system() {
    assert_eq!(
        chosen(at(&MSIX.replace('\\', "/")), nowhere),
        Kept::plain(Route::Store)
    );
}

#[test]
fn a_folder_is_named_windowsapps_or_it_is_not() {
    let alike = r"C:\Program Files\WindowsAppsBackup\Tisty\tisty.exe";

    assert_eq!(chosen(at(alike), nowhere), Kept::plain(Route::Download));
    assert_eq!(
        chosen(at(&MSIX.to_lowercase()), nowhere),
        Kept::plain(Route::Store),
        "Windows does not distinguish the case of a folder"
    );
}

#[test]
fn a_cask_answers_with_its_own_command() {
    assert_eq!(
        chosen(at(APP), only("/opt/homebrew/Caskroom/tisty")),
        Kept {
            route: Route::Brew,
            package: Some("tisty")
        }
    );
}

#[test]
fn the_command_line_is_a_formula_and_lives_where_formulas_do() {
    assert_eq!(
        chosen(
            at("/opt/homebrew/bin/tisty"),
            only("/opt/homebrew/Cellar/tisty-cli")
        ),
        Kept {
            route: Route::BrewCli,
            package: Some("tisty-cli")
        }
    );
    assert_eq!(
        chosen(at(APP), only("/opt/homebrew/Caskroom/tisty-cli")),
        Kept::plain(Route::Download),
        "no formula is ever kept under Caskroom"
    );
}

#[test]
fn a_candidate_is_upgraded_by_the_name_it_was_installed_under() {
    assert_eq!(
        chosen(at(APP), only("/opt/homebrew/Caskroom/tisty-beta")),
        Kept {
            route: Route::Brew,
            package: Some("tisty-beta")
        }
    );
    assert_eq!(
        chosen(
            at("/opt/homebrew/bin/tisty"),
            only("/opt/homebrew/Cellar/tisty-cli-beta")
        ),
        Kept {
            route: Route::BrewCli,
            package: Some("tisty-cli-beta")
        }
    );
}

#[test]
fn the_older_homebrew_root_answers_too() {
    assert_eq!(
        chosen(at(APP), only("/usr/local/Caskroom/tisty")),
        Kept {
            route: Route::Brew,
            package: Some("tisty")
        }
    );
    assert_eq!(
        chosen(
            at("/usr/local/bin/tisty"),
            only("/usr/local/Cellar/tisty-cli")
        ),
        Kept {
            route: Route::BrewCli,
            package: Some("tisty-cli")
        }
    );
}

#[test]
fn the_window_is_the_window_even_beside_its_own_command_line() {
    assert_eq!(
        chosen(at(APP), |_| true),
        Kept {
            route: Route::Brew,
            package: Some("tisty")
        }
    );
}

#[test]
fn what_is_running_wins_over_what_is_merely_installed() {
    assert_eq!(chosen(at(MSIX), |_| true), Kept::plain(Route::Store));
}

#[test]
fn everything_else_gets_the_page() {
    assert_eq!(
        chosen(at(r"C:\Program Files\Tisty\tisty.exe"), nowhere),
        Kept::plain(Route::Download)
    );
    assert_eq!(chosen(at(APP), nowhere), Kept::plain(Route::Download));
    assert_eq!(chosen(None, nowhere), Kept::plain(Route::Download));
}

#[test]
fn nothing_is_owed_before_the_interval_is_up() {
    let now: jiff::Timestamp = "2026-08-12T12:00:00Z".parse().unwrap();
    let recent: jiff::Timestamp = "2026-08-12T09:00:00Z".parse().unwrap();
    let old: jiff::Timestamp = "2026-08-10T09:00:00Z".parse().unwrap();

    let ahead: jiff::Timestamp = "2026-09-01T09:00:00Z".parse().unwrap();

    assert!(due(None, now), "a copy that never looked should look");
    assert!(
        due(Some(ahead), now),
        "a clock put back is not a look owed later"
    );
    assert!(!due(Some(recent), now));
    assert!(due(Some(old), now));
}
