use nix::errno::Errno;
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;
use std::fmt;
use std::os::fd::BorrowedFd;

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

#[derive(Debug, Default)]
pub enum StdStream {
    File {
        path: String,
        append: bool,
    },
    #[default]
    Terminal,
}

pub enum RuntimeStream<'a> {
    Configured(&'a StdStream),
    Pipe(BorrowedFd<'a>),
}

#[derive(Debug)]
pub enum Command {
    Simple(SimpleCommand),
    Pipeline(Vec<SimpleCommand>),
}

#[derive(Debug)]
pub struct SimpleCommand {
    pub args: Vec<String>,
    pub sout: StdStream,
    pub serr: StdStream,
    pub sin: StdStream,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Simple(command) => {
                write!(f, "{}", command.args.join(" "))
            }
            Command::Pipeline(commands) => {
                let pipeline = commands
                    .iter()
                    .map(|command| command.args.join(" "))
                    .collect::<Vec<_>>()
                    .join(" | ");

                write!(f, "{pipeline}")
            }
        }
    }
}

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    state: ProcessState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Stopped,
    Done,
}

impl Process {
    pub fn new(pid: Pid) -> Self {
        Process {
            pid,
            state: ProcessState::Running,
        }
    }

    pub fn get_state(&self) -> &ProcessState {
        &self.state
    }

    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessGroup {
    Foreground,
    Background,
}

#[derive(Debug)]
pub struct JobManager {
    jobs: Vec<Job>,
    next_job_id: u64,
}

#[derive(Debug)]
pub struct Job {
    id: u64,
    pgid: Pid,
    command: Command,
    mode: ProcessGroup,
    state: JobState,
    processes: Vec<Process>,
}

impl Job {
    pub fn new(command: Command, mode: ProcessGroup) -> Self {
        Job {
            id: 0,
            pgid: Pid::from_raw(0),
            command,
            mode,
            state: JobState::Running,
            processes: Vec::new(),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn add_process(&mut self, process: Process) {
        self.processes.push(process);
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

    pub fn get_state(&self) -> &JobState {
        &self.state
    }

    pub fn set_pgid(&mut self, pgid: Pid) {
        self.pgid = pgid;
    }

    pub fn update_state(&mut self) {
        if self.processes.iter().all(|p| p.state == ProcessState::Done) {
            self.state = JobState::Done;
        } else if self
            .processes
            .iter()
            .all(|p| matches!(p.state, ProcessState::Done | ProcessState::Stopped))
        {
            self.state = JobState::Stopped;
        } else {
            self.state = JobState::Running;
        }
    }

    pub fn set_process_state(&mut self, pid: Pid, state: ProcessState) {
        if let Some(process) = self.processes.iter_mut().find(|p| p.pid == pid) {
            process.set_state(state);
        }
    }

    pub fn set_stopped_processes_running(&mut self) {
        for process in &mut self.processes {
            if process.get_state() == &ProcessState::Stopped {
                process.set_state(ProcessState::Running);
            }
        }

        self.update_state();
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

    pub fn add_job(&mut self, mut job: Job) {
        job.id = self.next_job_id;
        job.state = JobState::Running;
        println!("[{}] {}", job.id, job.pgid);
        self.jobs.push(job);
        self.next_job_id += 1;
    }

    pub fn get_mut_job(&mut self, id: Option<u64>) -> Option<&mut Job> {
        match id {
            Some(id) => self.jobs.iter_mut().find(|job| job.get_id() == id),
            None => self.jobs.last_mut(),
        }
    }

    pub fn take_job(&mut self, id: Option<u64>) -> Option<Job> {
        match id {
            Some(id) => {
                let index = self.jobs.iter().position(|job| job.get_id() == id)?;
                Some(self.jobs.remove(index))
            }
            None => self.jobs.pop(),
        }
    }

    pub fn update_jobs(&mut self) -> Result<(), Error> {
        for job in self.jobs.iter_mut() {
            let pgid = Pid::from_raw(-job.get_pgid().as_raw());

            loop {
                match waitpid(
                    pgid,
                    Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED),
                ) {
                    Ok(WaitStatus::Exited(pid, _)) | Ok(WaitStatus::Signaled(pid, _, _)) => {
                        job.set_process_state(pid, ProcessState::Done);
                    }

                    Ok(WaitStatus::Stopped(pid, _)) => {
                        job.set_process_state(pid, ProcessState::Stopped);
                    }

                    Ok(WaitStatus::Continued(pid)) => {
                        job.set_process_state(pid, ProcessState::Running);
                    }

                    Ok(WaitStatus::StillAlive) => break,

                    Ok(status) => {
                        return Err(Error::UnhandledWaitStatus(status));
                    }

                    Err(Errno::ECHILD) => break,

                    Err(err) => {
                        return Err(Error::WaitPidFailed(err));
                    }
                }
            }

            job.update_state();
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
            if done_only && job.state != JobState::Done {
                continue;
            }
            let marker = if index == self.jobs.len() - 1 {
                '+'
            } else if index == self.jobs.len() - 2 {
                '-'
            } else {
                ' '
            };

            let state = &job.state;

            println!(
                "[{}]{}  {:<24} {}",
                job.id,
                marker,
                format!("{:?}", state),
                job.command
            );
        }

        self.jobs.retain(|job| job.state != JobState::Done);
    }
}
