use std::char;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Clone)]
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

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut iter = input.chars().peekable();
    let tokens: Vec<Token> = vec![];
    iter.parse_tokens(tokens)
}

trait TokenIterator<T>
where
    T: Iterator<Item = char>,
{
    fn parse_tokens(&mut self, tokens: Vec<Token>) -> Vec<Token>;
    fn parse_lparen(&mut self) -> Option<Token>;
    fn parse_rparen(&mut self) -> Option<Token>;
    fn parse_bool(&mut self) -> Option<Token>;
    fn parse_string(&mut self) -> Option<Token>;
    fn parse_symbol(&mut self) -> Option<Token>;
    fn parse_first_if(&mut self, cmp: char) -> Option<Token>;
    fn take_first_if(&mut self, cmp: char) -> Option<char>;
    fn advance_to_token(&mut self) -> &Self;
    fn take_until<F: Fn(&T::Item) -> bool>(&mut self, pred: F) -> IntoIter<T::Item>;
}

impl<T> TokenIterator<T> for Peekable<T>
where
    T: Iterator<Item = char>,
{
    fn parse_tokens(&mut self, mut tokens: Vec<Token>) -> Vec<Token> {
        self.advance_to_token();

        match self
            .parse_lparen()
            .or_else(|| self.parse_rparen())
            .or_else(|| self.parse_bool())
            .or_else(|| self.parse_string())
            .or_else(|| self.parse_symbol())
        {
            Some(token) => {
                tokens.push(token);
                return self.parse_tokens(tokens);
            }
            None => return tokens,
        };
    }

    fn parse_lparen(&mut self) -> Option<Token> {
        self.parse_first_if('(')
    }

    fn parse_rparen(&mut self) -> Option<Token> {
        self.parse_first_if(')')
    }

    fn parse_bool(&mut self) -> Option<Token> {
        self.take_first_if('#')?;
        match self.next() {
            Some('t') => Some(Token::Boolean(true)),
            Some('f') => Some(Token::Boolean(false)),
            Some(c) => panic!("Expected #t or #f, got #{c}"),
            None => panic!("Expected #t or #f, got nothing"),
        }
    }

    fn parse_string(&mut self) -> Option<Token> {
        self.take_first_if('"')?;

        let value: String = self.take_until(|c| c != &'"').collect();
        self.next(); //skip remaining quote

        Some(Token::Str(value))
    }

    fn parse_symbol(&mut self) -> Option<Token> {
        self.peek()?;

        let value: String = self
            .take_until(|c| c != &' ' && c != &'\n' && c != &')' && c != &'(')
            .collect();

        parse_number(&value).or_else(|| Some(Token::Symbol(value)))
    }

    fn parse_first_if(&mut self, cmp: char) -> Option<Token> {
        let token = self.next_if(|c| c == &cmp)?;
        Some(Token::find(token))
    }

    fn take_first_if(&mut self, cmp: char) -> Option<char> {
        let char = self.next_if(|c| c == &cmp)?;
        Some(char)
    }

    fn advance_to_token(&mut self) -> &Self {
        match self.next_if(|c| c == &' ' || c == &'\n') {
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
        let tokens = tokenize(&scm);
        assert_eq!(tokens, res);
    }

    #[test]
    fn test2() {
        let scm = "";
        let res: Vec<Token> = vec![];
        let tokens = tokenize(&scm);
        assert_eq!(tokens, res);
    }
}
