pub struct Shell {}

impl Shell {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) {
        self.print_prompt();
    }

    fn print_prompt(&self) {
        print!("$ ")
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
