use ansi_term::Colour;
use editor_helper::EditorHelper;
use interpreter::{IResult, Interpreter, Value};
use rustyline::{
    completion::FilenameCompleter, config::OutputStreamType, error::ReadlineError,
    highlight::MatchingBracketHighlighter, hint::HistoryHinter, validate::MatchingBracketValidator,
    Cmd, CompletionType, Config, EditMode, Editor, KeyEvent,
};
use std::{fs, io::Result as IOResult, path::PathBuf};
use structopt::StructOpt;

mod ast;
mod editor_helper;
mod interpreter;
mod lexer;
mod parser;
mod semantic_analyzer;
mod symbol;
mod token;

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
            Value::String(string) => println!("{}", Colour::Green.paint(format!("'{}'", string))),
            Value::NoValue => (),
        },
        Err(err) => eprintln!("{}", err),
    }
}

fn main() -> IOResult<()> {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    let args = CLIArgs::from_args();

    let mut interpreter = Interpreter::new();
    if let Some(file) = args.file {
        let result = interpreter.interpret(&fs::read_to_string(file)?);
        log_result(result);
        Ok(())
    } else {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .edit_mode(EditMode::Vi)
            .output_stream(OutputStreamType::Stdout)
            .build();
        let helper = EditorHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
        };
        let mut repl = Editor::with_config(config);
        repl.set_helper(Some(helper));
        repl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
        repl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
        let _ = repl.load_history("history.txt");
        loop {
            let readline = repl.readline("> ");
            match readline {
                Ok(line) => {
                    repl.add_history_entry(line.as_str());
                    let result = interpreter.interpret(&line);
                    log_result(result)
                }
                Err(ReadlineError::Interrupted) => break,
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        repl.save_history("history.txt").unwrap();
        Ok(())
    }
}
