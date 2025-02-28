mod active_parser;
mod cli;
mod errors;
mod finterpretor;
mod flat_ir;
mod formmatter;
mod lexer;
mod lsp;
mod parser;

use std::collections::HashMap;

use clap::Parser;
use errors::{ErrorType, FileLocation, ManagerError};
use lsp::{lsp, Lsp};

fn main() {
    let args = cli::Args::parse();

    match &args.cmd {
        cli::Command::Debug { file, .. } | cli::Command::Run { file, .. } => {
            let program = {
                let mut program = match std::fs::read_to_string(&file) {
                    Ok(program) => program,
                    Err(err) => {
                        eprintln!(
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
                        ErrorType::Parser(err) => println!("{}", err.prettify()),
                        ErrorType::Lexer(err) => println!("{}", err.prettify()),
                        _ => panic!(),
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

            let interpretor_out = match &args.cmd {
                cli::Command::Run { args, .. } => {
                    let flat_ir_out = flat_ir::flatten(&aparse_out, false);
                    finterpretor::interpret(&flat_ir_out, args, false)
                }
                cli::Command::Debug { args, .. } => {
                    let flat_ir_out = flat_ir::flatten(&aparse_out, true);
                    finterpretor::interpret(&flat_ir_out, args, true)
                }
                _ => panic!(),
            };

            if let Err(err) = interpretor_out {
                println!("{}", err.prettify());
            }
        }
        cli::Command::Format { file, to_stdout } => {
            let program = {
                let mut program = match std::fs::read_to_string(&file) {
                    Ok(program) => program,
                    Err(err) => {
                        eprintln!(
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

            let to_stdout = match to_stdout {
                Some(some) => *some,
                None => false,
            };

            match to_stdout {
                true => println!("{}", text),
                false => {
                    if let Err(err) = std::fs::write(&file, text) {
                        eprintln!(
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

                    eprintln!("{}", error.prettify());
                    return;
                }
            }
        }
        cli::Command::Lsp {
            file,
            line,
            col,
            run_parse,
        } => {
            let program = {
                let mut program = match std::fs::read_to_string(&file) {
                    Ok(program) => program,
                    Err(err) => {
                        eprintln!(
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
                    let lsp = Lsp {
                        vars: Vec::new(),
                        errors: vec![ErrorType::Lexer(err)],
                        structs: HashMap::new(),
                        functions: HashMap::new(),
                    };

                    println!("{}", lsp.json());
                    return;
                }
            };

            let run_parse = match run_parse {
                Some(some) => *some,
                None => false,
            };

            let lsp = lsp(lex_out, file, *line, *col, run_parse);
            println!("{}", lsp.json())
        }
    }
}
