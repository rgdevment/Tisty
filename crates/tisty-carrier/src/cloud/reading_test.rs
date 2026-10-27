use super::*;

fn along(done: usize, whole: usize) -> crate::Reached {
    crate::Reached::Along {
        stage: crate::Stage::Log,
        done,
        whole,
    }
}

#[test]
fn a_read_under_way_says_how_far_it_came_and_says_nothing_once_it_let_go() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(reading(home.path()), None);
    {
        let mut noting = Noting::new(home.path());
        noting.heard(&along(3, 40));
        let said = reading(home.path()).unwrap();
        assert_eq!((said.done, said.whole), (3, 40));
        noting.heard(&along(4, 40));
        assert_eq!(
            reading(home.path()).unwrap().done,
            3,
            "told at most twice a second"
        );
    }
    assert_eq!(reading(home.path()), None);
}

#[test]
fn a_read_that_finished_or_went_quiet_long_ago_is_not_under_way() {
    let home = tempfile::tempdir().unwrap();
    let write = |said: Reading| {
        std::fs::write(home.path().join(FILE), serde_json::to_vec(&said).unwrap()).unwrap();
    };
    write(Reading {
        done: 40,
        whole: 40,
        at: policy::now(),
    });
    assert_eq!(reading(home.path()), None);
    write(Reading {
        done: 3,
        whole: 40,
        at: policy::now() - FRESH_FOR - 1,
    });
    assert_eq!(reading(home.path()), None);
    write(Reading {
        done: 3,
        whole: 40,
        at: policy::now(),
    });
    assert!(reading(home.path()).is_some());
}
