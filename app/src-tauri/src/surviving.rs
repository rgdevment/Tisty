use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal};

pub(crate) fn survived<T>(
    work: impl FnOnce() -> T,
    on: &'static str,
    said: &'static str,
    facts: &[(&'static str, Fact)],
) -> Option<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
        .inspect_err(|_| witness::error(on, said, facts))
        .ok()
}

pub(crate) fn caught<T>(command: &'static str, work: impl FnOnce() -> T) -> Answer<T> {
    survived(
        work,
        channel::WINDOW,
        "a command broke and was answered with a refusal",
        &[("command", Fact::Word(command))],
    )
    .ok_or_else(|| Refusal::of("internal"))
}
