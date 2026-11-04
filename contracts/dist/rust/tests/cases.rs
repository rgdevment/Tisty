use serde_json::Value;
use tisty_contracts::{CONTRACT, check, check_html};

#[test]
fn every_case_says_what_the_contract_says() {
    let all: Value = serde_json::from_str(CONTRACT).unwrap();
    let contracts = all["contracts"].as_object().unwrap();
    let mut ran = 0;
    for (name, contract) in contracts {
        for one in contract["cases"].as_array().into_iter().flatten() {
            let wrong = check(name, &one["value"]);
            let holds = one["holds"].as_bool().unwrap();
            assert_eq!(wrong.is_empty(), holds, "{name}: {}: {wrong:?}", one["says"]);
            ran += 1;
        }
        for one in contract["html_cases"].as_array().into_iter().flatten() {
            let wrong = check_html(one["html"].as_str().unwrap());
            let holds = one["holds"].as_bool().unwrap();
            assert_eq!(wrong.is_empty(), holds, "{name} html: {}: {wrong:?}", one["says"]);
            ran += 1;
        }
    }
    assert!(ran > 0);
}
