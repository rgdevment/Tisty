use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use tisty_carrier::{
    Brought, Carrier, Elsewhere, Here, Holding, Keep, Kin, LetGo, Moved, Round, Signed, Stitched,
    Told, Trouble,
};

use super::*;

/// Answers only what opening an attachment asks; nothing else here is ever reached.
struct Opening(Brought);

impl Carrier for Opening {
    fn place(&self) -> Option<&Path> {
        None
    }
    fn lends_from(&self) -> Option<PathBuf> {
        None
    }
    fn frees_space(&self) -> bool {
        true
    }
    fn brought(&self, _: &Here, _: &str) -> Result<Brought, Trouble> {
        Ok(self.0.clone())
    }
    fn shelf(&self) -> Option<std::collections::BTreeMap<String, u64>> {
        unreachable!()
    }
    fn reachable(&self) -> bool {
        unreachable!()
    }
    fn theirs(&self) -> Option<String> {
        unreachable!()
    }
    fn been_here(&self, _: &Here) -> bool {
        unreachable!()
    }
    fn kin(&self, _: &Here) -> Kin {
        unreachable!()
    }
    fn signed(&self) -> Signed {
        unreachable!()
    }
    fn stirring(&self) -> u64 {
        unreachable!()
    }
    fn unclaimed(&self) -> Holding {
        unreachable!()
    }
    fn carry(&self, _: &Here, _: Round) -> Result<Moved, Trouble> {
        unreachable!()
    }
    fn bring_home(&self, _: &Here, _: Round) -> Result<Moved, Trouble> {
        unreachable!()
    }
    fn could_free(&self, _: &Here, _: SystemTime) -> LetGo {
        unreachable!()
    }
    fn set_free(&self, _: &Here, _: SystemTime, _: Told) -> Result<LetGo, Trouble> {
        unreachable!()
    }
    fn stitch(
        &self,
        _: &Here,
        _: Option<tisty_core::signing::SigningKey>,
    ) -> Result<Stitched, Trouble> {
        unreachable!()
    }
    fn let_go(&self, _: &Here, _: u64, _: Elsewhere, _: Told) -> Result<LetGo, Trouble> {
        unreachable!()
    }
    fn paper_waiting(&self, _: &str) -> bool {
        unreachable!()
    }
    fn paper_print(&self, _: &str) -> Option<String> {
        unreachable!()
    }
    fn both_papers(&self, _: &Here, _: &str) -> Result<(String, String), Trouble> {
        unreachable!()
    }
    fn settle(&self, _: &Here, _: &str, _: Keep) -> Result<Option<String>, Trouble> {
        unreachable!()
    }
    fn forget_paper(&self, _: &str) {
        unreachable!()
    }
}

const ASKED: &str = "attachments/ab/foto-0123456789.png";

fn looking(data: &Path, answer: Brought) -> Where {
    Where {
        data: data.to_path_buf(),
        avowed: None,
        shared: None,
        reached: data.join(tisty_core::lately::USED),
        cloud: Some((
            Arc::new(Opening(answer)),
            Here {
                data: data.to_path_buf(),
                aside: data.join("cache"),
                device: "dev_a".into(),
            },
        )),
        unvouched: Default::default(),
        weight: None,
    }
}

#[test]
fn an_attachment_not_here_is_asked_of_the_cloud_and_served_from_where_it_landed() {
    let data = tempfile::tempdir().unwrap();
    let landed = data.path().join("landed.png");
    let at = looking(data.path(), Brought::At(landed.clone()));

    assert_eq!(handed_over(ASKED, &at).unwrap(), landed);
    assert!(at.unvouched.lock().unwrap().is_none());
}

#[test]
fn what_came_down_with_no_history_behind_it_is_kept_to_be_written_down() {
    let data = tempfile::tempdir().unwrap();
    let landed = data.path().join("landed.png");
    let at = looking(
        data.path(),
        Brought::Unvouched {
            at: landed.clone(),
            sha256: "0123456789".into(),
            bytes: 7,
        },
    );

    assert_eq!(handed_over(ASKED, &at).unwrap(), landed);
    let held = at.unvouched.lock().unwrap().take().unwrap();
    assert_eq!((held.at.as_str(), held.bytes), (ASKED, 7));
}

#[test]
fn an_attachment_the_cloud_does_not_have_yet_is_said_to_be_on_its_way() {
    let data = tempfile::tempdir().unwrap();

    let absent = handed_over(ASKED, &looking(data.path(), Brought::Absent));

    assert!(absent.is_err_and(|refused| format!("{refused:?}").contains("attachNotYet")));
}

#[test]
fn a_moment_inside_an_attachment_names_the_attachment_itself() {
    assert_eq!(plain(&format!("{ASKED}#t=30")), ASKED);
    assert_eq!(plain(&format!("{ASKED}?page=2")), ASKED);
    assert_eq!(plain(ASKED), ASKED);
}

#[test]
fn an_attachment_let_go_of_here_still_weighs_what_the_log_says() {
    let data = tempfile::tempdir().unwrap();

    assert_eq!(weight_of(ASKED, data.path(), None, Some(42)), Some(42));
    assert_eq!(weight_of(ASKED, data.path(), None, None), None);
    let at = tisty_core::attach::resolve(ASKED, data.path()).unwrap();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, b"siete b").unwrap();
    assert_eq!(weight_of(ASKED, data.path(), None, Some(42)), Some(7));
}

#[test]
fn an_attachment_the_cloud_cannot_send_yet_says_so_in_words_of_the_cloud() {
    let data = tempfile::tempdir().unwrap();

    let later = handed_over(ASKED, &looking(data.path(), Brought::Later));

    assert!(later.is_err_and(|one| format!("{one:?}").contains("cloudLater")));
}

#[test]
fn an_attachment_found_nowhere_is_said_to_be_on_its_way_not_unreadable() {
    let data = tempfile::tempdir().unwrap();
    let mut at = looking(data.path(), Brought::Absent);
    at.cloud = None;

    let refused = handed_over(ASKED, &at).unwrap_err();

    assert_eq!(refused.code, "attachNotYet");
    assert_eq!(refused.name.as_deref(), Some(ASKED));
}
