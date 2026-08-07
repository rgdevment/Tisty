use tisty_core::Priority;

pub struct Vocabulary {
    pub today: &'static [&'static str],
    pub tomorrow: &'static [&'static str],
    pub day_after: &'static [&'static str],
    pub weekdays: [&'static [&'static str]; 7],
    pub months: [&'static [&'static str]; 12],
    pub next: &'static [&'static str],
    pub article: &'static [&'static str],
    pub date_prep: &'static [&'static str],
    pub deadline_prep: &'static [&'static str],
    pub time_prep: &'static [&'static str],
    /// Only these turn a bare integer into a clock: «a 12» is a volume.
    pub clock_prep: &'static [&'static str],
    /// «hace 3 días» looks ahead otherwise, and the past has no date here.
    pub past_prep: &'static [&'static str],
    /// «por la mañana» is a part of the day, not the day after.
    pub day_part: &'static [&'static str],
    /// What can follow a temporal phrase without being described by it.
    pub linker: &'static [&'static str],
    pub noon: &'static [&'static str],
    pub in_prep: &'static [&'static str],
    /// Marks a duration — «por 30 días» — which is not a date.
    pub spans_prep: &'static [&'static str],
    pub days_unit: &'static [&'static str],
    pub weeks_unit: &'static [&'static str],
    pub months_unit: &'static [&'static str],
    pub one: &'static [&'static str],
    pub this_week: &'static [&'static [&'static str]],
    pub end_of_month: &'static [&'static [&'static str]],
    pub weekend: &'static [&'static [&'static str]],
    /// Ordered P1 to P4; an empty slot has no spoken name.
    pub priorities: [&'static [&'static str]; 4],
}

pub const ES: Vocabulary = Vocabulary {
    today: &["hoy"],
    tomorrow: &["mañana"],
    day_after: &["pasado"],
    weekdays: [
        &["lunes"],
        &["martes"],
        &["miércoles", "miercoles"],
        &["jueves"],
        &["viernes"],
        &["sábado", "sabado"],
        &["domingo"],
    ],
    months: [
        &["enero", "ene"],
        &["febrero", "feb"],
        &["marzo", "mar"],
        &["abril", "abr"],
        &["mayo"],
        &["junio", "jun"],
        &["julio", "jul"],
        &["agosto", "ago"],
        &["septiembre", "setiembre", "sep"],
        &["octubre", "oct"],
        &["noviembre", "nov"],
        &["diciembre", "dic"],
    ],
    next: &[
        "próximo",
        "proximo",
        "próxima",
        "proxima",
        "siguiente",
        "este",
        "esta",
    ],
    article: &["el", "la", "los", "las", "de", "del"],
    date_prep: &["para"],
    deadline_prep: &["antes", "hasta", "vence"],
    time_prep: &["a", "al", "las", "la"],
    clock_prep: &["las"],
    past_prep: &["hace"],
    day_part: &["mañana", "tarde", "noche", "madrugada"],
    linker: &["y", "e", "o", "u", "pero"],
    noon: &["mediodía", "mediodia"],
    in_prep: &["en", "dentro"],
    spans_prep: &["por", "durante"],
    days_unit: &["día", "dia", "días", "dias"],
    weeks_unit: &["semana", "semanas"],
    months_unit: &["mes", "meses"],
    one: &["un", "una"],
    this_week: &[&["esta", "semana"]],
    end_of_month: &[&["fin", "de", "mes"], &["fin", "mes"]],
    weekend: &[&["finde"], &["fin", "de", "semana"]],
    priorities: [&["urgente"], &["alta"], &["media"], &[]],
};

pub const EN: Vocabulary = Vocabulary {
    today: &["today"],
    tomorrow: &["tomorrow"],
    day_after: &[],
    weekdays: [
        &["monday", "mon"],
        &["tuesday", "tue"],
        &["wednesday", "wed"],
        &["thursday", "thu"],
        &["friday", "fri"],
        &["saturday", "sat"],
        &["sunday", "sun"],
    ],
    months: [
        &["january", "jan"],
        &["february", "feb"],
        &["march", "mar"],
        &["april", "apr"],
        &["may"],
        &["june", "jun"],
        &["july", "jul"],
        &["august", "aug"],
        &["september", "sep"],
        &["october", "oct"],
        &["november", "nov"],
        &["december", "dec"],
    ],
    next: &["next", "this"],
    article: &["the", "on"],
    date_prep: &["on"],
    deadline_prep: &["by", "due", "before", "until"],
    time_prep: &["at"],
    clock_prep: &["at"],
    past_prep: &["ago"],
    day_part: &["morning", "afternoon", "evening", "night"],
    linker: &["and", "or", "to", "but"],
    noon: &["noon", "midday"],
    in_prep: &["in"],
    spans_prep: &["for", "during"],
    days_unit: &["day", "days"],
    weeks_unit: &["week", "weeks"],
    months_unit: &["month", "months"],
    one: &["a", "an", "one"],
    this_week: &[&["this", "week"]],
    end_of_month: &[&["end", "of", "month"], &["end", "of", "the", "month"]],
    weekend: &[&["this", "weekend"], &["weekend"]],
    priorities: [&["urgent"], &["high"], &["medium"], &[]],
};

/// What a system reports — `es-CL` — must not silently get English.
pub fn for_locale(code: &str) -> &'static Vocabulary {
    let tag = code.split(['_', '-', '.']).next().unwrap_or_default();
    match tag.to_lowercase().as_str() {
        "es" => &ES,
        _ => &EN,
    }
}

impl Vocabulary {
    pub fn weekday_index(&self, word: &str) -> Option<usize> {
        self.weekdays.iter().position(|w| w.contains(&word))
    }

    pub fn priority(&self, word: &str) -> Option<Priority> {
        let lower = word.to_lowercase();
        self.priorities
            .iter()
            .position(|names| names.contains(&lower.as_str()))
            .and_then(|i| Priority::try_from(i as u8 + 1).ok())
    }

    pub fn month_index(&self, word: &str) -> Option<u8> {
        self.months
            .iter()
            .position(|m| m.contains(&word))
            .map(|i| i as u8 + 1)
    }
}
