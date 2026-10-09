use std::time::Duration;

use tisty_sync::Trouble;

use crate::{Hitch, Reason};

pub enum Judged {
    Pause(Reason, Option<Duration>),
    Stop(Trouble),
}

pub fn judge(hitch: Hitch) -> Judged {
    match hitch {
        Hitch::Limited { wait } => Judged::Pause(Reason::Limit, Some(wait)),
        Hitch::Spent => Judged::Pause(Reason::Budget, None),
        Hitch::Unreachable(_) => Judged::Pause(Reason::Offline, None),
        Hitch::Full => Judged::Stop(Trouble::Refused("the cloud has no room left".into())),
        Hitch::Lost => Judged::Stop(Trouble::Refused("the cloud no longer lets this in".into())),
        Hitch::Elsewhere { found } => Judged::Stop(Trouble::Refused(format!(
            "the cloud is now signed in as {found}"
        ))),
        Hitch::Missing(name) | Hitch::Changed(name) => Judged::Stop(Trouble::Broke(name)),
        Hitch::Broke(why) => Judged::Stop(Trouble::Broke(why)),
    }
}
