use super::{Standing, print_of, standing_of};
use tisty_carrier::folio_limits::FolioMaker;
use tisty_contracts::Verdict;

fn maker() -> FolioMaker {
    FolioMaker {
        root: "R".repeat(43),
        name: Some("rgdevment".into()),
        domain: Some("rgdevment.dev".into()),
    }
}

fn other() -> String {
    "O".repeat(43)
}

fn never(_: &str, _: &str) -> bool {
    false
}

#[test]
fn the_makers_root_is_verified_while_its_signature_still_speaks_whatever_its_alias() {
    let maker = maker();
    let verified = Standing::Verified {
        maker: "rgdevment".into(),
        domain: Some("rgdevment.dev".into()),
    };
    assert_eq!(
        standing_of(
            &maker.root,
            Verdict::Holds,
            Some("anything"),
            Some(&maker),
            never
        ),
        verified
    );
    assert_eq!(
        standing_of(&maker.root, Verdict::Retired, None, Some(&maker), never),
        verified
    );
    assert_eq!(
        standing_of(
            &maker.root,
            Verdict::Doubtful,
            Some("rgdevment"),
            Some(&maker),
            never
        ),
        Standing::TakenBack
    );
}

#[test]
fn another_root_going_by_the_makers_name_imitates_it() {
    let maker = maker();
    assert_eq!(
        standing_of(
            &other(),
            Verdict::Holds,
            Some("rgdevrnent"),
            Some(&maker),
            never
        ),
        Standing::Imitates
    );
    assert_eq!(
        standing_of(
            &other(),
            Verdict::Holds,
            Some("rgdevment"),
            Some(&maker),
            |_, _| true
        ),
        Standing::Imitates
    );
}

#[test]
fn any_other_name_no_name_or_no_named_maker_is_plain() {
    let maker = maker();
    let nameless = FolioMaker {
        root: maker.root.clone(),
        ..FolioMaker::default()
    };
    assert_eq!(
        standing_of(&other(), Verdict::Holds, Some("Ana"), Some(&maker), never),
        Standing::Plain
    );
    assert_eq!(
        standing_of(&other(), Verdict::Holds, None, Some(&maker), never),
        Standing::Plain
    );
    assert_eq!(
        standing_of(&other(), Verdict::Holds, Some(""), Some(&maker), never),
        Standing::Plain
    );
    assert_eq!(
        standing_of(
            &maker.root,
            Verdict::Holds,
            Some("rgdevment"),
            Some(&nameless),
            never
        ),
        Standing::Plain
    );
    assert_eq!(
        standing_of(&other(), Verdict::Holds, Some("rgdevment"), None, never),
        Standing::Plain
    );
}

#[test]
fn a_domain_alias_is_checked_only_by_its_own_root_and_never_on_a_doubtful_signature() {
    let maker = maker();
    let only_ana = |root: &str, domain: &str| root == other() && domain == "ana.dev";
    assert_eq!(
        standing_of(
            &other(),
            Verdict::Holds,
            Some("ana.dev"),
            Some(&maker),
            only_ana
        ),
        Standing::Checked {
            domain: "ana.dev".into()
        }
    );
    assert_eq!(
        standing_of(&other(), Verdict::Holds, Some("ana.dev"), None, only_ana),
        Standing::Checked {
            domain: "ana.dev".into()
        }
    );
    assert_eq!(
        standing_of(
            &"B".repeat(43),
            Verdict::Holds,
            Some("ana.dev"),
            Some(&maker),
            only_ana
        ),
        Standing::Plain
    );
    assert_eq!(
        standing_of(
            &other(),
            Verdict::Doubtful,
            Some("ana.dev"),
            Some(&maker),
            only_ana
        ),
        Standing::Plain
    );
}

#[test]
fn a_party_reads_as_its_alias_beside_how_it_stands() {
    let party = super::Party {
        name: "rgdevrnent".into(),
        print: "70686 67560 35161 46791".into(),
        standing: Standing::Verified {
            maker: "rgdevment".into(),
            domain: Some("rgdevment.dev".into()),
        },
    };
    assert_eq!(
        serde_json::to_value(&party).unwrap(),
        serde_json::json!({
            "name": "rgdevrnent",
            "print": "70686 67560 35161 46791",
            "standing": "verified",
            "maker": "rgdevment",
            "domain": "rgdevment.dev"
        })
    );
}

#[test]
fn another_root_going_by_the_makers_domain_imitates_it_too() {
    let maker = maker();
    for alias in ["rgdevment.dev", "RG-DEVMENT.DEV"] {
        assert_eq!(
            standing_of(
                &other(),
                Verdict::Holds,
                Some(alias),
                Some(&maker),
                |_, _| true
            ),
            Standing::Imitates,
            "{alias}"
        );
    }
}

#[test]
fn a_maker_with_no_name_goes_by_its_domain_once() {
    let nameless = FolioMaker {
        root: "R".repeat(43),
        name: None,
        domain: Some("rgdevment.dev".into()),
    };
    assert_eq!(
        standing_of(
            &nameless.root,
            Verdict::Holds,
            Some("anything"),
            Some(&nameless),
            never
        ),
        Standing::Verified {
            maker: "rgdevment.dev".into(),
            domain: None
        }
    );
    assert_eq!(
        standing_of(
            &other(),
            Verdict::Holds,
            Some("rgdevment.dev"),
            Some(&nameless),
            never
        ),
        Standing::Imitates
    );
}

#[test]
fn a_root_prints_as_the_fingerprint_its_person_sees_in_security() {
    let person = tisty_share_proto::seed::Seed::from_bytes(&[7; 32])
        .root()
        .person();

    let print = print_of(&person.shown());

    assert_eq!(print, person.fingerprint());
    let groups: Vec<&str> = print.split(' ').collect();
    assert_eq!(groups.len(), 4, "{print}");
    assert!(
        groups
            .iter()
            .all(|five| five.len() == 5 && five.bytes().all(|b| b.is_ascii_digit())),
        "{print}"
    );
}

#[test]
fn a_root_that_is_no_key_prints_nothing() {
    assert_eq!(print_of("not a key"), "");
}
