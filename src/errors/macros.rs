macro_rules! prettify_macro {
    ($text:expr) => {
        // Convert to pretty Error
        pub fn prettify(&self) -> String {
            let file_location: &FileLocation = &self.1;
            let err_msg: &String = &self.0;
            let t: &str = $text;

            // Check to see if error has file position attached
            match file_location {
                // File position attached
                FileLocation::Loc {
                    file,
                    start_line,
                    end_line,
                    start_col,
                    end_col,
                } => {
                    let program = fs::read_to_string(file).unwrap();

                    // Create initial message
                    let mut msg = format!(
                        "{} {}:{}:{} ({})\n",
                        t.red(),
                        file.display(),
                        start_line + 1,
                        start_col + 1,
                        err_msg
                    );

                    // Get all the lines the error occurred on
                    let offset_start = if *start_line > 0 { 1 } else { 0 };
                    let line_range = (start_line - offset_start, end_line + 1);
                    let lines = &program.lines().collect::<Vec<&str>>()[line_range.0..line_range.1];
                    let lines = lines.into_iter().map(|x| x.to_string());

                    // Loop over lines and add them to message
                    for (line_idx, line) in lines.enumerate() {
                        let line_num = line_idx + start_line - offset_start;
                        msg += &format!("\n{: >5}|", line_num + 1);

                        if line_num == *start_line {
                            let start_col = if *start_col == 0 { 0 } else { start_col - 1 };

                            let good1 = &line[..start_col];
                            let (bad, good2) = if end_line == start_line {
                                let bad = &line[start_col..*end_col];
                                let good2 = &line[*end_col..];

                                (bad, good2)
                            } else {
                                let bad = &line[start_col..];
                                let good2 = "";

                                (bad, good2)
                            };

                            msg += &format!("{}{}{}", good1.green(), bad.red(), good2.green());
                        } else if line_num == *end_line {
                            let bad = &line[..*end_col];
                            let good = &line[*end_col..];
                            msg += &format!("{}{}", bad.red(), good.green())
                        } else if line_num > *start_line && line_num < *end_line {
                            msg += &format!("{}", line.red())
                        } else {
                            msg += &format!("{}", line.green())
                        }
                    }

                    // Return pretty message
                    return msg;
                }
                // End of file
                FileLocation::End { file } => {
                    let program = fs::read_to_string(file).unwrap();

                    // Get he initial message
                    let mut msg = format!("{} {} ({})\n", t.red(), file.display(), err_msg);

                    // Get the last line
                    let line = program.lines().last().unwrap();

                    // Update message
                    msg += format!("\n{: >5}|{}", program.lines().count(), line.green()).as_str();

                    // Return pretty message
                    return msg;
                }
                // File position non existent
                FileLocation::None => {
                    // Get he initial message
                    let msg = format!("{} ({})", t.red(), err_msg);
                    return msg;
                }
            }
        }

        pub fn json(&self) -> String {
            let mut json = String::from("{");

            match &self.1 {
                FileLocation::Loc {
                    file,
                    start_line,
                    end_line,
                    start_col,
                    end_col,
                } => {
                    json += &format!(
                        "\"loc\":\"{}:{start_line}:{start_col}-{end_line}:{end_col}\",",
                        file.display()
                    )
                }
                FileLocation::End { file } => json += &format!("\"loc\":\"{}\",", file.display()),
                FileLocation::None => {}
            };

            json += &format!("\"msg\":{:?}}}", self.0);
            return json;
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
