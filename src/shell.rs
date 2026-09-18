use std::io::{self, Write};

mod execute;

pub struct Shell {}

impl Shell {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) {
        loop {
            self.print_prompt();
            let Some(cmd) = self.read_input() else {
                break;
            };
            let args = self.parser(&cmd);
            if self.eval(&args) {
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

    fn parser(&self, command: &str) -> Vec<String> {
        command.split_whitespace().map(String::from).collect()
    }

    fn eval(&self, args: &[String]) -> bool {
        if args.is_empty() {
            return false;
        }

        execute::execute(args)
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
