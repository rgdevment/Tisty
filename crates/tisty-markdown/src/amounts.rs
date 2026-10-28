use std::cmp::Ordering;

use crate::Holds;

// Longer numbers are amiss rather than read, so no sum of a whole table can overflow.
const MOST_WHOLE: usize = 18;
const MOST_PART: usize = 9;

/// A decimal kept exact: `units` counted in tenths to the power of `scale`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Amount {
    pub(crate) units: i128,
    pub(crate) scale: u32,
}

impl Amount {
    // The one way a number is written in a file: digits, a point for decimals, no thousands mark.
    pub(crate) fn read(said: &str) -> Option<Self> {
        let (sign, digits) = match said.strip_prefix('-') {
            Some(rest) => (-1, rest),
            None => (1, said),
        };
        let (whole, part) = digits.split_once('.').unwrap_or((digits, ""));
        let fits = |one: &str| one.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty()
            || whole.len() > MOST_WHOLE
            || part.len() > MOST_PART
            || !fits(whole)
            || !fits(part)
            || (digits.contains('.') && part.is_empty())
        {
            return None;
        }
        let units: i128 = format!("{whole}{part}").parse().ok()?;
        Some(Self {
            units: sign * units,
            scale: u32::try_from(part.len()).ok()?,
        })
    }

    pub(crate) fn at(&self, scale: u32) -> i128 {
        self.units * 10_i128.pow(scale.saturating_sub(self.scale))
    }

    pub(crate) fn plus(&self, other: &Self) -> Self {
        let scale = self.scale.max(other.scale);
        Self {
            units: self.at(scale) + other.at(scale),
            scale,
        }
    }

    pub(crate) fn against(&self, other: &Self) -> Ordering {
        let scale = self.scale.max(other.scale);
        self.at(scale).cmp(&other.at(scale))
    }

    // Shared out among `many`, kept to `scale` decimals.
    pub(crate) fn shared(&self, many: i128, scale: u32) -> Self {
        let units = if scale >= self.scale {
            rounded(self.at(scale), many)
        } else {
            rounded(self.units, many * 10_i128.pow(self.scale - scale))
        };
        Self { units, scale }
    }

    // A plain number drops the zeros an average leaves at its end; money keeps its cents.
    pub(crate) fn trimmed(mut self, holds: &Holds) -> Self {
        if matches!(holds, Holds::Money { .. }) {
            return self;
        }
        while self.scale > 0 && self.units % 10 == 0 {
            self.units /= 10;
            self.scale -= 1;
        }
        self
    }

    fn split(&self) -> (bool, String, String) {
        let digits = self.units.unsigned_abs().to_string();
        let scale = self.scale as usize;
        let digits = format!("{digits:0>width$}", width = scale + 1);
        let (whole, part) = digits.split_at(digits.len() - scale);
        (self.units < 0, whole.to_string(), part.to_string())
    }

    pub(crate) fn plain(&self) -> String {
        let (negative, whole, part) = self.split();
        let sign = if negative { "-" } else { "" };
        if part.is_empty() {
            format!("{sign}{whole}")
        } else {
            format!("{sign}{whole}.{part}")
        }
    }
}

// Halves go away from zero, as on a till.
pub(crate) fn rounded(units: i128, many: i128) -> i128 {
    if many == 0 {
        return 0;
    }
    let half = many / 2;
    if units >= 0 {
        (units + half) / many
    } else {
        (units - half) / many
    }
}

pub(crate) struct Convention {
    pub(crate) symbol: String,
    pub(crate) decimals: u32,
    pub(crate) group: &'static str,
    pub(crate) point: &'static str,
    after: bool,
    spaced: bool,
}

// Each currency as it is written where it is spent, so an export reads the same in any language.
#[rustfmt::skip]
const CURRENCIES: &[(&str, &str, u32, &str, &str, bool, bool)] = &[
    ("CLP", "$", 0, ".", ",", false, false),
    ("ARS", "$", 2, ".", ",", false, false),
    ("COP", "$", 2, ".", ",", false, false),
    ("MXN", "MX$", 2, ",", ".", false, false),
    ("PEN", "S/", 2, ",", ".", false, true),
    ("BRL", "R$", 2, ".", ",", false, true),
    ("UYU", "$U", 2, ".", ",", false, true),
    ("USD", "US$", 2, ",", ".", false, false),
    ("CAD", "CA$", 2, ",", ".", false, false),
    ("AUD", "A$", 2, ",", ".", false, false),
    ("EUR", "€", 2, ".", ",", true, true),
    ("GBP", "£", 2, ",", ".", false, false),
    ("CHF", "CHF", 2, "'", ".", false, true),
    ("JPY", "¥", 0, ",", ".", false, false),
    ("CNY", "CN¥", 2, ",", ".", false, false),
];

pub(crate) fn convention(currency: Option<&str>) -> Convention {
    let Some(code) = currency else {
        return Convention {
            symbol: String::new(),
            decimals: 2,
            group: "",
            point: ".",
            after: false,
            spaced: false,
        };
    };
    let known = CURRENCIES.iter().find(|one| one.0 == code);
    let (symbol, decimals, group, point, after, spaced) = known.map_or(
        (code, 2, ",", ".", false, true),
        |&(_, symbol, decimals, group, point, after, spaced)| {
            (symbol, decimals, group, point, after, spaced)
        },
    );
    Convention {
        symbol: symbol.to_string(),
        decimals,
        group,
        point,
        after,
        spaced,
    }
}

pub(crate) fn money(amount: &Amount, currency: Option<&str>) -> String {
    let way = convention(currency);
    let fixed = Amount {
        units: amount.at(way.decimals),
        scale: way.decimals,
    };
    let (negative, whole, part) = fixed.split();
    let mut number = grouped(&whole, way.group);
    if !part.is_empty() {
        number = format!("{number}{}{part}", way.point);
    }
    let sign = if negative { "-" } else { "" };
    let gap = if way.spaced { " " } else { "" };
    if way.symbol.is_empty() {
        format!("{sign}{number}")
    } else if way.after {
        format!("{sign}{number}{gap}{}", way.symbol)
    } else {
        format!("{sign}{}{gap}{number}", way.symbol)
    }
}

fn grouped(whole: &str, group: &str) -> String {
    let mut out = String::new();
    for (at, digit) in whole.chars().enumerate() {
        if at > 0 && (whole.len() - at).is_multiple_of(3) {
            out.push_str(group);
        }
        out.push(digit);
    }
    out
}
