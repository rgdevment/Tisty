use serde_json::{Value, json};

use super::jsonrpc::told;

pub(super) fn topics() -> Vec<String> {
    tisty_help::pages(false)
        .iter()
        .map(|one| match one.slug() {
            slug if slug.is_empty() => "index".to_string(),
            slug => slug,
        })
        .collect()
}

// Asked in either language, a topic names the same page: the person's words may not be English.
fn page_of(spanish: bool, topic: &str) -> Option<&'static tisty_help::Page> {
    let slug = if topic == "index" { "" } else { topic };
    tisty_help::found(spanish, slug).or_else(|| {
        let other = tisty_help::found(!spanish, slug)?;
        tisty_help::pages(spanish)
            .iter()
            .find(|one| one.page == other.twin)
    })
}

fn indexed(spanish: bool) -> String {
    let mut out = String::from(
        "How Tisty itself works, from the pages its own Help and tisty.app show: tasks, documents, \
         sync, encryption, assistants, shortcuts. Ask `help` with a `topic` for one page. A link \
         written help:<topic> names another page to ask for.\n",
    );
    let mut section = "";
    for one in tisty_help::pages(spanish)
        .iter()
        .filter(|one| one.order > 0)
    {
        if one.section != section {
            section = one.section;
            out.push_str(&format!("\n{section}:\n"));
        }
        out.push_str(&format!("- {}: {}\n", one.slug(), one.title()));
    }
    out
}

pub(super) fn help(args: &Value) -> Value {
    let spanish = args["lang"].as_str() == Some("es");
    let asked = args["topic"].as_str();
    let said = match asked {
        None => indexed(spanish),
        Some(topic) => match page_of(spanish, topic) {
            Some(page) => format!("# {}\n{}", page.title(), page.readable(spanish)),
            None => format!("No topic is called {topic:?}.\n\n{}", indexed(spanish)),
        },
    };
    told(
        said.clone(),
        json!({
            "help": said,
            "topic": asked.unwrap_or("index"),
            "lang": if spanish { "es" } else { "en" },
        }),
    )
}

#[cfg(test)]
#[path = "helping_test.rs"]
mod tests;
