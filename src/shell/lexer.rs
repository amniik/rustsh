use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Tokenizer(State),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Tokenizer(state) => match state {
                State::NormalEscape => write!(f, "unexpected end of input after escape character"),
                State::SingleQuoted => write!(f, "unterminated single-quoted string"),
                State::DoubleQuoted => write!(f, "unterminated double-quoted string"),
                State::DoubleQuotedEscape => {
                    write!(
                        f,
                        "unexpected end of input after escape character in double quotes"
                    )
                }
                _ => write!(f, "unexpected error"),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Normal,
    SingleQuoted,
    DoubleQuoted,
    NormalEscape,
    DoubleQuotedEscape,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Word(String),
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn new() -> Self {
        Self
    }

    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        let mut token = String::new();
        let mut state = State::Normal;
        let mut word_started = false;

        for ch in input.chars() {
            match (&state, ch) {
                (State::Normal, ' ' | '\t') => {
                    if word_started {
                        tokens.push(Token::Word(std::mem::take(&mut token)));
                        word_started = false;
                    }
                }

                (State::Normal, '\'') => {
                    state = State::SingleQuoted;
                    word_started = true;
                }

                (State::Normal, '\\') => {
                    state = State::NormalEscape;
                }

                (State::Normal, '\"') => {
                    state = State::DoubleQuoted;
                    word_started = true;
                }

                (State::NormalEscape, _) => {
                    token.push(ch);
                    state = State::Normal;
                    word_started = true;
                }

                (State::SingleQuoted, '\'') => {
                    state = State::Normal;
                }

                (State::DoubleQuoted, '\"') => {
                    state = State::Normal;
                }

                (State::DoubleQuoted, '\\') => {
                    state = State::DoubleQuotedEscape;
                }

                (State::DoubleQuotedEscape, ch) => {
                    if ch == '\"' || ch == '\\' {
                        token.push(ch);
                    } else {
                        token.push('\\');
                        token.push(ch);
                    }
                    state = State::DoubleQuoted;
                }

                _ => {
                    token.push(ch);
                    word_started = true;
                }
            }
        }

        if state != State::Normal {
            return Err(Error::Tokenizer(state));
        }

        if word_started {
            tokens.push(Token::Word(token));
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenizer() -> Tokenizer {
        Tokenizer::new()
    }

    #[test]
    fn tokenize_empty_input() {
        assert_eq!(tokenizer().tokenize(""), Ok(vec![]));
    }

    #[test]
    fn tokenize_whitespace_only() {
        assert_eq!(tokenizer().tokenize(" \t  \t"), Ok(vec![]));
    }

    #[test]
    fn tokenize_single_word() {
        assert_eq!(
            tokenizer().tokenize("hello"),
            Ok(vec![Token::Word("hello".to_string())])
        );
    }

    #[test]
    fn tokenize_multiple_words() {
        assert_eq!(
            tokenizer().tokenize("echo hello world"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello".to_string()),
                Token::Word("world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_tabs_between_words() {
        assert_eq!(
            tokenizer().tokenize("echo\thello\tworld"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello".to_string()),
                Token::Word("world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_single_quoted_string() {
        assert_eq!(
            tokenizer().tokenize("echo 'hello world'"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_double_quoted_string() {
        assert_eq!(
            tokenizer().tokenize("echo \"hello world\""),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_empty_single_quoted_string() {
        assert_eq!(
            tokenizer().tokenize("echo ''"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word(String::new()),
            ])
        );
    }

    #[test]
    fn tokenize_empty_double_quoted_string() {
        assert_eq!(
            tokenizer().tokenize("echo \"\""),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word(String::new()),
            ])
        );
    }

    #[test]
    fn tokenize_adjacent_quoted_and_unquoted_text() {
        assert_eq!(
            tokenizer().tokenize("echo hello' world'"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_multiple_quoted_segments_in_one_word() {
        assert_eq!(
            tokenizer().tokenize("echo 'hello'\" world\""),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_escaped_space() {
        assert_eq!(
            tokenizer().tokenize(r"echo hello\ world"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_escaped_quote_in_double_quotes() {
        assert_eq!(
            tokenizer().tokenize(r#"echo "hello \"world\"""#),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word("hello \"world\"".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_escaped_backslash_in_double_quotes() {
        assert_eq!(
            tokenizer().tokenize(r#"echo "hello\\world""#),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word(r"hello\world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_backslash_in_single_quotes_is_literal() {
        assert_eq!(
            tokenizer().tokenize(r"echo 'hello\world'"),
            Ok(vec![
                Token::Word("echo".to_string()),
                Token::Word(r"hello\world".to_string()),
            ])
        );
    }

    #[test]
    fn tokenize_unterminated_single_quote() {
        assert!(matches!(
            tokenizer().tokenize("echo 'hello"),
            Err(Error::Tokenizer(State::SingleQuoted))
        ));
    }

    #[test]
    fn tokenize_unterminated_double_quote() {
        assert!(matches!(
            tokenizer().tokenize("echo \"hello"),
            Err(Error::Tokenizer(State::DoubleQuoted))
        ));
    }

    #[test]
    fn tokenize_trailing_escape() {
        assert!(matches!(
            tokenizer().tokenize("echo hello\\"),
            Err(Error::Tokenizer(State::NormalEscape))
        ));
    }

    #[test]
    fn tokenize_trailing_escape_in_double_quotes() {
        assert!(matches!(
            tokenizer().tokenize(r#"echo "hello\"#),
            Err(Error::Tokenizer(State::DoubleQuotedEscape))
        ));
    }
}
