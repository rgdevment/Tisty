//! Tisty's domain core.
//!
//! No terminal output belongs here: the CLI and the GUI are both clients of this
//! API, and anything printed from the core leaks into the GUI as garbage.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
