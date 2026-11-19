use tisty_markdown::{Block, Mark};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Highlight {
    pub line: usize,
    pub said: String,
}

/// What the writer highlighted with a pen or set in bold, as written, in reading order.
pub fn highlighted(body: &str) -> Vec<Highlight> {
    let doc = tisty_markdown::read(body);
    let mut spans = Vec::new();
    for block in doc.blocks() {
        spans_of(block, &mut spans);
    }
    spans.sort_by_key(|at: &std::ops::Range<usize>| (at.start, std::cmp::Reverse(at.end)));
    let mut kept: Vec<std::ops::Range<usize>> = Vec::new();
    for at in spans {
        if kept.last().is_some_and(|outer| at.end <= outer.end) {
            continue;
        }
        kept.push(at);
    }
    kept.into_iter()
        .filter_map(|at| {
            let said = doc.text().get(at.clone())?;
            Some(Highlight {
                line: doc.text()[..at.start].matches('\n').count() + 1,
                said: said.to_string(),
            })
        })
        .collect()
}

fn spans_of(block: &Block, into: &mut Vec<std::ops::Range<usize>>) {
    into.extend(
        block
            .spans
            .iter()
            .filter(|one| matches!(one.mark, Mark::Strong | Mark::Pen { .. }))
            .map(|one| one.at.clone()),
    );
    for inner in &block.inner {
        spans_of(inner, into);
    }
}

#[cfg(test)]
#[path = "highlighted_test.rs"]
mod tests;
