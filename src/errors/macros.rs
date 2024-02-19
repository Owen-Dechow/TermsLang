macro_rules! prettify_macro {
    ($text:expr) => {
        // Convert to pretty SyntaxError
        pub fn prettify(&self, program: &String) -> String {
            // Check to see if error has file position attached
            match &self.1 {
                // File position attached
                Some(pos) => {
                    let t = $text;

                    // Create initall mesage
                    let mut msg = format!(
                        "{t} [pos: {}:{}-{}:{}] ({})",
                        pos.start_line + 1,
                        pos.start_col,
                        pos.end_line + 1,
                        pos.end_col,
                        self.0
                    );

                    // Get all the lines the error occurred on
                    let lines =
                        &program.lines().collect::<Vec<&str>>()[pos.start_line..pos.end_line + 1];

                    // Loop over lines and add them to message
                    for (idx, line) in lines.iter().enumerate() {
                        // Trim whitespace
                        let start_len = line.len();
                        let line = line.trim_start();
                        let trimmed_len = start_len - line.len();

                        // Get he start position of the underline
                        let start = if idx == 0 { pos.start_col - 1 } else { 0 };

                        // Get the range of the underline
                        let mut range = if idx == (pos.end_line - pos.start_line) {
                            pos.end_col - start - 1
                        } else {
                            line.len() - start
                        };
                        if range == 0 {
                            range = 1;
                        }

                        // Create underline
                        let underline =
                            format!("{}{}", " ".repeat(start - trimmed_len), "^".repeat(range));

                        // Update message to include new line
                        msg += format!("\n\t{line}\n\t{underline}").as_str();
                    }

                    // Return pretty message
                    return msg;
                }
                // File position not attached (assume file ended early)
                None => {
                    // Get he initall message
                    let mut msg = format!("$type ({})", self.0);

                    // Get the last line
                    let line = program.lines().last().unwrap().trim();

                    // Create underline after end of line
                    let underline = format!("{}^^^", " ".repeat(line.len()));

                    // Update message
                    msg += format!("\n\t{line}\n\t{underline}").as_str();

                    // Return pretty message
                    return msg;
                }
            }
        }
    };
}
