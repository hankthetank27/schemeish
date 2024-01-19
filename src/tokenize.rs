use core::panic;
use std::char;
use std::vec::IntoIter;
use std::{iter::Peekable, vec};

#[derive(Debug, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Number(f64),
    Symbol(String),
    Boolean(bool),
    Str(String),
}

impl Token {
    fn find(c: char) -> Token {
        match c {
            '(' => Token::LParen,
            ')' => Token::RParen,
            _ => unreachable!(),
        }
    }
}

trait TokenIterator<T: Iterator> {
    fn advance_to_token(self) -> Self
    where
        T::Item: PartialEq<char>;

    fn take_until<F>(&mut self, pred: F) -> IntoIter<T::Item>
    where
        F: Fn(&T::Item) -> bool;

    fn parse_first_if(&mut self, cmp: char) -> Option<Token>
    where
        T: Iterator<Item = char>;

    fn take_first_if(&mut self, cmp: char) -> Option<char>
    where
        T: Iterator<Item = char>;
}

impl<T: Iterator> TokenIterator<T> for Peekable<T> {
    fn advance_to_token(mut self) -> Self
    where
        T::Item: PartialEq<char>,
    {
        match self.next_if(|c| c == &' ' || c == &'\n') {
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
            new.push(self.next().unwrap());
        }

        new.into_iter()
    }

    fn parse_first_if(&mut self, cmp: char) -> Option<Token>
    where
        T: Iterator<Item = char>,
    {
        let token = self.next_if(|c| c == &cmp)?;
        Some(Token::find(token))
    }

    fn take_first_if(&mut self, cmp: char) -> Option<char>
    where
        T: Iterator<Item = char>,
    {
        let char = self.next_if(|c| c == &cmp)?;
        Some(char)
    }
}

pub fn tokenize<T>(iter: Peekable<T>, mut tokens: Vec<Token>) -> Vec<Token>
where
    T: Iterator<Item = char>,
{
    let mut iter = iter.advance_to_token();

    match parse_lparen(&mut iter)
        .or_else(|| parse_rparen(&mut iter))
        .or_else(|| parse_bool(&mut iter))
        .or_else(|| parse_string(&mut iter))
        .or_else(|| parse_symbol(&mut iter))
    {
        Some(token) => {
            tokens.push(token);
            return tokenize(iter, tokens);
        }
        None => return tokens,
    };
}

fn parse_lparen<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    iter.parse_first_if('(')
}

fn parse_rparen<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    iter.parse_first_if(')')
}

fn parse_bool<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    iter.take_first_if('#')?;
    match iter.next() {
        Some('t') => Some(Token::Boolean(true)),
        Some('f') => Some(Token::Boolean(false)),
        Some(c) => panic!("Expected #t or #f, got #{c}"),
        None => panic!("Expected #t or #f, got nothing"),
    }
}

fn parse_string<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    iter.take_first_if('"')?;

    let value: String = iter.take_until(|c| c != &'"').collect();
    iter.next(); //skip remaining quote

    Some(Token::Str(value))
}

fn parse_symbol<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    iter.peek()?;

    let value: String = iter
        .take_until(|c| c != &' ' && c != &'\n' && c != &')' && c != &'(')
        .collect();

    parse_number(&value).or_else(|| Some(Token::Symbol(value)))
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

    use super::{Token, *};

    #[test]
    fn test1() {
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
        let tokens = tokenize(scm.chars().peekable(), vec![]);
        assert_eq!(tokens, res);
    }

    #[test]
    fn test2() {
        let scm = "";
        let res: Vec<Token> = vec![];
        let tokens = tokenize(scm.chars().peekable(), vec![]);
        assert_eq!(tokens, res);
    }
}
