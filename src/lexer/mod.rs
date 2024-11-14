pub mod syntax;
pub mod tokens;

use std::{collections::HashMap, path::PathBuf};

use crate::{errors::{FileLocation, LexerError}, active_parser::names::PREFIX_PROTECTED_NAMES};

use self::{
    syntax::{
        get_syntax_map, SyntaxMap, COMMENT, DECIMAL, IGNORED_IN_NUMBERS, LINE_TERMINATOR, NEW_LINE,
        STRING_QUOTES, VARIABLE_ALLOWED_EXTRA_CHARS,
    },
    tokens::{Operator, Token, TokenType},
};

#[derive(PartialEq, Debug)]
enum SectionState {
    None,
    Int,
    Float,
    Operator,
    Word,
    Comment,
    String(char),
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
    file: PathBuf,
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
}
impl FileLocationModel {
    fn build(&self) -> FileLocation {
        FileLocation::Loc {
            file: self.file.clone(),
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

// State machine to handle each character
fn handle_char(
    c: char,
    section: &mut Section,
    result: &mut Vec<Token>,
    syntax_map: &SyntaxMap,
    positioning: &mut FileLocationModel,
    id_prefix: &str,
    prefix_exclude: &[String],
    lex_comments: bool,
) -> Result<(), LexerError> {
    // Handel character based on state
    match &section.state {
        // If there is no current state
        SectionState::None => {
            // Skip white space
            if c.is_whitespace() {
                return Ok(());
            }

            // Update token positioning
            positioning.start_line = positioning.end_line;
            positioning.start_col = positioning.end_col;

            // Check for number
            if c.is_numeric() {
                if let Some(Token(
                    TokenType::Operator(Operator::Subtract),
                    FileLocation::Loc {
                        end_line,
                        end_col,
                        start_line,
                        start_col,
                        ..
                    },
                )) = result.last()
                {
                    if *end_line == positioning.start_line && *end_col == positioning.start_col {
                        section.content.push('-');
                        positioning.start_col = *start_col;
                        positioning.start_line = *start_line;

                        result.pop().unwrap();
                    }
                }

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
                section.state = SectionState::String(c);
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
            return handle_char(
                c,
                section,
                result,
                syntax_map,
                positioning,
                id_prefix,
                prefix_exclude,
                lex_comments,
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
            return handle_char(
                c,
                section,
                result,
                syntax_map,
                positioning,
                id_prefix,
                prefix_exclude,
                lex_comments,
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

                let mut content = section.content.clone();
                if !prefix_exclude.contains(&content) {
                    if !PREFIX_PROTECTED_NAMES.contains(&content.as_str()) { 
                        content = format!("{}{}", id_prefix, content);
                    }
                }

                result.push(Token(TokenType::Identity(content), positioning.build()));
            }

            // Reset state
            section.reset();
            return handle_char(
                c,
                section,
                result,
                syntax_map,
                positioning,
                id_prefix,
                prefix_exclude,
                lex_comments,
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
                    return handle_char(
                        c,
                        section,
                        result,
                        syntax_map,
                        positioning,
                        id_prefix,
                        prefix_exclude,
                        lex_comments,
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
                if lex_comments {
                    result.push(Token(
                        TokenType::Comment(section.content.clone()),
                        positioning.build(),
                    ))
                }
                section.reset();
            } else {
                section.content.push(c);
            }

            // Skip char in comment
            return Ok(());
        }

        // If state = string
        SectionState::String(quote) => {
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

            // Add char to string
            section.content.push(c);
            return Ok(());
        }
    }
}

// Lex a program (input)
pub fn lex(
    input: &String,
    lex_comments: bool,
    file: &PathBuf,
    id_prefix: &str,
    prefix_exclude: &[String],
) -> Result<Vec<Token>, LexerError> {
    // Create syntax map
    let syntax_map = get_syntax_map();

    // Resulting lex vector
    let mut result = Vec::new();

    // State machine
    let mut section: Section = Section::new();

    // Create token position tracker
    let mut positioning = FileLocationModel {
        file: file.clone(),
        start_line: 0,
        end_line: 0,
        start_col: 0,
        end_col: 0,
    };

    // Lex tokens
    let input = format!("{input}\n");
    let iter = input.chars().peekable();
    for c in iter {
        if c == '\n' {
            positioning.end_line += 1;
            positioning.end_col = 0;
        } else {
            positioning.end_col += 1;
        }

        if let Err(err) = handle_char(
            c,
            &mut section,
            &mut result,
            &syntax_map,
            &mut positioning,
            id_prefix,
            prefix_exclude,
            lex_comments,
        ) {
            return Err(err);
        }
    }

    return Ok(result);
}
