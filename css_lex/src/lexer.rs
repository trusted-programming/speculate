use std::str::FromStr;
use std::{char, str, sync::Arc};

use serde_json::{json, Value};

#[derive(Clone)]
pub struct NumericValue {
    pub representation: String,
    pub value: f64,
    pub int_value: Option<i64>,
}

impl PartialEq for NumericValue {
    fn eq(&self, other: &Self) -> bool {
        // Compare the `representation` and `int_value` fields for exact equality.
        self.representation == other.representation &&
        self.int_value == other.int_value &&
        // Compare the `value` fields with a tolerance for floating-point precision issues.
        (self.value - other.value).abs() < f64::EPSILON
    }
}
impl Eq for NumericValue {}

#[derive(Eq, PartialEq, Clone)]
pub struct SourceLocation {
    pub line: usize,   // First line is 1
    pub column: usize, // First character of a line is at column 1
}

impl SourceLocation {
    pub fn to_json(&self) -> Value {
        json!([self.line, self.column])
    }
}
#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash(String),
    IDHash(String),
    String(String),
    BadString,
    URL(String),
    BadURL,
    Delim(char),
    Number(NumericValue),
    Percentage(NumericValue),
    Dimension(NumericValue, String),
    UnicodeRange(u32, u32),
    IncludeMatch,
    DashMatch,
    PrefixMatch,
    SuffixMatch,
    SubstringMatch,
    Column,
    WhiteSpace,
    CDO,
    CDC,
    Colon,
    Semicolon,
    Comma,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    LeftCurlyBracket,
    RightCurlyBracket,
}

pub type Node = (Token, SourceLocation);

pub struct Tokenizer {
    input: Arc<String>,
    length: usize,
    position: usize,
    line: usize,
    last_line_start: usize,
}

impl Tokenizer {
    /**
     * Assumes `input` has already been preprocessed.
     */
    pub fn new(input: Arc<String>) -> Tokenizer {
        Tokenizer {
            length: input.len(),
            input,
            position: 0,
            line: 1,
            last_line_start: 0,
        }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.position >= self.length
    }

    // Assumes non-EOF
    #[inline]
    fn current_char(&self) -> char {
        self.char_at(0)
    }

    #[inline]
    fn char_at(&self, offset: usize) -> char {
        self.input.chars().nth(self.position + offset).unwrap()
    }

    #[inline]
    fn consume_char(&mut self) -> char {
        let (_, char) = self.input[self.position..].char_indices().next().unwrap();
        self.position += char.len_utf8();
        char
    }

    #[inline]
    fn starts_with(&self, needle: &str) -> bool {
        dbg!(needle);
        self.input[self.position..].starts_with(needle)
    }

    #[inline]
    fn new_line(&mut self) {
        if cfg!(test) {
            assert!(self.input.chars().nth(self.position - 1).unwrap() == '\n')
        }
        self.line += 1;
        self.last_line_start = self.position;
    }

    // Checks whether the Tokenizer has at least `num` characters remaining
    #[inline]
    fn has_more(&self, num: usize) -> bool {
        self.position + num < self.length
    }
}

pub fn tokenize(input: &str) -> Tokenizer {
    let input = preprocess(input);
    Tokenizer {
        length: input.len(),
        input: Arc::new(input),
        position: 0,
        line: 1,
        last_line_start: 0,
    }
}

impl Iterator for Tokenizer {
    type Item = Node;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        next_token(self)
    }
}

// From http://dev.w3.org/csswg/css-syntax/#input-preprocessing
pub fn preprocess(input: &str) -> String {
    // TODO: Is this faster if done in one pass?
    input
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .replace('\x0C', "\n")
        .replace('\x00', "\u{FFFD}")
}

macro_rules! is_match {
    ($value:expr, $($pattern:pat_param)|+) => {
        matches!($value, $($pattern)|+)
    };
}

// From http://dev.w3.org/csswg/css-syntax/#consume-a-token
fn next_token(tokenizer: &mut Tokenizer) -> Option<Node> {
    consume_comments(tokenizer);
    if tokenizer.is_eof() {
        return None;
    }
    let start_location = SourceLocation {
        line: tokenizer.line,
        // The start of the line is column 1:
        column: tokenizer.position - tokenizer.last_line_start + 1,
    };
    let c = tokenizer.current_char();
    let token = match c {
        '\t' | '\n' | ' ' => {
            while !tokenizer.is_eof() {
                match tokenizer.current_char() {
                    ' ' | '\t' => tokenizer.position += 1,
                    '\n' => {
                        tokenizer.position += 1;
                        tokenizer.new_line();
                    }
                    _ => break,
                }
            }
            Token::WhiteSpace
        }
        '\"' => consume_string(tokenizer, false),
        '#' => {
            tokenizer.position += 1;
            if is_ident_start(tokenizer) {
                Token::IDHash(consume_name(tokenizer))
            } else if !tokenizer.is_eof()
                && match tokenizer.current_char() {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
                    '\\' => !tokenizer.starts_with("\\\n"),
                    _ => c > '\x7F', // Non-ASCII
                }
            {
                Token::Hash(consume_name(tokenizer))
            } else {
                Token::Delim(c)
            }
        }
        '$' => {
            if tokenizer.starts_with("$=") {
                tokenizer.position += 2;
                Token::SuffixMatch
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        '\'' => consume_string(tokenizer, true),
        '\x28' => {
            tokenizer.position += 1;
            Token::LeftParen
        }
        '\x29' => {
            tokenizer.position += 1;
            Token::RightParen
        }
        '*' => {
            if tokenizer.starts_with("*=") {
                tokenizer.position += 2;
                Token::SubstringMatch
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        '+' => {
            if (tokenizer.has_more(1) && is_match!(tokenizer.char_at(1), '0'..='9'))
                || (tokenizer.has_more(2)
                    && tokenizer.char_at(1) == '.'
                    && is_match!(tokenizer.char_at(2), '0'..='9'))
            {
                consume_numeric(tokenizer)
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        ',' => {
            tokenizer.position += 1;
            Token::Comma
        }
        '-' => {
            if (tokenizer.has_more(1) && is_match!(tokenizer.char_at(1), '0'..='9'))
                || (tokenizer.has_more(2)
                    && tokenizer.char_at(1) == '.'
                    && is_match!(tokenizer.char_at(2), '0'..='9'))
            {
                consume_numeric(tokenizer)
            } else if is_ident_start(tokenizer) {
                consume_ident_like(tokenizer)
            } else if tokenizer.starts_with("-->") {
                tokenizer.position += 3;
                Token::CDC
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        '.' => {
            if tokenizer.has_more(1) && is_match!(tokenizer.char_at(1), '0'..='9') {
                consume_numeric(tokenizer)
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }

        // Handling of '/' would occur here, but comments are handled by
        // `consume_comments()` above
        ':' => {
            tokenizer.position += 1;
            Token::Colon
        }
        ';' => {
            tokenizer.position += 1;
            Token::Semicolon
        }
        '<' => {
            if tokenizer.starts_with("<!--") {
                tokenizer.position += 4;
                Token::CDO
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        '@' => {
            tokenizer.position += 1;
            if is_ident_start(tokenizer) {
                Token::AtKeyword(consume_name(tokenizer))
            } else {
                Token::Delim(c)
            }
        }
        '[' => {
            tokenizer.position += 1;
            Token::LeftBracket
        }
        '\\' => {
            if !tokenizer.starts_with("\\\n") {
                consume_ident_like(tokenizer)
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        ']' => {
            tokenizer.position += 1;
            Token::RightBracket
        }
        '^' => {
            if tokenizer.starts_with("^=") {
                tokenizer.position += 2;
                Token::PrefixMatch
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        '\x7b' => {
            tokenizer.position += 1;
            Token::LeftCurlyBracket
        }
        '\x7d' => {
            tokenizer.position += 1;
            Token::RightCurlyBracket
        }
        '0'..='9' => consume_numeric(tokenizer),

        'u' | 'U' => {
            if tokenizer.has_more(2)
                && tokenizer.char_at(1) == '+'
                && is_match!(tokenizer.char_at(2), '0'..='9' | 'a'..='f' | 'A'..='F' | '?')
            {
                tokenizer.position += 2;
                consume_unicode_range(tokenizer)
            } else {
                consume_ident_like(tokenizer)
            }
        }
        // Non-ASCII name-start code points are handled below
        'a'..='z' | 'A'..='Z' | '_' => consume_ident_like(tokenizer),

        '|' => {
            if tokenizer.starts_with("|=") {
                tokenizer.position += 2;
                Token::DashMatch
            } else if tokenizer.starts_with("||") {
                tokenizer.position += 2;
                Token::Column
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }

        '~' => {
            if tokenizer.starts_with("~=") {
                tokenizer.position += 2;
                Token::IncludeMatch
            } else {
                tokenizer.position += 1;
                Token::Delim(c)
            }
        }
        // Non-ASCII
        _ if c > '\x7F' => consume_ident_like(tokenizer),

        _ => {
            tokenizer.position += 1;
            Token::Delim(c)
        }
    };
    Some((token, start_location))
}

#[inline]
fn consume_comments(tokenizer: &mut Tokenizer) {
    while tokenizer.starts_with("/*") {
        tokenizer.position += 2; // +2 to consume "/*"
        while !tokenizer.is_eof() {
            match tokenizer.consume_char() {
                '*' => {
                    if !tokenizer.is_eof() && tokenizer.current_char() == '/' {
                        tokenizer.position += 1;
                        break;
                    }
                }
                '\n' => tokenizer.new_line(),
                _ => (),
            }
        }
    }
}

// From http://dev.w3.org/csswg/css-syntax/#consume-a-string-token0
fn consume_string(tokenizer: &mut Tokenizer, single_quote: bool) -> Token {
    match consume_quoted_string(tokenizer, single_quote) {
        Some(value) => Token::String(value),
        None => Token::BadString,
    }
}

// Return None on syntax error (ie. unescaped newline)
fn consume_quoted_string(tokenizer: &mut Tokenizer, single_quote: bool) -> Option<String> {
    tokenizer.position += 1; // Skip the initial quote
    let mut string: String = String::new();
    while !tokenizer.is_eof() {
        match tokenizer.consume_char() {
            '\"' if !single_quote => break,
            '\'' if single_quote => break,
            '\n' => {
                tokenizer.position -= 1;
                return None;
            }
            '\\' => {
                if !tokenizer.is_eof() {
                    if tokenizer.current_char() == '\n' {
                        // Escaped newline
                        tokenizer.position += 1;
                        tokenizer.new_line();
                    } else {
                        string.push(consume_escape(tokenizer))
                    }
                }
                // else: escaped EOF, do nothing.
            }
            c => string.push(c),
        }
    }
    Some(string)
}

#[inline]
fn is_ident_start(tokenizer: &mut Tokenizer) -> bool {
    !tokenizer.is_eof()
        && match tokenizer.current_char() {
            'a'..='z' | 'A'..='Z' | '_' => true,
            '-' => {
                tokenizer.has_more(1)
                    && match tokenizer.char_at(1) {
                        'a'..='z' | 'A'..='Z' | '_' => true,
                        '\\' => !tokenizer.input[(tokenizer.position + 1)..].starts_with("\\\n"),
                        c => c > '\x7F', // Non-ASCII
                    }
            }
            '\\' => !tokenizer.starts_with("\\\n"),
            c => c > '\x7F', // Non-ASCII
        }
}

// Consume an identifier-like token.
//
// From http://dev.w3.org/csswg/css-syntax/#consume-an-ident-like-token
fn consume_ident_like(tokenizer: &mut Tokenizer) -> Token {
    let value = consume_name(tokenizer);
    if !tokenizer.is_eof() && tokenizer.current_char() == '\x28' {
        // \x28 == (
        tokenizer.position += 1;
        if value.eq_ignore_ascii_case("url") {
            consume_url(tokenizer)
        } else {
            Token::Function(value)
        }
    } else {
        Token::Ident(value)
    }
}

// Consume a name
//
// From http://dev.w3.org/csswg/css-syntax/#consume-a-name
fn consume_name(tokenizer: &mut Tokenizer) -> String {
    let mut value = String::new();
    while !tokenizer.is_eof() {
        let c = tokenizer.current_char();
        value.push(match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => {
                tokenizer.position += 1;
                c
            }
            '\\' => {
                if tokenizer.starts_with("\\\n") {
                    break;
                }
                tokenizer.position += 1;
                consume_escape(tokenizer)
            }
            _ => {
                if c > '\x7F' {
                    tokenizer.consume_char()
                }
                // Non-ASCII
                else {
                    break;
                }
            }
        })
    }
    value
}

fn consume_numeric(tokenizer: &mut Tokenizer) -> Token {
    // Parse [+-]?\d*(\.\d+)?([eE][+-]?\d+)?
    // But this is always called so that there is at least one digit in \d*(\.\d+)?
    let mut representation = String::new();
    let mut is_integer = true;
    if is_match!(tokenizer.current_char(), '-' | '+') {
        representation.push(tokenizer.consume_char())
    }
    while !tokenizer.is_eof() {
        match tokenizer.current_char() {
            '0'..='9' => representation.push(tokenizer.consume_char()),
            _ => break,
        }
    }
    if tokenizer.has_more(1)
        && tokenizer.current_char() == '.'
        && is_match!(tokenizer.char_at(1), '0'..='9')
    {
        is_integer = false;
        representation.push(tokenizer.consume_char()); // '.'
        representation.push(tokenizer.consume_char()); // digit
        while !tokenizer.is_eof() {
            match tokenizer.current_char() {
                '0'..='9' => representation.push(tokenizer.consume_char()),
                _ => break,
            }
        }
    }
    if (tokenizer.has_more(1)
        && is_match!(tokenizer.current_char(), 'e' | 'E')
        && is_match!(tokenizer.char_at(1), '0'..='9'))
        || (tokenizer.has_more(2)
            && is_match!(tokenizer.current_char(), 'e' | 'E')
            && is_match!(tokenizer.char_at(1), '+' | '-')
            && is_match!(tokenizer.char_at(2), '0'..='9'))
    {
        is_integer = false;
        representation.push(tokenizer.consume_char()); // 'e' or 'E'
        representation.push(tokenizer.consume_char()); // sign or digit
                                                       // If the above was a sign, the first digit it consumed below
                                                       // and we make one extraneous is_eof() check.
        while !tokenizer.is_eof() {
            match tokenizer.current_char() {
                '0'..='9' => representation.push(tokenizer.consume_char()),
                _ => break,
            }
        }
    }

    let value = NumericValue {
        int_value: if is_integer {
            Some(
                // Remove any + sign as int::from_str() does not parse them.
                if !representation.starts_with('+') {
                    i64::from_str(&representation)
                } else {
                    i64::from_str(&representation[1..])
                }
                .unwrap(),
            )
        } else {
            None
        },
        value: f64::from_str(&representation).unwrap(),
        representation,
    };
    if !tokenizer.is_eof() && tokenizer.current_char() == '%' {
        tokenizer.position += 1;
        Token::Percentage(value)
    } else if is_ident_start(tokenizer) {
        Token::Dimension(value, consume_name(tokenizer))
    } else {
        Token::Number(value)
    }
}

// Consume a URL. Assumes that the initial "url(" has already been consumed
//
// From http://dev.w3.org/csswg/css-syntax/#consume-a-url-token0
fn consume_url(tokenizer: &mut Tokenizer) -> Token {
    while !tokenizer.is_eof() {
        match tokenizer.current_char() {
            '\t' | ' ' => tokenizer.position += 1,
            '\n' => {
                tokenizer.position += 1;
                tokenizer.new_line();
            }
            '\"' => return consume_quoted_url(tokenizer, false),
            '\'' => return consume_quoted_url(tokenizer, true),
            // '\x29' == ')'
            '\x29' => {
                tokenizer.position += 1;
                break;
            }
            _ => return consume_unquoted_url(tokenizer),
        }
    }
    return Token::URL(String::new());

    fn consume_quoted_url(tokenizer: &mut Tokenizer, single_quote: bool) -> Token {
        match consume_quoted_string(tokenizer, single_quote) {
            Some(value) => consume_url_end(tokenizer, value),
            None => consume_bad_url(tokenizer),
        }
    }

    fn consume_unquoted_url(tokenizer: &mut Tokenizer) -> Token {
        let mut string = String::new();
        while !tokenizer.is_eof() {
            let next_char = match tokenizer.consume_char() {
                ' ' | '\t' => return consume_url_end(tokenizer, string),
                '\n' => {
                    tokenizer.new_line();
                    return consume_url_end(tokenizer, string)
                },
                // '\x29' == ')'
                '\x29' => break,
                '\x00'..='\x08' | '\x0B' | '\x0E'..='\x1F' | '\x7F'  // non-printable
                    | '\"' | '\'' | '\x28' => return consume_bad_url(tokenizer),
                '\\' => {
                    if !tokenizer.is_eof() && tokenizer.current_char() == '\n' {
                        return consume_bad_url(tokenizer)
                    }
                    consume_escape(tokenizer)
                },
                c => c
            };
            string.push(next_char)
        }
        Token::URL(string)
    }

    fn consume_url_end(tokenizer: &mut Tokenizer, string: String) -> Token {
        while !tokenizer.is_eof() {
            match tokenizer.consume_char() {
                ' ' | '\t' => (),
                '\n' => tokenizer.new_line(),
                '\x29' => break,
                _ => return consume_bad_url(tokenizer),
            }
        }
        Token::URL(string)
    }

    fn consume_bad_url(tokenizer: &mut Tokenizer) -> Token {
        // Consume up to the closing )
        while !tokenizer.is_eof() {
            match tokenizer.consume_char() {
                '\x29' => break,
                '\\' => tokenizer.position += 1, // Skip an escaped ')' or '\'
                '\n' => tokenizer.new_line(),
                _ => (),
            }
        }
        Token::BadURL
    }
}

// Assumes the initial "u+" has already been consumed
//
// From http://dev.w3.org/csswg/css-syntax/#consume-a-unicode-range-token0
fn consume_unicode_range(tokenizer: &mut Tokenizer) -> Token {
    let mut hex = String::new();
    while hex.len() < 6
        && !tokenizer.is_eof()
        && is_match!(tokenizer.current_char(), '0'..='9' | 'A'..='F' | 'a'..='f')
    {
        hex.push(tokenizer.consume_char());
    }
    let max_question_marks = 6 - hex.len();
    let mut question_marks = 0;
    while question_marks < max_question_marks
        && !tokenizer.is_eof()
        && tokenizer.current_char() == '?'
    {
        question_marks += 1;
        tokenizer.position += 1
    }
    let start;
    let end;
    if question_marks > 0 {
        start = u32::from_str_radix(&(hex.clone() + &"0".repeat(question_marks)), 16).unwrap();
        end = u32::from_str_radix(&(hex + &"F".repeat(question_marks)), 16).unwrap();
    } else {
        start = u32::from_str_radix(&hex, 16).unwrap();
        hex = String::new();
        if !tokenizer.is_eof() && tokenizer.current_char() == '-' {
            tokenizer.position += 1;
            while hex.len() < 6 && !tokenizer.is_eof() {
                let c = tokenizer.current_char();
                match c {
                    '0'..='9' | 'A'..='F' | 'a'..='f' => {
                        hex.push(c);
                        tokenizer.position += 1
                    }
                    _ => break,
                }
            }
        }
        end = if !hex.is_empty() {
            u32::from_str_radix(&hex, 16).unwrap()
        } else {
            start
        }
    }
    Token::UnicodeRange(start, end)
}

// Assumes that the U+005C REVERSE SOLIDUS (\) has already been consumed
// and that the next input character has already been verified
// to not be a newline.
fn consume_escape(tokenizer: &mut Tokenizer) -> char {
    if tokenizer.is_eof() {
        return '\u{FFFD}';
    } // Escaped EOF
    let c = tokenizer.consume_char();
    match c {
        '0'..='9' | 'A'..='F' | 'a'..='f' => {
            let mut hex = c.to_string();
            while hex.len() < 6 && !tokenizer.is_eof() {
                let c = tokenizer.current_char();
                match c {
                    '0'..='9' | 'A'..='F' | 'a'..='f' => {
                        hex.push(c);
                        tokenizer.position += 1
                    }
                    _ => break,
                }
            }
            if !tokenizer.is_eof() {
                match tokenizer.current_char() {
                    ' ' | '\t' => tokenizer.position += 1,
                    '\n' => {
                        tokenizer.position += 1;
                        tokenizer.new_line()
                    }
                    _ => (),
                }
            }
            static REPLACEMENT_CHAR: char = '\u{FFFD}';
            let c: u32 = u32::from_str_radix(&hex, 16).unwrap();
            if c != 0 {
                let c = char::from_u32(c);
                c.unwrap_or(REPLACEMENT_CHAR)
            } else {
                REPLACEMENT_CHAR
            }
        }
        c => c,
    }
}
