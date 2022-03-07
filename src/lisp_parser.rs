use std::str::CharIndices;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextPosition {
    pub line: usize,
    pub column: usize,
}

struct ProgramWrapper<'program> {
    last_character_was_a_newline: bool,
    program_iterator: CharIndices<'program>,
    text_position: TextPosition,
}

struct Slicer<'string> {
    string: &'string str,
    start_index: usize,
}

impl<'string> Slicer<'string> {
    const fn new(string: &'string str, start_index: usize) -> Self {
        Self {
            string,
            start_index,
        }
    }

    fn slice(&self, to: usize) -> String {
        self.string[self.start_index..=to].to_string()
    }
}

impl<'program> ProgramWrapper<'program> {
    fn new(program: &'program str) -> Self {
        Self {
            last_character_was_a_newline: true,
            program_iterator: program.char_indices(),
            text_position: TextPosition { line: 0, column: 1 },
        }
    }
}

pub struct LispParser<'program> {
    program: &'program str,
    program_wrapper: ProgramWrapper<'program>,
}

impl<'program> Iterator for LispParser<'program> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        match self.program_wrapper.program_iterator.next() {
            Some((index, character)) => {
                if self.program_wrapper.last_character_was_a_newline {
                    self.program_wrapper.last_character_was_a_newline = false;
                    self.program_wrapper.text_position.column = 1;
                    self.program_wrapper.text_position.line += 1;
                } else {
                    self.program_wrapper.text_position.column += 1;
                }
                if character == '\n' {
                    self.program_wrapper.last_character_was_a_newline = true;
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
    List(Vec<Self>),
}

struct ParsedLispObject {
    lisp_object: LispObject,
    next_character_with_index: Option<(usize, char)>,
}

type LispObjectParsingResult = Result<ParsedLispObject, LispParsingError>;
pub type LispProgramParsingResult = Result<Vec<LispObject>, LispParsingError>;

impl<'program> LispParser<'program> {
    pub fn new(program: &'program str) -> Self {
        Self {
            program,
            program_wrapper: ProgramWrapper::new(program),
        }
    }

    const fn make_slicer(&self, start_index: usize) -> Slicer<'program> {
        Slicer::new(self.program, start_index)
    }

    const fn text_position(&self) -> TextPosition {
        self.program_wrapper.text_position
    }

    fn parse_string(&mut self, opening_quote_index: usize) -> LispObjectParsingResult {
        let slicer = self.make_slicer(opening_quote_index);
        let opening_quote_position = self.text_position();
        for (index, character) in self.make_iterator() {
            if character == '"' {
                return Ok(ParsedLispObject {
                    lisp_object: LispObject::String(slicer.slice(index)),
                    next_character_with_index: self.next(),
                });
            }
        }
        Err(LispParsingError::UnclosedQuote {
            opening_quote_position,
        })
    }

    pub fn parse_program(&mut self) -> LispProgramParsingResult {
        let mut list = Vec::new();
        let (mut index, mut character);
        match self.next() {
            None => return Ok(list),
            Some(character_with_index) => (index, character) = character_with_index,
        }
        loop {
            match self.parse_object((index, character)) {
                Ok(optional_object) => match optional_object {
                    Some(parsed_lisp_object) => {
                        match parsed_lisp_object.next_character_with_index {
                            None => return Ok(list),
                            Some(character_with_index) => {
                                (index, character) = character_with_index;
                            }
                        }
                        list.push(parsed_lisp_object.lisp_object);
                    }
                    None => return Ok(list),
                },
                Err(error) => return Err(error),
            }
        }
    }

    fn parse_word(&mut self, word_beginning_index: usize) -> ParsedLispObject {
        let slicer = self.make_slicer(word_beginning_index);
        let mut last_successful_index = word_beginning_index;
        for (index, character) in self.make_iterator() {
            if character.is_whitespace()
                || character == '"'
                || character == ')'
                || character == '('
            {
                return ParsedLispObject {
                    lisp_object: LispObject::String(slicer.slice(last_successful_index)),
                    next_character_with_index: Some((index, character)),
                };
            }
            last_successful_index = index;
        }
        ParsedLispObject {
            lisp_object: LispObject::String(self.program.to_string()),
            next_character_with_index: None,
        }
    }

    fn skip_whitespaces(&mut self, current_character: (usize, char)) -> Option<(usize, char)> {
        let (mut index, mut character) = current_character;
        let iterator = self.make_iterator();
        loop {
            if !character.is_whitespace() {
                return Some((index, character));
            }
            match iterator.next() {
                Some(character_with_index) => (index, character) = character_with_index,
                None => return None,
            }
        }
    }

    fn parse_object(
        &mut self,
        current_character: (usize, char),
    ) -> Result<Option<ParsedLispObject>, LispParsingError> {
        match self.skip_whitespaces(current_character) {
            None => Ok(None),
            Some((index, character)) => match character {
                '(' => Ok(Some(self.parse_list()?)),
                ')' => Err(LispParsingError::UnexpectedClosingParenthesis {
                    closing_parenthesis_position: self.text_position(),
                }),
                '"' => Ok(Some(self.parse_string(index)?)),
                _ => Ok(Some(self.parse_word(index))),
            },
        }
    }

    fn make_iterator(&mut self) -> &mut Self {
        self.by_ref()
    }

    fn parse_list(&mut self) -> LispObjectParsingResult {
        let mut list = Vec::new();
        let opening_parenthesis_position = self.text_position();
        let (mut index, mut character);
        match self.next() {
            None => {
                return Err(LispParsingError::UnclosedParenthesis {
                    opening_parenthesis_position,
                })
            }
            Some(character_with_index) => (index, character) = character_with_index,
        }
        loop {
            match self.parse_object((index, character)) {
                Err(LispParsingError::UnexpectedClosingParenthesis { .. }) => {
                    return Ok(ParsedLispObject {
                        lisp_object: LispObject::List(list),
                        next_character_with_index: self.next(),
                    })
                }
                Ok(optional_object) => match optional_object {
                    Some(parsed_lisp_object) => {
                        match parsed_lisp_object.next_character_with_index {
                            None => {
                                return Err(LispParsingError::UnclosedParenthesis {
                                    opening_parenthesis_position,
                                })
                            }
                            Some(character_with_index) => {
                                (index, character) = character_with_index;
                            }
                        }
                        list.push(parsed_lisp_object.lisp_object);
                    }
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
