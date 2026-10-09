use std::io::Write;
use std::process::{Command, Stdio};

pub fn run_shell(commands: &str) -> (String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(commands.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
