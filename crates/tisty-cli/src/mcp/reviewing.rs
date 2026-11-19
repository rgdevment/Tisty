use serde_json::{Value, json};
use tisty_core::{Paths, State, review};

pub(super) const HOW_TO_HELP: &str = "Offer to walk it with the person, one stop at a time: \
what waits to be decided, what was marked as past, what was forgotten. The window's Review shows \
all three. Help with what they ask, such as sorting an idea as `rules` «ideas» says, giving a \
task its list, or saying why a document looks past. The person decides: never archive, delete, \
mark or close anything on your own. Finishing the review is theirs, and closing this round brings \
the next one.";

pub(super) fn glance(
    paths: &Paths,
    state: &State,
    today: jiff::civil::Date,
) -> Option<(String, Value)> {
    let task = review::due(state, today)?;
    let opened = tisty_core::forgotten::opened_here(&paths.opened_file());
    let counted = review::waiting(&paths.docs(), state, &opened, tisty_core::lately::now());
    let list = task
        .list
        .and_then(|id| state.lists.get(&id))
        .map(|one| one.name.as_str())
        .unwrap_or_default();
    let day = task
        .date
        .as_ref()
        .map(|one| one.date().to_string())
        .unwrap_or_default();
    let when = match day == today.to_string() {
        true => "today".to_string(),
        false => format!("since {day}"),
    };
    let held = match list {
        "" => String::new(),
        named => format!(" in {named}"),
    };
    let said = format!(
        "The person's weekly review, «{}»{held}, is due {when}: {} task(s) with no list and \
         {} idea(s) on Brainstorm to decide, {} document(s) marked as past, and {} not opened on \
         this computer nor written for 60 days. {HOW_TO_HELP}",
        task.title, counted.unlisted, counted.ideas, counted.marked, counted.forgotten,
    );
    let glanced = json!({
        "task": task.id.to_string(),
        "title": task.title,
        "list": list,
        "due": day,
        "waiting": counted,
        "said": said,
    });
    Some((said, glanced))
}
