macro_rules! belongs {
    () => {
        " If someone will read it later (the person, another agent, you on another day), it belongs \
         in Tisty: work to pick up is a task, `propose`, or `note` and `tick` with `state` on the \
         task it is about; something kept to read again is a document, `brain` first and then \
         `append_doc`, `edit_doc` or `write_doc`. Then hand over its title, never a path. If the \
         person asked for a file in those words, a program reads it from there, or you do not have \
         Tisty's tools, make the same call again: it goes through, and this folder is not asked \
         about again in this session."
    };
}

pub(crate) const FILE_LEFT_OUTSIDE: &str = concat!(
    "This would leave a text file outside any repository.",
    belongs!()
);

pub(crate) const COMMAND_LEAVES_OUTSIDE: &str = concat!(
    "This command would leave a text file outside any repository, so none of it ran.",
    belongs!()
);

pub(crate) const CARD_LEFT_OUTSIDE: &str = "This would leave the work in a card only this app \
    shows. If it should reach another agent or another day, file it in Tisty first: `propose` it \
    in its list with what it is and its steps, or `note` it on the task it came from; the card can \
    then carry just that task's title. If the person asked for this card, or you do not have \
    Tisty's tools, make the same call again: it goes through, and cards are not asked about again \
    in this session.";

#[cfg(test)]
#[path = "outside_test.rs"]
mod tests;
