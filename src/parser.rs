use std::iter::Peekable;

use crate::error::ParseErr;
use crate::lexer::{Token, TokenStream};
use crate::primitives::pair::Pair;
use crate::procedure::Proc;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    List(Vec<Expr>),
    Atom(Token),
    Proc(Proc),
    Dotted(Pair),
    // Quoted(Box<Expr>),
    EmptyList,
}

pub struct Parser<'a> {
    tokens: Peekable<TokenStream<'a>>,
    exprs: Vec<Expr>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenStream<'a>) -> Self {
        Parser {
            tokens: tokens.peekable(),
            exprs: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Expr>, ParseErr> {
        while self.tokens.peek().is_some() {
            let next = self.read_from_tokens()?;
            self.exprs.push(next)
        }
        Ok(self.exprs)
    }

    fn read_from_tokens(&mut self) -> Result<Expr, ParseErr> {
        if let Some(token) = self.tokens.next() {
            match token? {
                Token::LParen => {
                    let mut exprs: Vec<Expr> = vec![];

                    while let Some(t) = self.tokens.peek() {
                        if let Ok(Token::RParen) = t {
                            self.tokens.next();
                            match exprs.len() {
                                0 => return Ok(Expr::EmptyList),
                                _ => return Ok(Expr::List(exprs)),
                            }
                        } else {
                            exprs.push(self.read_from_tokens()?)
                        }
                    }

                    Err(ParseErr::UnexpectedEnd)
                }
                Token::RParen => Err(ParseErr::UnexpectedToken("unexpected )".to_string())),
                t => Ok(Expr::Atom(t)),
            }
        } else {
            Err(ParseErr::UnexpectedEnd)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_parse() {
        let scm = "1 (+ 1 (+ 1 2))";
        let res: Vec<Expr> = vec![
            Expr::Atom(Token::Number(1.0)),
            Expr::List(vec![
                Expr::Atom(Token::Symbol("+".to_string())),
                Expr::Atom(Token::Number(1.0)),
                Expr::List(vec![
                    Expr::Atom(Token::Symbol("+".to_string())),
                    Expr::Atom(Token::Number(1.0)),
                    Expr::Atom(Token::Number(2.0)),
                ]),
            ]),
        ];
        let exprs = Parser::new(TokenStream::new(scm)).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    #[should_panic]
    fn extra_paren() {
        let scm = "(+ 1 2) (1))";
        Parser::new(TokenStream::new(scm)).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn opening_rparen() {
        let scm = ")(yo)";
        Parser::new(TokenStream::new(scm)).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn no_close() {
        let scm = "(+ 1 (1)";
        Parser::new(TokenStream::new(scm)).parse().unwrap();
    }
}
