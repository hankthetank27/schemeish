use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Number(f64),
    Symbol(String),
    Boolean(bool),
    Str(String),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    input.chars().peekable().collect_tokens(vec![])
}

trait TokenIterator<T>
where
    T: Iterator<Item = char>,
{
    fn collect_tokens(&mut self, tokens: Vec<Token>) -> Vec<Token>;
    fn parse_token(&mut self) -> Option<Token>;
    fn parse_bool(&mut self) -> Token;
    fn parse_string(&mut self) -> Token;
    fn parse_symbol(&mut self) -> Token;
    fn advance_to_token(&mut self) -> &Self;
    fn take_until<F: Fn(&T::Item) -> bool>(&mut self, pred: F) -> IntoIter<T::Item>;
}

impl<T> TokenIterator<T> for Peekable<T>
where
    T: Iterator<Item = char>,
{
    fn collect_tokens(&mut self, mut tokens: Vec<Token>) -> Vec<Token> {
        self.advance_to_token();
        match self.parse_token() {
            Some(token) => {
                tokens.push(token);
                self.collect_tokens(tokens)
            }
            None => tokens,
        }
    }

    //TODO: return result type, handling parse errors
    fn parse_token(&mut self) -> Option<Token> {
        match self.peek()? {
            '(' => {
                self.next();
                Some(Token::LParen)
            }
            ')' => {
                self.next();
                Some(Token::RParen)
            }
            '#' => {
                self.next();
                Some(self.parse_bool())
            }
            '"' => {
                self.next();
                Some(self.parse_string())
            }
            _ => Some(self.parse_symbol()),
        }
    }

    fn parse_bool(&mut self) -> Token {
        //TODO: handle charater sequences error, ie. #tt, #fasdf
        match self.next() {
            Some('t') => Token::Boolean(true),
            Some('f') => Token::Boolean(false),
            //TODO: impl error types
            Some(c) => panic!("Expected #t or #f, got #{c}"),
            None => panic!("Expected #t or #f, got nothing"),
        }
    }

    fn parse_string(&mut self) -> Token {
        //TODO: impl error for unclosed string
        let value: String = self.take_until(|c| c != &'"').collect();
        self.next(); //consume remaining quote
        Token::Str(value)
    }

    fn parse_symbol(&mut self) -> Token {
        let value: String = self
            .take_until(|c| !c.is_whitespace() && c != &')' && c != &'(')
            .collect();

        parse_number(&value).unwrap_or_else(|| Token::Symbol(value))
    }

    fn advance_to_token(&mut self) -> &Self {
        match self.next_if(|c| c.is_whitespace()) {
            Some(_) => &self.advance_to_token(),
            None => self,
        }
    }

    fn take_until<F: Fn(&T::Item) -> bool>(&mut self, pred: F) -> IntoIter<T::Item> {
        let mut new = vec![];

        while self.peek().map_or(false, &pred) {
            new.push(self.next().unwrap());
        }

        new.into_iter()
    }
}

fn parse_number(val: &str) -> Option<Token> {
    if let Ok(num) = val.parse::<f64>() {
        Some(Token::Number(num))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenize_string() {
        let scm = format!(r##"  (+ 1(+ 2  3)  2 "lolz")"omg"   (#t #f)"##);
        let res = vec![
            Token::LParen,
            Token::Symbol("+".to_string()),
            Token::Number(1.0),
            Token::LParen,
            Token::Symbol("+".to_string()),
            Token::Number(2.0),
            Token::Number(3.0),
            Token::RParen,
            Token::Number(2.0),
            Token::Str("lolz".to_string()),
            Token::RParen,
            Token::Str("omg".to_string()),
            Token::LParen,
            Token::Boolean(true),
            Token::Boolean(false),
            Token::RParen,
        ];
        let tokens = tokenize(&scm);
        assert_eq!(tokens, res);
    }

    #[test]
    fn tokenise_empty() {
        let scm = "";
        let res: Vec<Token> = vec![];
        let tokens = tokenize(&scm);
        assert_eq!(tokens, res);
    }

    #[test]
    fn tokenise_symbol() {
        let scm = "yoda";
        let res: Vec<Token> = vec![Token::Symbol("yoda".to_string())];
        let tokens = tokenize(&scm);
        assert_eq!(tokens, res);
    }

    //TODO: THIS SHOULD ERROR
    // #[test]
    // fn tokenise_unclosed_string() {
    //     let scm = format!(r##" "sup"##);
    //     let tokens = tokenize(&scm);
    //     assert_eq!(tokens, error_type_here);
    // }

    #[test]
    #[should_panic]
    fn hash_error() {
        let scm = "(#t #f #c)";
        tokenize(&scm);
    }
}
