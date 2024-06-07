pub mod syntax;
pub mod tokens;

use crate::errors::{FileLocation, LexerError};

use std::collections::HashMap;

use self::{
    syntax::{
        get_syntax_map, SyntaxMap, COMMENT, DECIMAL, FORMAT_STRING_GATES, IGNORED_IN_NUMBERS,
        LINE_TERMINATOR, NEW_LINE, STRING_QUOTES, VARIABLE_ALLOWED_EXTRA_CHARS, WHITE_SPACE,
    },
    tokens::{Operator, StringInterpolator, Token, TokenType},
};

#[derive(PartialEq, Debug)]
enum SectionState {
    None,
    Int,
    Float,
    Operator,
    Word,
    Comment,
    String(char, StringInterpolator),
}

struct Section {
    state: SectionState,
    content: String,
}

impl Section {
    fn new() -> Section {
        Section {
            state: SectionState::None,
            content: "".to_string(),
        }
    }

    fn reset(&mut self) {
        self.state = SectionState::None;
        self.content.clear();
    }
}

struct FileLocationModel {
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
}
impl FileLocationModel {
    fn build(&self) -> FileLocation {
        FileLocation::Loc {
            start_line: self.start_line,
            end_line: self.end_line,
            start_col: self.start_col,
            end_col: self.end_col,
        }
    }
}

// Check if string is section of valid operator: (&, &&) => true
fn map_partial_fit(pattern: &String, map: &HashMap<&str, Operator>) -> bool {
    for key in map.keys() {
        if key.contains(pattern) {
            return true;
        }
    }

    return false;
}

// State machine to handel each character
fn handel_char(
    c: char,
    section: &mut Section,
    result: &mut Vec<Token>,
    syntax_map: &SyntaxMap,
    seek_string_reenter: &mut (bool, char),
    positioning: &mut FileLocationModel,
) -> Result<(), LexerError> {
    // Handel character based on state
    match &section.state {
        // If there is no current state
        SectionState::None => {
            // Skip white space
            if WHITE_SPACE.contains(c) {
                return Ok(());
            }

            // Update token positioning
            positioning.start_line = positioning.end_line;
            positioning.start_col = positioning.end_col;

            // Check for number
            if c.is_numeric() {
                section.state = SectionState::Int;
                section.content.push(c);
                return Ok(());
            }

            // Check for kewword or variable
            if c.is_ascii_alphabetic() || VARIABLE_ALLOWED_EXTRA_CHARS.contains(c) {
                section.state = SectionState::Word;
                section.content.push(c);
                return Ok(());
            }

            // Check for comment char
            if c == COMMENT {
                section.state = SectionState::Comment;
                return Ok(());
            }

            // Check for end of line marker
            if c == LINE_TERMINATOR {
                result.push(Token(TokenType::Terminate, positioning.build()));
                return Ok(());
            }

            // Check for string
            if STRING_QUOTES.contains(c) {
                section.state = SectionState::String(c, StringInterpolator::None);
                return Ok(());
            }

            // Check for end of {} in interpolated string
            if seek_string_reenter.0 && c == FORMAT_STRING_GATES.1 {
                result.push(Token(
                    TokenType::Operator(Operator::CloseParen),
                    positioning.build(),
                ));
                result.push(Token(
                    TokenType::Operator(Operator::Add),
                    positioning.build(),
                ));

                section.state =
                    SectionState::String(seek_string_reenter.1, StringInterpolator::Interpolated);
                seek_string_reenter.0 = false;

                return Ok(());
            }

            // Default to operator
            section.state = SectionState::Operator;
            section.content.push(c);
            return Ok(());
        }

        // If state = integer
        SectionState::Int => {
            // Convert state to Float
            if c == DECIMAL {
                section.state = SectionState::Float;
                section.content.push(c);
                return Ok(());
            }

            // Add number
            if c.is_numeric() {
                section.content.push(c);
                return Ok(());
            }

            // Allow for ignored characters in numbers: 34_34 = 3434
            if IGNORED_IN_NUMBERS.contains(c) {
                return Ok(());
            }

            // Complete int token
            result.push(Token(
                TokenType::Int(section.content.parse().unwrap()),
                positioning.build(),
            ));
            section.reset();
            return handel_char(
                c,
                section,
                result,
                syntax_map,
                seek_string_reenter,
                positioning,
            );
        }

        // If state = float
        SectionState::Float => {
            // Add digit to float
            if c.is_numeric() {
                section.content.push(c);
                return Ok(());
            }

            // Allow for ignored characters in numbers: 34_34 = 3434
            if IGNORED_IN_NUMBERS.contains(c) {
                return Ok(());
            }

            // Complete float token
            result.push(Token(
                TokenType::Float(section.content.parse().unwrap()),
                positioning.build(),
            ));
            section.reset();
            return handel_char(
                c,
                section,
                result,
                syntax_map,
                seek_string_reenter,
                positioning,
            );
        }

        // If state = Word (Variable or Kewword)
        SectionState::Word => {
            // Add letter to word
            if c.is_alphanumeric() || VARIABLE_ALLOWED_EXTRA_CHARS.contains(c) {
                section.content.push(c);
                return Ok(());
            }

            // Take ownership of word
            let string_content = section.content.to_owned();
            let content = string_content.as_str();

            // Check to see if interpolated string identity: random_word"" => false, interpolate"" => true
            if STRING_QUOTES.contains(c) && syntax_map.string_interpolators.contains_key(content) {
                section.state =
                    SectionState::String(c, syntax_map.string_interpolators[content].clone());
                section.content.clear();

                return Ok(());
            }

            // Check if kewword
            if syntax_map.keywords.contains_key(content) {
                // Complete kewword token
                result.push(Token(
                    TokenType::KeyWord(syntax_map.keywords[content].clone()),
                    positioning.build(),
                ))
            } else if syntax_map.bools.contains_key(content) {
                // Complete bool token
                result.push(Token(
                    TokenType::Bool(syntax_map.bools[content].clone()),
                    positioning.build(),
                ))
            } else {
                // Complete identifier (variable) token
                result.push(Token(
                    TokenType::Identity(section.content.clone()),
                    positioning.build(),
                ));
            }

            // Reset state
            section.reset();
            return handel_char(
                c,
                section,
                result,
                syntax_map,
                seek_string_reenter,
                positioning,
            );
        }

        // If state = operator
        SectionState::Operator => {
            // Get combined operator
            let new = format!("{}{}", section.content, c);

            // Check to see if still valid operator pattern
            if map_partial_fit(&new, &syntax_map.operators) {
                // Update operator pattern
                section.content = new;
                return Ok(());
            } else {
                // Get old operator pattern
                let content = section.content.as_str();

                // Check if valid operator
                if syntax_map.operators.contains_key(content) {
                    // Complete operator token
                    result.push(Token(
                        TokenType::Operator(syntax_map.operators[content].clone()),
                        positioning.build(),
                    ));
                    section.reset();
                    return handel_char(
                        c,
                        section,
                        result,
                        syntax_map,
                        seek_string_reenter,
                        positioning,
                    );
                } else {
                    // Mark invalid operator
                    return Err(LexerError(
                        format!("Error invalid operator: {}", section.content),
                        positioning.build(),
                    ));
                }
            }
        }

        // If state = comment
        SectionState::Comment => {
            // Check to see if comment line complete
            if c == NEW_LINE {
                section.reset()
            }

            // Skip char in comment
            return Ok(());
        }

        // If state = string
        SectionState::String(quote, interpolator) => {
            // Check for string close
            if c == *quote {
                // Adjust positioning struct to include end quote
                positioning.end_col += 1;
                result.push(Token(
                    TokenType::String(section.content.clone()),
                    positioning.build(),
                ));
                positioning.end_col -= 1;

                section.reset();
                return Ok(());
            }

            // Check if interpolated string
            if *interpolator == StringInterpolator::Interpolated {
                // Check if found {} in intperpolated string
                if c == FORMAT_STRING_GATES.0 {
                    // Add current string as token
                    result.push(Token(
                        TokenType::String(section.content.clone()),
                        positioning.build(),
                    ));

                    // Add string join operator
                    result.push(Token(
                        TokenType::Operator(Operator::Add),
                        positioning.build(),
                    ));

                    // Add string func
                    result.push(Token(
                        TokenType::Identity("@String".to_string()),
                        positioning.build(),
                    ));

                    // Add dot
                    result.push(Token(
                        TokenType::Operator(Operator::Dot),
                        positioning.build(),
                    ));

                    // Add open parenthise operator
                    result.push(Token(
                        TokenType::Operator(Operator::OpenParen),
                        positioning.build(),
                    ));

                    // set seek_string_reenter to true to collect rest of string
                    *seek_string_reenter = (true, *quote);

                    // Reset state machine
                    section.reset();
                    return Ok(());
                }
            }

            // Add char to string
            section.content.push(c);
            return Ok(());
        }
    }
}

// Lex a program (input)
pub fn lex(input: &String) -> Result<Vec<Token>, LexerError> {
    // Create syntax map
    let syntax_map = get_syntax_map();

    // Resulting lex vector
    let mut result = Vec::new();

    // State machine
    let mut section: Section = Section::new();

    // In {} of interpolated string
    let mut seek_string_reenter = (false, '\0');

    // Create token position tracker
    let mut positioning = FileLocationModel {
        start_line: 0,
        end_line: 0,
        start_col: 0,
        end_col: 0,
    };

    // Lex tokens
    let iter = input.chars().peekable();
    for c in iter {
        if c == '\n' {
            positioning.end_line += 1;
            positioning.end_col = 0;
        } else {
            positioning.end_col += 1;
        }

        if let Err(err) = handel_char(
            c,
            &mut section,
            &mut result,
            &syntax_map,
            &mut seek_string_reenter,
            &mut positioning,
        ) {
            return Err(err);
        }
    }

    return Ok(result);
}
