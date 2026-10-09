mod common;

use common::run_shell;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_repl() {
    let (stdout, _) = run_shell("hello\n");

    assert_eq!(stdout, "$ hello: command not found\n$ ");
}

#[test]
fn test_empty_input() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustsh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
}
