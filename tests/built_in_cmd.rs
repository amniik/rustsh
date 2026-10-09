mod common;

use common::run_shell;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_echo_command() {
    let (stdout, _) = run_shell("echo hello\n");

    assert_eq!(stdout, "$ hello\n$ ");
}

#[test]
fn test_type_builtin() {
    let (stdout, _) = run_shell("type echo\n");

    assert!(stdout.contains("echo is a shell builtin"));
}

#[test]
fn test_type_unknown_command() {
    let (stdout, _) = run_shell("type hello\n");

    assert!(stdout.contains("hello: not found"));
}

#[test]
fn test_type_external_command() {
    let (stdout, _) = run_shell("type ls\n");

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

#[test]
fn test_pwd_command() {
    let (stdout, _) = run_shell("pwd\n");

    let s = env::current_dir().unwrap().into_string().unwrap();
    assert!(stdout.contains(&s));
}

#[test]
fn test_cd_absolute_path() {
    let (stdout, _) = run_shell("cd /tmp\npwd\n");
    assert!(
        stdout.contains("/tmp"),
        "Expected /tmp in stdout, got: {stdout:?}"
    );
}
#[test]
fn test_cd_parent_directory() {
    let initial_dir = std::env::current_dir().unwrap();
    let parent_dir = initial_dir.parent().unwrap();
    let commands = "cd ..\npwd\n".to_string();
    let (stdout, _) = run_shell(&commands);
    assert!(
        stdout.contains(&parent_dir.display().to_string()),
        "Expected parent directory in stdout, got: {stdout:?}"
    );
}
#[test]
fn test_cd_home() {
    let home = std::env::var_os("HOME").unwrap();
    let expected = std::path::PathBuf::from(home);
    let (stdout, _) = run_shell("cd\npwd\n");
    assert!(
        stdout.contains(&expected.display().to_string()),
        "Expected home directory in stdout, got: {stdout:?}"
    );
}

#[test]
fn test_cd_tilde() {
    let home = std::env::var_os("HOME").unwrap();
    let expected = std::path::PathBuf::from(home);
    let (stdout, _) = run_shell("cd ~\npwd\n");
    assert!(
        stdout.contains(&expected.display().to_string()),
        "Expected home directory in stdout, got: {stdout:?}"
    );
}

#[test]
fn test_cd_home_subdirectory() {
    let home = std::env::var_os("HOME").unwrap();
    let expected = std::path::PathBuf::from(home);
    let (stdout, _) = run_shell("cd ~/.\npwd\n");
    assert!(
        stdout.contains(&expected.display().to_string()),
        "Expected home directory path in stdout, got: {stdout:?}"
    );
}

#[test]
fn test_cd_nonexistent_directory() {
    let (_, stderr) = run_shell("cd /this-directory-should-not-exist-rustsh\n");
    assert!(
        stderr.contains("cd:"),
        "Expected a cd error, got: {stderr:?}"
    );
}

#[test]
fn test_cd_too_many_arguments() {
    let (_, stderr) = run_shell("cd /tmp /var\n");
    assert!(
        stderr.contains("cd: too many arguments"),
        "Expected too-many-arguments error, got: {stderr:?}"
    );
}
