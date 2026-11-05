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

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessGroup {
    Foreground,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    pub id: Option<u64>,
    pub command: Command,
    pub pg: ProcessGroup,
}
