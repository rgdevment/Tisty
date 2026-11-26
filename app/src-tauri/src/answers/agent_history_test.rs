use super::*;

#[test]
fn each_copy_names_its_file_and_keeps_its_order() {
    let when = jiff::Timestamp::from_second(1_800_000_000).unwrap();
    let kept = |n: u64, at: Option<jiff::Timestamp>| agents::Kept {
        n,
        bytes: 10 * n,
        pinned: n == 1,
        at,
    };
    let told = copies(vec![FileKept {
        at: "/home/rina/.claude/CLAUDE.md".into(),
        named: "Claude".into(),
        kept: vec![kept(2, Some(when)), kept(1, None)],
    }]);
    assert_eq!(
        told,
        [
            Copy {
                at: "/home/rina/.claude/CLAUDE.md".into(),
                named: "Claude".into(),
                n: 2,
                bytes: 20,
                pinned: false,
                when: Some(when.to_string()),
            },
            Copy {
                at: "/home/rina/.claude/CLAUDE.md".into(),
                named: "Claude".into(),
                n: 1,
                bytes: 10,
                pinned: true,
                when: None,
            },
        ]
    );
}
