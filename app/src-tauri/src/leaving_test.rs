use std::sync::Arc;

use tisty_core::config::CloudAt;

use super::*;
use crate::Session;

fn wiped(
    carrier: &dyn tisty_carrier::Carrier,
    here: &tisty_carrier::Here,
    alive: &[String],
) -> crate::Answer<usize> {
    crate::answers::relocating::all_home(carrier, here, alive)?;
    crate::answers::relocating::emptied(carrier, &mut |_, _| {})
}

fn machine(room: &Path, named: &str) -> Session {
    let paths = tisty_core::Paths::new(
        room.join(named).join("data"),
        room.join(named).join("config"),
    );
    Session::at(paths).unwrap()
}

fn vouches_for(by: &Session, of: &Session) {
    let key = tisty_core::signing::mine(&of.paths, &of.config.device_id).unwrap();
    tisty_core::vouched::confirm(
        by.paths.data(),
        &of.config.device_id,
        &tisty_core::signing::shown(&key),
    );
}

// Machines that sync together carry one store, and a word is only heeded within it.
fn shares_the_store(by: &Session, of: &Session) {
    std::fs::write(
        by.paths.store().join(tisty_core::store::MARKER),
        tisty_core::moved::store_of(of.paths.data()),
    )
    .unwrap();
}

fn word_in(folder: &Path) -> Option<String> {
    std::fs::read_to_string(folder.join(tisty_core::moved::NAMED)).ok()
}

#[test]
fn moving_away_keeps_tasks_and_documents_first_and_stops_the_machine_left_behind() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    let elsewhere = room.path().join("elsewhere");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&elsewhere).unwrap();
    let (mut one, two) = (machine(room.path(), "one"), machine(room.path(), "two"));
    vouches_for(&two, &one);
    shares_the_store(&two, &one);
    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();

    crate::answers::storing::choose(&mut one, Some(elsewhere.display().to_string())).unwrap();

    let kept = std::fs::read_dir(one.paths.private().join("before-a-change"))
        .unwrap()
        .count();
    assert_eq!(
        kept, 2,
        "a change went ahead without keeping tasks and documents first"
    );
    let stopped = heeded(
        word_in(&shared).as_deref(),
        two.paths.data(),
        &two.config.device_id,
        "folder",
    )
    .expect("the machine left behind went on syncing alone");
    assert_eq!(stopped.code, "movedAway");
    assert_eq!(stopped.name.as_deref(), Some("elsewhere"));
    assert!(
        heeded(
            word_in(&shared).as_deref(),
            one.paths.data(),
            &one.config.device_id,
            "folder"
        )
        .is_none(),
        "a machine stopped on its own word"
    );
}

#[test]
fn a_place_chosen_again_is_the_person_s_and_its_old_word_goes() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join(tisty_core::moved::NAMED), "un aviso viejo").unwrap();
    let mut one = machine(room.path(), "one");

    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();

    assert_eq!(word_in(&shared), None);
}

#[test]
fn a_word_the_old_place_could_not_take_is_owed_and_delivered_once_it_can() {
    let room = tempfile::tempdir().unwrap();
    let gone = room.path().join("unplugged");
    let one = machine(room.path(), "one");
    let left = Sync::folder(&gone);
    let old = tisty_carrier::chosen(Some(&left)).carrier;

    leave_word(&one.paths, &one.config, &left, old, &Sync::alone());

    assert!(one.paths.private().join(OWED).is_file());
    let owed = owing(&one.paths, &one.reach, None);
    assert_eq!(owed.len(), 1);
    assert!(!owed.into_iter().all(Owing::delivered));
    std::fs::create_dir_all(&gone).unwrap();
    let owed = owing(&one.paths, &one.reach, None);
    assert_eq!(owed.len(), 1);
    assert!(owed.into_iter().all(Owing::delivered));
    assert!(word_in(&gone).is_some_and(|said| said.contains("alone")));
    assert!(!one.paths.private().join(OWED).exists());
}

#[test]
fn the_same_account_under_a_new_address_is_the_same_place() {
    let at = |email: &str| {
        Sync::cloud(CloudAt {
            provider: "google".into(),
            account: "0123".into(),
            email: email.into(),
        })
    };

    assert!(same_place(
        &at("ana@example.com"),
        &at("ana.nueva@example.com")
    ));
    assert!(!same_place(
        &Sync::folder("G:/una"),
        &Sync::folder("G:/otra")
    ));
    assert!(!same_place(&Sync::alone(), &at("ana@example.com")));
}

#[test]
fn choosing_again_the_place_this_machine_was_stopped_in_takes_the_word_away() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let mut two = machine(room.path(), "two");
    crate::answers::storing::choose(&mut two, Some(shared.display().to_string())).unwrap();
    std::fs::write(shared.join(tisty_core::moved::NAMED), "otro equipo se fue").unwrap();

    crate::answers::storing::choose(&mut two, Some(shared.display().to_string())).unwrap();

    assert_eq!(
        word_in(&shared),
        None,
        "staying on purpose left the machine stopped"
    );
}

#[test]
fn a_word_owed_to_a_place_this_machine_came_back_to_is_never_delivered() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    let mut one = machine(room.path(), "one");
    let left = Sync::folder(&shared);
    leave_word(&one.paths, &one.config, &left, None, &Sync::alone());
    assert!(one.paths.private().join(OWED).is_file());
    std::fs::create_dir_all(&shared).unwrap();

    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();

    assert!(!one.paths.private().join(OWED).exists());
    assert!(owing(&one.paths, &one.reach, one.config.sync.as_ref()).is_empty());
    assert_eq!(word_in(&shared), None);
}

#[test]
fn a_word_owed_is_dropped_when_the_way_now_chosen_is_that_place() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let one = machine(room.path(), "one");
    let left = Sync::folder(&shared);
    owe(&one.paths, &left, "un aviso");

    assert!(owing(&one.paths, &one.reach, Some(&left)).is_empty());
    assert!(!one.paths.private().join(OWED).exists());
}

#[test]
fn leaving_this_machine_alone_owes_nobody_and_keeps_the_word_owed_to_a_real_place() {
    let room = tempfile::tempdir().unwrap();
    let unplugged = room.path().join("unplugged");
    let elsewhere = room.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let mut one = machine(room.path(), "one");
    one.keep(|c| c.sync = Some(Sync::alone())).unwrap();
    owe(
        &one.paths,
        &Sync::folder(&unplugged),
        "para la carpeta desconectada",
    );

    crate::answers::storing::choose(&mut one, Some(elsewhere.display().to_string())).unwrap();

    let owed = owed_all(&one.paths);
    assert_eq!(
        owed.iter().map(|one| &one.way).collect::<Vec<_>>(),
        [&Sync::folder(&unplugged)],
        "a word to nobody pushed out a real one"
    );
}

#[test]
fn a_word_owed_to_no_place_is_dropped_instead_of_kept_forever() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    owe(&one.paths, &Sync::alone(), "a nadie");

    assert!(owing(&one.paths, &one.reach, None).is_empty());
    assert!(!one.paths.private().join(OWED).exists());
}

#[test]
fn a_word_that_cannot_be_delivered_holds_back_none_owed_after_it() {
    let room = tempfile::tempdir().unwrap();
    let unplugged = room.path().join("unplugged");
    let reachable = room.path().join("reachable");
    std::fs::create_dir_all(&reachable).unwrap();
    let one = machine(room.path(), "one");
    owe(
        &one.paths,
        &Sync::folder(&unplugged),
        "para la desconectada",
    );
    owe(
        &one.paths,
        &Sync::folder(&reachable),
        "para la que responde",
    );

    for owed in owing(&one.paths, &one.reach, None) {
        owed.delivered();
    }

    assert!(word_in(&reachable).is_some_and(|said| said.contains("responde")));
    let left = owed_all(&one.paths);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].way, Sync::folder(&unplugged));
}

#[test]
fn a_change_that_is_refused_keeps_no_copy() {
    let room = tempfile::tempdir().unwrap();
    let mut one = machine(room.path(), "one");
    let later: Sync = toml::from_str("how = \"nube\"\nat = { cuenta = \"yo\" }").unwrap();
    one.keep(|c| c.sync = Some(later.clone())).unwrap();

    let refused = crate::answers::storing::choose(&mut one, Some("G:/otra".to_string()))
        .expect_err("left a way a newer build chose");

    assert_eq!(refused.code, "syncLaterToLeave");
    assert!(
        !one.paths.private().join("before-a-change").exists(),
        "a refused change pushed out a real copy"
    );
}

#[test]
fn machines_left_behind_are_told_the_cloud_the_others_moved_to() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let (mut one, two) = (machine(room.path(), "one"), machine(room.path(), "two"));
    vouches_for(&two, &one);
    shares_the_store(&two, &one);
    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();
    let cloud = Sync::cloud(CloudAt {
        provider: "google".into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    });

    leave_word(
        &one.paths,
        &one.config,
        &Sync::folder(&shared),
        Some(tisty_carrier::considering(&shared)),
        &cloud,
    );

    let stopped = heeded(
        word_in(&shared).as_deref(),
        two.paths.data(),
        &two.config.device_id,
        "folder",
    )
    .expect("the machine left behind went on syncing alone");
    assert_eq!(stopped.code, "movedToCloud");
    assert_eq!(
        stopped.name.as_deref(),
        Some("Google Drive · ana@example.com")
    );
}

fn paths_in(room: &Path) -> Paths {
    Paths::new(room.join("data"), room.join("config"))
}

fn ana() -> Sync {
    Sync::cloud(CloudAt {
        provider: "google".into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    })
}

#[test]
fn two_words_owed_to_two_places_are_both_kept_and_each_goes_once_delivered() {
    let room = tempfile::tempdir().unwrap();
    let paths = paths_in(room.path());
    let first = Sync::folder(room.path().join("first"));
    let second = Sync::folder(room.path().join("second"));

    owe(&paths, &first, "fui a otro lado");
    owe(&paths, &second, "y luego a otro");
    owe(&paths, &first, "otra vez desde el primero");

    let all = owed_all(&paths);
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .any(|owed| owed.said == "otra vez desde el primero")
    );
    assert!(all.iter().any(|owed| owed.said == "y luego a otro"));

    owed_no_more(&paths, &first);
    assert_eq!(owed_all(&paths).len(), 1);
    owed_no_more(&paths, &second);
    assert!(!paths.private().join(OWED).exists());
}

#[test]
fn a_word_kept_in_the_cache_by_an_older_build_is_read_once_and_kept_beside_the_settings() {
    let room = tempfile::tempdir().unwrap();
    let paths = paths_in(room.path());
    std::fs::create_dir_all(paths.cache()).unwrap();
    let one = Owed {
        way: ana(),
        said: "me fui".into(),
    };
    std::fs::write(paths.cache().join(OWED), serde_json::to_vec(&one).unwrap()).unwrap();

    assert_eq!(owed_all(&paths).len(), 1);
    assert!(!paths.cache().join(OWED).exists());
    assert!(paths.private().join(OWED).is_file());
    std::fs::remove_dir_all(paths.cache()).unwrap();
    assert!(
        owed_to(&paths, "0123"),
        "a wiped cache took the word with it"
    );
    assert!(!owed_to(&paths, "9999"));
}

/// A cloud that refuses the word and holds no copy of it, as Drive does when it is full or out of reach.
struct Refusing {
    said: tisty_carrier::Trouble,
    brought: usize,
    wiping: Result<usize, tisty_carrier::Trouble>,
    owed_at: PathBuf,
    owed_first: std::sync::atomic::AtomicBool,
}

impl Refusing {
    fn with(said: tisty_carrier::Trouble, paths: &Paths) -> Self {
        Self {
            said,
            brought: 0,
            wiping: Err(tisty_carrier::Trouble::NotThere(
                tisty_core::moved::NAMED.to_string(),
            )),
            owed_at: paths.private().join(OWED),
            owed_first: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl tisty_carrier::Carrier for Refusing {
    fn place(&self) -> Option<&Path> {
        None
    }
    fn lends_from(&self) -> Option<PathBuf> {
        None
    }
    fn frees_space(&self) -> bool {
        true
    }
    fn brought(
        &self,
        _: &tisty_carrier::Here,
        _: &str,
    ) -> Result<tisty_carrier::Brought, tisty_carrier::Trouble> {
        unreachable!()
    }
    fn shelf(&self) -> Option<std::collections::BTreeMap<String, u64>> {
        Some(Default::default())
    }
    fn reachable(&self) -> bool {
        true
    }
    fn theirs(&self) -> Option<String> {
        unreachable!()
    }
    fn been_here(&self, _: &tisty_carrier::Here) -> bool {
        unreachable!()
    }
    fn kin(&self, _: &tisty_carrier::Here) -> tisty_carrier::Kin {
        unreachable!()
    }
    fn signed(&self) -> tisty_carrier::Signed {
        unreachable!()
    }
    fn stirring(&self) -> u64 {
        unreachable!()
    }
    fn unclaimed(&self) -> tisty_carrier::Holding {
        unreachable!()
    }
    fn carry(
        &self,
        _: &tisty_carrier::Here,
        _: tisty_carrier::Round,
    ) -> Result<tisty_carrier::Moved, tisty_carrier::Trouble> {
        unreachable!()
    }
    fn bring_home(
        &self,
        _: &tisty_carrier::Here,
        _: tisty_carrier::Round,
    ) -> Result<tisty_carrier::Moved, tisty_carrier::Trouble> {
        Ok(tisty_carrier::Moved {
            brought: self.brought,
            ..Default::default()
        })
    }
    fn could_free(
        &self,
        _: &tisty_carrier::Here,
        _: std::time::SystemTime,
    ) -> tisty_carrier::LetGo {
        unreachable!()
    }
    fn set_free(
        &self,
        _: &tisty_carrier::Here,
        _: std::time::SystemTime,
        _: tisty_carrier::Told,
    ) -> Result<tisty_carrier::LetGo, tisty_carrier::Trouble> {
        unreachable!()
    }
    fn stitch(
        &self,
        _: &tisty_carrier::Here,
        _: Option<tisty_core::signing::SigningKey>,
    ) -> Result<tisty_carrier::Stitched, tisty_carrier::Trouble> {
        unreachable!()
    }
    fn let_go(
        &self,
        _: &tisty_carrier::Here,
        _: u64,
        _: tisty_carrier::Elsewhere,
        _: tisty_carrier::Told,
    ) -> Result<tisty_carrier::LetGo, tisty_carrier::Trouble> {
        unreachable!()
    }
    fn paper_waiting(&self, _: &str) -> bool {
        unreachable!()
    }
    fn paper_print(&self, _: &str) -> Option<String> {
        unreachable!()
    }
    fn both_papers(
        &self,
        _: &tisty_carrier::Here,
        _: &str,
    ) -> Result<(String, String), tisty_carrier::Trouble> {
        unreachable!()
    }
    fn settle(
        &self,
        _: &tisty_carrier::Here,
        _: &str,
        _: tisty_carrier::Keep,
        _: Option<&str>,
    ) -> Result<Option<String>, tisty_carrier::Trouble> {
        unreachable!()
    }
    fn forget_paper(&self, _: &str) {
        unreachable!()
    }
    fn leave_word(&self, _: &str) -> Result<(), tisty_carrier::Trouble> {
        self.owed_first
            .store(self.owed_at.is_file(), std::sync::atomic::Ordering::Relaxed);
        Err(self.said.clone())
    }
    fn wipe_along(
        &self,
        _along: &mut dyn FnMut(usize, usize),
    ) -> Result<usize, tisty_carrier::Trouble> {
        self.wiping.clone()
    }
}

fn refusals() -> [tisty_carrier::Trouble; 3] {
    [
        tisty_carrier::Trouble::CloudFull {
            provider: "Google Drive".into(),
        },
        tisty_carrier::Trouble::NotNow("the cloud cannot be used now: Limit".into()),
        tisty_carrier::Trouble::NotNow("the cloud cannot be used now: Offline".into()),
    ]
}

#[test]
fn a_word_a_full_limited_or_offline_cloud_did_not_take_stays_owed_and_its_copies_stay() {
    for refusal in refusals() {
        let room = tempfile::tempdir().unwrap();
        let one = machine(room.path(), "one");
        let refusing = Arc::new(Refusing::with(refusal.clone(), &one.paths));
        let old: tisty_carrier::Shared = refusing.clone();

        let told = left_said(&one.paths, &ana(), "me fui", Some(old));
        let wiped = wiped(&*refusing, &one.here(), &[]);

        assert!(
            !told,
            "{refusal:?}: a word that never landed was taken for told"
        );
        assert!(owed_to(&one.paths, "0123"), "{refusal:?}");
        assert_eq!(
            wiped.map_err(|one| one.code),
            Err("wipeUntold"),
            "{refusal:?}"
        );
    }
}

#[test]
fn a_word_is_owed_before_it_is_tried_so_a_window_closed_meanwhile_still_owes_it() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    let refusing = Arc::new(Refusing::with(
        tisty_carrier::Trouble::NotNow("busy".into()),
        &one.paths,
    ));
    let old: tisty_carrier::Shared = refusing.clone();

    left_said(&one.paths, &ana(), "me fui", Some(old));

    assert!(
        refusing
            .owed_first
            .load(std::sync::atomic::Ordering::Relaxed),
        "the word was tried before it was written down as owed"
    );
}

#[test]
fn a_cloud_that_brought_something_new_while_its_copies_were_about_to_go_keeps_them() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    let mut refusing = Refusing::with(tisty_carrier::Trouble::NotNow("busy".into()), &one.paths);
    refusing.brought = 1;

    let wiped = wiped(&refusing, &one.here(), &[]);

    assert_eq!(wiped.map_err(|one| one.code), Err("wipeNotHome"));
}

fn wiped_with(wiping: Result<usize, tisty_carrier::Trouble>) -> Answer<usize> {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    let mut refusing = Refusing::with(tisty_carrier::Trouble::NotNow("busy".into()), &one.paths);
    refusing.wiping = wiping;
    wiped(&refusing, &one.here(), &[])
}

fn refused(answer: Answer<usize>) -> Result<usize, (&'static str, Option<String>)> {
    answer.map_err(|one| (one.code, one.name))
}

#[test]
fn a_cloud_with_everything_home_is_emptied_and_says_how_much_went() {
    assert_eq!(refused(wiped_with(Ok(5))), Ok(5));
}

#[test]
fn a_cloud_that_holds_no_word_of_the_move_is_not_emptied_as_untold() {
    let untold = Err(tisty_carrier::Trouble::NotThere(
        tisty_core::moved::NAMED.to_string(),
    ));

    assert_eq!(refused(wiped_with(untold)), Err(("wipeUntold", None)));
}

#[test]
fn a_wipe_missing_some_other_file_says_the_meeting_place_is_gone_not_untold() {
    let elsewhere = Err(tisty_carrier::Trouble::NotThere("store.json".into()));

    assert_eq!(
        refused(wiped_with(elsewhere)),
        Err(("noMeetingPlace", Some("store.json".into())))
    );
}

#[test]
fn a_wipe_paused_by_a_limited_or_offline_cloud_says_it_stopped() {
    for paused in ["the cloud cannot be used now: Limit", "offline"] {
        let stopped = Err(tisty_carrier::Trouble::NotNow(paused.into()));

        assert_eq!(
            refused(wiped_with(stopped)),
            Err(("wipeStopped", None)),
            "{paused}"
        );
    }
}

#[test]
fn a_wipe_refused_for_any_other_reason_says_that_reason() {
    let full = Err(tisty_carrier::Trouble::CloudFull {
        provider: "Google Drive".into(),
    });

    assert_eq!(
        refused(wiped_with(full)),
        Err(("cloudFull", Some("Google Drive".into())))
    );
}

#[test]
fn a_cloud_still_bringing_something_home_is_never_emptied_even_when_it_could_be() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    let mut refusing = Refusing::with(tisty_carrier::Trouble::NotNow("busy".into()), &one.paths);
    refusing.brought = 1;
    refusing.wiping = Ok(5);

    let wiped = wiped(&refusing, &one.here(), &[]);

    assert_eq!(wiped.map_err(|one| one.code), Err("wipeNotHome"));
}

#[test]
fn a_move_owes_the_word_before_the_settings_change() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    let elsewhere = room.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let mut one = machine(room.path(), "one");
    one.keep(|c| c.sync = Some(Sync::folder(&shared))).unwrap();

    crate::answers::storing::choose(&mut one, Some(elsewhere.display().to_string())).unwrap();

    let owed = owed_all(&one.paths);
    assert_eq!(
        owed.iter().map(|one| &one.way).collect::<Vec<_>>(),
        [&Sync::folder(&shared)],
        "the place left behind, out of reach now, was owed nothing"
    );
}

#[test]
fn a_word_signed_here_names_the_store_and_the_place_it_was_left_in() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");

    let said = word_for(&one.paths, &one.config, &ana(), &Sync::alone()).unwrap();
    let word = tisty_core::moved::read(&said).unwrap();

    assert_eq!(word.place, "cloud:google:0123");
    assert_eq!(word.store, tisty_core::moved::store_of(one.paths.data()));
    assert!(!word.store.is_empty());
}

#[test]
fn a_cloud_lost_for_good_is_left_for_this_machine_alone_without_telling_it() {
    let room = tempfile::tempdir().unwrap();
    let mut one = machine(room.path(), "one");
    one.keep(|c| c.sync = Some(ana())).unwrap();

    crate::answers::relocating::left_for_alone(&mut one).unwrap();

    assert_eq!(one.config.sync, Some(Sync::alone()));
    assert!(
        one.paths.private().join("before-a-change").is_dir(),
        "it left without keeping tasks and documents first"
    );
    assert!(
        !owed_to(&one.paths, "0123"),
        "a word was owed to a cloud nobody can reach"
    );
}

#[test]
fn a_word_owed_to_a_cloud_nobody_can_reach_again_is_dropped_and_frees_its_account() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    let folder = Sync::folder(room.path().join("unplugged"));
    owe(&one.paths, &ana(), "me fui");
    owe(&one.paths, &folder, "para la carpeta");

    let _ = owing(&one.paths, &one.reach, None);

    assert!(
        !owed_to(&one.paths, "0123"),
        "a lost cloud kept its word forever"
    );
    assert_eq!(owed_all(&one.paths).len(), 1);
}

#[test]
fn a_word_dropped_for_one_account_keeps_every_other() {
    let room = tempfile::tempdir().unwrap();
    let paths = paths_in(room.path());
    let other = Sync::cloud(CloudAt {
        provider: "google".into(),
        account: "9999".into(),
        email: "otra@example.com".into(),
    });
    owe(&paths, &ana(), "me fui");
    owe(&paths, &other, "y de esta también");

    owed_no_more_to(&paths, "0123");

    assert!(!owed_to(&paths, "0123"));
    assert!(owed_to(&paths, "9999"));
}

#[test]
fn words_owed_from_many_threads_at_once_are_all_kept() {
    let room = tempfile::tempdir().unwrap();
    let paths = paths_in(room.path());
    let places: Vec<Sync> = (0..16)
        .map(|at| Sync::folder(room.path().join(format!("lugar-{at}"))))
        .collect();

    std::thread::scope(|scope| {
        for place in &places {
            let paths = &paths;
            scope.spawn(move || owe(paths, place, "me fui"));
        }
    });

    assert_eq!(owed_all(&paths).len(), places.len());
}
