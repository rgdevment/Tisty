use tisty_core::Priority;
use tisty_core::model::Unit;

pub struct Vocabulary {
    pub today: &'static [&'static str],
    pub tomorrow: &'static [&'static str],
    pub day_after: &'static [&'static str],
    pub spelled_day: &'static [&'static str],
    pub weekdays: [&'static [&'static str]; 7],
    pub months: [&'static [&'static str]; 12],
    pub next: &'static [&'static str],
    pub article: &'static [&'static str],
    pub date_prep: &'static [&'static str],
    pub deadline_prep: &'static [&'static str],
    pub ends_prep: &'static [&'static str],
    pub time_prep: &'static [&'static str],
    pub clock_prep: &'static [&'static str],
    pub past_prep: &'static [&'static str],
    pub day_part: &'static [&'static str],
    pub pm_part: &'static [&'static str],
    pub night_part: &'static [&'static str],
    pub part_prep: &'static [&'static str],
    pub linker: &'static [&'static str],
    pub genitive: &'static [&'static str],
    pub loose_ends: &'static [&'static str],
    pub idioms: &'static [&'static str],
    pub noon: &'static [&'static str],
    pub in_prep: &'static [&'static str],
    pub spans_prep: &'static [&'static str],
    pub days_unit: &'static [&'static str],
    pub weeks_unit: &'static [&'static str],
    pub months_unit: &'static [&'static str],
    pub years_unit: &'static [&'static str],
    pub every: &'static [&'static [&'static str]],
    pub cadences: &'static [(&'static [&'static str], Unit)],
    pub one: &'static [&'static str],
    pub this_week: &'static [&'static [&'static str]],
    pub next_week: &'static [&'static [&'static str]],
    pub next_month: &'static [&'static [&'static str]],
    pub first: &'static [&'static str],
    pub end_of_month: &'static [&'static [&'static str]],
    pub weekend: &'static [&'static [&'static str]],
    pub priorities: [&'static [&'static str]; 5],
}

pub const ES: Vocabulary = Vocabulary {
    today: &["hoy"],
    tomorrow: &["mañana"],
    day_after: &["pasado"],
    spelled_day: &[],
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
    ends_prep: &["hasta"],
    time_prep: &["a", "al", "las", "la"],
    clock_prep: &["las"],
    past_prep: &["hace"],
    day_part: &["mañana", "tarde", "noche", "madrugada"],
    pm_part: &["tarde", "noche"],
    night_part: &["noche", "madrugada"],
    part_prep: &["de", "del", "por"],
    linker: &[
        "y", "e", "o", "u", "pero", "sobre", "con", "sin", "desde", "hacia", "según", "segun",
        "que",
    ],
    genitive: &["de", "del"],
    loose_ends: &[
        "y", "e", "o", "u", "ni", "al", "a", "el", "la", "los", "las", "por",
    ],
    idioms: &["24/7"],
    noon: &["mediodía", "mediodia"],
    in_prep: &["en", "dentro"],
    spans_prep: &["por", "durante"],
    days_unit: &["día", "dia", "días", "dias"],
    weeks_unit: &["semana", "semanas"],
    months_unit: &["mes", "meses"],
    years_unit: &["año", "ano", "años", "anos"],
    every: &[&["cada"], &["todos", "los"], &["todas", "las"]],
    cadences: &[
        (&["diariamente"], Unit::Day),
        (&["semanalmente"], Unit::Week),
        (&["mensualmente"], Unit::Month),
        (&["anualmente"], Unit::Year),
    ],
    one: &["un", "una"],
    this_week: &[&["esta", "semana"]],
    next_week: &[
        &["próxima", "semana"],
        &["proxima", "semana"],
        &["semana", "que", "viene"],
        &["siguiente", "semana"],
    ],
    next_month: &[
        &["próximo", "mes"],
        &["proximo", "mes"],
        &["mes", "que", "viene"],
        &["siguiente", "mes"],
    ],
    first: &["primero", "primer", "1º", "1o"],
    end_of_month: &[&["fin", "de", "mes"], &["fin", "mes"]],
    weekend: &[&["finde"], &["fin", "de", "semana"]],
    priorities: [
        &["hacer", "ahora"],
        &["planificar", "decidir", "importante"],
        &["delegar", "delega"],
        &["prescindible", "descartable"],
        &["sinclasificar", "ninguna"],
    ],
};

pub const EN: Vocabulary = Vocabulary {
    today: &["today"],
    tomorrow: &["tomorrow"],
    day_after: &["after"],
    spelled_day: &["day"],
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
    ends_prep: &["until"],
    time_prep: &["at"],
    clock_prep: &["at"],
    past_prep: &["ago"],
    day_part: &["morning", "afternoon", "evening", "night"],
    pm_part: &["afternoon", "evening", "night"],
    night_part: &["night"],
    part_prep: &["in", "at", "of"],
    linker: &[
        "and",
        "or",
        "to",
        "but",
        "about",
        "with",
        "without",
        "from",
        "into",
        "regarding",
        "that",
    ],
    genitive: &["of"],
    loose_ends: &["and", "or", "nor", "to", "for", "a"],
    idioms: &["24/7"],
    noon: &["noon", "midday"],
    in_prep: &["in", "within"],
    spans_prep: &["for", "during"],
    days_unit: &["day", "days"],
    weeks_unit: &["week", "weeks"],
    months_unit: &["month", "months"],
    years_unit: &["year", "years"],
    every: &[&["every"], &["each"]],
    cadences: &[
        (&["daily"], Unit::Day),
        (&["weekly"], Unit::Week),
        (&["monthly"], Unit::Month),
        (&["yearly", "annually"], Unit::Year),
    ],
    one: &["a", "an", "one"],
    this_week: &[&["this", "week"]],
    next_week: &[&["next", "week"]],
    next_month: &[&["next", "month"]],
    first: &["first"],
    end_of_month: &[&["end", "of", "month"], &["end", "of", "the", "month"]],
    weekend: &[&["this", "weekend"], &["weekend"]],
    priorities: [
        &["do", "now"],
        &["schedule", "decide", "important"],
        &["delegate"],
        &["minor", "wont"],
        &["unclassified", "none"],
    ],
};

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

    pub fn spoken(&self, p: Priority) -> &'static str {
        let placed = [
            Priority::Do,
            Priority::Decide,
            Priority::Delegate,
            Priority::Minor,
            Priority::Unset,
        ];
        placed
            .iter()
            .position(|one| *one == p)
            .and_then(|i| self.priorities[i].first().copied())
            .unwrap_or("")
    }

    pub fn priority(&self, word: &str) -> Option<Priority> {
        let lower = word.to_lowercase();
        let placed = [
            Priority::Do,
            Priority::Decide,
            Priority::Delegate,
            Priority::Minor,
            Priority::Unset,
        ];
        self.priorities
            .iter()
            .position(|names| names.contains(&lower.as_str()))
            .map(|i| placed[i])
    }

    pub fn month_index(&self, word: &str) -> Option<u8> {
        self.months
            .iter()
            .position(|m| m.contains(&word))
            .map(|i| i as u8 + 1)
    }
}
