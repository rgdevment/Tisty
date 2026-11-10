use std::collections::{BTreeMap, BTreeSet};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use tisty_contracts::{Review, Reviewed, Served, Verdict};
use tisty_share_proto::identity::{Certificate, Signed};
use tisty_share_proto::reviewing::{self, Kept, Said};

fn decoded(said: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(said).ok()
}

struct Bytes {
    of: Vec<u8>,
    key: Option<Vec<u8>>,
    print: Option<Vec<u8>>,
}

fn bytes_of(review: &Review) -> Option<Bytes> {
    Some(Bytes {
        of: decoded(&review.of)?,
        key: review.key.as_deref().and_then(decoded),
        print: review.print.as_deref().and_then(decoded),
    })
}

fn said<'a>(anchor: &'a str, review: &'a Review, bytes: &'a Bytes) -> Option<Said<'a>> {
    let kept = match (
        review.kept,
        &review.document,
        &review.lang,
        &bytes.key,
        &bytes.print,
    ) {
        (false, ..) => None,
        (true, Some(document), Some(lang), Some(key), Some(print)) => Some(Kept {
            document,
            lang,
            title: review.title.as_deref(),
            key,
            print,
        }),
        _ => return None,
    };
    Some(Said {
        anchor,
        of: &bytes.of,
        seq: review.seq,
        kept,
    })
}

fn holds(anchor: &str, owner: &str, anchor_key: &[u8], review: &Review) -> bool {
    if review.root != owner {
        return false;
    }
    let (Some(bytes), Ok(since)) = (bytes_of(review), review.since.parse()) else {
        return false;
    };
    let Some(said) = said(anchor, review, &bytes) else {
        return false;
    };
    let named =
        |kept: &Kept| reviewing::of(anchor_key, kept.document).is_some_and(|its| its == bytes.of);
    if said.kept.as_ref().is_some_and(|kept| !named(kept)) {
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
    reviewing::holds(&said, &signed)
}

fn newer(one: &Review, was: &Review) -> bool {
    (one.seq, decoded(&one.machine)) > (was.seq, decoded(&was.machine))
}

fn latest<'a>(
    anchor: &str,
    owner: &str,
    anchor_key: &[u8],
    reviews: &'a [Review],
) -> BTreeMap<&'a str, &'a Review> {
    let mut latest: BTreeMap<&str, &Review> = BTreeMap::new();
    for one in reviews
        .iter()
        .filter(|one| holds(anchor, owner, anchor_key, one))
    {
        if latest
            .get(one.of.as_str())
            .is_none_or(|was| newer(one, was))
        {
            latest.insert(&one.of, one);
        }
    }
    latest
}

fn reviews_hash(anchor: &str, reviews: &[Review]) -> Option<Vec<u8>> {
    let mut each = Vec::with_capacity(reviews.len());
    for review in reviews {
        let bytes = bytes_of(review)?;
        let said = said(anchor, review, &bytes)?;
        let row = reviewing::row(&decoded(&review.root)?, &decoded(&review.machine)?, &said);
        each.push(Sha256::digest(row).to_vec());
    }
    each.sort();
    let mut whole = Sha256::new();
    whole.update(b"tisty-language-reviews\0");
    for one in each {
        whole.update(one);
    }
    Some(whole.finalize().to_vec())
}

/// What a reader reads of a language, so sealing it again under another head, number or key leaves it as it was.
pub fn print_of(parts: &[Vec<u8>]) -> String {
    let mut whole = Sha256::new();
    whole.update(b"tisty-language-print\0");
    for one in parts {
        whole.update(Sha256::digest(one));
    }
    URL_SAFE_NO_PAD.encode(whole.finalize())
}

pub struct Weighing<'a> {
    pub anchor: &'a str,
    pub owner: &'a str,
    pub anchor_key: &'a str,
    pub served: &'a Served,
    pub seen: u64,
}

fn word_counts(weighing: &Weighing, head: &Reviewed, reviews: &[Review]) -> bool {
    if head.anchor != weighing.anchor || head.root != weighing.owner || head.seq < weighing.seen {
        return false;
    }
    let Some(hash) = reviews_hash(weighing.anchor, reviews) else {
        return false;
    };
    if decoded(&head.reviews).as_ref() != Some(&hash) {
        return false;
    }
    let arrival = Served {
        received: weighing.served.reviewed_received.clone(),
        revocations: weighing.served.revocations.clone(),
        reviewed: None,
        reviewed_received: None,
    };
    let Ok(signature) = serde_json::to_value(head) else {
        return false;
    };
    let said = reviewing::reviewed(weighing.anchor, head.seq, &hash);
    matches!(
        crate::reading::verdict_on(&signature, reviewing::REVIEWED, &said, &arrival),
        Verdict::Holds | Verdict::Retired
    )
}

/// None while the owner's word served beside the anchor does not name these very reviews.
pub fn kept(weighing: &Weighing, reviews: &[Review]) -> Vec<Review> {
    let Some(head) = weighing.served.reviewed.as_ref() else {
        return Vec::new();
    };
    let Some(anchor_key) = decoded(weighing.anchor_key) else {
        return Vec::new();
    };
    if !word_counts(weighing, head, reviews) {
        return Vec::new();
    }
    latest(weighing.anchor, weighing.owner, &anchor_key, reviews)
        .into_values()
        .filter(|one| one.kept)
        .cloned()
        .collect()
}

pub fn keeps(kept: &[Review], document: &str, lang: Option<&str>, print: &str) -> bool {
    kept.iter().any(|one| {
        one.document.as_deref() == Some(document)
            && one.lang.as_deref() == lang
            && one.print.as_deref() == Some(print)
    })
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
        let gone: BTreeSet<String> = latest(anchor, owner, &anchor_key, reviews)
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
