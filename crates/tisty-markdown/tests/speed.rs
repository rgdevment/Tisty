use std::time::Instant;

const SAMPLE: &str = include_str!("../../../app/src-tauri/resources/guide/es/muestra.md");

fn big() -> String {
    let plain: String = SAMPLE
        .lines()
        .filter(|line| !["]:", "[^", ":::"].iter().any(|one| line.contains(one)))
        .map(|line| format!("{line}\n"))
        .collect();
    let mut text = String::new();
    while text.len() < 500 * 1024 {
        text.push_str(&plain);
        text.push_str("\n\n");
    }
    text
}

fn timed<T>(label: &str, rounds: u32, mut run: impl FnMut() -> T) {
    let start = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(run());
    }
    let each = start.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds);
    println!("{label}: {each:.2} ms");
}

#[test]
#[ignore = "a measure, run by hand with --release -- --ignored --nocapture"]
fn a_key_on_a_page_of_500_kb() {
    let text = big();
    let middle = text.len() / 2;
    println!("text: {} KB", text.len() / 1024);
    timed("read", 10, || tisty_markdown::read(&text));
    let doc = tisty_markdown::read(&text);
    timed("tags_in", 10, || tisty_markdown::tags::tags_in(&text));
    timed("placed_near", 10, || {
        tisty_markdown::placed_near(&doc, middle, middle + 1, Some(doc.labels()))
    });
    let heading = text[middle..]
        .find("\n# ")
        .map_or(middle, |at| middle + at + 3);
    timed("placed_near in a heading", 10, || {
        tisty_markdown::placed_near(&doc, heading, heading + 1, Some(doc.labels()))
    });
    timed("read + placed_near", 10, || {
        tisty_markdown::placed_near(
            &tisty_markdown::read(&text),
            middle,
            middle + 1,
            Some(doc.labels()),
        )
    });
    let at = (0..=middle)
        .rev()
        .find(|at| text.is_char_boundary(*at))
        .unwrap_or(0);
    let typed = format!("{}x{}", &text[..at], &text[at..]);
    let mut docs: Vec<_> = (0..10).map(|_| doc.clone()).collect();
    timed("reread a key", 10, || {
        tisty_markdown::reread(docs.pop().unwrap_or_else(|| doc.clone()), &typed)
    });
    let mut docs: Vec<_> = (0..10).map(|_| doc.clone()).collect();
    timed("reread an edit", 10, || {
        tisty_markdown::reread_edit(docs.pop().unwrap_or_else(|| doc.clone()), at..at, "x")
    });
}
