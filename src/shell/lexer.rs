#[derive(Debug, PartialEq, Eq)]
enum State {
    Normal,
    SingleQuoted,
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

    pub fn tokenize(&self, input: &str) -> Vec<Token> {
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

                (State::SingleQuoted, '\'') => {
                    state = State::Normal;
                }

                _ => {
                    token.push(ch);
                    word_started = true;
                }
            }
        }

        if word_started {
            tokens.push(Token::Word(token));
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Tokenizer::new().tokenize(input)
    }

    #[test]
    fn tokenize_empty_input() {
        assert_eq!(tokenize(""), vec![]);
    }

    #[test]
    fn tokenize_whitespace_only() {
        assert_eq!(tokenize("   \t  "), vec![]);
    }

    #[test]
    fn tokenize_single_word() {
        assert_eq!(tokenize("echo"), vec![Token::Word("echo".to_string())]);
    }

    #[test]
    fn tokenize_multiple_words() {
        assert_eq!(
            tokenize("echo hello world"),
            vec![
                Token::Word("echo".to_string()),
                Token::Word("hello".to_string()),
                Token::Word("world".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_multiple_spaces_and_tabs() {
        assert_eq!(
            tokenize("echo   hello\tworld"),
            vec![
                Token::Word("echo".to_string()),
                Token::Word("hello".to_string()),
                Token::Word("world".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_single_quoted_string() {
        assert_eq!(
            tokenize("echo 'hello world'"),
            vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_quoted_and_unquoted_text() {
        assert_eq!(
            tokenize("echo hello' world'"),
            vec![
                Token::Word("echo".to_string()),
                Token::Word("hello world".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_empty_quoted_string() {
        assert_eq!(
            tokenize("echo ''"),
            vec![Token::Word("echo".to_string()), Token::Word(String::new()),]
        );
    }

    #[test]
    fn tokenize_multiple_quoted_strings() {
        assert_eq!(
            tokenize("'hello world' 'rust shell'"),
            vec![
                Token::Word("hello world".to_string()),
                Token::Word("rust shell".to_string()),
            ]
        );
    }
}
