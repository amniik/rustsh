/// Returns `true` when the shell should exit.
pub fn execute(args: &[String]) -> bool {
    match args[0].as_str() {
        "exit" => true,
        _ => {
            println!("{}: command not found", args[0]);
            false
        }
    }
}
