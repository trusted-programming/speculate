use crate::lexer::*;
use serde_json::{json, Value};

pub fn json_almost_equals(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-6
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|(a, b)| json_almost_equals(a, b))
        }
        (Value::Object(_), Value::Object(_)) => todo!(),
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

impl Token {
    pub fn to_json(&self) -> Value {
        match self {
            Token::Ident(value) => json!(["ident", value]),
            Token::Function(name) => json!(["function", name]),
            Token::AtKeyword(value) => json!(["at-keyword", value]),
            Token::Hash(value) => json!(["hash", value, "unrestricted"]),
            Token::IDHash(value) => json!(["hash", value, "id"]),
            Token::String(value) => json!(["string", value]),
            Token::BadString => json!(["error", "bad-string"]),
            Token::URL(value) => json!(["url", value]),
            Token::BadURL => json!(["error", "bad-url"]),
            Token::Delim('\\') => json!("\\"),
            Token::Delim(value) => json!(value.to_string()),
            Token::Number(value) => {
                let mut res = json!(vec![json!("number")]);
                let value_array = value.to_json();

                if let (Some(first), Some(second)) = (res.as_array_mut(), value_array.as_array()) {
                    first.extend(second.clone());
                }

                res
            }
            Token::Percentage(value) => {
                let mut res = json!(vec![json!("percentage")]);
                let value_array = value.to_json();

                if let (Some(first), Some(second)) = (res.as_array_mut(), value_array.as_array()) {
                    first.extend(second.clone());
                }
                res
            }
            Token::Dimension(value, unit) => {
                let mut res = json!(vec![json!("dimension")]);
                let value_array = value.to_json();

                if let (Some(first), Some(second)) = (res.as_array_mut(), value_array.as_array()) {
                    first.extend(second.clone());
                    first.push(json!(unit));
                }
                res
            }
            Token::UnicodeRange(s, e) => {
                json!(vec![json!("unicode-range"), json!(s), json!(e)])
            }
            Token::IncludeMatch => json!("~="),
            Token::DashMatch => json!("|="),
            Token::PrefixMatch => json!("^="),
            Token::SuffixMatch => json!("$="),
            Token::SubstringMatch => json!("*="),
            Token::Column => json!("||"),
            Token::WhiteSpace => json!(" "),
            Token::CDO => json!("<!--"),
            Token::CDC => json!("-->"),
            Token::Colon => json!(":"),
            Token::Semicolon => json!(";"),
            Token::Comma => json!(","),

            Token::LeftBracket => json!("["),
            Token::RightBracket => json!("]"),
            Token::LeftParen => json!("("),
            Token::RightParen => json!(")"),
            Token::LeftCurlyBracket => json!("{"),
            Token::RightCurlyBracket => json!("}"),
        }
    }
}

impl NumericValue {
    pub fn to_json(&self) -> Value {
        json!([
            self.representation,
            self.value,
            match self.int_value {
                Some(_) => "integer",
                None => "number",
            }
        ])
    }
}

pub fn list_to_json(list: &[(Token, SourceLocation)]) -> Vec<Value> {
    list.iter().map(|(c, _)| c.to_json()).collect()
}
