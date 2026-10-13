use std::time::Duration;

use tisty_sync::Trouble;

use crate::{Hitch, Reason};

pub enum Judged {
    Pause(Reason, Option<Duration>),
    Stop(Trouble),
}

pub fn judge(hitch: Hitch, provider: &str) -> Judged {
    let provider = provider.to_string();
    match hitch {
        Hitch::Limited { wait } => Judged::Pause(Reason::Limit, Some(wait)),
        Hitch::Busy { wait } => Judged::Pause(Reason::Busy, Some(wait)),
        Hitch::Spent => Judged::Pause(Reason::Budget, None),
        Hitch::Paused => Judged::Pause(Reason::Paused, None),
        Hitch::Unreachable(_) => Judged::Pause(Reason::Offline, None),
        Hitch::Troubled(_) => Judged::Pause(Reason::Troubled, None),
        Hitch::Binned => Judged::Stop(Trouble::CloudBinned { provider }),
        Hitch::Full => Judged::Stop(Trouble::CloudFull { provider }),
        Hitch::Lost => Judged::Stop(Trouble::CloudLost { provider }),
        Hitch::Elsewhere { found } => Judged::Stop(Trouble::CloudElsewhere { provider, found }),
        Hitch::Missing(name) | Hitch::Changed(name) => Judged::Stop(Trouble::Broke(name)),
        Hitch::Broke(why) => Judged::Stop(Trouble::Broke(why)),
    }
}

#[cfg(test)]
#[path = "judge_test.rs"]
mod tests;
