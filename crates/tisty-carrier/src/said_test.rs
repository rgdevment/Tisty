use std::time::Duration;

use super::{WAITED_UNTOLD, retry_after, token_refused};
use crate::Hitch;

#[test]
fn a_wait_is_read_in_seconds_and_never_under_one() {
    assert_eq!(retry_after(Some(" 7 ")), Some(Duration::from_secs(7)));
    assert_eq!(retry_after(Some("0")), Some(Duration::from_secs(1)));
    assert_eq!(retry_after(Some("soon")), None);
    assert_eq!(retry_after(None), None);
}

#[test]
fn a_refused_token_says_whether_signing_in_again_mends_it() {
    let lost = ["invalid_grant", "interaction_required"];
    assert_eq!(
        token_refused(
            "Microsoft",
            400,
            None,
            r#"{"error":"interaction_required"}"#,
            &lost
        ),
        Hitch::Lost
    );
    assert_eq!(
        token_refused("Google", 400, None, r#"{"error":"invalid_client"}"#, &lost),
        Hitch::Broke("Google refused the token: invalid_client".into())
    );
    assert_eq!(
        token_refused("Google", 429, None, "", &lost),
        Hitch::Limited {
            wait: WAITED_UNTOLD
        }
    );
    assert_eq!(
        token_refused("Google", 503, Some(Duration::from_secs(9)), "", &lost),
        Hitch::Limited {
            wait: Duration::from_secs(9)
        }
    );
    assert_eq!(
        token_refused("Microsoft", 502, None, "", &lost),
        Hitch::Unreachable(None, "Microsoft answered 502".into())
    );
}
