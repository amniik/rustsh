use std::path::PathBuf;
use std::{env, os::unix::fs::PermissionsExt};

/// Returns `true` when the shell should exit.
pub fn execute(args: &[String]) -> bool {
    match args[0].as_str() {
        "exit" => true,
        "echo" => {
            echo(&args[1..]);
            false
        }
        "type" => {
            type_cmd(&args[1..]);
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

fn is_builtin_commad(command: &str) -> bool {
    matches!(command, "echo" | "type" | "exit")
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);

        if candidate.is_file()
            && candidate
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        {
            return Some(candidate);
        }
    }

    None
}

fn type_cmd(args: &[String]) {
    for arg in args {
        if is_builtin_commad(arg) {
            println!("{arg} is a shell builtin");
        } else if let Some(command) = find_executable(arg) {
            println!("{arg} is {}", command.display());
        } else {
            println!("{}: not found", arg);
        }
    }
}
