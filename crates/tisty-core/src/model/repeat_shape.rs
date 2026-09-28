use super::*;

#[test]
fn what_was_written_before_until_existed_still_reads() {
    let old = r#"{"from":"due","each":{"every":1,"unit":"day"}}"#;
    let read: Repeat = serde_json::from_str(old).expect("old shape");

    assert_eq!(read.from, From::Due);
    assert_eq!(read.each.every, 1);
    assert_eq!(read.until, None);
    assert_eq!(serde_json::to_string(&read).unwrap(), old);
}
