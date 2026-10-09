use std::path::PathBuf;
use std::sync::Arc;

use tisty_core::event::{DeviceId, DocAdd, TaskAdd};
use tisty_core::{Op, Store};
use ulid::Ulid;

use crate::fakes::{self, Counting};
use crate::{
    Cloud, Folder, Here, Hitch, Moved, Reason, Remote, Round, STORE, Shared, Trouble, Way,
};
use tisty_core::config::Holds;

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

fn counting(setting: &Setting) -> &Arc<dyn Counting> {
    match &setting.place {
        Place::Cloud(remote) => remote,
        Place::Folder(_) => unreachable!("only a cloud counts requests"),
    }
}

impl Desk {
    fn store(&self) -> Store {
        Store::open(
            self.here.data.join(STORE),
            DeviceId(self.here.device.clone()),
        )
        .unwrap()
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
        let (one, two) = (desk(&setting, "dev_a"), desk(&setting, "dev_b"));
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
        let one = desk(&setting, "dev_a");
        for title in ["primera", "segunda", "tercera"] {
            one.wrote(title);
            one.round(Way::Both);
        }
        let two = desk(&setting, "dev_b");

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
        let (one, two) = (desk(&setting, "dev_a"), desk(&setting, "dev_b"));
        one.wrote("lo de uno");
        one.round(Way::Both);
        two.round(Way::Both);
        one.round(Way::Both);

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
        let (one, two) = (desk(&setting, "dev_a"), desk(&setting, "dev_b"));
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
    }
}

#[test]
fn an_attachment_goes_up_and_comes_down() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = (desk(&setting, "dev_a"), desk(&setting, "dev_b"));
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
        let one = desk(&setting, "dev_a");
        one.filed(DOC, "# Plan\n\nuno\n");
        one.round(Way::Both);
        assert!(one.carrier.paper_waiting(DOC), "{who}");

        one.carrier.forget_paper(DOC);

        assert!(!one.carrier.paper_waiting(DOC), "{who}");
    }
}

#[test]
fn a_quiet_cloud_round_asks_once_and_moves_no_content() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let one = desk(&setting, "dev_a");
        one.wrote("lo de uno");
        one.round(Way::Both);
        one.round(Way::Both);
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
        let (one, two) = (desk(&setting, "dev_a"), desk(&setting, "dev_b"));
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
        let one = desk(&setting, "dev_a");
        one.wrote("lo de uno");
        one.round(Way::Both);
        one.wrote("lo de despues");
        remote.fail_after(
            1,
            Hitch::Limited {
                wait: std::time::Duration::from_secs(9),
            },
        );

        let moved = one.round(Way::Both);

        let deferred = moved.deferred.unwrap();
        assert_eq!(deferred.reason, Reason::Limit, "{who}");
        assert_eq!(
            deferred.retry_after,
            Some(std::time::Duration::from_secs(9)),
            "{who}"
        );
        let later = one.round(Way::Both);
        assert_eq!(later.deferred, None, "{who}");
        assert_eq!(
            desk(&setting, "dev_b").carrier.theirs(),
            one.carrier.theirs(),
            "{who}"
        );
    }
}

#[test]
fn a_provider_that_will_not_let_us_in_is_a_refusal_not_a_deferral() {
    for setting in clouds() {
        let remote = counting(&setting);
        let one = desk(&setting, "dev_a");
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
fn two_rounds_never_run_at_once() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let one = desk(&setting, "dev_a");
        one.wrote("lo de uno");
        let held = super::lock::Round::take(&setting.room.path().join("dev_a").join("cloud"));
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
        let one = desk(&setting, "dev_a");
        one.wrote("lo de uno");
        one.round(Way::Both);
        let home = setting.room.path().join("dev_a").join("cloud");
        std::fs::remove_dir_all(&home).unwrap();

        let moved = one.round(Way::Both);

        assert_eq!(moved.deferred, None, "{who}");
        assert!(home.join("tree").join("tisty.toml").is_file(), "{who}");
        assert_eq!(one.titles(), ["lo de uno"], "{who}");
        assert!(one.carrier.theirs().is_some(), "{who}");
    }
}

#[test]
fn a_lost_index_costs_no_download_when_the_mirror_is_intact() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let one = desk(&setting, "dev_a");
        one.wrote("lo de uno");
        one.round(Way::Both);
        let home = setting.room.path().join("dev_a").join("cloud");
        std::fs::remove_file(home.join("index.json")).unwrap();
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
fn an_upload_the_provider_defers_keeps_the_original_and_the_next_round_sends_it() {
    for setting in clouds().into_iter().take(1) {
        let who = &setting.name;
        let remote = counting(&setting);
        let one = desk(&setting, "dev_a");
        one.wrote("lo de uno");
        one.round(Way::Both);
        let big = vec![7u8; usize::try_from(tisty_core::attach::COPIED_UP_TO).unwrap() + 1024];
        let kept = one.kept("grande", &big);
        remote.fail_after(
            1,
            Hitch::Limited {
                wait: std::time::Duration::from_secs(1),
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
        let one = desk(&setting, "dev_a");
        let litter = setting.room.path().join("litter");
        std::fs::write(&litter, b"a half-written thing").unwrap();
        let name = "docs/dev_x-0001.01ARZ3NDEKTSV4RRFFQ69G5FAV.3.part";
        remote.put(name, &litter, crate::Expect::Absent).unwrap();
        one.wrote("lo de uno");

        one.round(Way::Both);
        one.round(Way::Both);

        let home = setting.room.path().join("dev_a").join("cloud");
        assert!(
            !home
                .join("tree")
                .join("docs")
                .join("dev_x-0001.01ARZ3NDEKTSV4RRFFQ69G5FAV.3.part")
                .exists(),
            "{who}"
        );
        assert!(
            remote.about(name).unwrap().is_some(),
            "{who}: it is not ours to remove"
        );
    }
}
