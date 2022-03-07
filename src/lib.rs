mod lisp_parser;

pub use crate::lisp_parser::{LispObject, LispParsingError, LispProgramParsingResult, TextPosition};

/// A function that parses a LISP program into `LispObject`s.
/// Each `LispObject` is a `List` or a `String`, and numbers are `Strings` here too.
///
/// # Errors
/// If something is wrong with the `program` passed, an error may be returned:
/// * `UnclosedQuote`:
///     there is an opening quote for a string literal that was not closed.
///     Enum contents: `opening_quote_position` (`TextPosition`) - where an opening quote was in
///     text.
///     Example:
///       abc (def) "ghi
///                 ^ Unclosed quote is here
/// * `UnclosedParenthesis`:
///     there is an opened parenthesis for a list literal that was not closed.
///     Enum contents: `opening_parenthesis_position` (`TextPosition`) - where an opening
///     parenthesis was in text.
///     Example:
///       (abc def "ghi"
///       ^ Unclosed parenthesis is here
/// * `UnexpectedClosingParenthesis`:
///     there is a closing parenthesis, but it does not correspond to any opening parenthesis.
///     Enum contents: `closing_parenthesis_position` (`TextPosition`) - where an unexpected closing
///     parenthesis was in text.
///     Example:
///       ( ) abc def)
///                  ^ Unexpected closing parenthesis is here
pub fn parse_lisp_program(program: &str) -> lisp_parser::LispProgramParsingResult {
    let mut parser = lisp_parser::LispParser::new(program);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::{
        parse_lisp_program,
        LispObject::{self, List, String},
        LispParsingError, TextPosition,
    };
    #[test]
    fn complex_program_parsing_test() {
        fn str(string: &str) -> LispObject {
            String(string.to_string())
        }
        let program = "
            a
            (b c (d e f))
            \"ghi jkl\" (m n\"o\"(p q r(s t\"u v) w\")))
            x y z
            ";
        let parsed_program = vec![
            str("a"),
            List(vec![
                str("b"),
                str("c"),
                List(vec![str("d"), str("e"), str("f")]),
            ]),
            str("\"ghi jkl\""),
            List(vec![
                str("m"),
                str("n"),
                str("\"o\""),
                List(vec![
                    str("p"),
                    str("q"),
                    str("r"),
                    List(vec![str("s"), str("t"), str("\"u v) w\"")]),
                ]),
            ]),
            str("x"),
            str("y"),
            str("z"),
        ];
        assert_eq!(parse_lisp_program(program).unwrap(), parsed_program,);
    }

    #[test]
    fn test_unclosed_parenthesis_error() {
        assert_eq!(
            parse_lisp_program("("),
            Err(LispParsingError::UnclosedParenthesis {
                opening_parenthesis_position: TextPosition { line: 1, column: 1 },
            })
        );
    }

    #[test]
    fn test_unexpected_closing_parenthesis_error() {
        assert_eq!(
            parse_lisp_program("( )\n)"),
            Err(LispParsingError::UnexpectedClosingParenthesis {
                closing_parenthesis_position: TextPosition { line: 2, column: 1 },
            })
        );
    }

    #[test]
    fn test_unclosed_quote() {
        assert_eq!(
            parse_lisp_program("(\"\nabc)"),
            Err(LispParsingError::UnclosedQuote {
                opening_quote_position: TextPosition { line: 1, column: 2 },
            })
        );
    }

    #[test]
    fn test_empty_program() {
        assert_eq!(parse_lisp_program(""), Ok(Vec::new()));
    }

    #[test]
    fn test_whitespaces_program() {
        assert_eq!(
            parse_lisp_program("   \t \n  \n\t    \r    "),
            Ok(Vec::new())
        );
    }
}
