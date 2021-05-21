use interpreter::{Interpreter, Value};
use std::io;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod semantic_analyzer;
mod symbol;
mod token;

fn main() {
    let mut interpreter = Interpreter::new();
    loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();

        if input == "exit" {
            break;
        }

        match interpreter.interpret(&input) {
            Ok(result) => match result {
                Value::NoValue => (),
                val => println!("{:?}", val),
            },
            Err(err) => eprintln!("ERROR: {}", err),
        };
    }
}
