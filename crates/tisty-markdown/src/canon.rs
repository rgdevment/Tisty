use crate::Holds;
use crate::amounts::convention;
use crate::typed::understood;
use crate::values::seen;

/// What was typed in a cell, in the one form its column's type is written in the file.
/// What cannot be read as that type is kept as typed, to be shown marked, never refused.
pub fn canonical(words: &str, typed: &str) -> String {
    let said = typed.trim();
    let holds = understood(words).map(|one| one.0).unwrap_or_default();
    let made = match &holds {
        Holds::Number => numbered(said),
        Holds::Money { currency } => priced(said, currency.as_deref()),
        Holds::Percent => numbered(said.trim_end_matches('%').trim_end()),
        Holds::Duration => lasted(said),
        Holds::Rating => rated(said),
        Holds::Check => ticked(said),
        Holds::Url => webbed(said),
        Holds::Tags => tagged(said),
        _ => None,
    };
    match made {
        Some(one) if !seen(&holds, &one).amiss => one,
        _ => said.to_string(),
    }
}

// A comma is a decimal point unless three digits after it make it a thousands mark.
fn numbered(said: &str) -> Option<String> {
    let said: String = said.chars().filter(|one| !one.is_whitespace()).collect();
    match said.split_once(',') {
        None => Some(said),
        Some((whole, part)) if !said.contains('.') && !part.contains(',') && part.len() != 3 => {
            Some(format!("{whole}.{part}"))
        }
        Some(_) => None,
    }
}

// Money is read as its own currency writes it: `$6.490` in pesos is six thousand, `$6.49` in dollars is not.
fn priced(said: &str, currency: Option<&str>) -> Option<String> {
    let way = convention(currency);
    let mut kept = said.to_string();
    for mark in [currency.unwrap_or_default(), way.symbol.as_str(), "$"] {
        if !mark.is_empty() {
            kept = kept.replace(mark, "");
        }
    }
    let kept: String = kept.chars().filter(|one| !one.is_whitespace()).collect();
    if currency.is_none() || way.group.is_empty() {
        return numbered(&kept);
    }
    let (sign, digits) = kept
        .strip_prefix('-')
        .map_or(("", kept.as_str()), |rest| ("-", rest));
    let (whole, part) = digits.rsplit_once(way.point).unwrap_or((digits, ""));
    let grouped = whole.split(way.group).collect::<Vec<_>>();
    let rightly = grouped.len() == 1
        || (grouped[0].len() <= 3 && grouped[1..].iter().all(|one| one.len() == 3));
    if !rightly || part.contains(way.group) {
        return None;
    }
    let whole = grouped.concat();
    Some(if part.is_empty() {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{part}")
    })
}

// `1 h 30 min`, `1h30`, `90 min` and `1:30` are all an hour and a half.
fn lasted(said: &str) -> Option<String> {
    let said: String = said
        .to_lowercase()
        .chars()
        .filter(|one| !one.is_whitespace())
        .collect();
    let number = |one: &str| -> Option<u64> {
        (!one.is_empty() && one.len() < 8 && one.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| one.parse().ok())
            .flatten()
    };
    let minutes = if let Some((hours, rest)) = said.split_once(':') {
        number(hours)? * 60 + number(rest)?
    } else {
        let (hours, rest) = said.split_once('h').unwrap_or(("", said.as_str()));
        let rest = rest.trim_end_matches("min").trim_end_matches('m');
        let hours = if hours.is_empty() { 0 } else { number(hours)? };
        let rest = if rest.is_empty() { 0 } else { number(rest)? };
        hours * 60 + rest
    };
    if said.is_empty() {
        return None;
    }
    Some(match (minutes / 60, minutes % 60) {
        (0, rest) => format!("{rest}m"),
        (hours, 0) => format!("{hours}h"),
        (hours, rest) => format!("{hours}h{rest}m"),
    })
}

fn rated(said: &str) -> Option<String> {
    let stars = said.chars().filter(|one| *one == '★').count();
    if stars > 0 && said.chars().all(|one| one == '★' || one == '☆') {
        return Some(stars.to_string());
    }
    Some(said.to_string())
}

const YES: &[&str] = &["[x]", "x", "✓", "sí", "si", "yes", "y", "true", "1"];
const NO: &[&str] = &["[ ]", "no", "n", "false", "0"];

fn ticked(said: &str) -> Option<String> {
    let said = said.to_lowercase();
    if YES.contains(&said.as_str()) {
        Some("[x]".to_string())
    } else if NO.contains(&said.as_str()) {
        Some("[ ]".to_string())
    } else {
        None
    }
}

// A bare place on the web is taken for the secure one, as a browser does.
fn webbed(said: &str) -> Option<String> {
    if said.contains("://") || said.contains(char::is_whitespace) || !said.contains('.') {
        return Some(said.to_string());
    }
    Some(format!("https://{said}"))
}

fn tagged(said: &str) -> Option<String> {
    let mut kept: Vec<&str> = Vec::new();
    for one in said.split(',').map(str::trim).filter(|one| !one.is_empty()) {
        if !kept.iter().any(|was| was.eq_ignore_ascii_case(one)) {
            kept.push(one);
        }
    }
    Some(kept.join(", "))
}

#[cfg(test)]
#[path = "canon_test.rs"]
mod tests;
