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

#[derive(Debug)]
pub struct StdStatus {
    pub path: String,
    pub append: bool,
}

#[derive(Debug)]
pub enum Command {
    SimpleCommand {
        args: Vec<String>,
        stdout: Option<StdStatus>,
        stderr: Option<StdStatus>,
    },
}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Result<Command, Error> {
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
            }
        }
        if expecting_stdout {
            Err(Error::StdoutNotFound)
        } else if expecting_stderr {
            Err(Error::StderrNotFound)
        } else {
            Ok(Command::SimpleCommand {
                args,
                stdout,
                stderr,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_input() {
        let parser = Parser::new();

        assert!(matches!(parser.parse(vec![]), Err(Error::TokensIsEmpty)));
    }

    #[test]
    fn parse_command_without_arguments() {
        let parser = Parser::new();
        let tokens = vec![Token::Word("pwd".to_string())];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: None, stderr: None })
                if args == vec!["pwd"]
        ));
    }

    #[test]
    fn parse_command_with_arguments() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::Word("world".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: None, stderr: None })
                if args == vec!["echo", "hello", "world"]
        ));
    }

    #[test]
    fn parse_command_with_empty_argument() {
        let parser = Parser::new();
        let tokens = vec![Token::Word("echo".to_string()), Token::Word(String::new())];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: None, stderr: None })
                if args == vec!["echo", ""]
        ));
    }

    #[test]
    fn parse_preserves_argument_order() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("command".to_string()),
            Token::Word("first".to_string()),
            Token::Word("second".to_string()),
            Token::Word("third".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: None, stderr: None })
                if args == vec!["command", "first", "second", "third"]
        ));
    }

    #[test]
    fn parse_stdout_redirection() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout(false),
            Token::Word("out.txt".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: Some(StdStatus{ path, append }), stderr:None })
                if args == vec!["echo", "hello"] && path == "out.txt" && !append
        ));
    }

    #[test]
    fn parse_redirection_without_filename() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout(false),
        ];

        assert!(matches!(parser.parse(tokens), Err(Error::StdoutNotFound)));
    }

    #[test]
    fn parse_redirection_with_arguments_after_target() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout(false),
            Token::Word("out.txt".to_string()),
            Token::Word("world".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: Some(StdStatus{ path, append }), stderr: None })
                if args == vec!["echo", "hello", "world"] && path == "out.txt" && !append
        ));
    }

    #[test]
    fn test_stderr_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("error".into()),
            Token::RedirectStderr(false),
            Token::Word("error.txt".into()),
        ];

        let command = Parser.parse(tokens).unwrap();

        assert!(matches!(
            command,
            Command::SimpleCommand { args, stdout: None, stderr: Some(StdStatus{ path, append }) }
                if args == vec!["echo", "error"] && path == "error.txt" && !append
        ));
    }

    #[test]
    fn test_stderr_redirection_without_filename() {
        let tokens = vec![Token::Word("echo".into()), Token::RedirectStderr(false)];

        assert!(matches!(Parser.parse(tokens), Err(Error::StderrNotFound)));
    }

    #[test]
    fn test_stdout_and_stderr_redirection() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RedirectStdout(false),
            Token::Word("output.txt".into()),
            Token::RedirectStderr(false),
            Token::Word("error.txt".into()),
        ];

        let command = Parser.parse(tokens).unwrap();

        assert!(matches!(
            command,
            Command::SimpleCommand { args, stdout: Some(StdStatus{ path, append }), stderr: Some(StdStatus{ path: path_err, append: append_err }) }
                if args == vec!["echo", "hello"]
                    && path == "output.txt"
                    && path_err == "error.txt"
                    && !append && !append_err
        ));
    }

    #[test]
    fn parse_stdout_append_redirection() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout(true),
            Token::Word("out.txt".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand {
                args,
                stdout: Some(StdStatus { path, append: true }),
                stderr: None,
            }) if args == vec!["echo", "hello"] && path == "out.txt"
        ));
    }

    #[test]
    fn parse_stderr_append_redirection() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStderr(true),
            Token::Word("error.txt".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand {
                args,
                stdout: None,
                stderr: Some(StdStatus { path, append: true }),
            }) if args == vec!["echo", "hello"] && path == "error.txt"
        ));
    }
}
