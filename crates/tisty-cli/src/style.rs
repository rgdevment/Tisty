use std::io::IsTerminal;
use std::sync::OnceLock;

pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const GREEN: &str = "\x1b[32m";

/// Honours NO_COLOR and pipes: a redirected listing must not carry escape
/// sequences into whatever reads it.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal())
}

pub fn paint(code: &str, text: &str) -> String {
    if enabled() {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}

pub fn dim(text: &str) -> String {
    paint(DIM, text)
}

pub fn bold(text: &str) -> String {
    paint(BOLD, text)
}
