use serde::Serialize;
use tisty_carrier::folio_limits::FolioMaker;
use tisty_contracts::Verdict;
use tisty_share_proto::identity::Person;
use tisty_share_proto::name::imitates;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "standing", rename_all = "camelCase")]
pub enum Standing {
    Verified {
        maker: String,
        domain: Option<String>,
    },
    Imitates,
    TakenBack,
    Checked {
        domain: String,
    },
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Party {
    pub name: String,
    pub print: String,
    #[serde(flatten)]
    pub standing: Standing,
}

// Read the way Settings › Security shows it, so whoever shared it can compare the two.
pub fn print_of(root: &str) -> String {
    Person::read(root)
        .map(|person| person.fingerprint())
        .unwrap_or_default()
}

fn still_speaks(verdict: Verdict) -> bool {
    matches!(verdict, Verdict::Holds | Verdict::Retired)
}

// Folio's rule, root for root: only the maker's root is verified, and a name says nothing on its own.
pub fn standing_of(
    root: &str,
    verdict: Verdict,
    alias: Option<&str>,
    maker: Option<&FolioMaker>,
    checked: impl FnOnce(&str, &str) -> bool,
) -> Standing {
    let alias = alias.filter(|one| !one.is_empty());
    if let Some(maker) = maker
        && let Some(shown) = maker.name.as_ref().or(maker.domain.as_ref())
    {
        if root == maker.root {
            return match still_speaks(verdict) {
                true => Standing::Verified {
                    maker: shown.clone(),
                    domain: maker.name.as_ref().and(maker.domain.clone()),
                },
                false => Standing::TakenBack,
            };
        }
        let mut names = maker.name.iter().chain(&maker.domain);
        if alias.is_some_and(|one| names.any(|name| imitates(one, name))) {
            return Standing::Imitates;
        }
    }
    match alias {
        Some(domain) if still_speaks(verdict) && checked(root, domain) => Standing::Checked {
            domain: domain.to_owned(),
        },
        _ => Standing::Plain,
    }
}

#[cfg(test)]
#[path = "folio_standing_test.rs"]
mod tests;
