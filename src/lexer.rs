/// Responsible for lexing the source input.
/// This is the first of three parsing stages.

use std::vec::Vec;
use std::collections::HashSet;
use std::iter::FromIterator;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum LexemeKind<'a> {
    Whitespace(&'a str),
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
    Keyword(&'a str),
    Identifier(&'a str),
    IntLiteral(i32),
}

#[derive(Debug, PartialEq)]
pub struct Lexeme<'a> {
    pub kind: LexemeKind<'a>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum LexError {
    UnrecognizedInput { line: usize, column: usize },
}

// All the patterns that are used to match stuff
lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = HashSet::from_iter(vec![
        "return", "int"
    ]);

    static ref AFTER_LAST_NEWLINE_REGEX: Regex = Regex::new(r"\n([^\n]*)$").unwrap();
    static ref WHITESPACE_REGEX: Regex = Regex::new(r"^\s+").unwrap();
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r"^[a-zA-Z]\w*").unwrap();
    static ref INT_LITERAL_REGEX: Regex = Regex::new(r"^[0-9]+").unwrap();
    static ref SYMBOL_REGEX: Regex = Regex::new(r"^[\(\)\{\};]").unwrap();
}

fn try_get<'a, F>(current_input: &'a str, pattern: &Regex, transformer: F) -> Option<(&'a str, &'a str, LexemeKind<'a>)>
where F: Fn(&'a str) -> LexemeKind<'a>,
{
    if let Some(matched_result) = pattern.find(current_input) {
        let matched_str = matched_result.as_str();
        let new_input = &current_input[matched_result.end()..];
        let kind = transformer(matched_str);
        Some((new_input, matched_str, kind))
    } else {
        None
    }
}

fn convert_identifier_str<'a>(identifier: &'a str) -> LexemeKind<'a> {
    if KEYWORDS.contains(identifier) {
        LexemeKind::Keyword(identifier)
    }
    else {
        LexemeKind::Identifier(identifier)
    }
}

fn convert_symbol_str<'a>(symbol: &'a str) -> LexemeKind<'a> {
    match symbol {
        "{" => LexemeKind::OpenBrace,
        "}" => LexemeKind::CloseBrace,
        "(" => LexemeKind::OpenParen,
        ")" => LexemeKind::CloseParen,
        ";" => LexemeKind::Semicolon,
        _ => unreachable!()
    }
}

fn get_next_token<'a>(current_input: &'a str) -> Option<(&'a str, &'a str, LexemeKind<'a>)> {
    try_get(current_input, &WHITESPACE_REGEX, |s| LexemeKind::Whitespace(s))
        .or_else(|| try_get(current_input, &IDENTIFIER_REGEX, convert_identifier_str))
        .or_else(|| try_get(current_input, &SYMBOL_REGEX, convert_symbol_str))
        .or_else(|| try_get(current_input, &INT_LITERAL_REGEX, |s| LexemeKind::IntLiteral(s.parse().unwrap())))
}

pub fn lex_str(input: &str) -> Result<Vec<Lexeme>, LexError> {
    let mut result = Vec::new();
    let mut current_input = input;
    let mut current_line: usize = 1;
    let mut current_column: usize = 1;

    loop {
        if let Some((new_input, consumed_input, lexeme_kind)) = get_next_token(current_input) {
            current_input = new_input;

            // Skip over whitespace
            match lexeme_kind {
                LexemeKind::Whitespace(_) => {},
                _ => result.push(Lexeme {
                    kind: lexeme_kind,
                    line: current_line,
                    column: current_column,
                }),
            }

            // Now update the current line and column info
            // Collect all the newlines in the string
            let line_change_count = consumed_input.matches("\n").count();
            current_line += line_change_count;

            // If the line count changed...
            if line_change_count > 0 {
                // ...reset the column...
                current_column = 1;

                // ...and increment by the amount of characters after the last newline.
                if let Some(matched) = AFTER_LAST_NEWLINE_REGEX.find(consumed_input) {
                    let amount = matched.as_str().len();
                    current_column += amount;
                }
            }
            // ...otherwise, just increment the column.
            else {
                current_column += consumed_input.len();
            }
        } else {
            break
        }
    }

    if !current_input.is_empty() {
        return Err(LexError::UnrecognizedInput {
            line: current_line,
            column: current_column,
        })
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_get_test() {
        let (new_input, consumed_input, lexed_kind) = try_get("test", &IDENTIFIER_REGEX, |s| LexemeKind::Identifier(s)).unwrap();
        assert_eq!(new_input, "");
        assert_eq!(consumed_input, "test");
        assert_eq!(lexed_kind, LexemeKind::Identifier("test"));
    }

    #[test]
    fn get_next_token_test() {
        let (new_input, consumed_input, lexed_kind) = get_next_token("  test").unwrap();
        assert_eq!(new_input, "test");
        assert_eq!(consumed_input, "  ");
        assert_eq!(lexed_kind, LexemeKind::Whitespace("  "));

        let (new_input, consumed_input, lexed_kind) = get_next_token(new_input).unwrap();
        assert_eq!(new_input, "");
        assert_eq!(consumed_input, "test");
        assert_eq!(lexed_kind, LexemeKind::Identifier("test"));
    }

    #[test]
    fn lex_str_test() {
        let lexed = lex_str("test foo bar").unwrap();
        assert_eq!(lexed.len(), 3);
        assert_eq!(lexed, vec![
            Lexeme {
                kind: LexemeKind::Identifier("test"),
                line: 1,
                column: 1,
            },
            Lexeme {
                kind: LexemeKind::Identifier("foo"),
                line: 1,
                column: 6,
            }, 
            Lexeme {
                kind: LexemeKind::Identifier("bar"),
                line: 1,
                column: 10,
            }
        ]);
    }

    #[test]
    fn lexing_identifiers_vs_keywords() {
        let lexed = lex_str("test return").unwrap();
        assert_eq!(lexed, vec![
            Lexeme {
                kind: LexemeKind::Identifier("test"),
                line: 1,
                column: 1,
            },
            Lexeme {
                kind: LexemeKind::Keyword("return"),
                line: 1,
                column: 6,
            }
        ]);
    }

    #[test]
    fn lexing_symbols() {
        let lexed = lex_str("{}();").unwrap();
        assert_eq!(lexed, vec![
            Lexeme {
                kind: LexemeKind::OpenBrace,
                line: 1,
                column: 1,
            },
            Lexeme {
                kind: LexemeKind::CloseBrace,
                line: 1,
                column: 2,
            },
            Lexeme {
                kind: LexemeKind::OpenParen,
                line: 1,
                column: 3,
            },
            Lexeme {
                kind: LexemeKind::CloseParen,
                line: 1,
                column: 4,
            },
            Lexeme {
                kind: LexemeKind::Semicolon,
                line: 1,
                column: 5,
            },
        ]);
    }

    #[test]
    fn int_literals() {
        let lexed = lex_str("123 456").unwrap();
        assert_eq!(lexed, vec![
            Lexeme {
                kind: LexemeKind::IntLiteral(123),
                line: 1,
                column: 1,
            },
            Lexeme {
                kind: LexemeKind::IntLiteral(456),
                line: 1,
                column: 5,
            }
        ]);
    }
}