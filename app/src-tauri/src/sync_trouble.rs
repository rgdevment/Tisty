use tisty_core::witness::{self, Fact, channel};

use crate::Refusal;

pub(crate) fn said(trouble: tisty_carrier::Trouble) -> Refusal {
    let (provider, why) = told_apart(&trouble);
    let refusal = refused_for(trouble);
    told_in_the_log(refusal.code, provider, why);
    refusal
}

fn told_apart(trouble: &tisty_carrier::Trouble) -> (Option<String>, Option<String>) {
    use tisty_carrier::Trouble;
    match trouble {
        Trouble::CloudFull { provider }
        | Trouble::CloudEmptied { provider }
        | Trouble::CloudLost { provider }
        | Trouble::CloudElsewhere { provider, .. }
        | Trouble::CloudSpent { provider } => (Some(provider.clone()), None),
        Trouble::Broke(why)
        | Trouble::Unreadable(why)
        | Trouble::Refused(why)
        | Trouble::NotNow(why)
        | Trouble::StuckOn { why, .. } => (None, Some(why.clone())),
        _ => (None, None),
    }
}

fn refused_for(trouble: tisty_carrier::Trouble) -> Refusal {
    match trouble {
        tisty_carrier::Trouble::NotThere(at) => Refusal::about("noMeetingPlace", at),
        tisty_carrier::Trouble::OtherStore { theirs } => Refusal::about("otherStore", theirs),
        tisty_carrier::Trouble::Newer(who) => Refusal::about("syncNewer", who),
        tisty_carrier::Trouble::Unreadable(why) => Refusal::about("syncUnreadable", why),
        tisty_carrier::Trouble::Refused(why) => Refusal::about("syncRefused", why),
        tisty_carrier::Trouble::Broke(_) => Refusal::of("syncFailed"),
        tisty_carrier::Trouble::WouldReset { theirs } => Refusal::about("wouldReset", theirs),
        tisty_carrier::Trouble::NotAllowed(who) => Refusal::about("notAllowed", who),
        tisty_carrier::Trouble::Emptied(at) => Refusal::about("emptiedPlace", at),
        tisty_carrier::Trouble::Shape(at) => Refusal::about("syncShape", at),
        tisty_carrier::Trouble::Needs(release) => Refusal::about("syncNeeds", release),
        tisty_carrier::Trouble::Unshaped(at) => Refusal::about("syncUnshaped", at),
        tisty_carrier::Trouble::CloudFull { provider } => Refusal::about("cloudFull", provider),
        tisty_carrier::Trouble::CloudEmptied { provider } => {
            Refusal::about("cloudEmptied", provider)
        }
        tisty_carrier::Trouble::CloudLost { provider } => Refusal::about("cloudLost", provider),
        tisty_carrier::Trouble::CloudElsewhere { provider, found } => {
            Refusal::about("cloudElsewhere", format!("{provider} · {found}"))
        }
        tisty_carrier::Trouble::SameName(who) => {
            Refusal::about("sameName", tisty_core::config::nicknamed(&who))
        }
        tisty_carrier::Trouble::CloudSpent { provider } => Refusal::about("cloudSpent", provider),
        tisty_carrier::Trouble::StuckOn { name, .. } => match name.strip_prefix("docs/") {
            Some(_) => Refusal::about("syncStuckDoc", name),
            None => Refusal::about(
                "syncStuckHistory",
                tisty_core::config::nicknamed(name.split('/').nth(1).unwrap_or_default()),
            ),
        },
        tisty_carrier::Trouble::NotNow(_) => Refusal::of("cloudNotNow"),
        tisty_carrier::Trouble::Moved(_) => Refusal::of("movedUnderfoot"),
        tisty_carrier::Trouble::Sealed(_) => Refusal::of("cloudSealed"),
        tisty_carrier::Trouble::Sealing(_) => Refusal::of("cloudSealing"),
        tisty_carrier::Trouble::Unsealed(_) => Refusal::of("cloudUnsealed"),
        tisty_carrier::Trouble::OtherPhrase(_) => Refusal::of("cloudOtherPhrase"),
        tisty_carrier::Trouble::MarkDamaged(_) => Refusal::of("cloudMarkDamaged"),
        tisty_carrier::Trouble::OtherPerson(_) => Refusal::of("otherPerson"),
    }
}

// What a provider or the system says is English and technical, so the person reads a sentence and the log keeps it.
fn told_in_the_log(code: &'static str, provider: Option<String>, why: Option<String>) {
    let mut facts = vec![("code", Fact::Code(code))];
    facts.extend(provider.map(|one| ("provider", Fact::Id(one))));
    facts.extend(why.map(|one| ("why", Fact::Why(one))));
    match code {
        "cloudNotNow" | "movedUnderfoot" | "cloudSealing" => {
            witness::standing(channel::SYNC, "a sync could not be finished", &facts)
        }
        _ => witness::warn(channel::SYNC, "a sync could not be finished", &facts),
    }
}

/// As `said`, with a document that held the round back named by its title.
pub(crate) fn said_carrying(paths: &tisty_core::Paths, trouble: tisty_carrier::Trouble) -> Refusal {
    let refusal = said(trouble);
    let titled = refusal
        .name
        .as_deref()
        .filter(|_| refusal.code == "syncStuckDoc")
        .and_then(|name| name.strip_prefix("docs/"))
        .and_then(|leaf| leaf.strip_suffix(".md"))
        .and_then(|id| tisty_core::docs::title_of(&paths.docs(), id))
        .filter(|title| !title.trim().is_empty());
    match titled {
        Some(title) => Refusal {
            name: Some(title),
            ..refusal
        },
        None => refusal,
    }
}

#[cfg(test)]
#[path = "sync_trouble_test.rs"]
mod tests;
