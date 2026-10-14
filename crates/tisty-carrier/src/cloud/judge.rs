use std::time::Duration;

use tisty_sync::Trouble;

use crate::{Cause, Hitch, Reason};

pub enum Judged {
    Pause(Reason, Option<Duration>, Option<Cause>),
    Stop(Trouble),
}

pub fn judge(hitch: Hitch, provider: &str) -> Judged {
    let provider = provider.to_string();
    match hitch {
        Hitch::Limited { wait } => Judged::Pause(Reason::Limit, Some(wait), None),
        Hitch::Busy { wait } => Judged::Pause(Reason::Busy, Some(wait), None),
        Hitch::Spent => Judged::Pause(Reason::Budget, None, None),
        Hitch::Paused => Judged::Pause(Reason::Paused, None, None),
        Hitch::Unreachable(cause, _) => Judged::Pause(Reason::Offline, None, Some(cause)),
        Hitch::Troubled(_) => Judged::Pause(Reason::Troubled, None, None),
        Hitch::Full => Judged::Stop(Trouble::CloudFull { provider }),
        Hitch::Emptied => Judged::Stop(Trouble::CloudEmptied { provider }),
        Hitch::Lost => Judged::Stop(Trouble::CloudLost { provider }),
        Hitch::Elsewhere { found } => Judged::Stop(Trouble::CloudElsewhere { provider, found }),
        Hitch::Missing(name) | Hitch::Changed(name) => Judged::Stop(Trouble::Broke(name)),
        Hitch::Broke(why) => Judged::Stop(Trouble::Broke(why)),
    }
}

#[cfg(test)]
#[path = "judge_test.rs"]
mod tests;
