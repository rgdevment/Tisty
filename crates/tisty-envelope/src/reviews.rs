use std::collections::{BTreeMap, BTreeSet};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tisty_contracts::Review;
use tisty_share_proto::identity::{Certificate, Signed};
use tisty_share_proto::reviewing::{self, Kept, Said};

fn decoded(said: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(said).ok()
}

fn holds(anchor: &str, owner: &str, anchor_key: &[u8], review: &Review) -> bool {
    if review.root != owner {
        return false;
    }
    let (Some(of), Ok(since)) = (decoded(&review.of), review.since.parse()) else {
        return false;
    };
    let key = review.key.as_deref().and_then(decoded);
    let print = review.print.as_deref().and_then(decoded);
    let kept = match (review.kept, &review.document, &review.lang, &key, &print) {
        (false, ..) => None,
        (true, Some(document), Some(lang), Some(key), Some(print)) => Some(Kept {
            document,
            lang,
            title: review.title.as_deref(),
            key,
            print,
        }),
        _ => return false,
    };
    let named = |kept: &Kept| reviewing::of(anchor_key, kept.document).is_some_and(|its| its == of);
    if kept.as_ref().is_some_and(|kept| !named(kept)) {
        return false;
    }
    let signed = Signed {
        certificate: Certificate {
            root: review.root.clone(),
            share: review.machine.clone(),
            since,
            sig: review.certificate.clone(),
        },
        sig: review.signed.clone(),
    };
    let said = Said {
        anchor,
        of: &of,
        seq: review.seq,
        kept,
    };
    reviewing::holds(&said, &signed)
}

fn newer(one: &Review, was: &Review) -> bool {
    (one.seq, decoded(&one.machine)) > (was.seq, decoded(&was.machine))
}

pub struct LeftOut {
    anchor_key: Vec<u8>,
    gone: BTreeSet<String>,
}

impl LeftOut {
    /// Only the latest review of each language counts, and only one its owner signed.
    pub fn by(
        anchor_key: Option<&str>,
        anchor: &str,
        owner: &str,
        reviews: &[Review],
    ) -> Option<Self> {
        let anchor_key = decoded(anchor_key?)?;
        let mut latest: BTreeMap<&str, &Review> = BTreeMap::new();
        for one in reviews
            .iter()
            .filter(|one| holds(anchor, owner, &anchor_key, one))
        {
            if latest
                .get(one.of.as_str())
                .is_none_or(|was| newer(one, was))
            {
                latest.insert(&one.of, one);
            }
        }
        let gone: BTreeSet<String> = latest
            .into_values()
            .filter(|one| !one.kept)
            .map(|one| one.of.clone())
            .collect();
        (!gone.is_empty()).then_some(Self { anchor_key, gone })
    }

    pub fn names(&self, id: &str) -> bool {
        reviewing::of(&self.anchor_key, id)
            .is_some_and(|of| self.gone.contains(&URL_SAFE_NO_PAD.encode(of)))
    }
}

#[cfg(test)]
#[path = "reviews_test.rs"]
mod tests;
