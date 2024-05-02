use core::str::Chars;
use std::iter::Peekable;

use crate::error::EvalErr;
use crate::special_form::MutCell;
use crate::utils::SoftIter;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LParen,
    RParen,
    If,
    And,
    Or,
    Define,
    Lambda,
    Cond,
    Else,
    Assignment,
    QuoteTick,
    QuoteProc,
    MutatePair(MutCell),
    Number(f64),
    Boolean(bool),
    Str(String),
    Symbol(String),
}

pub type TokenRes<T> = Result<T, EvalErr>;

pub struct TokenStream<'a>(Peekable<Chars<'a>>);

impl<'a> TokenStream<'a> {
    pub fn new(input: &'a str) -> Self {
        TokenStream(input.chars().peekable())
    }

    pub fn collect_tokens(self) -> Result<Vec<Token>, EvalErr> {
        let (tokens, errors) = self.fold((vec![], vec![]), |(mut tokens, mut errs), token| {
            match token {
                Ok(t) => tokens.push(t),
                Err(e) => errs.push(e),
            };
            (tokens, errs)
        });

        match errors.len() {
            0 => Ok(tokens),
            _ => Err(EvalErr::LexingFailures(errors)),
        }
    }

    fn parse_token(&mut self) -> Option<TokenRes<Token>> {
        match self.advance_to_token()?.peek()? {
            '(' => {
                self.0.next();
                Some(Ok(Token::LParen))
            }
            ')' => {
                self.0.next();
                Some(Ok(Token::RParen))
            }
            '#' => {
                self.0.next();
                Some(self.parse_bool())
            }
            '"' => {
                self.0.next();
                Some(self.parse_string())
            }
            '\'' => {
                self.0.next();
                Some(Ok(Token::QuoteTick))
            }
            c if c.is_numeric() => Some(self.parse_number()),
            _ => Some(self.parse_symbol()),
        }
    }

    fn parse_bool(&mut self) -> TokenRes<Token> {
        match self.0.next() {
            Some('t') => Ok(Token::Boolean(true)),
            Some('f') => Ok(Token::Boolean(false)),
            Some(c) => Err(EvalErr::UnexpectedToken(c.to_string())),
            None => Err(EvalErr::MalformedToken(
                "expected charater indicating bool type",
            )),
        }
    }

    fn parse_string(&mut self) -> TokenRes<Token> {
        let value: String = self.0.take_until(|c| c != &'"').collect();
        self.0
            .next() //consume remaining quote
            .ok_or(EvalErr::MalformedToken("unclosed string"))?;
        Ok(Token::Str(value))
    }

    fn parse_symbol(&mut self) -> TokenRes<Token> {
        let value: String = self.0.take_until(|c| !end_of_token(c)).collect();

        Ok(match value.as_str() {
            "if" => Token::If,
            "define" => Token::Define,
            "lambda" => Token::Lambda,
            "quote" => Token::QuoteProc,
            "and" => Token::And,
            "or" => Token::Or,
            "cond" => Token::Cond,
            "set!" => Token::Assignment,
            "else" => Token::Else,
            "set-car!" => Token::MutatePair(MutCell::Car),
            "set-cdr!" => Token::MutatePair(MutCell::Cdr),
            _ => Token::Symbol(value),
        })
    }

    fn parse_number(&mut self) -> TokenRes<Token> {
        let err = EvalErr::MalformedToken("failed to parse number");
        let value: String = self
            .0
            .take_until(|c| c.is_numeric() && !end_of_token(c))
            .collect();

        match self.0.peek() {
            Some(c) if !c.is_numeric() && !end_of_token(c) => Err(err),
            _ => Ok(Token::Number(value.parse().map_err(|_| err)?)),
        }
    }

    fn advance_to_token(&mut self) -> Option<&mut Peekable<Chars<'a>>> {
        while self
            .consume_comment()?
            .peek()
            .map_or(false, |c| c.is_whitespace())
        {
            self.0.next();
        }

        Some(&mut self.0)
    }

    fn consume_comment(&mut self) -> Option<&mut Peekable<Chars<'a>>> {
        if self.0.peek()? == &';' {
            self.0.take_until(|c| c != &'\n');
        }
        Some(&mut self.0)
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = TokenRes<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_token()
    }
}

fn end_of_token(c: &char) -> bool {
    c.is_whitespace() || c == &')' || c == &'(' || c == &';'
}

#[cfg(test)]
mod test {
    use super::*;

    fn tokenize(input: &str) -> TokenRes<Vec<Token>> {
        TokenStream::new(input).collect()
    }

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
    fn tokenise_number() {
        let scm = " 123";
        let res: Vec<Token> = vec![Token::Number(123.0)];
        let tokens = tokenize(&scm).unwrap();
        assert_eq!(tokens, res);
    }

    #[test]
    fn tokenise_parse_fail() {
        let scm = " 123)";
        let res: Vec<Token> = vec![Token::Number(123.0), Token::RParen];
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
    fn parse_num_failure() {
        let scm = "5d";
        tokenize(&scm).unwrap();
    }

    #[test]
    #[should_panic]
    fn hash_error() {
        let scm = "(#t #f #c)";
        tokenize(&scm).unwrap();
    }
}
