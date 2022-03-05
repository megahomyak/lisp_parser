mod lisp_parser {
    use std::str::CharIndices;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct TextPosition {
        pub line: usize,
        pub column: usize,
    }

    pub struct LispParser<'program> {
        program: &'program str,
        text_position: TextPosition,
        last_character_was_a_newline: bool,
        program_iterator: CharIndices<'program>,
    }

    impl<'program> Iterator for LispParser<'program> {
        type Item = (usize, char);

        fn next(&mut self) -> Option<Self::Item> {
            match self.program_iterator.next() {
                Some((index, character)) => {
                    if self.last_character_was_a_newline {
                        self.last_character_was_a_newline = false;
                        self.text_position.column = 1;
                        self.text_position.line += 1;
                    } else {
                        self.text_position.column += 1;
                    }
                    if character == '\n' {
                        self.last_character_was_a_newline = true;
                    }
                    Some((index, character))
                }
                None => None,
            }
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    pub enum LispParsingError {
        UnclosedQuote {
            opening_quote_position: TextPosition,
        },
        UnclosedParenthesis {
            opening_parenthesis_position: TextPosition,
        },
        UnexpectedClosingParenthesis {
            closing_parenthesis_position: TextPosition,
        },
    }

    #[derive(PartialEq, Eq, Debug)]
    pub enum LispObject {
        String(String),
        List(Vec<LispObject>),
    }

    type LispObjectParsingResult = Result<LispObject, LispParsingError>;
    pub type LispProgramParsingResult = Result<Vec<LispObject>, LispParsingError>;

    impl<'program> LispParser<'program> {
        pub fn new(program: &'program str) -> Self {
            Self {
                program,
                text_position: TextPosition { line: 1, column: 1 },
                last_character_was_a_newline: false,
                program_iterator: program.char_indices(),
            }
        }

        fn slice(&mut self, to: usize) -> &'program str {
            let slice = &self.program[..to];
            self.set_program(&self.program[to..]);
            return slice;
        }

        fn set_program(&mut self, new_program: &'program str) {
            self.program = new_program;
            self.program_iterator = new_program.char_indices();
        }

        fn parse_string(&mut self) -> LispObjectParsingResult {
            let opening_quote_position = self.text_position;
            for (index, character) in self.get_iterator() {
                if character == '"' {
                    return Ok(LispObject::String(self.slice(index).to_string()));
                }
            }
            Err(LispParsingError::UnclosedQuote {
                opening_quote_position,
            })
        }

        pub fn parse_program(&mut self) -> LispProgramParsingResult {
            let mut list = Vec::new();
            loop {
                match self.parse_object() {
                    Ok(optional_object) => match optional_object {
                        Some(object) => list.push(object),
                        None => return Ok(list),
                    },
                    Err(error) => return Err(error),
                }
            }
        }

        fn parse_word(&mut self) -> LispObjectParsingResult {
            for (index, character) in self.get_iterator() {
                if character.is_whitespace()
                    || character == '"'
                    || character == ')'
                    || character == '('
                {
                    return Ok(LispObject::String(self.slice(index - 1).to_string()));
                }
            }
            let program = self.program;
            self.set_program("");
            return Ok(LispObject::String(program.to_string()));
        }

        fn skip_whitespaces(&mut self) {
            for (index, character) in self.get_iterator() {
                if !character.is_whitespace() {
                    self.set_program(&self.program[index..]);
                    return;
                }
            }
        }

        fn parse_object(&mut self) -> Result<Option<LispObject>, LispParsingError> {
            self.skip_whitespaces();
            let opening_parenthesis_position = self.text_position;
            match self.next() {
                None => return Ok(None),
                Some((_index, character)) => match character {
                    '(' => Ok(Some(self.parse_list()?)),
                    ')' => Err(LispParsingError::UnclosedParenthesis {
                        opening_parenthesis_position,
                    }),
                    '"' => Ok(Some(self.parse_string()?)),
                    _ => Ok(Some(self.parse_word()?)),
                },
            }
        }

        fn get_iterator(&mut self) -> &mut Self {
            self.by_ref()
        }

        fn parse_list(&mut self) -> LispObjectParsingResult {
            let mut list = Vec::new();
            let opening_parenthesis_position = self.text_position;
            loop {
                match self.parse_object() {
                    Err(LispParsingError::UnexpectedClosingParenthesis { .. }) => {
                        return Ok(LispObject::List(list))
                    }
                    Ok(optional_object) => match optional_object {
                        Some(object) => list.push(object),
                        None => {
                            return Err(LispParsingError::UnclosedParenthesis {
                                opening_parenthesis_position,
                            })
                        }
                    },
                    Err(other_error) => return Err(other_error),
                }
            }
        }
    }
}

pub use lisp_parser::{LispObject, LispParsingError, LispProgramParsingResult, TextPosition};

pub fn parse_lisp_program<'program>(
    program: &'program str,
) -> lisp_parser::LispProgramParsingResult {
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
        assert_eq!(parse_lisp_program(&program).unwrap(), parsed_program,);
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

