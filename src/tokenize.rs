use core::f64;
use std::vec::IntoIter;
use std::{iter::Peekable, vec};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LParen,
    RParen,
    Number(f64),
    Symbol(String),
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

trait SoftIterator<T: Iterator> {
    fn take_until<F>(&mut self, pred: F) -> IntoIter<T::Item>
    where
        F: Fn(&T::Item) -> bool;
}

impl<T: Iterator> SoftIterator<T> for Peekable<T> {
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
}

pub fn tokenize<T>(iter: Peekable<T>, mut tokens: Vec<Token>) -> Vec<Token>
where
    T: Iterator<Item = char>,
{
    let mut iter = advance_whitespace(iter);

    match parse_lparen(&mut iter)
        .or_else(|| parse_rparen(&mut iter))
        .or_else(|| parse_symbol(&mut iter))
    {
        Some(token) => {
            tokens.push(token);
            return tokenize(iter, tokens);
        }
        None => return tokens,
    };
}

fn advance_whitespace<T>(mut iter: Peekable<T>) -> Peekable<T>
where
    T: Iterator<Item = char>,
{
    match iter.next_if(|c| c == &' ' || c == &'\n') {
        Some(_) => advance_whitespace(iter),
        None => iter,
    }
}

fn parse_lparen<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    parse_first_char(iter, '(')
}

fn parse_rparen<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    parse_first_char(iter, ')')
}

fn parse_symbol<T>(iter: &mut Peekable<T>) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    if !iter.peek().is_some() {
        return None;
    }

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

fn parse_first_char<T>(iter: &mut Peekable<T>, cmp: char) -> Option<Token>
where
    T: Iterator<Item = char>,
{
    let token = iter.next_if(|c| c == &cmp)?;
    Some(Token::find(token))
}

#[cfg(test)]
mod test {
    use super::{Token, *};

    #[test]
    fn test1() {
        let scm = "  (+ 1(+ 2  3)  2)   ";
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
