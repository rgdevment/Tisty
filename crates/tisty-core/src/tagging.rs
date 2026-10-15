use crate::model::Tag;

pub use tisty_markdown::tags::AT_MOST;

pub fn worth_keeping(tags: &[Tag]) -> Vec<Tag> {
    tags.iter()
        .filter(|one| one.worth_reading())
        .cloned()
        .collect()
}

pub fn tags_in(body: &str) -> Vec<Tag> {
    tisty_markdown::tags::tags_in(body)
        .iter()
        .filter_map(|name| Tag::new(name).ok())
        .collect()
}

#[cfg(test)]
#[path = "tagging_test.rs"]
mod tests;
