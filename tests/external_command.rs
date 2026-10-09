mod common;

use common::run_shell;

#[test]
fn test_command_in_path() {
    let (stdout, _) = run_shell("which echo\n");

    assert_eq!(stdout, "$ /usr/bin/echo\n$ ");
}

#[test]
fn test_absolute_path_command() {
    let (stdout, _) = run_shell("/usr/bin/echo hello\n");

    assert_eq!(stdout, "$ hello\n$ ");
}

#[test]
fn test_command_not_found() {
    let (stdout, _) = run_shell("does-not-exist\n");

    assert_eq!(stdout, "$ does-not-exist: command not found\n$ ");
}
