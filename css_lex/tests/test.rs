use css_lex::lexer::*;
use css_lex::to_json::*;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;

fn run_json_tests<T, F>(json_data: &str, parse: F)
where
    T: Serialize + Debug,
    F: Fn(String) -> T,
{
    let items = match serde_json::from_str::<Value>(json_data) {
        Ok(Value::Array(items)) => items,
        _ => panic!("Invalid JSON"),
    };
    assert!(items.len() % 2 == 0);
    let mut input: Option<String> = None;
    for item in items.into_iter() {
        match (&input, item) {
            (&None, Value::String(string)) => input = Some(string),
            (&Some(_), expected) => {
                let css = input.take().expect("Expected input to be Some");
                let result =
                    serde_json::to_value(parse(css.clone())).expect("Serialization failed");
                if !json_almost_equals(&result, &expected) {
                    panic!("got: {:?}\nexpected: {:?}", result, expected);
                }
            }
            _ => panic!("Unexpected JSON"),
        };
    }
}

#[test]
fn tokenize_simple() {
    let mut t = tokenize("a");
    assert!(
        t.next()
            == Some((
                Token::Ident("a".to_string()),
                SourceLocation { line: 1, column: 1 }
            ))
    );
}

#[test]
fn test_tokenize_json() {
    run_json_tests(include_str!("tokens.json"), |input| {
        let tokenizer = tokenize(&input);
        let mut token_list = vec![];
        for token in tokenizer {
            token_list.push(token);
        }
        list_to_json(&token_list)
    });
}
