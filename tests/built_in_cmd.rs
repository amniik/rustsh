use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_echo_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"echo hello\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "$ hello\n$ ");
}

#[test]
fn test_type_builtin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"type echo\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("echo is a shell builtin"));
}

#[test]
fn test_type_unknown_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"type hello\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("hello: not found"));
}

#[test]
fn test_type_external_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"type ls\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("ls is /usr/bin/ls"));
}

#[test]
fn test_exit_command() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"exit\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
}
