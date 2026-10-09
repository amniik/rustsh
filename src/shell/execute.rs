use libc::_exit;
use nix::{
    errno::Errno,
    sys::wait::waitpid,
    unistd::{ForkResult, execv, fork},
};
use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::{env, os::unix::fs::PermissionsExt};

pub struct Executer {
    builtin_commands: HashMap<&'static str, Builtin>,
}

pub enum ExecuteResult {
    Continue,
    Exit,
    NotFound,
}

type Builtin = fn(&[String]) -> ExecuteResult;

impl Executer {
    pub fn new() -> Self {
        Self {
            builtin_commands: HashMap::from([
                ("echo", echo as Builtin),
                ("type", type_cmd),
                ("pwd", pwd),
                ("exit", exit),
            ]),
        }
    }

    pub fn execute(&self, args: &[String]) -> ExecuteResult {
        if let Some(command) = self.builtin_commands.get(args[0].as_str()) {
            return command(&args[1..]);
        }

        let executable = if args[0].contains('/') {
            find_executable(args[0].as_str())
        } else {
            find_executable_in_path(args[0].as_str())
        };

        let Some(executable) = executable else {
            return ExecuteResult::NotFound;
        };

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                if let Err(err) = waitpid(child, None) {
                    eprintln!("waitpid failed: {err}");
                }
            }

            Ok(ForkResult::Child) => {
                exec(executable.as_path(), args);
            }

            Err(err) => {
                eprintln!("fork failed: {err}");
            }
        }

        ExecuteResult::Continue
    }
}

fn exec(path: &Path, args: &[String]) {
    let cpath = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

    let eargs: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.as_bytes()).unwrap())
        .collect();

    match execv(&cpath, &eargs) {
        Ok(_) => unreachable!(),

        Err(err) => {
            eprintln!("{}: {err}", args[0]);

            unsafe {
                _exit(if err == Errno::ENOENT { 127 } else { 126 });
            }
        }
    }
}

fn exit(_: &[String]) -> ExecuteResult {
    ExecuteResult::Exit
}

fn echo(args: &[String]) -> ExecuteResult {
    println!("{}", args.join(" "));
    ExecuteResult::Continue
}

fn pwd(_: &[String]) -> ExecuteResult {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(err) => eprintln!("pwd: {err}"),
    }

    ExecuteResult::Continue
}

fn type_cmd(args: &[String]) -> ExecuteResult {
    for arg in args {
        if is_builtin_commad(arg) {
            println!("{arg} is a shell builtin");
        } else if let Some(command) = find_executable_in_path(arg) {
            println!("{arg} is {}", command.display());
        } else {
            println!("{}: not found", arg);
        }
    }
    ExecuteResult::Continue
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);

        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(name);
    if is_executable(&candidate) {
        return Some(candidate);
    }

    None
}

fn is_executable(candidate: &Path) -> bool {
    candidate.is_file()
        && candidate
            .metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

fn is_builtin_commad(command: &str) -> bool {
    matches!(command, "echo" | "type" | "exit")
}
