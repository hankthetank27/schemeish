use std::iter::Peekable;
use std::vec::IntoIter;

use crate::error::ParseErr;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LParen,
    RParen,
    Number(f64),
    Symbol(String),
    Boolean(bool),
    Str(String),
}

pub fn tokenize(input: &str) -> TokenRes<Vec<Token>> {
    input.chars().peekable().collect_tokens(vec![])
}

type TokenRes<T> = Result<T, ParseErr>;

trait TokenIterator<T>
where
    T: Iterator<Item = char>,
{
    fn collect_tokens(&mut self, tokens: Vec<Token>) -> TokenRes<Vec<Token>>;
    fn parse_token(&mut self) -> Option<TokenRes<Token>>;
    fn parse_bool(&mut self) -> TokenRes<Token>;
    fn parse_string(&mut self) -> TokenRes<Token>;
    fn parse_symbol(&mut self) -> TokenRes<Token>;
    fn advance_to_token(&mut self) -> &Self;
    fn take_until<F: Fn(&T::Item) -> bool>(&mut self, pred: F) -> IntoIter<T::Item>;
}

impl<T> TokenIterator<T> for Peekable<T>
where
    T: Iterator<Item = char>,
{
    fn collect_tokens(&mut self, mut tokens: Vec<Token>) -> TokenRes<Vec<Token>> {
        self.advance_to_token();
        match self.parse_token() {
            Some(token) => {
                tokens.push(token?);
                self.collect_tokens(tokens)
            }
            None => Ok(tokens),
        }
    }

    fn parse_token(&mut self) -> Option<TokenRes<Token>> {
        match self.peek()? {
            '(' => {
                self.next();
                Some(Ok(Token::LParen))
            }
            ')' => {
                self.next();
                Some(Ok(Token::RParen))
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

    fn parse_bool(&mut self) -> TokenRes<Token> {
        match self.next() {
            Some('t') => Ok(Token::Boolean(true)),
            Some('f') => Ok(Token::Boolean(false)),
            Some(c) => Err(ParseErr::UnexpectedToken(
                format!("expected #t or #f, got #{}", c).to_string(),
            )),
            None => Err(ParseErr::MalformedToken(
                "expected charater indicating bool type",
            )),
        }
    }

    fn parse_string(&mut self) -> TokenRes<Token> {
        let value: String = self.take_until(|c| c != &'"').collect();
        self.next()
            .ok_or(ParseErr::MalformedToken("unclosed string"))?; //consume remaining quote
        Ok(Token::Str(value))
    }

    fn parse_symbol(&mut self) -> TokenRes<Token> {
        let value: String = self
            .take_until(|c| !c.is_whitespace() && c != &')' && c != &'(')
            .collect();

        Ok(parse_number(&value).unwrap_or(Token::Symbol(value)))
    }

    fn advance_to_token(&mut self) -> &Self {
        match self.next_if(|c| c.is_whitespace()) {
            Some(_) => self.advance_to_token(),
            None => self,
        }
    }

    fn take_until<F>(&mut self, pred: F) -> IntoIter<T::Item>
    where
        F: Fn(&T::Item) -> bool,
    {
        let mut new = vec![];

        while self.peek().map_or(false, &pred) {
            new.push(self.next().unwrap())
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
        let tokens = tokenize(&scm).unwrap();
        assert_eq!(tokens, res);
    }

    #[test]
    fn tokenise_empty() {
        let scm = "";
        let res: Vec<Token> = vec![];
        let tokens = tokenize(&scm).unwrap();
        assert_eq!(tokens, res);
    }

    #[test]
    fn tokenise_symbol() {
        let scm = "yoda";
        let res: Vec<Token> = vec![Token::Symbol("yoda".to_string())];
        let tokens = tokenize(&scm).unwrap();
        assert_eq!(tokens, res);
    }

    #[test]
    #[should_panic]
    fn tokenise_unclosed_string() {
        let scm = format!(r##" "sup"##);
        tokenize(&scm).unwrap();
    }

    #[test]
    #[should_panic]
    fn hash_error() {
        let scm = "(#t #f #c)";
        tokenize(&scm).unwrap();
    }
}
