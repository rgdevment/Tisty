use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tisty_core::config::Holds;
use tisty_core::event::{DeviceId, DocAdd, TaskAdd};
use tisty_core::signing::SigningKey;
use tisty_core::{Op, Store};
use ulid::Ulid;

use crate::fakes::{self, Counting, Fake};
use crate::{
    Carrier, Cloud, Expect, Folder, Here, Hitch, Moved, Reason, Remote, Round, STORE, Shared,
    Trouble, Way,
};

const DOC: &str = "dev_a-0001";

enum Place {
    Folder(PathBuf),
    Cloud(Arc<dyn Counting>),
}

struct Setting {
    name: String,
    room: tempfile::TempDir,
    place: Place,
}

struct Desk {
    here: Here,
    carrier: Shared,
}

fn settings() -> Vec<Setting> {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let mut all = vec![Setting {
        name: "folder".to_string(),
        room,
        place: Place::Folder(shared),
    }];
    for fake in fakes::every() {
        all.push(Setting {
            name: format!("cloud over {}", fake.who()),
            room: tempfile::tempdir().unwrap(),
            place: Place::Cloud(Arc::from(fake)),
        });
    }
    all
}

fn clouds() -> Vec<Setting> {
    settings()
        .into_iter()
        .filter(|one| matches!(one.place, Place::Cloud(_)))
        .collect()
}

fn desk(setting: &Setting, device: &str) -> Desk {
    let base = setting.room.path().join(device);
    let here = Here {
        data: base.join("data"),
        aside: base.join("cache"),
        device: device.to_string(),
    };
    let carrier: Shared = match &setting.place {
        Place::Folder(shared) => Arc::new(Folder::at(shared.clone())),
        Place::Cloud(remote) => {
            let remote: Arc<dyn Remote> = remote.clone();
            Arc::new(Cloud::over(remote, base.join("cloud")))
        }
    };
    Desk { here, carrier }
}

fn pair(setting: &Setting) -> (Desk, Desk) {
    let (one, two) = (desk(setting, "dev_a"), desk(setting, "dev_b"));
    one.vouches_for(&two);
    two.vouches_for(&one);
    (one, two)
}

fn counting(setting: &Setting) -> &Arc<dyn Counting> {
    match &setting.place {
        Place::Cloud(remote) => remote,
        Place::Folder(_) => unreachable!("only a cloud counts requests"),
    }
}

fn home_of(setting: &Setting, device: &str) -> PathBuf {
    setting.room.path().join(device).join("cloud")
}

fn doc_name() -> String {
    format!("docs/{DOC}.md")
}

impl Desk {
    fn paths(&self) -> tisty_core::Paths {
        tisty_core::Paths::new(self.here.data.clone(), self.here.data.join("config"))
    }

    fn whose(&self) -> DeviceId {
        DeviceId(self.here.device.clone())
    }

    fn key(&self) -> Option<SigningKey> {
        tisty_core::signing::mine(&self.paths(), &self.whose())
    }

    fn confirm(&self, whose: &DeviceId, shown: &str) {
        std::fs::create_dir_all(&self.here.data).unwrap();
        tisty_core::vouched::confirm(&self.here.data, whose, shown);
    }

    fn vouches_for(&self, other: &Desk) {
        if let Some(shown) = other.key().as_ref().map(tisty_core::signing::shown) {
            self.confirm(&other.whose(), &shown);
        }
    }

    fn store(&self) -> Store {
        let key = self.key();
        if let Some(shown) = key.as_ref().map(tisty_core::signing::shown) {
            self.confirm(&self.whose(), &shown);
        }
        Store::open(self.here.data.join(STORE), self.whose())
            .unwrap()
            .signing_with(key)
    }

    fn wrote(&self, title: &str) {
        self.store()
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new(title, "a0"),
            })
            .unwrap();
    }

    fn titles(&self) -> Vec<String> {
        let events = tisty_core::store::read_all(self.here.data.join(STORE)).unwrap();
        let mut all: Vec<String> = tisty_core::State::replay(&events)
            .tasks
            .values()
            .map(|task| task.title.clone())
            .collect();
        all.sort();
        all
    }

    fn carry(&self, way: Way, holds: Holds) -> Result<Moved, Trouble> {
        let mut saying = |_| {};
        self.carrier.carry(
            &self.here,
            Round {
                way,
                alive: &[],
                holds,
                saying: &mut saying,
            },
        )
    }

    fn round(&self, way: Way) -> Moved {
        self.carry(way, Holds::Everywhere).unwrap()
    }

    fn filed(&self, file: &str, body: &str) {
        self.store()
            .append(Op::DocAdd {
                id: Ulid::generate(),
                d: DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: file.to_string(),
                    order: "a0".into(),
                    folder: None,
                    page_of: None,
                },
            })
            .unwrap();
        self.wrote_doc(file, body);
    }

    fn wrote_doc(&self, file: &str, body: &str) {
        tisty_core::docs::write(&self.here.data.join("docs"), file, body).unwrap();
    }

    fn edited(&self, file: &str, body: &str) {
        self.wrote_doc(file, body);
        let events = tisty_core::store::read_all(self.here.data.join(STORE)).unwrap();
        let id = tisty_core::State::replay(&events)
            .docs
            .values()
            .find(|paper| paper.file == file)
            .map(|paper| paper.id)
            .unwrap();
        self.store()
            .append(Op::DocSaid {
                id,
                d: tisty_core::event::Said::of(body),
            })
            .unwrap();
    }

    fn doc(&self, file: &str) -> Option<String> {
        std::fs::read_to_string(self.here.data.join("docs").join(format!("{file}.md"))).ok()
    }

    fn kept(&self, name: &str, body: &[u8]) -> tisty_core::attach::Kept {
        let source = self.here.data.join(format!("source-{name}"));
        std::fs::create_dir_all(&self.here.data).unwrap();
        std::fs::write(&source, body).unwrap();
        tisty_core::attach::keep(&source, &self.here.data, tisty_core::attach::COPIED_IN_DOC)
            .unwrap()
    }

    fn held(&self, reference: &str) -> Option<Vec<u8>> {
        let at = tisty_core::attach::resolve(reference, &self.here.data).unwrap();
        std::fs::read(at).ok()
    }
}

#[test]
fn what_one_machine_wrote_reaches_the_other_and_back() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["lo de uno"], "{who}");

        two.wrote("lo de dos");
        two.round(Way::Both);
        one.round(Way::Both);

        assert_eq!(one.titles(), ["lo de dos", "lo de uno"], "{who}");
        assert_eq!(two.titles(), one.titles(), "{who}");
    }
}

#[test]
fn a_machine_joining_a_place_that_has_a_history_takes_all_of_it() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        for title in ["primera", "segunda", "tercera"] {
            one.wrote(title);
            one.round(Way::Both);
        }

        two.round(Way::Pull);

        assert_eq!(two.titles(), ["primera", "segunda", "tercera"], "{who}");
        assert_eq!(two.carrier.theirs(), one.carrier.theirs(), "{who}");
        assert!(two.carrier.theirs().is_some(), "{who}");
    }
}

#[test]
fn a_round_with_nothing_new_moves_nothing() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        two.round(Way::Both);
        two.wrote("lo de dos");
        two.round(Way::Both);
        one.round(Way::Both);
        assert_eq!(one.titles().len(), 2, "{who}: the exchange happened");

        let again = one.round(Way::Both);
        let other = two.round(Way::Both);

        for moved in [again, other] {
            assert_eq!((moved.sent, moved.brought), (0, 0), "{who}");
            assert_eq!(moved.deferred, None, "{who}");
        }
    }
}

#[test]
fn a_document_travels_and_an_edit_comes_back() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);

        two.round(Way::Both);

        assert_eq!(two.doc(DOC).as_deref(), Some("# Plan\n\nuno\n"), "{who}");
        assert!(one.carrier.paper_waiting(DOC), "{who}");
        assert_eq!(
            one.carrier.paper_print(DOC),
            two.carrier.paper_print(DOC),
            "{who}"
        );

        two.edited(DOC, "# Plan\n\nuno y dos\n");
        two.round(Way::Both);
        one.round(Way::Both);

        assert_eq!(
            one.doc(DOC).as_deref(),
            Some("# Plan\n\nuno y dos\n"),
            "{who}"
        );
    }
}

#[test]
fn an_attachment_goes_up_and_comes_down() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.wrote("con foto");
        one.round(Way::Both);

        let moved = two.round(Way::Both);

        assert_eq!(
            two.held(&kept.at).as_deref(),
            Some(b"los bytes de una foto".as_slice()),
            "{who}"
        );
        assert_eq!(moved.took_in.len(), 1, "{who}");
    }
}

#[test]
fn a_document_the_place_forgets_is_no_longer_waiting_there() {
    for setting in settings() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);
        assert!(one.carrier.paper_waiting(DOC), "{who}");

        one.carrier.forget_paper(DOC);

        assert!(!one.carrier.paper_waiting(DOC), "{who}");
    }
}

#[test]
fn a_document_forgotten_here_is_removed_from_the_cloud_by_the_next_round() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);
        assert!(remote.about(&doc_name()).unwrap().is_some(), "{who}");
        std::fs::remove_file(one.here.data.join("docs").join(format!("{DOC}.md"))).unwrap();
        one.carrier.forget_paper(DOC);
        assert!(
            remote.about(&doc_name()).unwrap().is_some(),
            "{who}: not yet"
        );

        one.round(Way::Both);

        assert_eq!(remote.about(&doc_name()).unwrap(), None, "{who}");
    }
}

#[test]
fn a_quiet_cloud_round_asks_once_and_moves_no_content() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        one.round(Way::Both);
        let home = home_of(&setting, "dev_a");
        assert!(
            home.join("index.json").is_file(),
            "{who}: the index was kept"
        );
        remote.forget_counts();

        let moved = one.round(Way::Both);

        let spent = remote.counts();
        assert_eq!((spent.sent, spent.received), (0, 0), "{who}");
        let asked = match remote.native_changes() {
            true => 1,
            false => remote.limits().requests_to_list(8),
        };
        assert!(
            spent.requests <= asked.max(1),
            "{who}: {} requests",
            spent.requests
        );
        assert_eq!(moved.deferred, None, "{who}");
    }
}

#[test]
fn a_cut_connection_defers_the_round_and_loses_nothing() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        remote.fail_next(Hitch::Unreachable("sin red".into()));

        let cut = one.round(Way::Both);

        let deferred = cut.deferred.unwrap();
        assert_eq!(deferred.reason, Reason::Offline, "{who}");
        assert_eq!((cut.sent, cut.brought), (0, 0), "{who}");
        one.round(Way::Both);
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["lo de uno"], "{who}");
    }
}

#[test]
fn a_limit_from_the_provider_defers_with_the_wait_it_asked_for() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        one.wrote("lo de despues");
        remote.fail_after(
            1,
            Hitch::Limited {
                wait: Duration::from_secs(9),
            },
        );

        let moved = one.round(Way::Both);

        let deferred = moved.deferred.unwrap();
        assert_eq!(deferred.reason, Reason::Limit, "{who}");
        assert_eq!(deferred.retry_after, Some(Duration::from_secs(9)), "{who}");
        assert!(deferred.left > 0, "{who}: something was left");
        let later = one.round(Way::Both);
        assert_eq!(later.deferred, None, "{who}");
        two.round(Way::Both);
        assert_eq!(two.titles(), ["lo de despues", "lo de uno"], "{who}");
    }
}

#[test]
fn a_provider_that_will_not_let_us_in_is_a_refusal_not_a_deferral() {
    for setting in clouds() {
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");

        remote.fail_next(Hitch::Lost);
        let lost = one.carry(Way::Both, Holds::Everywhere);
        remote.fail_next(Hitch::Full);
        let full = one.carry(Way::Both, Holds::Everywhere);

        assert!(matches!(lost, Err(Trouble::Refused(_))), "{}", setting.name);
        assert!(matches!(full, Err(Trouble::Refused(_))), "{}", setting.name);
    }
}

#[test]
fn a_refusal_while_sending_still_tells_what_the_round_did() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        remote.fail_after(1, Hitch::Full);

        let moved = one.round(Way::Both);

        assert!(moved.sent > 0, "{who}: the local side moved");
        assert_eq!(moved.deferred.unwrap().reason, Reason::Refused, "{who}");
        one.round(Way::Both);
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["lo de uno"], "{who}");
    }
}

#[test]
fn two_rounds_never_run_at_once() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        let held = super::lock::Round::take(&home_of(&setting, "dev_a")).unwrap();
        assert!(held.is_some(), "{who}");
        remote.forget_counts();

        let busy = one.round(Way::Both);

        assert_eq!(busy.deferred.unwrap().reason, Reason::Busy, "{who}");
        assert_eq!(remote.counts().requests, 0, "{who}");
        drop(held);
        assert_eq!(one.round(Way::Both).deferred, None, "{who}");
    }
}

#[test]
fn a_mirror_that_was_deleted_is_rebuilt_from_the_cloud() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        let home = home_of(&setting, "dev_a");
        std::fs::remove_dir_all(&home).unwrap();

        let moved = one.round(Way::Both);

        assert_eq!(moved.deferred, None, "{who}");
        assert!(home.join("tree").join("tisty.toml").is_file(), "{who}");
        assert_eq!(one.titles(), ["lo de uno"], "{who}");
        assert!(one.carrier.theirs().is_some(), "{who}");
    }
}

#[test]
fn a_mirror_that_lost_files_gets_them_back_and_takes_nothing_from_the_cloud() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        let tree = home_of(&setting, "dev_a").join("tree");
        let before = remote.list("").unwrap().len();
        std::fs::remove_file(tree.join("tisty.toml")).unwrap();
        std::fs::remove_dir_all(tree.join("store")).unwrap();

        let moved = one.carry(Way::Both, Holds::Everywhere);

        assert!(moved.is_ok(), "{who}: {moved:?}");
        assert_eq!(
            remote.list("").unwrap().len(),
            before,
            "{who}: nothing was removed up there"
        );
        assert!(tree.join("tisty.toml").is_file(), "{who}");
        assert!(tree.join("store").is_dir(), "{who}");
    }
}

#[test]
fn a_lost_index_costs_no_download_when_the_mirror_is_intact() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        std::fs::remove_file(home_of(&setting, "dev_a").join("index.json")).unwrap();
        remote.forget_counts();

        let moved = one.round(Way::Both);

        assert_eq!(moved.deferred, None, "{who}");
        assert_eq!(
            remote.counts().received,
            0,
            "{who}: nothing needed to come down"
        );
        assert_eq!(
            remote.counts().sent,
            0,
            "{who}: and nothing needed to go up"
        );
    }
}

#[test]
fn a_lost_index_does_not_bring_back_what_another_machine_deleted() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);
        remote.delete(&doc_name(), None).unwrap();
        std::fs::remove_file(one.here.data.join("docs").join(format!("{DOC}.md"))).unwrap();
        std::fs::remove_file(home_of(&setting, "dev_a").join("index.json")).unwrap();

        one.round(Way::Both);

        assert_eq!(remote.about(&doc_name()).unwrap(), None, "{who}");
        let mirrored = home_of(&setting, "dev_a").join("tree").join(doc_name());
        assert!(!mirrored.exists(), "{who}");
    }
}

#[test]
fn an_edit_that_did_not_go_up_is_not_overwritten_by_what_another_machine_sent_meanwhile() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        one.filed(DOC, "# Plan\n\nbase\n");
        one.round(Way::Both);
        two.round(Way::Both);
        one.edited(DOC, "# Plan\n\nlo de uno\n");
        remote.fail_after(
            1,
            Hitch::Limited {
                wait: Duration::from_secs(1),
            },
        );
        let held_back = one.round(Way::Both);
        assert_eq!(held_back.deferred.unwrap().reason, Reason::Limit, "{who}");
        two.edited(DOC, "# Plan\n\nlo de dos\n");
        two.round(Way::Both);

        let after = one.round(Way::Both);

        assert!(
            after.undecided_ids().contains(&DOC.to_string()),
            "{who}: {after:?}"
        );
        assert_eq!(
            one.doc(DOC).as_deref(),
            Some("# Plan\n\nlo de uno\n"),
            "{who}"
        );
    }
}

#[test]
fn history_goes_up_before_documents_and_the_stamp_goes_last() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.wrote("lo de uno");

        one.round(Way::Both);

        let sent = remote.uploaded();
        let last_history = sent
            .iter()
            .rposition(|name| name.starts_with("store/"))
            .unwrap();
        let first_document = sent
            .iter()
            .position(|name| name.starts_with("docs/"))
            .unwrap();
        assert!(last_history < first_document, "{who}: {sent:?}");
        assert_eq!(
            sent.last().map(String::as_str),
            Some("tisty.toml"),
            "{who}: {sent:?}"
        );
    }
}

#[test]
fn a_name_that_cannot_be_a_path_is_left_alone() {
    let room = tempfile::tempdir().unwrap();
    let drive = Arc::new(Fake::drive());
    drive.plant("x\\evil.md", b"1");
    drive.plant("docs/../../evil.md", b"1");
    drive.plant("C:/evil.md", b"1");
    let home = room.path().join("dev_a").join("cloud");
    let remote: Arc<dyn Remote> = drive.clone();
    let carrier = Cloud::over(remote, home.clone());
    let here = Here {
        data: room.path().join("dev_a").join("data"),
        aside: room.path().join("dev_a").join("cache"),
        device: "dev_a".to_string(),
    };

    let mut saying = |_| {};
    let _ = carrier.carry(
        &here,
        Round {
            way: Way::Pull,
            alive: &[],
            holds: Holds::Everywhere,
            saying: &mut saying,
        },
    );

    let outside = [
        home.join("evil.md"),
        room.path().join("evil.md"),
        room.path().join("dev_a").join("evil.md"),
    ];
    for at in outside {
        assert!(!at.exists(), "{at:?}");
    }
    let index = super::index::Index::load(&home);
    assert!(index.tree.keys().all(|name| !name.contains("evil")));
}

#[test]
fn a_file_rewritten_with_the_same_size_in_the_same_instant_is_still_sent() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(tree.join("docs")).unwrap();
    let remote = Fake::drive();
    let mut index = super::index::Index::default();
    let at = tree.join("docs").join(format!("{DOC}.md"));
    std::fs::write(&at, b"[ ] una tarea").unwrap();
    let (_, hitch) = super::mirror::push(&remote, &tree, &mut index);
    assert_eq!(hitch, None);
    let stamped = std::fs::metadata(&at).unwrap().modified().unwrap();
    std::fs::write(&at, b"[x] una tarea").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&at)
        .unwrap()
        .set_modified(stamped)
        .unwrap();

    let (_, hitch) = super::mirror::push(&remote, &tree, &mut index);

    assert_eq!(hitch, None);
    let mut up = Vec::new();
    remote.fetch(&doc_name(), 0, &mut up).unwrap();
    assert_eq!(up, b"[x] una tarea");
}

#[test]
fn an_upload_the_provider_defers_keeps_the_original_and_the_next_round_sends_it() {
    for setting in clouds().into_iter().take(1) {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        let big = vec![7u8; usize::try_from(tisty_core::attach::COPIED_UP_TO).unwrap() + 1024];
        let kept = one.kept("grande", &big);
        remote.fail_after(
            1,
            Hitch::Limited {
                wait: Duration::from_secs(1),
            },
        );

        let held_back = one.carry(Way::Both, Holds::Shared).unwrap();

        assert_eq!(held_back.deferred.unwrap().reason, Reason::Limit, "{who}");
        assert_eq!(held_back.let_go, Vec::<String>::new(), "{who}");
        assert!(
            one.held(&kept.at).is_some(),
            "{who}: the original is still here"
        );

        let sent = one.carry(Way::Both, Holds::Shared).unwrap();

        assert_eq!(sent.let_go, std::slice::from_ref(&kept.at), "{who}");
        assert!(
            one.held(&kept.at).is_none(),
            "{who}: it left once it was up there"
        );
        let up = remote.about(&kept.at).unwrap().unwrap();
        assert_eq!(up.bytes, big.len() as u64, "{who}");
    }
}

#[test]
fn what_another_left_in_the_cloud_that_is_not_ours_to_mirror_is_neither_fetched_nor_removed() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let litter = setting.room.path().join("litter");
        std::fs::write(&litter, b"a half-written thing").unwrap();
        let name = "docs/dev_x-0001.01ARZ3NDEKTSV4RRFFQ69G5FAV.3.part";
        remote.put(name, &litter, Expect::Absent).unwrap();
        one.wrote("lo de uno");

        one.round(Way::Both);
        one.round(Way::Both);

        let mirrored = home_of(&setting, "dev_a").join("tree").join(name);
        assert!(!mirrored.exists(), "{who}");
        assert!(
            remote.about(name).unwrap().is_some(),
            "{who}: it is not ours to remove"
        );
    }
}
