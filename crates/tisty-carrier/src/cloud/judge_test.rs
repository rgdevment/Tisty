use super::*;

#[test]
fn waiting_on_another_process_renewing_is_busy_and_never_the_providers_limit() {
    let wait = Duration::from_secs(30);

    let judged = judge(Hitch::Busy { wait }, "drive");

    assert!(
        matches!(judged, Judged::Pause(Reason::Busy, Some(then)) if then == wait),
        "a lock on this machine was told as the provider holding Tisty back"
    );
}
