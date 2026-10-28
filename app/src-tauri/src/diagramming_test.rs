use super::found_in;

// One per line: a body | each diagram found, as which one it is and what it says.
const FOUND: &str = "
# Sin diagramas⏎⏎texto | 
```mermaid⏎graph TD⏎A-->B⏎``` | 0:graph TD⏎A-->B⏎
```mermaid⏎a⏎```⏎⏎> ```mmd⏎> b⏎> ``` | 0:a⏎ ; 1:b⏎
";

#[test]
fn every_diagram_of_a_document_is_offered_with_the_document_it_comes_from() {
    for case in FOUND.lines().filter(|line| !line.trim().is_empty()) {
        let (body, said) = case.split_once(" | ").expect("a case has two cells");
        let found = found_in("ab12-0001", "Pagos", &body.replace('⏎', "\n"));
        let expected: Vec<String> = said
            .split(" ; ")
            .map(str::trim)
            .filter(|one| !one.is_empty())
            .map(|one| one.replace('⏎', "\n"))
            .collect();
        let got: Vec<String> = found
            .iter()
            .map(|one| format!("{}:{}", one.nth, one.source))
            .collect();
        assert_eq!(got, expected, "{case}");
        assert!(
            found
                .iter()
                .all(|one| one.file == "ab12-0001" && one.title == "Pagos")
        );
    }
}
