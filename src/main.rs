pub mod active_parser;
pub mod cli;
pub mod errors;
pub mod finterpretor;
pub mod flat_ir;
pub mod formmatter;
pub mod lexer;
pub mod parser;

use clap::Parser;
use errors::{FileLocation, ManagerError};
use std;

fn main() {
    let args = cli::Args::parse();

    match &args.cmd {
        cli::Command::Debug { file, .. } | cli::Command::Run { file, .. } => {
            let program = {
                let mut program = match std::fs::read_to_string(&file) {
                    Ok(program) => program,
                    Err(err) => {
                        println!(
                            "{}",
                            ManagerError(
                                format!("Could not open program file | {err}"),
                                FileLocation::None
                            )
                            .prettify()
                        );
                        return;
                    }
                };
                program.push(' ');
                program
            };

            let lex_out = match lexer::lex(&program, false, &file, "", &[]) {
                Ok(lex) => lex,
                Err(err) => {
                    println!("{}", err.prettify());
                    return;
                }
            };
            let parse_out = match parser::parse(lex_out, &file) {
                Ok(parse) => parse,
                Err(err) => {
                    match err {
                        parser::ErrType::Parser(err) => println!("{}", err.prettify()),
                        parser::ErrType::Lexer(err) => println!("{}", err.prettify()),
                    };
                    return;
                }
            };

            let aparse_out = match active_parser::aparse(&parse_out) {
                Ok(aparse) => aparse,
                Err(err) => {
                    println!("{}", err.prettify());
                    return;
                }
            };

            let flat_ir_out = flat_ir::flatten(&aparse_out);
            match &args.cmd {
                cli::Command::Run { args, .. } => {
                    finterpretor::interpret(&flat_ir_out, args, false)
                }
                cli::Command::Debug { args, .. } => {
                    finterpretor::interpret(&flat_ir_out, args, true)
                }
                _ => panic!(),
            }
        }
        cli::Command::Format { file } => {
            let program = {
                let mut program = match std::fs::read_to_string(&file) {
                    Ok(program) => program,
                    Err(err) => {
                        println!(
                            "{}",
                            ManagerError(
                                format!("Could not open program file. | {err}"),
                                FileLocation::None
                            )
                            .prettify()
                        );
                        return;
                    }
                };
                program.push(' ');
                program
            };

            let text = formmatter::format(&program, 4);

            if let Err(err) = std::fs::write(&file, text) {
                println!(
                    "{}",
                    ManagerError(
                        format!("Could not write to program file. | {err}"),
                        FileLocation::None,
                    )
                    .prettify()
                );

                return;
            }
        }
        cli::Command::Update => {
            println!("Starting update.");
            let result = std::process::Command::new("cargo")
                .arg("install")
                .arg("termslang")
                .output();

            match result {
                Ok(_) => println!("Update complete via cargo."),
                Err(err) => {
                    let error = ManagerError(
                        format!(
                            "Attempt to update TermsLang spawned an error.\n{}\n\n{}",
                            "Ensure you have cargo installed on your PATH.", err
                        ),
                        FileLocation::None,
                    );

                    println!("{}", error.prettify());
                    return;
                }
            }
        }
    }
}
