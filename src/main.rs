use ansi_term::Colour;
use enviroment::{FunctionType, Value};
use interpreter::{IResult, Interpreter};
use repl::Repl;
use rustyline::error::ReadlineError;
use std::{fs, io::Result as IOResult, path::PathBuf};
use structopt::StructOpt;

mod ast;
mod editor_helper;
mod enviroment;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod semantic_analyzer;
mod symbol;
mod symbol_table;
mod token;
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Neko")]
struct CLIArgs {
    /// file to process
    file: Option<PathBuf>,
}

fn log_result(result: IResult) {
    match result {
        Ok(val) => match val {
            Value::Number(num) => println!("{}", Colour::Yellow.paint(num.to_string())),
            Value::Boolean(boolean) => println!("{}", Colour::Yellow.paint(boolean.to_string())),
            Value::String(string) => println!("{}", Colour::Green.paint(format!("{:?}", string))),
            Value::Function(function_type, _) => println!(
                "{}",
                match function_type {
                    FunctionType::Function(function) =>
                        Colour::Green.paint(format!("[Function: {}]", function.name)),
                    FunctionType::Lambda(_) => Colour::Green.paint("[Function: (lambda)]"),
                }
            ),
            Value::NoValue => (),
        },
        Err(err) => eprintln!("{}", err),
    }
}

const HELP: &str = r#".editor   Enter editor mode
.exit     Exit the REPL
.help     Print this help message
.load     Load Neko from a file into the REPL session
.save     Save all evaluated commands in this REPL session to a file"#;

fn main() -> IOResult<()> {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    let args = CLIArgs::from_args();

    if let Some(file) = args.file {
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&fs::read_to_string(file)?);
        Ok(())
    } else {
        println!(
            "Neko v{} | [{}, {}] for {}.\nType '.help' for more information.",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.unwrap_or("unknown"),
            built_info::BUILT_TIME_UTC.split(' ').collect::<Vec<&str>>()[..5]
                .join(", ")
                .replacen(",", "", 4),
            built_info::TARGET,
        );

        let mut interpreter = Interpreter::new();
        let mut repl = Repl::new();
        let _ = repl.editor.load_history("history.txt");
        loop {
            let readline = if repl.editor_enabled {
                repl.editor.readline("")
            } else {
                repl.editor.readline("> ")
            };
            match readline {
                Ok(line) => {
                    repl.editor.add_history_entry(&line);
                    if repl.editor_enabled {
                        repl.disable_editor();
                        repl.bind()
                    }
                    match line.as_str() {
                        ".editor" => {
                            println!(
                                "// Entering editor mode (Ctrl+D to finish, Ctrl+C to cancel)"
                            );
                            repl.toggle_editor();
                        }
                        ".help" => {
                            println!("{}", HELP);
                        }
                        ".exit" => break,
                        _ => {
                            let mut split = line.split(' ');
                            match split.next() {
                                Some(".load") => {
                                    match split.next() {
                                        Some(path) => match fs::read_to_string(path) {
                                            Ok(content) => {
                                                let result = interpreter.interpret(&&content);
                                                if result.is_ok() {
                                                    repl.add_history(&line);
                                                };
                                                log_result(result);
                                            }
                                            Err(err) => eprintln!("{}", err),
                                        },
                                        None => eprintln!(
                                            "Provide a path to the file you want to load."
                                        ),
                                    };
                                }
                                Some(".save") => {
                                    match split.next() {
                                        Some(path) => {
                                            match fs::write(path, repl.session_history.join("\n")) {
                                                Ok(()) => println!("Session saved to {}", path),
                                                Err(err) => eprintln!("{}", err),
                                            };
                                        }
                                        None => eprintln!(
                                            "Provide a path to the file you want to save to."
                                        ),
                                    };
                                }
                                _ => {
                                    let result = interpreter.interpret(&line);
                                    if result.is_ok() {
                                        repl.add_history(&line);
                                    };
                                    log_result(result);
                                }
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    if repl.editor_enabled {
                        repl.toggle_editor();
                    } else {
                        break;
                    }
                }
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        repl.editor.save_history("history.txt").unwrap();
        Ok(())
    }
}
