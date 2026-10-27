use super::{Place, as_read, at_place, diagrams, redrawn, still};

// One case per line: body | the diagram sitting where the page put it (`-` when none does) | which one it was counting from the top | was | now | body after (`-` when refused).
// `⏎` is a line break and `␍` a carriage return, so a whole document fits on its line.
const CASES: &str = "
```mermaid⏎graph TD⏎A-->B⏎```⏎ | 0 | 0 | graph TD⏎A-->B⏎ | flowchart LR⏎A-->C⏎ | ```mermaid⏎flowchart LR⏎A-->C⏎```⏎
```mmd⏎graph TD⏎```⏎ | 0 | 0 | graph TD⏎ | mindmap⏎  root⏎ | ```mmd⏎mindmap⏎  root⏎```⏎
~~~mermaid⏎graph TD⏎~~~⏎ | 0 | 0 | graph TD⏎ | graph LR⏎ | ~~~mermaid⏎graph LR⏎~~~⏎
# Title⏎⏎```mermaid⏎a⏎```⏎⏎after⏎ | 0 | 0 | a⏎ | b⏎c⏎ | # Title⏎⏎```mermaid⏎b⏎c⏎```⏎⏎after⏎
> ```mermaid⏎> graph TD⏎> A-->B⏎> ```⏎ | 0 | 0 | graph TD⏎A-->B⏎ | graph LR⏎X-->Y⏎ | > ```mermaid⏎> graph LR⏎> X-->Y⏎> ```⏎
- item⏎⏎  ```mermaid⏎  graph TD⏎  ```⏎ | 0 | 0 | graph TD⏎ | graph LR⏎A-->B⏎ | - item⏎⏎  ```mermaid⏎  graph LR⏎  A-->B⏎  ```⏎
- ```mermaid⏎  graph TD⏎  ```⏎ | 0 | 0 | graph TD⏎ | graph LR⏎A⏎ | - ```mermaid⏎  graph LR⏎  A⏎  ```⏎
```mermaid⏎```⏎ | 0 | 0 |  | mindmap⏎  root⏎ | ```mermaid⏎mindmap⏎  root⏎```⏎
> ```mermaid⏎> ```⏎ | 0 | 0 |  | graph TD⏎ | > ```mermaid⏎> graph TD⏎> ```⏎
```mermaid⏎graph TD⏎```⏎ | 0 | 0 | graph LR⏎ | flowchart TB⏎ | -
```mermaid⏎a⏎```⏎⏎```mermaid⏎a⏎```⏎ | 1 | 1 | a⏎ | b⏎ | ```mermaid⏎a⏎```⏎⏎```mermaid⏎b⏎```⏎
intro⏎⏎```mermaid⏎a⏎```⏎ | - | 0 | a⏎ | b⏎ | intro⏎⏎```mermaid⏎b⏎```⏎
```mermaid⏎a⏎```⏎⏎```mermaid⏎c⏎```⏎ | - | 1 | c⏎ | d⏎ | ```mermaid⏎a⏎```⏎⏎```mermaid⏎d⏎```⏎
```mermaid⏎x⏎```⏎⏎```mermaid⏎a⏎```⏎ | 0 | 0 | a⏎ | b⏎ | -
```mermaid⏎new⏎```⏎⏎```mermaid⏎a⏎```⏎ | - | 0 | a⏎ | b⏎ | -
```mermaid⏎a⏎```⏎⏎```mermaid⏎a⏎```⏎ | - | 1 | a⏎ | b⏎ | -
```mermaid␍⏎graph TD␍⏎```␍⏎ | 0 | 0 | graph TD⏎ | graph LR⏎A⏎ | ```mermaid␍⏎graph LR␍⏎A␍⏎```␍⏎
intro␍⏎⏎```mermaid⏎a⏎```⏎ | 0 | 0 | a⏎ | b⏎ | intro␍⏎⏎```mermaid⏎b⏎```⏎
```js⏎graph TD⏎```⏎ | - | 0 | graph TD⏎ | graph LR⏎ | -
```mermaid⏎graph TD⏎```⏎ | 0 | 0 | graph TD⏎ |  | ```mermaid⏎```⏎
```mermaid⏎graph TD⏎```⏎ | 0 | 0 | graph TD⏎ | graph LR | ```mermaid⏎graph LR⏎```⏎
";

fn unfolded(said: &str) -> String {
    said.trim().replace('⏎', "\n").replace('␍', "\r")
}

#[test]
fn a_diagram_is_written_over_only_while_it_still_says_what_was_opened() {
    for case in CASES.lines().filter(|line| !line.trim().is_empty()) {
        let cells: Vec<&str> = case.split(" | ").collect();
        let [body, sitting, nth, was, now, after] = cells[..] else {
            panic!("a case has six cells: {case}");
        };
        let body = unfolded(body);
        let at = sitting
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|index| diagrams(&body).get(index).map(|one| one.at.start))
            .unwrap_or(usize::MAX);
        let place = Place {
            at,
            nth: nth.trim().parse().expect(case),
        };
        let wanted = (after.trim() != "-").then(|| unfolded(after));

        assert_eq!(
            redrawn(&body, place, &unfolded(was), &unfolded(now)).map(|(body, _)| body),
            wanted,
            "{case}"
        );
    }
}

#[test]
fn every_diagram_is_found_in_reading_order_even_inside_a_quote_or_a_list() {
    let body = "```mermaid\na\n```\n\n> ```mmd\n> b\n> ```\n\n- one\n\n  ```mermaid\n  c\n  ```\n\n```rust\nd\n```\n";

    let said: Vec<String> = diagrams(body).into_iter().map(|one| one.source).collect();

    assert_eq!(said, ["a\n", "b\n", "c\n"]);
}

#[test]
fn a_diagram_written_over_keeps_its_place_and_one_found_elsewhere_says_where() {
    let body = "intro\n\n```mermaid\na\n```\n";
    let here = Place {
        at: diagrams(body)[0].at.start,
        nth: 0,
    };

    assert_eq!(
        redrawn(body, here, "a\n", "b\n").map(|(_, at)| at),
        Some(here)
    );
    let moved = Place { at: 0, nth: 0 };
    assert_eq!(
        redrawn(body, moved, "a\n", "b\n").map(|(_, at)| at),
        Some(here)
    );
}

#[test]
fn the_place_of_a_diagram_is_the_one_the_page_names() {
    let body = "# T\n\n```mermaid\na\n```\n";

    let found = diagrams(body);

    assert_eq!(found.len(), 1);
    assert_eq!(&body[found[0].at.clone()], "```mermaid\na\n```");
}

#[test]
fn what_is_written_reads_back_as_the_window_will_hold_it() {
    for now in [
        "graph TD\nA-->B",
        "graph TD\r\nA-->B\r\n",
        "graph TD\nA-->B\n",
    ] {
        let first = Place { at: 0, nth: 0 };
        let (body, place) = redrawn("```mermaid\nx\n```\n", first, "x\n", now).expect(now);
        let back = diagrams(&body).into_iter().nth(place.nth);

        assert_eq!(back.map(|one| one.source), Some(as_read(now)), "{now:?}");
    }
}

#[test]
fn a_window_finds_its_diagram_again_by_place_or_order_whatever_it_says_now() {
    let body = "intro\n\n```mermaid\nchanged\n```\n";
    let sitting = diagrams(body)[0].at.start;
    let here = Place {
        at: sitting,
        nth: 0,
    };

    assert_eq!(at_place(body, here), Some((here, "changed\n".to_string())));
    assert_eq!(
        at_place(body, Place { at: 0, nth: 0 }),
        Some((here, "changed\n".to_string()))
    );
    assert_eq!(at_place(body, Place { at: 0, nth: 1 }), None);
}

// One hand-written document per line (`⏎` is a line break): the page must name each diagram where the window finds it.
const WRITTEN_BY_HAND: &str = "
# Pagos⏎⏎```mermaid⏎erDiagram⏎  a ||--o{ b : has⏎```⏎
> Nota⏎>⏎> ```mmd⏎> mindmap⏎>   root⏎> ```⏎
- uno⏎⏎  ~~~mermaid⏎  graph TD⏎  ~~~⏎⏎- dos⏎
---⏎title: Con propiedades⏎---⏎⏎```mermaid⏎classDiagram⏎```⏎⏎```mermaid⏎sequenceDiagram⏎```⏎
";

#[test]
fn the_page_names_each_diagram_where_the_window_finds_it() {
    for case in WRITTEN_BY_HAND
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let body = unfolded(case);
        let page = crate::html(&crate::read(&body));
        let found = diagrams(&body);

        assert!(!found.is_empty(), "{case}");
        for one in found {
            let named = format!("data-at=\"{}-{}\"", one.at.start, one.at.end);
            let figure = page.contains(&format!("<figure class=\"diagram\" {named}"));
            assert!(figure, "{case}: no figure {named} in {page}");
        }
    }
}

// One case per line: body | place the page sent | order the page sent | what the page drew | the order opened (`-` when refused).
const OPENED: &str = "
intro⏎⏎```mermaid⏎a⏎```⏎ | 0 | 0 | a⏎ | 0
```mermaid⏎new⏎```⏎⏎```mermaid⏎a⏎```⏎ | 0 | 0 | a⏎ | -
```mermaid⏎a⏎```⏎⏎```mermaid⏎b⏎```⏎ | 999 | 1 | b⏎ | 1
```mermaid⏎a⏎```⏎⏎```mermaid⏎b⏎```⏎ | 999 | 1 | c⏎ | -
```mermaid⏎a⏎```⏎⏎```mermaid⏎a⏎```⏎ | 999 | 0 | a⏎ | -
";

#[test]
fn a_page_a_save_behind_still_opens_the_diagram_it_drew_and_never_a_neighbour() {
    for case in OPENED.lines().filter(|line| !line.trim().is_empty()) {
        let cells: Vec<&str> = case.split(" | ").collect();
        let [body, at, nth, drew, opened] = cells[..] else {
            panic!("a case has five cells: {case}");
        };
        let place = Place {
            at: at.trim().parse().expect(case),
            nth: nth.trim().parse().expect(case),
        };
        let wanted = opened.trim().parse::<usize>().ok();

        let found = still(&unfolded(body), place, &unfolded(drew)).map(|(place, _)| place.nth);

        assert_eq!(found, wanted, "{case}");
    }
}
