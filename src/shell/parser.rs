use crate::shell::lexer::Token;
use std::fmt;

pub struct Parser;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    StdoutNotFound,
    TokensIsEmpty,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::StdoutNotFound => write!(f, "Stdout is not found for redirection"),
            Error::TokensIsEmpty => write!(f, "Tokens passed to parser is empty"),
        }
    }
}

#[derive(Debug)]
pub enum Command {
    SimpleCommand {
        args: Vec<String>,
        stdout: Option<String>,
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

        let mut expecting_stdout = false;
        let mut args: Vec<String> = Vec::new();
        let mut stdout = None;
        for token in tokens.into_iter() {
            match token {
                Token::RedirectStdout => {
                    if expecting_stdout {
                        return Err(Error::StdoutNotFound);
                    }
                    expecting_stdout = true
                }
                Token::Word(word) => {
                    if expecting_stdout {
                        stdout = Some(word);
                        expecting_stdout = false;
                    } else {
                        args.push(word);
                    }
                }
            }
        }
        if expecting_stdout {
            Err(Error::StdoutNotFound)
        } else {
            Ok(Command::SimpleCommand { args, stdout })
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
            Ok(Command::SimpleCommand { args, stdout: None })
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
            Ok(Command::SimpleCommand { args, stdout: None })
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
            Ok(Command::SimpleCommand { args, stdout: None })
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
            Ok(Command::SimpleCommand { args, stdout: None })
                if args == vec!["command", "first", "second", "third"]
        ));
    }

    #[test]
    fn parse_stdout_redirection() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout,
            Token::Word("out.txt".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: Some(stdout) })
                if args == vec!["echo", "hello"] && stdout == "out.txt"
        ));
    }

    #[test]
    fn parse_redirection_without_filename() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout,
        ];

        assert!(matches!(parser.parse(tokens), Err(Error::StdoutNotFound)));
    }

    #[test]
    fn parse_redirection_with_arguments_after_target() {
        let parser = Parser::new();
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::RedirectStdout,
            Token::Word("out.txt".to_string()),
            Token::Word("world".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Ok(Command::SimpleCommand { args, stdout: Some(stdout) })
                if args == vec!["echo", "hello", "world"] && stdout == "out.txt"
        ));
    }
}
