use super::*;

#[test]
fn only_known_extensions_have_an_interpreter() {
    assert_eq!(
        interpreter("exe").map(|_| ()),
        Err(NotRun::NoInterpreter("none"))
    );
}

#[test]
fn a_ran_script_blocks_only_on_two_within_its_time() {
    let ran = |code, timed_out| Ran {
        code: Some(code),
        said: String::new(),
        timed_out,
        took: Duration::ZERO,
    };
    assert!(ran(2, false).blocks());
    assert!(!ran(2, true).blocks());
    assert!(!ran(1, false).blocks());
}
