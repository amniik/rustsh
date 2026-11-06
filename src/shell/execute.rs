use crate::shell::command::{Command, Job, ProcessGroup, StdStatus};
use libc::_exit;
use nix::fcntl::OFlag;
use nix::fcntl::open;
use nix::sys::stat::Mode;
use nix::unistd::{dup, dup2_stderr, dup2_stdout};
use nix::{
    errno::Errno,
    sys::signal::{self, SigHandler, Signal},
    sys::wait::{WaitPidFlag, waitpid},
    unistd::{ForkResult, Pid, execv, fork, getpgrp, setpgid, tcsetpgrp},
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
    TerminalOpenFailed(nix::Error),
    SetPGFailed(nix::Error),
    SetTerminalPG(nix::Error),
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
            Error::TerminalOpenFailed(e) => write!(f, "opening terminal is faild: {e}"),
            Error::SetPGFailed(e) => write!(f, "set pg is failed: {e}"),
            Error::SetTerminalPG(e) => write!(f, "set terminal pg is failed: {e}"),
        }
    }
}

pub struct Executer {
    builtin_commands: HashMap<&'static str, Builtin>,
    shell_pgid: Pid,
    terminal_fd: OwnedFd,
}

type Builtin = fn(&[String]) -> Result<(), Error>;

impl Executer {
    pub fn new() -> Result<Self, Error> {
        let terminal_fd = open("/dev/tty", OFlag::O_RDWR | OFlag::O_CLOEXEC, Mode::empty())
            .map_err(Error::TerminalOpenFailed)?;

        let shell_pgid = getpgrp();

        // ignore control signals
        for signal in [
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTSTP,
            Signal::SIGTTIN,
            Signal::SIGTTOU,
        ] {
            unsafe {
                signal::signal(signal, SigHandler::SigIgn).unwrap();
            }
        }

        Ok(Self {
            builtin_commands: HashMap::from([
                ("echo", echo as Builtin),
                ("type", type_cmd),
                ("pwd", pwd),
                ("cd", cd),
                ("exit", exit),
            ]),
            shell_pgid,
            terminal_fd,
        })
    }

    pub fn execute(&self, job: Job) -> Result<(), Error> {
        match job.pg {
            ProcessGroup::Foreground if self.is_simple_builtin_command(&job.command) => {
                self.execute_simple_builtin_command(job.command)
            }

            ProcessGroup::Foreground => self.execute_foreground_job(job),
        }
    }

    fn execute_foreground_job(&self, job: Job) -> Result<(), Error> {
        match job.command {
            Command::SimpleCommand {
                args,
                stdout,
                stderr,
            } => {
                let pgid = self.execute_simple_command(&args, stdout, stderr)?;

                tcsetpgrp(&self.terminal_fd, pgid).map_err(Error::SetTerminalPG)?;

                let wait_result =
                    waitpid(Pid::from_raw(-pgid.as_raw()), Some(WaitPidFlag::WUNTRACED));

                let restore_result =
                    tcsetpgrp(&self.terminal_fd, self.shell_pgid).map_err(Error::SetTerminalPG);

                wait_result.map_err(Error::WaitPidFailed)?;
                restore_result?;

                Ok(())
            }
        }
    }

    fn is_simple_builtin_command(&self, command: &Command) -> bool {
        match command {
            Command::SimpleCommand {
                args,
                stdout: _,
                stderr: _,
            } => {
                let Some(program) = args.first() else {
                    return false;
                };
                self.builtin_commands.contains_key(program.as_str())
            }
        }
    }

    fn execute_simple_builtin_command(&self, command: Command) -> Result<(), Error> {
        let Command::SimpleCommand {
            args,
            stdout,
            stderr,
        } = command;
        let cmd = self
            .builtin_commands
            .get(args.first().unwrap().as_str())
            .unwrap();
        let mut terminal_stdout: Option<OwnedFd> = None;
        let mut terminal_stderr: Option<OwnedFd> = None;
        if let Some(std) = stdout {
            terminal_stdout = Some(save_stdout()?);
            redirect_stdout(&std)?;
        }
        if let Some(std) = stderr {
            terminal_stderr = Some(save_stderr()?);
            redirect_stderr(&std)?;
        }
        let result = cmd(&args[1..]);
        if let Some(out) = terminal_stdout {
            // Restore the Stdout
            dup2_stdout(out).map_err(Error::DupFailed)?;
        }
        if let Some(err) = terminal_stderr {
            // Restore the Stdout
            dup2_stderr(err).map_err(Error::DupFailed)?;
        }
        result
    }

    fn execute_simple_command(
        &self,
        args: &[String],
        stdout: Option<StdStatus>,
        stderr: Option<StdStatus>,
    ) -> Result<Pid, Error> {
        let Some(program) = args.first() else {
            return Err(Error::ArgsEmpty);
        };

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                setpgid(child, child).map_err(Error::SetPGFailed)?;
                Ok(child)
            }

            Ok(ForkResult::Child) => {
                restore_child_signals();
                setpgid(Pid::from_raw(0), Pid::from_raw(0)).map_err(Error::SetPGFailed)?;
                if let Some(std) = stdout
                    && let Err(err) = redirect_stdout(&std)
                {
                    eprintln!("{program}: {err}");
                    unsafe {
                        _exit(1);
                    }
                }
                if let Some(std) = stderr
                    && let Err(err) = redirect_stderr(&std)
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
                unreachable!()
            }

            Err(err) => Err(Error::ForkFailed(err)),
        }
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

fn redirect_stdout(std: &StdStatus) -> Result<(), Error> {
    let StdStatus { path, append } = std;
    let flag = if *append {
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_APPEND
    } else {
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_TRUNC
    };
    let f = open(
        path.as_str(),
        flag,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .map_err(Error::StdoutRedirectionFailed)?;
    dup2_stdout(f).map_err(Error::DupFailed)
}

fn save_stdout() -> Result<OwnedFd, Error> {
    dup(stdout()).map_err(Error::DupFailed)
}

fn redirect_stderr(std: &StdStatus) -> Result<(), Error> {
    let StdStatus { path, append } = std;
    let flag = if *append {
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_APPEND
    } else {
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_TRUNC
    };
    let f = open(
        path.as_str(),
        flag,
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

fn restore_child_signals() {
    for signal in [
        Signal::SIGINT,
        Signal::SIGQUIT,
        Signal::SIGTSTP,
        Signal::SIGTTIN,
        Signal::SIGTTOU,
        Signal::SIGCHLD,
    ] {
        unsafe {
            signal::signal(signal, SigHandler::SigDfl).unwrap();
        }
    }
}
