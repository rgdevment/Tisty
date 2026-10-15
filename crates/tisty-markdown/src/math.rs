use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};

const AT_MOST: usize = 8 * 1024;

/// MathML the system draws by itself, or nothing when the formula cannot be read.
pub fn mathml(latex: &str, block: bool) -> Option<String> {
    if latex.trim().is_empty() || latex.len() > AT_MOST {
        return None;
    }
    let config = MathCoreConfig {
        ignore_unknown_commands: true,
        ..Default::default()
    };
    let display = if block {
        MathDisplay::Block
    } else {
        MathDisplay::Inline
    };
    crate::guarded(|| {
        let converter = LatexToMathML::new(config, Vec::new()).ok()?;
        converter
            .convert_with_local_state(latex, display)
            .ok()
            .map(|done| done.mathml)
    })
}

// Strict on purpose, so prices never read as formulas: each `$` touches its words, no digit after.
pub(crate) fn fold(spans: &mut Vec<crate::Span>, text: &str) {
    let dollars = dollars(spans, text);
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < dollars.len() {
        let open = dollars[at];
        let opens = bytes
            .get(open + 1)
            .is_some_and(|b| !b.is_ascii_whitespace());
        let shut = opens
            .then(|| {
                dollars[at + 1..].iter().position(|close| {
                    let close = *close;
                    !bytes[close - 1].is_ascii_whitespace()
                        && !bytes.get(close + 1).is_some_and(u8::is_ascii_digit)
                        && !text[open..close].contains('\n')
                })
            })
            .flatten();
        let Some(step) = shut else {
            at += 1;
            continue;
        };
        let close = dollars[at + 1 + step];
        spans.push(crate::Span {
            mark: crate::Mark::Math,
            at: open..close + 1,
        });
        at += step + 2;
    }
    spans.sort_by_key(|one| (one.at.start, std::cmp::Reverse(one.at.end)));
}

fn dollars(spans: &[crate::Span], text: &str) -> Vec<usize> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    for span in spans.iter().filter(|one| one.mark == crate::Mark::Text) {
        for at in span.at.clone() {
            let lone = bytes[at] == b'$'
                && (at == 0 || !matches!(bytes[at - 1], b'\\' | b'$'))
                && bytes.get(at + 1) != Some(&b'$');
            if lone {
                found.push(at);
            }
        }
    }
    found
}
