use crate::shell::command::{Command, Job, ProcessGroup, StdStatus};
use crate::shell::lexer::Token;
use std::fmt;

pub struct Parser;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    StdoutNotFound,
    StderrNotFound,
    TokensIsEmpty,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::StdoutNotFound => write!(f, "Stdout is not found for redirection"),
            Error::StderrNotFound => write!(f, "Stderr is not found for redirection"),
            Error::TokensIsEmpty => write!(f, "Tokens passed to parser is empty"),
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Result<Job, Error> {
        if tokens.is_empty() {
            return Err(Error::TokensIsEmpty);
        }

        let mut expecting_stderr = false;
        let mut expecting_stdout = false;
        let mut args: Vec<String> = Vec::new();
        let mut stdout = None;
        let mut std_out_append = false;
        let mut std_err_append = false;
        let mut stderr = None;
        let mut pg = ProcessGroup::Foreground;
        for token in tokens.into_iter() {
            match token {
                Token::RedirectStdout(append) => {
                    if expecting_stdout {
                        return Err(Error::StdoutNotFound);
                    }
                    if expecting_stderr {
                        return Err(Error::StderrNotFound);
                    }
                    std_out_append = append;
                    expecting_stdout = true;
                }
                Token::RedirectStderr(append) => {
                    if expecting_stdout {
                        return Err(Error::StdoutNotFound);
                    }
                    if expecting_stderr {
                        return Err(Error::StderrNotFound);
                    }
                    std_err_append = append;
                    expecting_stderr = true;
                }
                Token::Word(word) => {
                    if expecting_stdout {
                        stdout = Some(StdStatus {
                            path: word,
                            append: std_out_append,
                        });
                        expecting_stdout = false;
                    } else if expecting_stderr {
                        stderr = Some(StdStatus {
                            path: word,
                            append: std_err_append,
                        });
                        expecting_stderr = false;
                    } else {
                        args.push(word);
                    }
                }
                Token::Ampersand => {
                    pg = ProcessGroup::Background;
                }
            }
        }
        if expecting_stdout {
            Err(Error::StdoutNotFound)
        } else if expecting_stderr {
            Err(Error::StderrNotFound)
        } else {
            Ok(Job::new(
                Command::SimpleCommand {
                    args,
                    stdout,
                    stderr,
                },
                pg,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(tokens: Vec<Token>) -> Result<Job, Error> {
        Parser::new().parse(tokens)
    }

    fn is_foreground_simple_command(
        job: &Job,
        expected_args: &[&str],
        expected_stdout: Option<(&str, bool)>,
        expected_stderr: Option<(&str, bool)>,
    ) -> bool {
        let ProcessGroup::Foreground = job.get_mode() else {
            return false;
        };

        let Command::SimpleCommand {
            args,
            stdout,
            stderr,
        } = job.get_command();

        let stdout_matches = match (stdout, expected_stdout) {
            (None, None) => true,
            (Some(actual), Some((path, append))) => actual.path == path && actual.append == append,
            _ => false,
        };

        let stderr_matches = match (stderr, expected_stderr) {
            (None, None) => true,
            (Some(actual), Some((path, append))) => actual.path == path && actual.append == append,
            _ => false,
        };

        args.iter().map(String::as_str).collect::<Vec<_>>() == expected_args
            && stdout_matches
            && stderr_matches
    }

    #[test]
    fn parse_empty_input() {
        assert!(matches!(parse(vec![]), Err(Error::TokensIsEmpty)));
    }

    #[test]
    fn parse_command_without_arguments() {
        let tokens = vec![Token::Word("pwd".into())];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(&job, &["pwd"], None, None,));
    }

    #[test]
    fn parse_command_with_arguments() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::Word("world".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "hello", "world"],
            None,
            None,
        ));
    }

    #[test]
    fn parse_command_with_empty_argument() {
        let tokens = vec![Token::Word("echo".into()), Token::Word(String::new())];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", ""],
            None,
            None,
        ));
    }

    #[test]
    fn parse_preserves_argument_order() {
        let tokens = vec![
            Token::Word("command".into()),
            Token::Word("first".into()),
            Token::Word("second".into()),
            Token::Word("third".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["command", "first", "second", "third"],
            None,
            None,
        ));
    }

    #[test]
    fn parse_stdout_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStdout(false),
            Token::Word("out.txt".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "hello"],
            Some(("out.txt", false)),
            None,
        ));
    }

    #[test]
    fn parse_redirection_without_filename() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStdout(false),
        ];

        assert!(matches!(parse(tokens), Err(Error::StdoutNotFound)));
    }

    #[test]
    fn parse_redirection_with_arguments_after_target() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStdout(false),
            Token::Word("out.txt".into()),
            Token::Word("world".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "hello", "world"],
            Some(("out.txt", false)),
            None,
        ));
    }

    #[test]
    fn parse_stderr_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("error".into()),
            Token::RedirectStderr(false),
            Token::Word("error.txt".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "error"],
            None,
            Some(("error.txt", false)),
        ));
    }

    #[test]
    fn parse_stderr_redirection_without_filename() {
        let tokens = vec![Token::Word("echo".into()), Token::RedirectStderr(false)];

        assert!(matches!(parse(tokens), Err(Error::StderrNotFound)));
    }

    #[test]
    fn parse_stdout_and_stderr_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStdout(false),
            Token::Word("output.txt".into()),
            Token::RedirectStderr(false),
            Token::Word("error.txt".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "hello"],
            Some(("output.txt", false)),
            Some(("error.txt", false)),
        ));
    }

    #[test]
    fn parse_stdout_append_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStdout(true),
            Token::Word("out.txt".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "hello"],
            Some(("out.txt", true)),
            None,
        ));
    }

    #[test]
    fn parse_stderr_append_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStderr(true),
            Token::Word("error.txt".into()),
        ];

        let job = parse(tokens).unwrap();

        assert!(is_foreground_simple_command(
            &job,
            &["echo", "hello"],
            None,
            Some(("error.txt", true)),
        ));
    }

    #[test]
    fn parse_foreground_job_by_default() {
        let job = parse(vec![Token::Word("sleep".into())]).unwrap();

        assert!(matches!(job.get_mode(), ProcessGroup::Foreground));
    }

    #[test]
    fn parse_background_job() {
        let tokens = vec![
            Token::Word("sleep".to_string()),
            Token::Word("10".to_string()),
            Token::Ampersand,
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_mode(), &ProcessGroup::Background);

        assert!(matches!(
            result.get_command(),
            Command::SimpleCommand { args, .. } if args == &["sleep", "10"]
        ));
    }
}
