/// Returns `true` when the shell should exit.
pub fn execute(args: &[String]) -> bool {
    match args[0].as_str() {
        "exit" => true,
        "echo" => {
            echo(&args[1..]);
            false
        }
        _ => {
            println!("{}: command not found", args[0]);
            false
        }
    }
}

fn echo(args: &[String]) {
    println!("{}", args.join(" "));
}
