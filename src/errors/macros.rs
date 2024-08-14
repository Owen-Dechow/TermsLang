macro_rules! prettify_macro {
    ($text:expr) => {
        // Convert to pretty SyntaxError
        pub fn prettify(&self, program: &String) -> String {
            let file_location: &FileLocation = &self.1;
            let err_msg: &String = &self.0;
            let t: &str = $text;

            // Check to see if error has file position attached
            match file_location {
                // File position attached
                FileLocation::Loc {
                    start_line,
                    end_line,
                    start_col,
                    end_col,
                } => {
                    // Create initall mesage
                    let mut msg = format!(
                        "{t} [pos: {}:{}-{}:{}] ({})",
                        start_line + 1,
                        start_col,
                        end_line + 1,
                        end_col,
                        err_msg
                    );

                    // Get all the lines the error occurred on
                    let line_range = (start_line.clone(), end_line + 1);
                    let lines = &program.lines().collect::<Vec<&str>>()[line_range.0..line_range.1];

                    // Loop over lines and add them to message
                    for (line_idx, line) in lines.iter().enumerate() {                        
                        // Get he start position of the underline
                        let start = if line_idx == 0 {
                            if start_col > &0 {start_col -1} else {*start_col}
                        } else {
                            0 
                        };
                        
                        // Get the range of the underline
                        let range = {
                            if (start_line + line_idx) == *end_line {
                                end_col + 1
                            } else {
                                line.len() - start
                            }
                        };

                        // Create underline
                        let underline =
                            format!("{}{}", " ".repeat(if start > 0 {start} else {0}), "^".repeat(range));

                        // Update message to include new line
                        msg += format!("\n{}\n{}", line, underline).as_str();
                    }

                    // Return pretty message
                    return msg;
                }
                // End of file
                FileLocation::End => {
                    // Get he initall message
                    let mut msg = format!("{t} ({})", err_msg);

                    // Get the last line
                    let line = program.lines().last().unwrap().trim();

                    // Create underline after end of line
                    let underline = format!("{}^^^", " ".repeat(line.len()));

                    // Update message
                    msg += format!("\n\t{line}\n\t{underline}").as_str();

                    // Return pretty message
                    return msg;
                }
                // File position non existent
                FileLocation::None => {
                    // Get he initall message
                    let msg = format!("{t} ({})", err_msg);
                    return msg;
                }
            }
        }
    };
}

macro_rules! from_for_err_macro {
    ($err_type:ty) => {
        impl From<$err_type> for std::io::Error {
            fn from(err: $err_type) -> Self {
                std::io::Error::new(std::io::ErrorKind::Other, err.0)
            }
        }
    };
}
