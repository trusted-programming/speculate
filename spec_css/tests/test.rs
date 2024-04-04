use css_lex::{json_almost_equals, list_to_json};
use serde::Serialize;
use serde_json::Value;
use spec_css::*;
use std::sync::Arc;

fn run_json_tests<T: Serialize>(json_data: &str, parse: &dyn Fn(String) -> T) {
    let items = serde_json::from_str(json_data).expect("Invalid JSON");
    match items {
        Value::Array(items) => {
            assert!(items.len() % 2 == 0, "Items list should be even");

            let mut input: Option<String> = None;
            for item in items {
                match (&input, &item) {
                    (None, Value::String(string)) => input = Some(string.clone()),
                    (Some(_), expected) => {
                        let css = input.take().expect("Input was None");
                        let result =
                            serde_json::to_value(parse(css.clone())).expect("Serialization failed");
                        if !json_almost_equals(&result, expected) {
                            panic!("got: {}\nexpected: {}", result, expected);
                        }
                    }
                    _ => panic!("Unexpected JSON"),
                };
            }
        }
        _ => panic!("Invalid JSON structure"),
    }
}

// This could be replaced with JSON tests.
#[test]
fn test_next_token_start() {
    let css = Arc::new(String::from("cls1 : cls2 {prop: val;}"));

    assert!(next_token_start(css.clone(), 8) == 11);
    assert!(next_token_start(css.clone(), 4) == 4);
    assert!(next_token_start(css.clone(), 13) == 13);
    assert!(next_token_start(css.clone(), 14) == 17);
    assert!(next_token_start(css.clone(), 0) == 0);
}

#[test]
fn test_spec_token_json() {
    run_json_tests(include_str!("../../css_lex/tests/tokens.json"), &|input| {
        list_to_json(&spec_tokenize(input, 1).1)
    });
    run_json_tests(include_str!("../../css_lex/tests/tokens.json"), &|input| {
        list_to_json(&spec_tokenize(input, 2).1)
    });
    run_json_tests(include_str!("../../css_lex/tests/tokens.json"), &|input| {
        list_to_json(&spec_tokenize(input, 3).1)
    });
}
