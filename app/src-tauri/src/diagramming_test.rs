use super::found_in;

// One per line: a body | each diagram found, as the heading it sits under, which one it is and what it says.
const FOUND: &str = "
# Sin diagramas⏎⏎texto |
```mermaid⏎graph TD⏎A-->B⏎``` | :0:graph TD⏎A-->B⏎
```mermaid⏎a⏎```⏎⏎> ```mmd⏎> b⏎> ``` | :0:a⏎ ; :1:b⏎
# Pagos⏎⏎```mermaid⏎a⏎```⏎⏎## Cobro⏎⏎```mermaid⏎b⏎```⏎⏎### Vuelto⏎⏎texto⏎⏎```mermaid⏎c⏎``` | :0:a⏎ ; Cobro:1:b⏎ ; Vuelto:2:c⏎
";

#[test]
fn every_diagram_of_a_document_is_offered_with_its_document_and_the_heading_it_sits_under() {
    for case in FOUND.lines().filter(|line| !line.trim().is_empty()) {
        let (body, said) = case.split_once(" |").expect("a case has two cells");
        let found = found_in("ab12-0001", "Pagos", &body.replace('⏎', "\n"));
        let expected: Vec<String> = said
            .split(" ; ")
            .map(str::trim)
            .filter(|one| !one.is_empty())
            .map(|one| one.replace('⏎', "\n"))
            .collect();
        let got: Vec<String> = found
            .iter()
            .map(|one| format!("{}:{}:{}", one.section, one.nth, one.source))
            .collect();
        assert_eq!(got, expected, "{case}");
        assert!(
            found
                .iter()
                .all(|one| one.file == "ab12-0001" && one.title == "Pagos")
        );
    }
}
