pub(crate) fn today() -> jiff::civil::Date {
    jiff::Zoned::now().date()
}

pub(crate) fn zone() -> String {
    jiff::Zoned::now()
        .time_zone()
        .iana_name()
        .unwrap_or("UTC")
        .to_string()
}
