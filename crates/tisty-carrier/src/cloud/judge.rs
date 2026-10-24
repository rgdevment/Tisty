use std::time::Duration;

use tisty_core::witness::{self, Fact, channel};
use tisty_sync::Trouble;

use crate::{Cause, Hitch, Reason};

pub enum Judged {
    Pause(Reason, Option<Duration>, Option<Cause>),
    Stop(Trouble),
}

pub fn judge(hitch: Hitch, provider: &str) -> Judged {
    told(&hitch, provider);
    let provider = provider.to_string();
    match hitch {
        Hitch::Limited { wait } => Judged::Pause(Reason::Limit, Some(wait), None),
        Hitch::Busy { wait } => Judged::Pause(Reason::Busy, Some(wait), None),
        Hitch::Spent => Judged::Pause(Reason::Budget, None, None),
        Hitch::Paused => Judged::Pause(Reason::Paused, None, None),
        Hitch::Unreachable(cause, _) => Judged::Pause(Reason::Offline, None, cause),
        Hitch::Troubled(_) => Judged::Pause(Reason::Troubled, None, None),
        Hitch::Full => Judged::Stop(Trouble::CloudFull { provider }),
        Hitch::Emptied => Judged::Stop(Trouble::CloudEmptied { provider }),
        Hitch::Barred(trouble) => Judged::Stop(trouble),
        Hitch::Lost => Judged::Stop(Trouble::CloudLost { provider }),
        Hitch::Elsewhere { found } => Judged::Stop(Trouble::CloudElsewhere { provider, found }),
        Hitch::Missing(name) | Hitch::Changed(name) => Judged::Stop(Trouble::Broke(name)),
        Hitch::Broke(why) => Judged::Stop(Trouble::Broke(why)),
    }
}

/// What the provider said is otherwise lost here, where it becomes a pause or a stop.
fn told(hitch: &Hitch, provider: &str) {
    let why = match hitch {
        Hitch::Missing(why)
        | Hitch::Changed(why)
        | Hitch::Unreachable(_, why)
        | Hitch::Troubled(why)
        | Hitch::Broke(why) => why.as_str(),
        _ => "",
    };
    let facts = [
        ("provider", Fact::Id(provider.to_string())),
        ("kind", Fact::Code(hitch.code())),
        ("why", Fact::Why(why.to_string())),
    ];
    match hitch {
        Hitch::Limited { .. }
        | Hitch::Busy { .. }
        | Hitch::Spent
        | Hitch::Paused
        | Hitch::Unreachable(..)
        | Hitch::Troubled(_) => {
            witness::standing(channel::CLOUD, "a cloud round was put off", &facts)
        }
        _ => witness::standing(
            channel::CLOUD,
            "a cloud round stopped on what the provider answered",
            &facts,
        ),
    }
}

#[cfg(test)]
#[path = "judge_test.rs"]
mod tests;
