mod common;

use common::run_shell;

#[test]
fn execute_foreground_pipeline() {
    let (stdout, _) = run_shell("printf hello | wc -c\n");

    assert_eq!(stdout, "$ 5\n$ ");
}

#[test]
fn background_pipeline_tracks_all_processes() {
    let (stdout, stderr) = run_shell(
        "sleep 1 | sleep 4 &\n\
         jobs\n\
         sleep 2\n\
         jobs\n\
         sleep 3\n\
         jobs\n\
         exit\n",
    );

    assert!(stderr.is_empty(), "stderr: {stderr}");

    let running_count = stdout.matches("Running").count();
    let done_count = stdout.matches("Done").count();

    assert!(
        running_count >= 2,
        "expected the job to be Running twice, output: {stdout}"
    );

    assert!(
        done_count >= 1,
        "expected the job to eventually become Done, output: {stdout}"
    );
}

#[test]
fn background_pipeline_with_partially_stopped_process() {
    let tempdir = tempfile::tempdir().unwrap();
    let pid_file = tempdir.path().join("first.pid");
    let pid_file = pid_file.to_str().unwrap();

    let command = format!(
        "sh -c 'echo $$ > {pid_file}; kill -STOP $$; sleep 1' \
         | sh -c 'sleep 4; kill -CONT $(cat {pid_file})' &\n\
         sleep 1\n\
         jobs\n\
         sleep 5\n\
         jobs\n\
         exit\n"
    );

    let (stdout, stderr) = run_shell(&command);

    assert!(stderr.is_empty(), "stderr: {stderr}");

    assert!(
        stdout.contains("Running"),
        "expected partially stopped pipeline to remain Running: {stdout}"
    );

    assert!(
        stdout.contains("Done"),
        "expected pipeline to eventually become Done: {stdout}"
    );
}

#[test]
fn background_resumes_stopped_job() {
    let (stdout, stderr) = run_shell(
        "sh -c 'kill -STOP $$; echo resumed' &\n\
         sleep 1\n\
         bg\n\
         sleep 1\n\
         exit\n",
    );

    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert!(
        stdout.contains("resumed"),
        "expected process to resume, stdout: {stdout}"
    );
}

#[test]
fn foreground_resumes_stopped_job() {
    let (stdout, stderr) = run_shell(
        "sh -c 'kill -STOP $$; echo resumed' &\n\
         sleep 1\n\
         fg\n\
         echo after_fg\n\
         exit\n",
    );

    assert!(stderr.is_empty(), "stderr: {stderr}");

    assert!(
        stdout.contains("resumed"),
        "expected process to resume, stdout: {stdout}"
    );

    assert!(
        stdout.contains("after_fg"),
        "expected shell to continue after fg, stdout: {stdout}"
    );
}

#[test]
fn foreground_selects_job_by_id() {
    let (stdout, stderr) = run_shell(
        "sh -c 'kill -STOP $$; echo first' &\n\
         sh -c 'kill -STOP $$; echo second' &\n\
         sleep 1\n\
         fg 1\n\
         echo after_fg\n\
         exit\n",
    );

    assert!(stderr.is_empty(), "stderr: {stderr}");

    assert!(stdout.contains("first"));
    assert!(stdout.contains("after_fg"));
}

#[test]
fn foreground_invalid_job_id() {
    let (_, stderr) = run_shell("fg 999\nexit\n");

    assert!(
        stderr.contains("fg: no jobs to bring to foreground"),
        "unexpected stderr: {stderr}"
    );
}
