use super::*;

#[test]
fn waiting_on_another_process_renewing_is_busy_and_never_the_providers_limit() {
    let wait = Duration::from_secs(30);

    let judged = judge(Hitch::Busy { wait }, "drive");

    assert!(
        matches!(judged, Judged::Pause(Reason::Busy, Some(then), None) if then == wait),
        "a lock on this machine was told as the provider holding Tisty back"
    );
}

#[test]
fn a_provider_out_of_reach_pauses_saying_what_stood_in_the_way() {
    for cause in [Cause::Blocked, Cause::Filtered, Cause::Slow] {
        let judged = judge(Hitch::Unreachable(cause, "said".into()), "drive");

        assert!(
            matches!(judged, Judged::Pause(Reason::Offline, None, Some(kind)) if kind == cause),
            "{cause:?} was lost on the way to the pause"
        );
    }
    assert!(matches!(
        judge(Hitch::Troubled("503".into()), "drive"),
        Judged::Pause(Reason::Troubled, None, None)
    ));
}
