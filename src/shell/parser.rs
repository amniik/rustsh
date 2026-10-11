use crate::shell::lexer::{Token, Token::Word};

pub struct Parser;
pub enum Command {
    SimpleCommand { args: Vec<String> },
}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Option<Command> {
        if tokens.is_empty() {
            return None;
        }

        let args = tokens
            .into_iter()
            .map(|token| match token {
                Word(word) => word,
            })
            .collect();

        Some(Command::SimpleCommand { args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_input() {
        let parser = Parser::new();

        assert!(parser.parse(vec![]).is_none());
    }

    #[test]
    fn parse_command_without_arguments() {
        let parser = Parser::new();
        let tokens = vec![Word("pwd".to_string())];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Some(Command::SimpleCommand { args })
                if args == vec!["pwd"]
        ));
    }

    #[test]
    fn parse_command_with_arguments() {
        let parser = Parser::new();
        let tokens = vec![
            Word("echo".to_string()),
            Word("hello".to_string()),
            Word("world".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Some(Command::SimpleCommand { args })
                if args == vec!["echo", "hello", "world"]
        ));
    }

    #[test]
    fn parse_command_with_empty_argument() {
        let parser = Parser::new();
        let tokens = vec![Word("echo".to_string()), Word(String::new())];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Some(Command::SimpleCommand { args })
                if args == vec!["echo", ""]
        ));
    }

    #[test]
    fn parse_preserves_argument_order() {
        let parser = Parser::new();
        let tokens = vec![
            Word("command".to_string()),
            Word("first".to_string()),
            Word("second".to_string()),
            Word("third".to_string()),
        ];

        let command = parser.parse(tokens);

        assert!(matches!(
            command,
            Some(Command::SimpleCommand { args })
                if args == vec!["command", "first", "second", "third"]
        ));
    }
}
