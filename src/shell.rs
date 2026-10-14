use std::io::{self, Write};

mod execute;
mod lexer;
mod parser;

use execute::Executer;
use lexer::Tokenizer;
use parser::Parser;

pub struct Shell {
    tokenizer: Tokenizer,
    parser: Parser,
    executer: Executer,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            tokenizer: Tokenizer::new(),
            parser: Parser::new(),
            executer: Executer::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.print_prompt();
            let Some(cmd) = self.read_input() else {
                break;
            };
            if self.eval(&cmd) {
                break;
            }
        }
    }

    fn print_prompt(&self) {
        print!("$ ");
        io::stdout().flush().unwrap();
    }

    fn read_input(&self) -> Option<String> {
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.is_empty() {
            return None;
        }
        Some(command.trim().to_string())
    }

    fn eval(&self, command: &str) -> bool {
        match self.tokenizer.tokenize(command) {
            Err(e) => {
                println!("{}", e);
                false
            }
            Ok(tokens) => {
                if tokens.is_empty() {
                    return false;
                }
                match self.parser.parse(tokens) {
                    Err(e) => {
                        println!("{}", e);
                        false
                    }
                    Ok(command) => match self.executer.execute(command) {
                        Err(e) => match e {
                            execute::Error::Exit => true,
                            _ => {
                                eprintln!("{}", e);
                                false
                            }
                        },
                        Ok(()) => false,
                    },
                }
            }
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
