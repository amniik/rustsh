use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_command_in_path() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"which echo\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "$ /usr/bin/echo\n$ ");
}

#[test]
fn test_absolute_path_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"/usr/bin/echo hello\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "$ hello\n$ ");
}

#[test]
fn test_command_not_found() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"does-not-exist\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "$ does-not-exist: command not found\n$ ");
}
