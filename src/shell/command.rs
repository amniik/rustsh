use nix::errno::Errno;
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnhandledWaitStatus(WaitStatus),
    WaitPidFailed(nix::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnhandledWaitStatus(status) => {
                write!(f, "jobs: unhandled wait status: {status:?}")
            }
            Error::WaitPidFailed(err) => {
                write!(f, "jobs: waitpid failed: {err}")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StdStatus {
    pub path: String,
    pub append: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    SimpleCommand {
        args: Vec<String>,
        stdout: Option<StdStatus>,
        stderr: Option<StdStatus>,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::SimpleCommand { args, .. } => {
                write!(f, "{}", args.join(" "))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessGroup {
    Foreground,
    Background,
}

#[derive(Debug, PartialEq, Eq)]
pub struct JobManager {
    jobs: Vec<Job>,
    next_job_id: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    id: u64,
    pgid: Pid,
    command: Command,
    mode: ProcessGroup,
    state: Option<JobState>,
}

impl Job {
    pub fn new(command: Command, mode: ProcessGroup) -> Self {
        Job {
            id: 0,
            pgid: Pid::from_raw(0),
            command,
            mode,
            state: None,
        }
    }

    pub fn get_command(&self) -> &Command {
        &self.command
    }

    pub fn get_mode(&self) -> &ProcessGroup {
        &self.mode
    }

    pub fn get_pgid(&self) -> &Pid {
        &self.pgid
    }

    pub fn set_state(&mut self, state: JobState) {
        self.state = Some(state);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Done,
}

impl JobManager {
    pub fn new() -> Self {
        JobManager {
            jobs: Vec::new(),
            next_job_id: 1,
        }
    }

    pub fn add_job(&mut self, pgid: Pid, mut job: Job) {
        job.id = self.next_job_id;
        job.pgid = pgid;
        job.state = Some(JobState::Running);
        println!("[{}] {}", job.id, pgid);
        self.jobs.push(job);
        self.next_job_id += 1;
    }

    pub fn update_jobs(&mut self) -> Result<(), Error> {
        for job in self.jobs.iter_mut() {
            let pgid = job.get_pgid();

            match waitpid(
                Pid::from_raw(-pgid.as_raw()),
                Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED),
            ) {
                Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                    job.set_state(JobState::Done);
                }

                Ok(WaitStatus::Stopped(_, _)) => {
                    job.set_state(JobState::Stopped);
                }

                Ok(WaitStatus::Continued(_)) => {
                    job.set_state(JobState::Running);
                }

                Ok(WaitStatus::StillAlive) => {
                    job.set_state(JobState::Running);
                }

                Ok(status) => {
                    return Err(Error::UnhandledWaitStatus(status));
                }

                Err(Errno::ECHILD) => {
                    // No remaining unwaited-for child in this process group.
                    job.set_state(JobState::Done);
                }

                Err(err) => {
                    return Err(Error::WaitPidFailed(err));
                }
            }
        }
        Ok(())
    }

    pub fn recyle_next_job_id(&mut self) {
        if self.jobs.is_empty() {
            self.next_job_id = 1;
        } else if self.jobs.len() == 1 && self.jobs.last().unwrap().id == 1 {
            self.next_job_id = 2;
        }
    }

    pub fn print_jobs(&mut self, done_only: bool) {
        for (index, job) in self.jobs.iter().enumerate() {
            if done_only && job.state != Some(JobState::Done) {
                continue;
            }
            let marker = if index == self.jobs.len() - 1 {
                '+'
            } else if index == self.jobs.len() - 2 {
                '-'
            } else {
                ' '
            };

            let state = job.state.as_ref().unwrap();

            println!(
                "[{}]{}  {:<24} {}",
                job.id,
                marker,
                format!("{:?}", state),
                job.command
            );
        }

        self.jobs.retain(|job| job.state != Some(JobState::Done));
    }
}
