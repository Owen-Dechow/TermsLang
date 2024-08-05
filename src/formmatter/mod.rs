enum Ignore {
    String(char),
    Comment,
    None,
}

pub fn format(program: &String, indent_size: usize) -> String {
    let mut program_text = String::new();
    let mut indent_level = 0;
    let mut ignore = Ignore::None;

    let get_indent = |lev: &usize| String::from(" ").repeat(indent_size).repeat(*lev);

    for line in program.lines() {
        let line = if let Ignore::String(..) = ignore {
            line
        } else {
            line.trim()
        }
        .to_string();

        if let Ignore::None = ignore {
            if !program_text.ends_with(&get_indent(&indent_level)) {
                if line.len() != 0 {
                    program_text.push_str(&get_indent(&indent_level))
                }
            }
        }

        for (ch_idx, ch) in line.chars().enumerate() {
            if let Ignore::None = ignore {
                match ch {
                    '{' => {
                        indent_level += 1;
                        program_text = program_text.trim_end().to_string();
                        program_text.push(' ');
                        program_text.push(ch);
                        program_text.push('\n');
                    }
                    '}' => {
                        if indent_level > 0 {
                            indent_level -= 1
                        }
                        program_text = program_text.trim_end().to_string();
                        program_text.push('\n');
                        program_text
                            .push_str(&String::from(" ").repeat(indent_size).repeat(indent_level));
                        program_text.push(ch);
                    }
                    '#' => {
                        ignore = Ignore::Comment;
                        program_text.push(ch);
                    }
                    '"' | '`' | '\'' => {
                        ignore = Ignore::String(ch);
                        program_text.push(ch);
                    }
                    ' ' | '\t' => {
                        if !program_text.ends_with(' ') {
                            program_text.push(ch);
                        }
                    }
                    '~' => {
                        program_text = program_text.trim_end().to_string();
                        program_text.push(' ');
                        program_text.push('~');
                        if ch_idx != line.len() - 1 {
                            program_text.push('\n');
                            program_text.push_str(&get_indent(&indent_level));
                        }
                    }
                    ':' => {
                        program_text.push(ch);
                        program_text.push(' ');
                    }
                    '+' | '<' | '>' | '%' | '!' | '*' | '/' | '^' => {
                        if !program_text.ends_with(' ') {
                            program_text.push(' ');
                        }

                        program_text.push(ch);

                        if Some("=") != line.get((ch_idx + 1)..(ch_idx + 2)) {
                            program_text.push(' ');
                        }
                    }
                    '&' | '|' => {
                        if !program_text.ends_with(ch) {
                            if !program_text.ends_with(' ') {
                                program_text.push(' ')
                            }
                        }

                        program_text.push(ch);
                        if Some(ch.to_string().as_str()) != line.get((ch_idx + 1)..(ch_idx + 2)) {
                            program_text.push(' ');
                        }
                    }
                    '=' => {
                        if !program_text.ends_with(|s| "+-=<>%*/! ".contains(s)) {
                            program_text.push(' ');
                        }
                        program_text.push(ch)
                    }
                    ']' => {
                        program_text.push(ch);
                        if Some(".") != line.get((ch_idx + 1)..(ch_idx + 2)) {
                            program_text.push(' ');
                        }
                    }
                    '(' => {
                        if program_text.trim_end().ends_with('$') {
                            program_text = program_text.trim_end().to_string();
                        }
                        program_text.push(ch);
                    }
                    '-' => {
                        if !program_text.ends_with(' ') {
                            program_text.push(' ');
                        }
                        program_text.push(ch);

                        if let Some(next) = line.get((ch_idx + 1)..(ch_idx + 2)) {
                            if !"1234567890=".contains(next) {
                                program_text.push(' ');
                            }
                        }
                    }
                    _ => {
                        program_text.push(ch);
                    }
                }
            } else if let Ignore::String(quote) = ignore {
                if quote == ch {
                    ignore = Ignore::None
                }
                program_text.push(ch);
            } else if let Ignore::Comment = ignore {
                program_text.push(ch);
            }
        }

        if let Ignore::Comment = ignore {
            ignore = Ignore::None;
            program_text.push('\n');
        } else if let Ignore::None = ignore {
            if !program_text.trim_end().ends_with('{') {
                if !program_text.ends_with("\n\n") {
                    program_text.push('\n');
                }
            }
        } else if let Ignore::String(..) = ignore {
            program_text.push('\n');
        }
    }

    program_text = program_text.trim_end().to_owned();
    program_text.push('\n');

    return program_text;
}
