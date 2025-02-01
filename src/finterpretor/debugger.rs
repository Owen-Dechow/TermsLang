use crate::{errors::RuntimeError, flat_ir::CMD};
use colored::Colorize;
use std::io::{stdin, Write};

use super::Runner;

pub struct Debugger<'a> {
    debug_out: String,
    runner: Runner<'a>,
    commands_run: u128,
    complete: bool,
    padding: Vec<usize>,
}
impl Debugger<'_> {
    pub fn new<'a>(runner: Runner<'a>) -> Debugger<'a> {
        let mut pad = 0;
        let mut padding = Vec::new();

        for x in &runner.prog.tape {
            pad -= match x {
                CMD::TRelease => 1,
                _ => 0,
            };

            padding.push(pad);

            pad += match x {
                CMD::SplitScope => 1,
                _ => 0,
            };
        }

        return Debugger {
            debug_out: String::new(),
            runner,
            commands_run: 0,
            complete: false,
            padding,
        };
    }

    fn print_state(&mut self) {
        self.clear_terminal();
        let term_size = termion::terminal_size().unwrap();

        let min_width = 75;
        if term_size.0 < min_width {
            println!(
                "Terminal width is {} must be at least {} to run debugger.",
                term_size.0, min_width
            )
        }

        let w = (term_size.0 - 1) as f32;
        let r_50 = ((50.0 / 130.0) * w) as usize - 2;
        let r_30 = ((30.0 / 130.0) * w) as usize - 3;

        let program_string = box_string(
            self.get_program_string(term_size.1 - 6),
            r_50,
            "Program Tape",
        );
        let stack_string = box_string(self.get_stack_string(), r_30, "Stack");
        let refer_string = box_string(self.get_refer_string(), r_30, "Refer Stack");
        let data_string = box_string(self.get_data_string(), r_30, "Heap Data");
        let scopes_string = box_string(self.get_scopes_string(), r_30, "Scope Pointers");
        let std_out = box_string(self.get_std_out_string(term_size.1 - 10), r_50, "STD Out");
        let stats_string = box_string(self.get_stats_string(), r_50, "Stats");

        let col1 = program_string;
        let col2 = join_rows(stats_string, std_out);
        let col3 = join_rows(
            stack_string,
            join_rows(data_string, join_rows(refer_string, scopes_string)),
        );

        let mut string = format!("{}", join_cols(col1, join_cols(col2, col3)));
        string += "[x] to exit, [enter] to continue: ";

        println!("{}", string);
    }

    fn get_std_out_string(&self, max_height: u16) -> String {
        let lines: Vec<&str> = self.debug_out.lines().into_iter().collect();
        let max_height = max_height as usize;

        if lines.len() >= max_height {
            lines[(lines.len() - max_height)..].join("\n")
        } else {
            self.debug_out.clone()
        }
    }

    fn get_program_string(&self, max_height: u16) -> String {
        let mut string = String::new();

        let mut top = 0;
        let mut bottom = self.runner.prog.tape.len();

        if bottom > max_height as usize {
            let mut ntop = (self.runner.current_postion as i64) - (max_height as i64) / 2;
            if ntop < 0 {
                ntop = 0
            }

            let mut nbottom = ntop + max_height as i64;
            if nbottom > bottom as i64 {
                nbottom = bottom as i64;
                ntop = nbottom - (max_height as i64);
            }

            top = ntop as usize;
            bottom = nbottom as usize;
        }

        for idx in top..bottom {
            let x = &self.runner.prog.tape[idx];
            let x = match x {
                CMD::InternalOp(op, _) => format!("InternalOp({})", op),
                _ => format!("{x:?}"),
            };
            let padding = self.padding[idx];
            let idx = match self.runner.current_postion == idx {
                true => format!(">> {idx}",),
                false => format!("{idx}"),
            };

            string += &format!(
                "{}{idx}: {}{x}\n",
                String::from(" ").repeat(6 - string_width(&format!("{idx}"))),
                String::from("|  ").repeat(padding),
            );
        }

        return string;
    }

    fn get_stack_string(&self) -> String {
        let mut string = String::new();

        for item in &self.runner.stack {
            string += &format!("{:?}\n", item);
        }

        return string;
    }

    fn get_refer_string(&self) -> String {
        let mut string = String::new();

        for r in &self.runner.refer_stack {
            string += &format!("{r:?}\n");
        }

        return string;
    }

    fn get_data_string(&self) -> String {
        let mut string = String::new();

        for (key, val) in &self.runner.data.get_valid_data() {
            string += &format!("{} ({}): {:?}\n", key, val.1, val.0);
        }

        return string;
    }

    fn get_scopes_string(&self) -> String {
        let mut string = String::new();

        for (key, ptrs) in &self.runner.scopes {
            string += &format!("{}:\n", key);
            for ptr in ptrs.into_iter().rev().enumerate() {
                string += &format!(" {:?}", ptr.1);
                if ptr.0 < ptrs.len() - 1 {
                    string += ","
                } else {
                    string += "\n"
                }
            }
        }

        return string;
    }

    fn get_stats_string(&self) -> String {
        format!(
            "Program Complete: {}\nCommands Run: {}",
            self.complete, self.commands_run
        )
    }

    pub fn debug(&mut self) -> Result<(), RuntimeError> {
        print!("{}", termion::clear::All);

        loop {
            self.print_state();

            let mut input = String::new();
            let _ = stdin().read_line(&mut input);
            if input.trim().to_lowercase() == "x" {
                break;
            }

            if !self.complete {
                match self.runner.prog.tape[self.runner.current_postion] {
                    CMD::Print => {
                        let v = self.runner.stack_pop();
                        self.debug_out += v.string(&self.runner);
                        self.runner.current_postion += 1;
                    }
                    CMD::PrintLn => {
                        let v = self.runner.stack_pop();
                        self.debug_out += v.string(&self.runner);
                        self.debug_out += "\n";
                        self.runner.current_postion += 1;
                    }
                    _ => {
                        self.complete = self.runner.run_command()?;
                    }
                }

                self.commands_run += 1;
            }
        }

        return Ok(());
    }

    fn clear_terminal(&self) {
        print!("{}", termion::cursor::Goto(1, 1));
        print!("{}", termion::clear::AfterCursor);
        std::io::stdout().flush().unwrap();
    }
}

fn box_string(string: String, width: usize, title: &str) -> String {
    let mut new_string = create_box_bar(width, title).green().to_string();
    let mut inside = String::new();
    let pipe = "|".green();

    for line in string.lines() {
        let sw = string_width(line);

        let bar = if sw <= width {
            format!("{}{}", line, String::from(" ").repeat(width - sw))
        } else {
            format!("{}", &line[..width])
        };

        inside += &format!("\n{pipe}{bar}{pipe}");
    }

    if inside.is_empty() {
        inside = format!("\n{pipe}{}{pipe}", String::from(" ").repeat(width));
    }

    new_string += &inside;
    new_string += "\n";
    new_string += &create_box_bar(width, "").green().to_string();
    return new_string;
}

fn create_box_bar(width: usize, title: &str) -> String {
    let title_width = string_width(title);

    format!(
        "+{}{}+",
        title,
        String::from("-").repeat(width - title_width)
    )
}

fn join_rows(a: String, b: String) -> String {
    format!("{a}\n{b}")
}

fn join_cols(a: String, b: String) -> String {
    let mut string = String::new();

    let a = a.lines().into_iter().collect::<Vec<&str>>();
    let b = b.lines().into_iter().collect::<Vec<&str>>();

    let mut idx = 0;

    let a_width = string_width(a.first().unwrap());

    while a.len() >= idx || b.len() >= idx {
        let a = a.get(idx);
        let b = b.get(idx);

        string += &match a {
            Some(a) => format!(
                "{a} {}\n",
                match b {
                    Some(b) => b,
                    None => "",
                }
            ),
            None => format!(
                "{} {}\n",
                String::from(" ").repeat(a_width),
                match b {
                    Some(b) => b,
                    None => "",
                }
            ),
        };

        idx += 1;
    }

    return string;
}

fn string_width(string: &str) -> usize {
    string
        .replace("\u{1b}[32m", "")
        .replace("\u{1b}[0m", "")
        .chars()
        .count()
}
