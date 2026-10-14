mod common;

use common::run_shell;
use std::io::Write;
use std::process::{Command, Stdio};

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

#[test]
fn test_stdout_redirection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let input = format!("echo hello > {}\n", output_file.display());
    let (stdout, stderr) = run_shell(&input);

    assert_eq!(stdout, "$ $ ");
    assert_eq!(stderr, "");
    assert_eq!(std::fs::read_to_string(&output_file).unwrap(), "hello\n");
}

#[test]
fn test_stdout_redirection_truncates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    std::fs::write(&output_file, "old content\n").unwrap();

    let input = format!("echo new > {}\n", output_file.display());
    let (_, stderr) = run_shell(&input);

    assert_eq!(stderr, "");
    assert_eq!(std::fs::read_to_string(&output_file).unwrap(), "new\n");
}

#[test]
fn test_command_not_found_with_stdout_redirection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let input = format!("does-not-exist > {}\n", output_file.display());
    let (stdout, stderr) = run_shell(&input);

    assert_eq!(stdout, "$ $ ");
    assert_eq!(stderr, "does-not-exist: command not found\n");
    assert_eq!(std::fs::read_to_string(&output_file).unwrap(), "");
}

#[test]
fn test_builtin_stdout_restored_after_redirection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let input = format!(
        "echo redirected > {}\necho terminal\n",
        output_file.display()
    );
    let (stdout, stderr) = run_shell(&input);

    assert_eq!(stdout, "$ $ terminal\n$ ");
    assert_eq!(stderr, "");
    assert_eq!(
        std::fs::read_to_string(&output_file).unwrap(),
        "redirected\n"
    );
}
