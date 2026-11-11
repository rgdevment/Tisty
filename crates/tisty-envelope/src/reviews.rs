use std::collections::{BTreeMap, BTreeSet};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use tisty_contracts::{Review, Reviewed, Served, Verdict};
use tisty_share_proto::identity::{Certificate, ShareKey, Signed};
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

pub fn reviews_hash(anchor: &str, reviews: &[Review]) -> Option<Vec<u8>> {
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

/// Over what opens, so sealing it again under another head, number or key leaves it as it was.
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

pub struct Counted {
    pub seq: u64,
    pub reviews: Vec<Review>,
}

pub fn kept(weighing: &Weighing, reviews: &[Review]) -> Option<Counted> {
    let head = weighing.served.reviewed.as_ref()?;
    let anchor_key = decoded(weighing.anchor_key)?;
    if !word_counts(weighing, head, reviews) {
        return None;
    }
    let reviews = latest(weighing.anchor, weighing.owner, &anchor_key, reviews)
        .into_values()
        .filter(|one| one.kept)
        .cloned()
        .collect();
    Some(Counted {
        seq: head.seq,
        reviews,
    })
}

pub fn keeps(kept: &[Review], document: &str, lang: Option<&str>, print: &str) -> bool {
    kept.iter().any(|one| {
        one.document.as_deref() == Some(document)
            && one.lang.as_deref() == lang
            && one.print.as_deref() == Some(print)
    })
}

pub struct Read<'a> {
    pub lang: &'a str,
    pub title: Option<&'a str>,
    pub key: &'a str,
    pub print: &'a str,
}

pub struct Signer<'a> {
    pub key: &'a ShareKey,
    pub certificate: &'a Certificate,
}

fn review_of(said: &Said, signed: Signed) -> Review {
    let kept = said.kept.as_ref();
    Review {
        of: URL_SAFE_NO_PAD.encode(said.of),
        kept: kept.is_some(),
        document: kept.map(|one| one.document.to_owned()),
        lang: kept.map(|one| one.lang.to_owned()),
        title: kept.and_then(|one| one.title.map(str::to_owned)),
        key: kept.map(|one| URL_SAFE_NO_PAD.encode(one.key)),
        print: kept.map(|one| URL_SAFE_NO_PAD.encode(one.print)),
        seq: said.seq,
        root: signed.certificate.root,
        machine: signed.certificate.share,
        since: signed.certificate.since.to_string(),
        certificate: signed.certificate.sig,
        signed: signed.sig,
    }
}

pub fn review_signed(
    signer: &Signer,
    anchor: &str,
    anchor_key: &str,
    document: &str,
    read: Option<&Read>,
    seq: u64,
) -> Option<Review> {
    let of = reviewing::of(&decoded(anchor_key)?, document)?;
    let bytes = match read {
        Some(read) => Some((decoded(read.key)?, decoded(read.print)?)),
        None => None,
    };
    let kept = read.zip(bytes.as_ref()).map(|(read, (key, print))| Kept {
        document,
        lang: read.lang,
        title: read.title,
        key,
        print,
    });
    let said = Said {
        anchor,
        of: &of,
        seq,
        kept,
    };
    let signed = reviewing::signed(signer.key, signer.certificate, &said)?;
    Some(review_of(&said, signed))
}

// The owner never writes its word over a review a machine it revoked signed, so that review is signed again as it was.
pub fn signed_again(signer: &Signer, anchor: &str, review: &Review) -> Option<Review> {
    let bytes = bytes_of(review)?;
    let said = said(anchor, review, &bytes)?;
    let signed = reviewing::signed(signer.key, signer.certificate, &said)?;
    Some(review_of(&said, signed))
}

pub fn word_signed(
    signer: &Signer,
    anchor: &str,
    seq: u64,
    reviews: &[Review],
) -> Option<Reviewed> {
    let hash = reviews_hash(anchor, reviews)?;
    let said = reviewing::reviewed(anchor, seq, &hash);
    let signed = signer
        .key
        .sign(signer.certificate, reviewing::REVIEWED, &said)?;
    Some(Reviewed {
        anchor: anchor.to_owned(),
        seq,
        reviews: URL_SAFE_NO_PAD.encode(hash),
        root: signed.certificate.root,
        machine: signed.certificate.share,
        since: signed.certificate.since.to_string(),
        certificate: signed.certificate.sig,
        signed: signed.sig,
    })
}

pub fn next_seq(reviews: &[Review], of: &str) -> u64 {
    reviews
        .iter()
        .filter(|one| one.of == of)
        .map(|one| one.seq)
        .max()
        .unwrap_or(0)
        + 1
}

pub fn with_review(reviews: &[Review], review: Review) -> Vec<Review> {
    let mut out: Vec<Review> = reviews
        .iter()
        .filter(|one| one.of != review.of)
        .cloned()
        .collect();
    out.push(review);
    out
}

pub fn word_on(joining: &Joining, reviews: &[Review], document: &str) -> Option<bool> {
    let anchor_key = decoded(joining.anchor_key)?;
    let of = URL_SAFE_NO_PAD.encode(reviewing::of(&anchor_key, document)?);
    let latest = latest(joining.anchor, joining.owner, &anchor_key, reviews);
    latest.get(of.as_str()).map(|one| one.kept)
}

pub fn of_language(anchor_key: &str, id: &str) -> Option<String> {
    reviewing::of(&decoded(anchor_key)?, id).map(|of| URL_SAFE_NO_PAD.encode(of))
}

pub struct Joining<'a> {
    pub anchor: &'a str,
    pub owner: &'a str,
    pub anchor_key: &'a str,
}

// Only what its owner signed comes in from another's version, and one review stays for each language, its latest.
pub fn joined(joining: &Joining, mine: &[Review], theirs: &[Review]) -> Vec<Review> {
    let Some(anchor_key) = decoded(joining.anchor_key) else {
        return mine.to_vec();
    };
    let both: Vec<Review> = mine.iter().chain(theirs).cloned().collect();
    latest(joining.anchor, joining.owner, &anchor_key, &both)
        .into_values()
        .cloned()
        .collect()
}

pub struct LeftOut {
    anchor_key: Vec<u8>,
    gone: BTreeSet<String>,
}

impl LeftOut {
    pub fn by(anchor_key: &str, anchor: &str, owner: &str, reviews: &[Review]) -> Option<Self> {
        let anchor_key = decoded(anchor_key)?;
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
