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
const BACKED: &str = "cloud over a folder";

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
    let room = tempfile::tempdir().unwrap();
    let backed = crate::folder_backed::FolderBacked::at(room.path().join("provider"));
    all.push(Setting {
        name: BACKED.to_string(),
        room,
        place: Place::Cloud(Arc::new(backed)),
    });
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
fn an_attachment_goes_up_and_a_cloud_brings_it_down_only_once_it_is_opened() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        one.round(Way::Both);

        let moved = two.round(Way::Both);

        if let Place::Folder(_) = setting.place {
            assert_eq!(
                two.held(&kept.at).as_deref(),
                Some(b"los bytes de una foto".as_slice()),
                "{who}"
            );
            assert_eq!(moved.took_in.len(), 1, "{who}");
            continue;
        }
        assert_eq!(two.held(&kept.at), None, "{who}: a round spent quota on it");
        assert!(moved.took_in.is_empty(), "{who}");
        two.carrier.brought(&two.here, &kept.at).unwrap();
        assert_eq!(
            two.held(&kept.at).as_deref(),
            Some(b"los bytes de una foto".as_slice()),
            "{who}: opened, it was not kept here"
        );
        assert!(
            tisty_core::attach::digests(&two.here.data).contains_key(&kept.at),
            "{who}: kept here without being written down"
        );
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
        assert_eq!(
            (spent.lists, spent.fetches, spent.puts, spent.deletes),
            (1, 0, 0, 0),
            "{who}: a quiet round asks for one listing and touches nothing"
        );
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
        assert_eq!(moved.sent, 0, "{who}: nothing reached the cloud");
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

        let provider = remote.limits().provider.to_string();
        remote.fail_next(Hitch::Lost);
        let lost = one.carry(Way::Both, Holds::Everywhere);
        remote.fail_next(Hitch::Full);
        let full = one.carry(Way::Both, Holds::Everywhere);
        remote.fail_next(Hitch::Elsewhere {
            found: "otra@ejemplo.cl".into(),
        });
        let elsewhere = one.carry(Way::Both, Holds::Everywhere);

        let who = &setting.name;
        assert_eq!(
            lost,
            Err(Trouble::CloudLost {
                provider: provider.clone()
            }),
            "{who}"
        );
        assert_eq!(
            full,
            Err(Trouble::CloudFull {
                provider: provider.clone()
            }),
            "{who}"
        );
        assert_eq!(
            elsewhere,
            Err(Trouble::CloudElsewhere {
                provider,
                found: "otra@ejemplo.cl".into()
            }),
            "{who}"
        );
    }
}

#[test]
fn a_refusal_before_anything_went_up_is_an_error() {
    for setting in clouds() {
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        remote.fail_after(1, Hitch::Full);

        let refused = one.carry(Way::Both, Holds::Everywhere);

        assert!(
            matches!(refused, Err(Trouble::CloudFull { .. })),
            "{}: {refused:?}",
            setting.name
        );
        one.round(Way::Both);
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["lo de uno"], "{}", setting.name);
    }
}

#[test]
fn a_refusal_after_part_of_the_round_went_up_still_tells_what_the_round_did() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        remote.fail_after(2, Hitch::Full);

        let moved = one.round(Way::Both);

        assert!(moved.sent > 0, "{who}: something did go up");
        assert!(
            matches!(moved.refused, Some(Trouble::CloudFull { .. })),
            "{who}: {:?}",
            moved.refused
        );
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
fn an_upload_the_provider_defers_goes_with_the_next_round_and_the_original_stays() {
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

        let deferred = held_back.deferred.clone().unwrap();
        assert_eq!(deferred.reason, Reason::Limit, "{who}");
        assert!(
            deferred.names.contains(&kept.at),
            "{who}: what was left lost its name: {:?}",
            deferred.names
        );
        assert_eq!(held_back.let_go, Vec::<String>::new(), "{who}");
        assert!(
            one.held(&kept.at).is_some(),
            "{who}: the original is still here"
        );

        let sent = one.carry(Way::Both, Holds::Shared).unwrap();

        assert_eq!(sent.let_go, Vec::<String>::new(), "{who}");
        assert!(
            one.held(&kept.at).is_some(),
            "{who}: a cloud let go of a copy this machine keeps"
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

#[test]
fn the_index_survives_being_saved_and_read_back() {
    let room = tempfile::tempdir().unwrap();
    let mut index = super::index::Index {
        cursor: Some("42".to_string()),
        ..Default::default()
    };
    let seen = crate::Seen {
        name: "docs/dev_a-0001.md".to_string(),
        bytes: 7,
        hash: "md5:abc".to_string(),
        revision: "r9".to_string(),
    };
    index.tree.insert(
        seen.name.clone(),
        super::index::Mirrored {
            seen: seen.clone(),
            len: 7,
            stamp: 1_760_000_000_123_456_789_000,
            digest: "abc".to_string(),
        },
    );
    index
        .shelf
        .insert("attachments/ab/foto-1a2b3c4d.png".to_string(), seen);

    index.save(room.path()).unwrap();

    assert_eq!(super::index::Index::load(room.path()), index);
}

#[test]
fn names_that_are_not_history_or_documents_are_neither_fetched_nor_removed() {
    // Only a provider takes a name like «a*b.md»; a folder on Windows refuses to hold it.
    for setting in clouds().into_iter().filter(|one| one.name != BACKED) {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let stray = setting.room.path().join("stray");
        std::fs::write(&stray, b"not ours").unwrap();
        let names = [
            "Videos/viaje.mp4",
            "otra/nota.md",
            "docs/a*b.md",
            "docs/c?.md",
        ];
        for name in names {
            remote.put(name, &stray, Expect::Any).unwrap();
        }
        one.wrote("lo de uno");

        one.round(Way::Both);
        one.round(Way::Both);

        let tree = home_of(&setting, "dev_a").join("tree");
        assert!(
            !tree.join("Videos").exists() && !tree.join("otra").exists(),
            "{who}"
        );
        for name in names {
            assert!(
                remote.about(name).unwrap().is_some(),
                "{who}: {name} was removed"
            );
        }
    }
}

#[test]
fn a_cloud_that_never_got_the_stamp_does_not_leave_this_machine_unshaped() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        remote.fail_after(
            1,
            Hitch::Limited {
                wait: Duration::from_secs(1),
            },
        );
        assert!(one.round(Way::Both).deferred.is_some(), "{who}");
        std::fs::remove_dir_all(home_of(&setting, "dev_a")).unwrap();

        let again = one.carry(Way::Both, Holds::Everywhere);

        assert!(again.is_ok(), "{who}: {again:?}");
        assert!(remote.about("tisty.toml").unwrap().is_some(), "{who}");
    }
}

#[test]
fn an_attachment_that_fails_offline_does_not_hold_back_the_history_and_both_are_reported() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.kept("foto", b"los bytes de una foto");
        one.wrote("lo de uno");
        remote.fail_after(1, Hitch::Unreachable("sin red".into()));
        remote.fail_next(Hitch::Limited {
            wait: Duration::from_secs(4),
        });

        let moved = one.round(Way::Both);

        let deferred = moved.deferred.unwrap();
        assert_eq!(deferred.reason, Reason::Offline, "{who}");
        assert_eq!(deferred.retry_after, Some(Duration::from_secs(4)), "{who}");
        assert!(deferred.left >= 2, "{who}: {deferred:?}");
    }
}

#[test]
fn a_document_forgotten_while_a_round_runs_is_still_forgotten() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);
        std::fs::remove_file(one.here.data.join("docs").join(format!("{DOC}.md"))).unwrap();
        let mirrored = home_of(&setting, "dev_a").join("tree").join(doc_name());
        let running = super::lock::Round::take(&home_of(&setting, "dev_a")).unwrap();

        one.carrier.forget_paper(DOC);

        assert!(mirrored.exists(), "{who}: the round has the mirror");
        drop(running);
        one.round(Way::Both);
        assert!(!mirrored.exists(), "{who}");
        assert_eq!(remote.about(&doc_name()).unwrap(), None, "{who}");
    }
}

#[test]
fn a_forgetting_still_being_written_is_left_for_the_next_round() {
    let setting = Setting {
        name: "cloud over drive".to_string(),
        room: tempfile::tempdir().unwrap(),
        place: Place::Cloud(Arc::new(Fake::drive())),
    };
    let one = desk(&setting, "dev_a");
    one.filed(DOC, "# Plan\n\nuno\n");
    one.round(Way::Both);
    let home = home_of(&setting, "dev_a");
    std::fs::write(home.join(super::FORGOTTEN), format!("{DOC}\n")).unwrap();
    let writing = super::lock::Held::take(&home, super::lock::FORGETTING)
        .unwrap()
        .unwrap();

    one.round(Way::Both);

    assert!(
        home.join(super::FORGOTTEN).exists(),
        "the list was taken from under a writer, whose line would land where nobody reads"
    );
    drop(writing);
    one.round(Way::Both);
    assert!(!home.join(super::FORGOTTEN).exists());
}

#[test]
fn a_provider_that_gives_no_hashes_still_gets_each_file_once_and_a_quiet_round_sends_nothing() {
    let room = tempfile::tempdir().unwrap();
    let plain = Arc::new(Fake::dropbox().without_hashes());
    let setting = Setting {
        name: "cloud without hashes".to_string(),
        room,
        place: Place::Cloud(plain.clone()),
    };
    let (one, _) = pair(&setting);
    one.filed(DOC, "# Plan\n\nuno\n");
    one.wrote("lo de uno");

    one.round(Way::Both);

    let sent = plain.uploaded();
    let once: std::collections::BTreeSet<&String> = sent.iter().collect();
    assert_eq!(once.len(), sent.len(), "{sent:?}");
    one.round(Way::Both);
    plain.forget_counts();
    one.round(Way::Both);
    assert_eq!(plain.counts().sent, 0);
}

#[test]
fn work_that_must_not_fail_waits_for_a_running_round_to_finish() {
    let room = tempfile::tempdir().unwrap();
    let home = room.path().to_path_buf();
    let running = super::lock::Round::take(&home).unwrap().unwrap();
    let short = super::lock::Round::wait_for(&home, Duration::from_millis(100)).unwrap();
    assert!(short.is_none(), "a held lock was given away");
    let waiter = std::thread::spawn({
        let home = home.clone();
        move || {
            super::lock::Round::wait_for(&home, Duration::from_secs(10))
                .unwrap()
                .is_some()
        }
    });
    std::thread::sleep(Duration::from_millis(200));

    drop(running);

    assert!(waiter.join().unwrap(), "it never got the lock");
}

#[test]
fn what_a_dead_round_left_behind_is_swept_and_what_is_recent_is_not() {
    let room = tempfile::tempdir().unwrap();
    let docs = room.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let old = std::time::SystemTime::now() - Duration::from_secs(3 * 24 * 60 * 60);
    let part = docs.join("dev_x-0001.01ARZ3NDEKTSV4RRFFQ69G5FAV.7.part");
    let tmp = docs.join("dev_x-0001.4242.3.tmp");
    let recent = docs.join("dev_x-0002.01ARZ3NDEKTSV4RRFFQ69G5FAV.8.part");
    for at in [&part, &tmp, &recent] {
        std::fs::write(at, b"half").unwrap();
    }
    for at in [&part, &tmp] {
        std::fs::File::options()
            .write(true)
            .open(at)
            .unwrap()
            .set_modified(old)
            .unwrap();
    }

    super::mirror::sweep(room.path());

    assert!(!part.exists() && !tmp.exists());
    assert!(
        recent.exists(),
        "a round that may still be running keeps its file"
    );
}

fn each_kind(shape: impl Fn(Fake) -> Fake) -> Vec<(Setting, Arc<Fake>)> {
    [Fake::drive(), Fake::onedrive(), Fake::dropbox()]
        .into_iter()
        .map(|fake| {
            let fake = Arc::new(shape(fake));
            let setting = Setting {
                name: format!("cloud over {}", fake.who()),
                room: tempfile::tempdir().unwrap(),
                place: Place::Cloud(fake.clone()),
            };
            (setting, fake)
        })
        .collect()
}

#[test]
fn a_download_that_comes_short_is_never_installed_and_the_next_round_takes_it_whole() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.filed(DOC, "# Plan\n\nlo escrito\n");
        one.wrote("lo de uno");
        one.round(Way::Both);
        fake.truncate_next(usize::MAX);

        let short = two.round(Way::Pull);

        assert_eq!(short.brought, 0, "{who}: {short:?}");
        assert_eq!(two.doc(DOC), None, "{who}: half a body was installed");
        fake.truncate_next(0);
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["lo de uno"], "{who}");
        assert_eq!(two.doc(DOC), one.doc(DOC), "{who}");
    }
}

#[test]
fn a_file_that_cannot_come_down_owes_itself_without_holding_the_cursor_back() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.filed(DOC, "# Plan\n\nlo escrito\n");
        one.wrote("lo de uno");
        one.round(Way::Both);
        fake.truncate_next(usize::MAX);

        two.round(Way::Pull);

        let index = super::index::Index::load(&home_of(&setting, "dev_b"));
        assert!(
            index.cursor.is_some(),
            "{who}: the cursor stayed behind the file"
        );
        assert!(
            !index.owed.is_empty(),
            "{who}: what could not come down was forgotten"
        );
        fake.truncate_next(0);
        one.wrote("lo de después");
        one.round(Way::Both);
        two.round(Way::Pull);
        assert_eq!(two.doc(DOC), one.doc(DOC), "{who}");
        assert_eq!(two.titles().len(), 2, "{who}: {:?}", two.titles());
        let index = super::index::Index::load(&home_of(&setting, "dev_b"));
        assert!(index.owed.is_empty(), "{who}: {:?}", index.owed.keys());
    }
}

#[test]
fn a_listing_that_lags_behind_loses_nothing_and_sends_nothing_twice() {
    for (setting, fake) in each_kind(|fake| fake.lagging(2)) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.filed(DOC, "# Plan\n\nlo escrito\n");
        one.wrote("lo de uno");

        for _ in 0..4 {
            one.round(Way::Both);
            two.round(Way::Both);
        }

        assert_eq!(two.titles(), ["lo de uno"], "{who}");
        assert_eq!(two.doc(DOC), one.doc(DOC), "{who}");
        assert!(
            fake.about(&doc_name()).unwrap().is_some(),
            "{who}: a write the listing did not show yet was taken for one that went away"
        );
        fake.forget_counts();
        one.round(Way::Both);
        two.round(Way::Both);
        assert_eq!(
            fake.counts().sent,
            0,
            "{who}: what was already up went up again"
        );
    }
}

#[test]
fn a_conflict_copy_a_desktop_client_left_is_neither_taken_in_nor_removed() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.filed(DOC, "# Plan\n\nlo escrito\n");
        one.wrote("lo de uno");
        one.round(Way::Both);
        let copies = [
            fake.conflict_copy(&doc_name()),
            fake.conflict_copy(&format!("{STORE}/dev_a/active.tisty")),
        ];

        two.round(Way::Both);
        one.round(Way::Both);

        assert_eq!(two.titles(), ["lo de uno"], "{who}");
        assert_eq!(two.doc(DOC), one.doc(DOC), "{who}");
        for copy in &copies {
            assert!(
                fake.about(copy).unwrap().is_some(),
                "{who}: {copy} was removed"
            );
            assert!(
                !home_of(&setting, "dev_b").join("tree").join(copy).exists(),
                "{who}: {copy} was taken in"
            );
        }
    }
}

#[test]
fn a_machine_on_the_folder_and_one_on_a_cloud_over_the_same_files_keep_one_history() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    let folder = Setting {
        name: "folder".to_string(),
        room: tempfile::tempdir().unwrap(),
        place: Place::Folder(shared.clone()),
    };
    let cloud = Setting {
        name: BACKED.to_string(),
        room: tempfile::tempdir().unwrap(),
        place: Place::Cloud(Arc::new(crate::folder_backed::FolderBacked::at(&shared))),
    };
    let (one, two) = (desk(&folder, "dev_a"), desk(&cloud, "dev_b"));
    one.vouches_for(&two);
    two.vouches_for(&one);
    one.filed(DOC, "# Plan\n\nlo escrito en la carpeta\n");
    one.wrote("lo de uno");

    one.round(Way::Both);
    two.round(Way::Both);
    two.wrote("lo de dos");
    two.edited(DOC, "# Plan\n\nlo escrito en la nube\n");
    two.round(Way::Both);
    one.round(Way::Both);

    assert_eq!(one.titles(), ["lo de dos", "lo de uno"]);
    assert_eq!(two.titles(), one.titles());
    assert_eq!(
        one.doc(DOC).as_deref(),
        Some("# Plan\n\nlo escrito en la nube\n")
    );
    let tree = home_of(&cloud, "dev_b").join("tree");
    for name in [
        "store/dev_a/active.tisty",
        "store/dev_b/active.tisty",
        "docs/dev_a-0001.md",
    ] {
        assert_eq!(
            std::fs::read(tree.join(name)).unwrap(),
            std::fs::read(shared.join(name)).unwrap(),
            "the mirror of {name} is not the folder byte for byte"
        );
    }
}

#[test]
fn a_sibling_of_a_kind_this_build_has_no_name_for_still_travels() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        let later = one.here.data.join(STORE).join("dev_a").join("active.later");
        std::fs::write(&later, b"lo que escribe una version futura").unwrap();

        one.round(Way::Both);
        two.round(Way::Both);

        assert_eq!(
            std::fs::read(two.here.data.join(STORE).join("dev_a").join("active.later")).ok(),
            Some(b"lo que escribe una version futura".to_vec()),
            "{who}: a sibling a later build wrote was left behind"
        );
    }
}

#[test]
fn another_machine_s_write_stirs_the_cloud_without_a_byte_of_content_coming_down() {
    for (setting, fake) in each_kind(|fake| fake.polled_every(Duration::ZERO)) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        two.round(Way::Both);
        let still = two.carrier.stirring();
        assert_eq!(
            still,
            two.carrier.stirring(),
            "{who}: a quiet cloud stirred"
        );

        one.wrote("lo nuevo de uno");
        one.round(Way::Both);
        fake.forget_counts();

        assert_ne!(
            still,
            two.carrier.stirring(),
            "{who}: a write elsewhere went unnoticed"
        );
        let spent = fake.counts();
        assert_eq!(
            (spent.received, spent.fetches),
            (0, 0),
            "{who}: looking brought content down"
        );
        assert_eq!(spent.lists, 1, "{who}");
    }
}

#[test]
fn a_cloud_asked_again_before_its_provider_s_interval_is_not_asked_at_all() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        fake.forget_counts();

        for _ in 0..3 {
            desk(&setting, "dev_a").carrier.stirring();
        }

        assert_eq!(fake.counts().lists, 1, "{who}: every glance cost a request");
    }
}

#[test]
fn what_this_machine_sent_up_is_not_taken_for_a_stir_from_elsewhere() {
    for (setting, _) in each_kind(|fake| fake.polled_every(Duration::ZERO)) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.filed(DOC, "# Plan\n\nuno\n");
        one.wrote("lo de uno");
        one.round(Way::Both);
        let still = desk(&setting, "dev_a").carrier.stirring();

        one.wrote("lo nuevo de uno");
        one.edited(
            DOC,
            "# Plan

dos
",
        );
        one.round(Way::Both);

        assert_eq!(
            still,
            desk(&setting, "dev_a").carrier.stirring(),
            "{who}: its own write set off another round"
        );
    }
}

#[test]
fn an_attachment_only_the_shared_place_holds_is_brought_when_it_is_asked_for() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        one.round(Way::Both);
        if let Place::Cloud(_) = setting.place {
            two.round(Way::Pull);
        }

        let at = two.carrier.brought(&two.here, &kept.at).unwrap();

        let crate::Brought::At(at) = at else {
            panic!("{who}: nothing was brought: {at:?}");
        };
        assert_eq!(
            std::fs::read(&at).unwrap(),
            b"los bytes de una foto",
            "{who}"
        );
        match &setting.place {
            Place::Folder(_) => assert!(
                two.carrier
                    .lends_from()
                    .is_some_and(|lent| at.starts_with(lent)),
                "{who}: what was brought lies outside what the window may show"
            ),
            Place::Cloud(_) => assert_eq!(
                Some(at),
                tisty_core::attach::resolve(&kept.at, &two.here.data).ok(),
                "{who}: what was opened did not stay among this machine's attachments"
            ),
        }
        assert_eq!(
            two.carrier
                .brought(&two.here, "docs/dev_a-0001.md")
                .unwrap(),
            crate::Brought::Absent,
            "{who}: something that is not an attachment was handed out"
        );
        if let Place::Cloud(remote) = &setting.place {
            remote.forget_counts();
            two.carrier.brought(&two.here, &kept.at).unwrap();
            assert_eq!(remote.counts().fetches, 0, "{who}: brought down twice");
            let other = one.kept("otra", b"otra foto que aun no baja");
            one.store().append(other.told()).unwrap();
            one.round(Way::Both);
            two.round(Way::Pull);
            remote.fail_next(Hitch::Unreachable("sin red".into()));
            assert_eq!(
                two.carrier.brought(&two.here, &other.at).unwrap(),
                crate::Brought::Later,
                "{who}: a cut connection was told as something that is not there"
            );
        }
    }
}

#[test]
fn an_attachment_whose_bytes_changed_up_there_is_never_brought() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        one.round(Way::Both);
        let swapped = setting.room.path().join("swapped");
        std::fs::write(&swapped, b"otros bytes, otra foto").unwrap();
        remote.put(&kept.at, &swapped, Expect::Any).unwrap();
        two.round(Way::Pull);

        let refused = two.carrier.brought(&two.here, &kept.at);

        assert!(refused.is_err(), "{who}: {refused:?}");
        assert!(two.held(&kept.at).is_none(), "{who}");
        let shelf = tisty_core::attach::resolve(&kept.at, &two.here.data)
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let left: Vec<_> = std::fs::read_dir(&shelf)
            .map(|all| {
                all.filter_map(|one| one.ok())
                    .map(|one| one.file_name())
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            left.is_empty(),
            "{who}: {left:?} were left where the window looks"
        );
        assert_eq!(
            two.carrier
                .brought(&two.here, "attachments/ab/nada-0123456789.png")
                .unwrap(),
            crate::Brought::Later,
            "{who}: no history names it yet"
        );
        assert_eq!(
            two.carrier.brought(&two.here, "../fuera.png").unwrap(),
            crate::Brought::Absent,
            "{who}"
        );
    }
}

fn spent_today(setting: &Setting, device: &str, units: u64) {
    let home = home_of(setting, device);
    let day = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / (24 * 60 * 60);
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(
        home.join("budget.json"),
        format!("{{\"day\":{day},\"units\":{units},\"bytes\":0}}"),
    )
    .unwrap();
}

fn units_spent(setting: &Setting, device: &str) -> u64 {
    let said = std::fs::read_to_string(home_of(setting, device).join("budget.json")).unwrap();
    let said: serde_json::Value = serde_json::from_str(&said).unwrap();
    said["units"].as_u64().unwrap()
}

#[test]
fn history_goes_up_on_its_reserve_once_attachments_have_spent_their_share() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.wrote("lo de uno");
        spent_today(&setting, "dev_a", fake.limits().units_a_day / 100 * 95);

        let moved = one.round(Way::Both);

        assert_eq!(
            moved.deferred.map(|one| one.reason),
            Some(Reason::Budget),
            "{who}"
        );
        assert!(
            fake.about(&kept.at).unwrap().is_none(),
            "{who}: an attachment ate into what history keeps"
        );
        two.round(Way::Pull);
        assert_eq!(
            two.titles(),
            ["lo de uno"],
            "{who}: history waited for attachments"
        );
    }
}

#[test]
fn a_day_spent_to_the_last_unit_moves_nothing_and_says_why() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        spent_today(&setting, "dev_a", fake.limits().units_a_day);
        fake.forget_counts();

        let moved = one.round(Way::Both);

        assert_eq!(
            moved.deferred.map(|one| one.reason),
            Some(Reason::Budget),
            "{who}"
        );
        assert_eq!(fake.counts().requests, 0, "{who}: a spent day still asked");
    }
}

#[test]
fn what_the_person_asks_for_comes_down_past_a_spent_day_and_is_counted() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", b"los bytes de una foto");
        one.store().append(kept.told()).unwrap();
        one.wrote("con foto");
        one.round(Way::Both);
        two.round(Way::Pull);
        let all = fake.limits().units_a_day;
        spent_today(&setting, "dev_b", all);

        let brought = two.carrier.brought(&two.here, &kept.at).unwrap();

        assert!(
            matches!(brought, crate::Brought::At(_)),
            "{who}: {brought:?}"
        );
        assert!(
            units_spent(&setting, "dev_b") > all,
            "{who}: it was not counted"
        );
    }
}

#[test]
fn a_new_day_starts_with_nothing_spent() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        let home = home_of(&setting, "dev_a");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            home.join("budget.json"),
            format!(
                "{{\"day\":0,\"units\":{},\"bytes\":0}}",
                fake.limits().units_a_day
            ),
        )
        .unwrap();

        let moved = one.round(Way::Both);

        assert_eq!(
            moved.deferred, None,
            "{who}: yesterday's spending held today back"
        );
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["lo de uno"], "{who}");
    }
}

#[test]
fn what_a_round_spends_is_still_counted_by_the_next_command() {
    for (setting, _) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        let first = units_spent(&setting, "dev_a");

        desk(&setting, "dev_a").round(Way::Both);

        assert!(first > 0, "{who}: a round spent nothing");
        assert!(
            units_spent(&setting, "dev_a") > first,
            "{who}: the count started over"
        );
    }
}

#[test]
fn a_cursor_the_provider_has_expired_is_read_whole_again_instead_of_wedging_the_sync() {
    for (setting, fake) in each_kind(|fake| fake.expiring_cursors()) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        two.round(Way::Both);
        fake.forget_feed();
        one.wrote("lo nuevo de uno");
        one.round(Way::Both);

        let moved = two.carry(Way::Both, Holds::Everywhere);

        assert!(moved.is_ok(), "{who}: {moved:?}");
        assert_eq!(two.titles(), ["lo de uno", "lo nuevo de uno"], "{who}");
    }
}

#[test]
fn an_attachment_the_cloud_has_not_listed_yet_is_later_not_gone() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo de uno");
        one.round(Way::Both);
        two.round(Way::Both);
        fake.forget_counts();

        let asked = two
            .carrier
            .brought(&two.here, "attachments/ab/nada-0123456789.png")
            .unwrap();

        assert_eq!(asked, crate::Brought::Later, "{who}");
        assert_eq!(
            fake.counts().requests,
            0,
            "{who}: asked the provider for what it never listed"
        );
    }
}

#[path = "converge_test.rs"]
mod converge;

#[path = "shift_test.rs"]
mod shift;

#[path = "freed_test.rs"]
mod freed;

#[test]
fn a_round_held_back_twice_names_the_reason_the_person_can_do_least_about() {
    let owed = Some(crate::Deferred {
        reason: Reason::Changed,
        left: 1,
        retry_after: None,
        names: vec!["attachments/ab/a.png".into()],
    });
    let spent = Some(crate::Deferred {
        reason: Reason::Budget,
        left: 2,
        retry_after: None,
        names: vec!["attachments/cd/b.pdf".into(), "attachments/ef/c.mp4".into()],
    });

    let said = super::merged(owed.clone(), spent.clone()).unwrap();

    assert_eq!(
        said.reason,
        Reason::Budget,
        "a spent day was told as a file that changed"
    );
    assert_eq!(said.left, 3);
    assert_eq!(
        said.names,
        [
            "attachments/ab/a.png",
            "attachments/cd/b.pdf",
            "attachments/ef/c.mp4"
        ],
        "what was left lost its names on the way"
    );
    assert_eq!(super::merged(spent, owed).unwrap().reason, Reason::Budget);
}
