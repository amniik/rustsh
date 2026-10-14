use crate::shell::parser::Command;
use libc::_exit;
use nix::fcntl::OFlag;
use nix::fcntl::open;
use nix::sys::stat::Mode;
use nix::unistd::{dup, dup2_stderr, dup2_stdout};
use nix::{
    errno::Errno,
    sys::wait::waitpid,
    unistd::{ForkResult, execv, fork},
};
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::io::{stderr, stdout};
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::{env, os::unix::fs::PermissionsExt};

#[derive(Debug)]
pub enum Error {
    ArgsEmpty,
    DupFailed(nix::Error),
    StdoutRedirectionFailed(nix::Error),
    StderrRedirectionFailed(nix::Error),
    FileStatusFailed(std::io::Error),
    CommandNotFound(String),
    WaitPidFailed(nix::Error),
    ForkFailed(nix::Error),
    Exit,
    PwdFailed(std::io::Error),
    CdFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ArgsEmpty => {
                write!(f, "empty arguments passed to the executor")
            }
            Error::DupFailed(e) => write!(f, "dup failed: {e}"),
            Error::StdoutRedirectionFailed(e) => {
                write!(f, "stdout redirection failed: {e}")
            }
            Error::StderrRedirectionFailed(e) => {
                write!(f, "stderr redirection failed: {e}")
            }
            Error::FileStatusFailed(e) => {
                write!(f, "failed to obtain file status: {e}")
            }
            Error::CommandNotFound(command) => write!(f, "{}: command not found", command),
            Error::WaitPidFailed(e) => write!(f, "waitpid failed: {e}"),
            Error::ForkFailed(e) => write!(f, "fork failed: {e}"),
            Error::Exit => Ok(()),
            Error::PwdFailed(e) => write!(f, "pwd: {e}"),
            Error::CdFailed(s) => write!(f, "cd: {s}"),
        }
    }
}

pub struct Executer {
    builtin_commands: HashMap<&'static str, Builtin>,
}

type Builtin = fn(&[String]) -> Result<(), Error>;

impl Executer {
    pub fn new() -> Self {
        Self {
            builtin_commands: HashMap::from([
                ("echo", echo as Builtin),
                ("type", type_cmd),
                ("pwd", pwd),
                ("cd", cd),
                ("exit", exit),
            ]),
        }
    }

    pub fn execute(&self, command: Command) -> Result<(), Error> {
        match command {
            Command::SimpleCommand {
                args,
                stdout,
                stderr,
            } => self.execute_simple_command(&args, stdout, stderr),
        }
    }

    pub fn execute_simple_command(
        &self,
        args: &[String],
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> Result<(), Error> {
        let Some(program) = args.first() else {
            return Err(Error::ArgsEmpty);
        };
        if let Some(command) = self.builtin_commands.get(program.as_str()) {
            let mut terminal_stdout: Option<OwnedFd> = None;
            let mut terminal_stderr: Option<OwnedFd> = None;
            if let Some(path) = stdout {
                terminal_stdout = Some(save_stdout()?);
                redirect_stdout(&path)?;
            }
            if let Some(path) = stderr {
                terminal_stderr = Some(save_stderr()?);
                redirect_stderr(&path)?;
            }
            let result = command(&args[1..]);
            if let Some(out) = terminal_stdout {
                // Restore the Stdout
                dup2_stdout(out).map_err(Error::DupFailed)?;
            }
            if let Some(err) = terminal_stderr {
                // Restore the Stdout
                dup2_stderr(err).map_err(Error::DupFailed)?;
            }
            return result;
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                if let Err(err) = waitpid(child, None) {
                    return Err(Error::WaitPidFailed(err));
                }
            }

            Ok(ForkResult::Child) => {
                if let Some(path) = stdout
                    && let Err(err) = redirect_stdout(&path)
                {
                    eprintln!("{program}: {err}");
                    unsafe {
                        _exit(1);
                    }
                }
                if let Some(path) = stderr
                    && let Err(err) = redirect_stderr(&path)
                {
                    eprintln!("{program}: {err}");
                    unsafe {
                        _exit(1);
                    }
                }
                let executable = if program.contains('/') {
                    find_executable(program)
                } else {
                    find_executable_in_path(program)
                };

                let Some(executable) = executable else {
                    eprintln!("{}", Error::CommandNotFound(String::from(program)));
                    unsafe {
                        _exit(127);
                    }
                };
                exec(executable.as_path(), args);
            }

            Err(err) => {
                return Err(Error::ForkFailed(err));
            }
        }

        Ok(())
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

fn redirect_stdout(path: &str) -> Result<(), Error> {
    let f = open(
        path,
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_TRUNC,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .map_err(Error::StdoutRedirectionFailed)?;
    dup2_stdout(f).map_err(Error::DupFailed)
}

fn save_stdout() -> Result<OwnedFd, Error> {
    dup(stdout()).map_err(Error::DupFailed)
}

fn redirect_stderr(path: &str) -> Result<(), Error> {
    let f = open(
        path,
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_TRUNC,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .map_err(Error::StderrRedirectionFailed)?;
    dup2_stderr(f).map_err(Error::DupFailed)
}

fn save_stderr() -> Result<OwnedFd, Error> {
    dup(stderr()).map_err(Error::DupFailed)
}

fn exit(_: &[String]) -> Result<(), Error> {
    Err(Error::Exit)
}

fn echo(args: &[String]) -> Result<(), Error> {
    println!("{}", args.join(" "));
    Ok(())
}

fn pwd(_: &[String]) -> Result<(), Error> {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(err) => return Err(Error::PwdFailed(err)),
    }

    Ok(())
}

fn cd(args: &[String]) -> Result<(), Error> {
    if args.len() > 1 {
        return Err(Error::CdFailed(String::from("too many arguments")));
    }

    let path = match args.first() {
        None => match env::var_os("HOME") {
            Some(home) => PathBuf::from(home),
            None => {
                return Err(Error::CdFailed(String::from("HOME not set")));
            }
        },
        Some(dir) if dir == "~" => match env::var_os("HOME") {
            Some(home) => PathBuf::from(home),
            None => {
                return Err(Error::CdFailed(String::from("HOME not set")));
            }
        },
        Some(dir) if dir.starts_with("~/") => match env::var_os("HOME") {
            Some(home) => {
                let mut path = PathBuf::from(home);
                path.push(&dir[2..]);
                path
            }
            None => {
                return Err(Error::CdFailed(String::from("HOME not set")));
            }
        },
        Some(dir) => PathBuf::from(dir),
    };

    if let Err(err) = env::set_current_dir(&path) {
        eprintln!("cd: {}: {err}", path.display());
    }

    Ok(())
}

fn type_cmd(args: &[String]) -> Result<(), Error> {
    for arg in args {
        if is_builtin_commad(arg) {
            println!("{arg} is a shell builtin");
        } else if let Some(command) = find_executable_in_path(arg) {
            println!("{arg} is {}", command.display());
        } else {
            println!("{}: not found", arg);
        }
    }
    Ok(())
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);

        if let Ok(exist) = is_executable(&candidate)
            && exist
        {
            return Some(candidate);
        }
    }

    None
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(name);
    if let Ok(exist) = is_executable(&candidate)
        && exist
    {
        return Some(candidate);
    }

    None
}

fn is_executable(candidate: &Path) -> Result<bool, Error> {
    candidate
        .metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .map_err(Error::FileStatusFailed)
}

fn is_builtin_commad(command: &str) -> bool {
    matches!(command, "echo" | "type" | "exit")
}
